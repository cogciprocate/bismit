use common;
use ocl;
use envoy::{ Envoy };
use cortical_areas::{ CorticalAreas, Width };
use cortical_regions::{ CorticalRegions, CorticalRegionType, CorticalLayerClass };

use std::num;
use std::rand;
use std::mem;
use std::rand::distributions::{ Normal, IndependentSample, Range };
use std::num::{ NumCast, Int, FromPrimitive };
use std::default::{ Default };
use std::fmt::{ Display };


pub struct Cells {
	pub width: u32,
	pub height_antecellular: u8,
	pub height_cellular: u8,
	pub axns: Axons,
	pub dst_dens: Dendrites,
	pub prx_dens: Dendrites,
	ocl: ocl::Ocl,
}
impl Cells {
	pub fn new(regions: &CorticalRegions, areas: &CorticalAreas, ocl: &ocl::Ocl) -> Cells {
		let region_type = CorticalRegionType::Sensory;				// NEED TO CHANGE WHEN WE ADD OTHER REGIONS
		let (height_antecellular, height_cellular) = regions.height(CorticalRegionType::Sensory); // CHANGE
		let height_total = height_antecellular + height_cellular;
		let width = areas.width(CorticalRegionType::Sensory);		// NEED TO CHANGE WHEN WE ADD OTHER REGIONS  ^^^

		Cells {
			width: width,
			height_antecellular: height_antecellular,
			height_cellular: height_cellular,
			axns:	Axons::new(width, height_antecellular, height_cellular, ocl),
			dst_dens: Dendrites::new(width, height_cellular, regions, ocl),
			prx_dens: Dendrites::new(width, height_cellular, regions, ocl),
			ocl: ocl.clone(),
		}
	}

	pub fn cycle(&self) {
		self.dst_dens.syns.cycle(&self.axns, &self.ocl);
		self.dst_dens.cycle(&self.ocl);
		self.axns.cycle(&self.ocl);
	}
}


pub struct Axons {
	height_antecellular: u8,
	height_cellular: u8,
	width: u32,
	padding: u32,
	pub states: Envoy<ocl::cl_char>,
}

impl Axons {
	pub fn new(width: u32, height_antecellular: u8, height_cellular: u8,	ocl: &ocl::Ocl) -> Axons {
		let padding: u32 = num::cast(common::AXONS_MARGIN * 2).expect("Axons::new()");
		let height = height_cellular + height_antecellular;

		Axons {
			height_antecellular: height_antecellular,
			height_cellular: height_cellular,
			width: width,
			padding: padding,
			states: Envoy::<ocl::cl_char>::with_padding(padding, width, height, 0i8, ocl),
		}
	}

	fn cycle(&self, ocl: &ocl::Ocl) {
		//let width: u32 = self.areas.width(CorticalRegionType::Sensory);
		//let (height_antecellular, height_cellular) = self.regions.height(CorticalRegionType::Sensory);

		let kern = ocl::new_kernel(ocl.program, "cycle_axns");
		ocl::set_kernel_arg(0, self.states.buf, kern);
		ocl::set_kernel_arg(1, self.states.buf, kern);
		ocl::set_kernel_arg(2, self.height_antecellular as u32, kern);

		let gws = (self.height_cellular as usize, self.width as usize);

		ocl::enqueue_2d_kernel(kern, ocl.command_queue, &gws);

	}
}


pub struct Dendrites {
	height: u8,
	width: u32,
	pub thresholds: Envoy<ocl::cl_char>,
	pub states: Envoy<ocl::cl_char>,
	pub syns: Synapses,
}

