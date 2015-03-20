use common;
use ocl::{ self, Ocl, WorkSize };
use envoy::{ Envoy };
use cortical_areas::{ CorticalAreas, Width };
use cortical_regions::{ CorticalRegion, CorticalRegionType };
use protocell::{ CellKind, Protocell, DendriteKind };
use synapses::{ Synapses };
use dendrites::{ Dendrites };
use axons::{ Axons };
use cells:: { Aux };

use std::num;
use std::ops;
use std::rand;
use std::mem;
use std::rand::distributions::{ Normal, IndependentSample, Range };
use std::rand::{ ThreadRng };
use std::num::{ NumCast, Int, FromPrimitive };
use std::default::{ Default };
use std::fmt::{ Display };


pub struct Columns {
	width: u32,
	kern_cycle: ocl::Kernel,
	pub states: Envoy<ocl::cl_uchar>,
	pub syns: ColumnSynapses,
	
}

impl Columns {
	pub fn new(width: u32, region: &CorticalRegion, axons: &Axons, aux: &Aux, ocl: &Ocl) -> Columns {
		let height: u8 = 1;
		let syns_per_cell = common::DENDRITES_PER_CELL_PROXIMAL * common::SYNAPSES_PER_DENDRITE_PROXIMAL;

		let states = Envoy::<ocl::cl_uchar>::new(width, height, common::STATE_ZERO, ocl);
		let syns = ColumnSynapses::new(width, syns_per_cell, region, axons, aux, ocl);

		let mut kern_cycle = ocl.new_kernel("col_cycle", WorkSize::TwoDim(height as usize, width as usize));
		kern_cycle.new_arg_envoy(&syns.states);
		kern_cycle.new_arg_envoy(&states);
		
		Columns {
			width: width,
			kern_cycle: kern_cycle,
			states: states,
			syns: syns,
		}
	}

	pub fn cycle(&self) {
		self.syns.cycle();
		self.kern_cycle.enqueue();
	}
}


pub struct ColumnSynapses {
	width: u32,
	height: u8,
	per_cell: u32,
	src_row_ids_list: Vec<u8>,
	kern_cycle: ocl::Kernel,
	pub states: Envoy<ocl::cl_uchar>,
	pub strengths: Envoy<ocl::cl_char>,
	pub src_ofs: Envoy<ocl::cl_char>,
	pub src_row_ids: Envoy<ocl::cl_uchar>,
}

impl ColumnSynapses {
	pub fn new(width: u32, per_cell: u32, region: &CorticalRegion, axons: &Axons, aux: &Aux, ocl: &Ocl) -> ColumnSynapses {
		let syns_per_row = width * per_cell;
		let src_row_ids_list: Vec<u8> = region.src_row_ids(region.col_input_row(), DendriteKind::Proximal);
		let src_rows_len = src_row_ids_list.len() as u8;
		let height = src_rows_len;
		let wg_size = common::SYNAPSES_WORKGROUP_SIZE;
		//let dens_per_wg: u32 = wg_size / (common::SYNAPSES_PER_DENDRITE_PROXIMAL);
		let syns_per_cell_l2: u32 = common::SYNAPSES_PER_CELL_PROXIMAL_LOG2;
		//let dens_per_wg: u32 = 1;

		print!("\nNew Column Synapses with: height: {}, syns_per_row: {},", height, syns_per_row);

		let states = Envoy::<ocl::cl_uchar>::new(syns_per_row, height, common::STATE_ZERO, ocl);
		let strengths = Envoy::<ocl::cl_char>::new(syns_per_row, height, 1i8, ocl);
		let src_ofs = Envoy::<ocl::cl_char>::shuffled(syns_per_row, height, -128, 127, ocl);
		let src_row_ids= Envoy::<ocl::cl_uchar>::new(syns_per_row, height, 0u8, ocl);

		let mut kern_cycle = ocl.new_kernel("syns_cycle", 
			WorkSize::TwoDim(height as usize, width as usize))
			.lws(WorkSize::TwoDim(1 as usize, wg_size as usize));
		kern_cycle.new_arg_envoy(&axons.states);
		kern_cycle.new_arg_envoy(&src_ofs);
		kern_cycle.new_arg_envoy(&src_row_ids);
		kern_cycle.new_arg_scalar(syns_per_cell_l2);
		kern_cycle.new_arg_envoy(&aux.ints_0);
		kern_cycle.new_arg_envoy(&aux.ints_1);
		kern_cycle.new_arg_envoy(&states);
		
		//println!("src_row_ids_list[0]: {}", src_row_ids_list[0]);
		
		let mut syns = ColumnSynapses {
			width: width,
			height: height,
			per_cell: per_cell,
			src_row_ids_list: src_row_ids_list,
			states: states,
			strengths: strengths,
			src_ofs: src_ofs,
			src_row_ids: src_row_ids,
			kern_cycle: kern_cycle,
		};

		syns.init(region);

		syns
	}

	fn init(&mut self, region: &CorticalRegion) {
		let row_len = self.width * self.per_cell;
		let mut rng = rand::weak_rng();
		let ei_start = 0usize;
		let ei_end = ei_start + row_len as usize;
		let src_row_idx_range: Range<usize> = Range::new(0, self.src_row_ids_list.len());
		//println!("\nInitializing Column Synapses: ei_start: {}, ei_end: {}, self.src_row_ids: {:?}, self.src_row_ids.len(): {}", ei_start, ei_end, self.src_row_ids_list, self.src_row_ids_list.len());

		for i in range(ei_start, ei_end) {
			self.src_row_ids[i] = self.src_row_ids_list[src_row_idx_range.ind_sample(&mut rng)];
		}
		self.src_row_ids.write();
	}

	pub fn cycle(&self) {
		self.kern_cycle.enqueue();
	}
}
