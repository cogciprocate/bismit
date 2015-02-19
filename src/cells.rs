use common;
use ocl;
use envoy::{ Envoy };
use cortical_areas::{ CorticalAreas, Width };
use cortical_regions::{ CorticalRegions, CorticalRegionType };
use protocell::{  };
use cortical_layer::{ CellKind, DendriteType };


use std::num;
use std::rand;
use std::mem;
use std::rand::distributions::{ Normal, IndependentSample, Range };
use std::rand::{ ThreadRng };
use std::num::{ NumCast, Int, FromPrimitive };
use std::default::{ Default };
use std::fmt::{ Display };



pub struct Cells {
	pub width: u32,
	pub height_noncellular: u8,
	pub height_cellular: u8,
	pub axns: Axons,
	pub dst_dens: Dendrites,
	pub prx_dens: Dendrites,
	ocl: ocl::Ocl,
}
impl Cells {
	pub fn new(regions: &CorticalRegions, areas: &CorticalAreas, ocl: &ocl::Ocl) -> Cells {
		let region_type = CorticalRegionType::Sensory;				// NEED TO CHANGE WHEN WE ADD OTHER REGIONS
		let (height_noncellular, height_cellular) = regions.height(CorticalRegionType::Sensory); // CHANGE
		println!("Cells::new(): height_noncellular: {}, height_cellular: {}", height_noncellular, height_cellular);
		assert!(height_cellular > 0, "cells::Cells::new(): Region has no cellular layers.");
		let height_total = height_noncellular + height_cellular;
		let width = areas.width(CorticalRegionType::Sensory);		// NEED TO CHANGE WHEN WE ADD OTHER REGIONS  ^^^

		Cells {
			width: width,
			height_noncellular: height_noncellular,
			height_cellular: height_cellular,
			axns:	Axons::new(width, height_noncellular, height_cellular, ocl),
			dst_dens: Dendrites::new(width, height_cellular, DendriteType::Distal, regions, ocl),
			prx_dens: Dendrites::new(width, height_cellular, DendriteType::Proximal, regions, ocl),
			ocl: ocl.clone(),
		}
	}

	pub fn cycle(&self) {
		self.dst_dens.cycle(&self.axns, &self.ocl);
		self.prx_dens.syns.cycle(&self.axns, &self.ocl);
		self.axns.cycle(&self.dst_dens, &self.ocl);
	}
}

pub struct Somata {	
	pub states: Envoy<ocl::cl_char>,
}

impl Somata {
	pub fn new(width: u32, height: u8, ocl: &ocl::Ocl) -> Somata {
		Somata { states: Envoy::<ocl::cl_char>::new(width, height, 0i8, ocl), }
	}
}


pub struct Axons {
	height_noncellular: u8,
	height_cellular: u8,
	width: u32,
	padding: u32,
	pub states: Envoy<ocl::cl_char>,
}

impl Axons {
	pub fn new(width: u32, height_noncellular: u8, height_cellular: u8, ocl: &ocl::Ocl) -> Axons {
		let padding: u32 = num::cast(common::AXONS_MARGIN * 2).expect("Axons::new()");
		let height = height_cellular + height_noncellular;

		//println!("New Axons with: height_ac: {}, height_c: {}, width: {}", height_noncellular, height_cellular, width);

		Axons {
			height_noncellular: height_noncellular,
			height_cellular: height_cellular,
			width: width,
			padding: padding,
			states: Envoy::<ocl::cl_char>::with_padding(padding, width, height, 0i8, ocl),
		}
	}

	fn cycle(&self, dst_dens: &Dendrites, ocl: &ocl::Ocl) {
		//let width: u32 = self.areas.width(CorticalRegionType::Sensory);
		//let (height_noncellular, height_cellular) = self.regions.height(CorticalRegionType::Sensory);

		let kern = ocl::new_kernel(ocl.program, "cycle_axns");
		ocl::set_kernel_arg(0, dst_dens.states.buf, kern);
		ocl::set_kernel_arg(1, self.states.buf, kern);
		ocl::set_kernel_arg(2, self.height_noncellular as u32, kern);

		let gws = (self.height_cellular as usize, self.width as usize);

		ocl::enqueue_2d_kernel(kern, ocl.command_queue, &gws);

	}
}