impl Dendrites {
	pub fn new(width: u32, height: u8, regions: &CorticalRegions, ocl: &ocl::Ocl) -> Dendrites {
		let width_dens = width * num::cast(common::DENDRITES_PER_NEURON).expect("cells::Dendrites::new()");
		Dendrites {
			width: width,
			height: height,
			thresholds: Envoy::<ocl::cl_char>::new(width_dens, height, common::DENDRITE_INITIAL_THRESHOLD, ocl),
			states: Envoy::<ocl::cl_char>::new(width_dens, height, 0i8, ocl),
			syns: Synapses::new(width, height, regions, ocl),
		}
	}

	fn cycle(&self, ocl: &ocl::Ocl) {

		//let width: u32 = self.areas.width(CorticalRegionType::Sensory);
		//let (_, height_cellular) = self.regions.height(CorticalRegionType::Sensory);

		let width_dens: usize = self.height as usize * self.width as usize * common::DENDRITES_PER_NEURON;

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
	pub states: Envoy<ocl::cl_char>,
	pub strengths: Envoy<ocl::cl_char>,
	pub axn_row_ids: Envoy<ocl::cl_uchar>,
	pub axn_col_offs: Envoy<ocl::cl_char>,
}

impl Synapses {
	pub fn new(width: u32, height: u8, regions: &CorticalRegions, ocl: &ocl::Ocl) -> Synapses {
		let width_syns = width * num::cast(common::SYNAPSES_PER_NEURON).expect("cells::Synapses::new()");

		let mut axn_row_ids = Envoy::<ocl::cl_uchar>::new(width_syns, height, 0, ocl);
		let mut axn_col_offs = Envoy::<ocl::cl_char>::new(width_syns, height, 0, ocl);

		let mut dst_syns = Synapses {
			width: width,
			height: height,
			states: Envoy::<ocl::cl_char>::new(width_syns, height, 0, ocl),
			strengths: Envoy::<ocl::cl_char>::new(width_syns, height, common::SYNAPSE_STRENGTH_ZERO, ocl),
			axn_row_ids: axn_row_ids,
			axn_col_offs: axn_col_offs,
		};

		dst_syns.init(regions);

		dst_syns
	}

	fn init(&mut self, regions: &CorticalRegions) {
		assert!(self.axn_col_offs.width() == self.axn_row_ids.width(), "[cells::Synapse::init(): width mismatch]");
		let width = self.axn_col_offs.width();

		let mut rng = rand::thread_rng();

		let col_off_range: Range<i8> = Range::new(-126, 127);


		let ref r = regions[CorticalRegionType::Sensory];

		for (&ln, l) in r.layers.iter() {
			let row_ids = r.layer_row_ids_ct(ln);
			let src_row_ids: Vec<u8> =	match l.class {
				CorticalLayerClass::Interlaminar(_, _) => {
					r.layer_src_row_ids(ln)
				},
				_ => continue,
			};

			for &ri in row_ids.iter() {

				let src_row_idx_count: u8 = num::cast(src_row_ids.len()).expect("cells::Synapses::init()");
				let src_row_idx_range: Range<u8> = Range::new(0, src_row_idx_count);

					//	Envoy Indexes
				let ei_start = width as usize * ri as usize;
				let ei_end = ei_start + width as usize;

				for i in range(ei_start, ei_end) {
					self.axn_row_ids[i] = src_row_ids[src_row_idx_range.ind_sample(&mut rng) as usize];
					self.axn_col_offs[i] = col_off_range.ind_sample(&mut rng);
				}
			}
		}

		self.axn_col_offs.write();
		self.axn_row_ids.write();
	}

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

		let gws = (self.height as usize, self.width as usize, common::SYNAPSES_PER_NEURON);

		//println!("gws: {:?}", gws);

		ocl::enqueue_3d_kernel(kern, ocl.command_queue, &gws);

	}
}


/*pub struct Somata {	
	pub states: Envoy<ocl::cl_char>,
}

impl Somata {
	pub fn new(width: u32, height: u8, ocl: &ocl::Ocl) -> Somata {
		Somata { states: Envoy::<ocl::cl_char>::new(width, height, 0i8, ocl), }
	}
}*/