pub struct Dendrites {
	height: u8,
	width: u32,
	per_cell: u32,
	den_type: DendriteType,
	pub thresholds: Envoy<ocl::cl_char>,
	pub states: Envoy<ocl::cl_char>,
	pub syns: Synapses,
}

impl Dendrites {
	pub fn new(width: u32, height: u8, den_type: DendriteType, regions: &CorticalRegions, ocl: &ocl::Ocl) -> Dendrites {
		let per_cell = match den_type {
			DendriteType::Distal =>		common::DENDRITES_PER_NEURON_DISTAL,
			DendriteType::Proximal =>	common::DENDRITES_PER_NEURON_PROXIMAL,
		};
		let width_dens = width * per_cell;	//	num::cast(common::DENDRITES_PER_NEURON).expect("cells::Dendrites::new()");

		//println!("New Dendrites with: height: {}, width_dens: {}", height, width_dens);

		Dendrites {
			height: height,
			width: width,
			per_cell: per_cell,
			den_type: den_type,
			thresholds: Envoy::<ocl::cl_char>::new(width_dens, height, common::DENDRITE_INITIAL_THRESHOLD, ocl),
			states: Envoy::<ocl::cl_char>::new(width_dens, height, 0i8, ocl),
			syns: Synapses::new(width, height, per_cell * common::SYNAPSES_PER_DENDRITE, den_type, regions, ocl),
		}
	}

	fn cycle(&self, axns: &Axons, ocl: &ocl::Ocl) {
		self.syns.cycle(axns, ocl);

		//let width: u32 = self.areas.width(CorticalRegionType::Sensory);
		//let (_, height_cellular) = self.regions.height(CorticalRegionType::Sensory);

		let width_dens: usize = self.height as usize * self.width as usize * self.per_cell as usize;

		let kern = ocl::new_kernel(ocl.program, "cycle_dens");

		ocl::set_kernel_arg(0, self.syns.states.buf, kern);
		ocl::set_kernel_arg(1, self.thresholds.buf, kern);
		ocl::set_kernel_arg(2, self.states.buf, kern);

		ocl::enqueue_kernel(kern, ocl.command_queue, width_dens);

	}
}


pub struct Synapses {
	height: u8,
	width: u32,
	per_cell: u32,
	den_type: DendriteType,
	pub states: Envoy<ocl::cl_char>,
	pub strengths: Envoy<ocl::cl_char>,
	pub axn_row_ids: Envoy<ocl::cl_uchar>,
	pub axn_col_offs: Envoy<ocl::cl_char>,
}

impl Synapses {
	pub fn new(width: u32, height: u8, per_cell: u32, den_type: DendriteType, regions: &CorticalRegions, ocl: &ocl::Ocl) -> Synapses {
		let width_syns = width * per_cell;

		println!("New Synapses with: height: {}, width: {}, per_cell(row depth): {}, width_syns(row area): {}", height, width, per_cell, width_syns);

		let mut axn_row_ids = Envoy::<ocl::cl_uchar>::new(width_syns, height, 0, ocl);
		let mut axn_col_offs = Envoy::<ocl::cl_char>::new(width_syns, height, 0, ocl);


		let mut syns = Synapses {
			width: width,
			height: height,
			per_cell: per_cell,
			den_type: den_type,
			states: Envoy::<ocl::cl_char>::new(width_syns, height, 0, ocl),
			strengths: Envoy::<ocl::cl_char>::new(width_syns, height, common::SYNAPSE_STRENGTH_ZERO, ocl),
			axn_row_ids: axn_row_ids,
			axn_col_offs: axn_col_offs,
		};


		syns.init(regions);

		syns
	}

	fn init(&mut self, regions: &CorticalRegions) {
		assert!((self.axn_col_offs.width() == self.axn_row_ids.width()) && ((self.axn_row_ids.width() == (self.width * self.per_cell))), "[cells::Synapse::init(): width mismatch]");

		let ref region = regions[CorticalRegionType::Sensory];
		assert!(region.layers.len() > 0, "cells::Synapses::init(): Region has no layers.");

		let row_len = self.width * self.per_cell;
		let mut rng = rand::thread_rng();


			/* LOOP THROUGH LAYERS */
		for (&ln, l) in region.layers.iter() {
			let src_row_ids: Vec<u8> =	match l.cell {
				Some(_) => {
					region.src_row_ids(ln, self.den_type)
				},
				_ => continue,
			};
			let kind_base_row_pos = l.kind_base_row_pos;
			let src_row_idx_len: usize = src_row_ids.len();
			let src_row_idx_range: Range<usize> = Range::new(0, src_row_idx_len);
			//let row_ids = region.row_ids(vec!(ln));
			//println!("Layer: \"{}\": row_ids: {:?}, src_row_ids: {:?}", ln, row_ids, src_row_ids);
			
				/* LOOP THROUGH ROWS OF LIKE KIND (WITHIN LAYER)*/
			for row_pos in range(kind_base_row_pos, kind_base_row_pos + l.height) {
				let ei_start = row_len as usize * row_pos as usize;
				let ei_end = ei_start + row_len as usize;
				//println!("	ei_start: {}, ei_end: {}, idx_len: {}", ei_start, ei_end, src_row_idx_len);

				let col_off_range: Range<i8> = Range::new(-126, 127);

				/* LOOP THROUGH ENVOY VECTOR ELEMENTS (WITHIN ROW) */
				match self.den_type {
					DendriteType::Distal => {
						for i in range(ei_start, ei_end) {
							self.axn_row_ids[i] = src_row_ids[src_row_idx_range.ind_sample(&mut rng)];
							self.axn_col_offs[i] = col_off_range.ind_sample(&mut rng);
						}
					},
					DendriteType::Proximal => {
						for i in range(ei_start, ei_end) {
							self.axn_row_ids[i] = src_row_ids[src_row_idx_range.ind_sample(&mut rng)];
							self.axn_col_offs[i] = 0;
						}
					},
				}
			}
		}
		self.axn_col_offs.write();
		self.axn_row_ids.write();		
	}

	/*fn _init_dist_row(&mut self, ei_start: usize, ei_end: usize, src_row_ids: &Vec<u8>, src_row_idx_range: Range<usize>, rng: &ThreadRng) {
		
	}

	fn _init_prox_row(&mut self, ei_start: usize, ei_end: usize, src_row_ids: &Vec<u8>, src_row_idx_range: Range<usize>, rng: &ThreadRng) {
		for i in range(ei_start, ei_end) {
			self.axn_row_ids[i] = src_row_ids[src_row_idx_range.ind_sample(&mut rng) as usize];
			
		}
	}*/

	pub fn cycle(&self, axns: &Axons, ocl: &ocl::Ocl) {

		//let width: u32 = self.width;
		//let height_total: u8 = self.regions.height_total(CorticalRegionType::Sensory);
		//let height_cellular = self.height;
		//let len: u32 = width * height_total as u32;

		//let test_envoy = Envoy::<ocl::cl_int>::new(width, height_total, 0, &self.ocl);

		//println!("cycle_cel_syns running with width = {}, height = {}", width, height_total);

		let kern = ocl::new_kernel(ocl.program, "cycle_syns");
		ocl::set_kernel_arg(0, axns.states.buf, kern);
		ocl::set_kernel_arg(1, self.axn_row_ids.buf, kern);
		ocl::set_kernel_arg(2, self.axn_col_offs.buf, kern);
		ocl::set_kernel_arg(3, self.strengths.buf, kern);
		ocl::set_kernel_arg(4, self.states.buf, kern);

		//println!("height_total: {}, height_cellular: {}, width_syn_row: {}", height_total, height_cellular, width_syn_row);

		let gws = (self.height as usize, self.width as usize, self.per_cell as usize);

		//println!("gws: {:?}", gws);

		ocl::enqueue_3d_kernel(kern, ocl.command_queue, &gws);

	}
}


