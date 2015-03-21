use common;
use ocl::{ self, Ocl, WorkSize };
use envoy::{ Envoy };
use cortical_areas::{ CorticalAreas, Width };
use cortical_regions::{ CorticalRegion, CorticalRegionKind };
use protocell::{ CellKind, Protocell, DendriteKind };
use synapses::{ Synapses };
use dendrites::{ Dendrites };
use axons::{ Axons };
use cells:: { Aux };
use aspiny:: { AspinyStellate };

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
	kern_axns_cycle: ocl::Kernel,
	pub states: Envoy<ocl::cl_uchar>,
	pub asps: AspinyStellate,
	pub syns: ColumnSynapses,
	
}

impl Columns {
	pub fn new(width: u32, region: &CorticalRegion, axons: &Axons, aux: &Aux, ocl: &Ocl) -> Columns {
		let il = region.col_input_layer();
		let height: u8 = il.height();

		let syns_per_cell_l2: u32 = common::SYNAPSES_PER_CELL_PROXIMAL_LOG2;
		let syns_per_cell: u32 = 1 << syns_per_cell_l2;

		let states = Envoy::<ocl::cl_uchar>::new(width, height, common::STATE_ZERO, ocl);

		let asps = AspinyStellate::new(width, height, region, &states, ocl);
		let syns = ColumnSynapses::new(width, height, syns_per_cell, region, axons, aux, ocl);

		let mut kern_cycle = ocl.new_kernel("dens_cycle", WorkSize::TwoDim(height as usize, width as usize))
			.arg_env(&syns.states)
			.arg_scl(syns_per_cell_l2)
			.arg_env(&states);

		println!("\ncol base_row_pos: {}", il.base_row_pos());

		let mut kern_axns_cycle = ocl.new_kernel("col_axns_cycle_unoptd", WorkSize::TwoDim(height as usize, width as usize))
			.lws(WorkSize::TwoDim(1 as usize, common::AXONS_WORKGROUP_SIZE as usize))
			.arg_env(&asps.ids)
			.arg_env(&asps.states)
			.arg_env(&asps.wins)
			.arg_env(&states)
			.arg_env(&axons.states)
			.arg_env(&aux.ints_0)
			.arg_env(&aux.ints_1)
			//self.kern_cycle.arg_local(0u8, common::AXONS_WORKGROUP_SIZE / common::ASPINY_SPAN as usize);
			.arg_scl(il.base_row_pos() as u32);
		
		Columns {
			width: width,
			kern_cycle: kern_cycle,
			kern_axns_cycle: kern_axns_cycle,
			states: states,
			asps: asps,
			syns: syns,
		}
	}

	pub fn cycle(&mut self) {
		self.syns.cycle();
		self.kern_cycle.enqueue();
		self.asps.cycle();
		self.kern_axns_cycle.enqueue();
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
	pub fn new(width: u32, height: u8, per_cell: u32, region: &CorticalRegion, axons: &Axons, aux: &Aux, ocl: &Ocl) -> ColumnSynapses {

		let syns_per_row = width * per_cell;
		let src_row_ids_list: Vec<u8> = region.src_row_ids(region.col_input_layer_name(), DendriteKind::Proximal);
		let src_rows_len = src_row_ids_list.len() as u8;
		//let height = src_rows_len;
		let wg_size = common::SYNAPSES_WORKGROUP_SIZE;
		//let dens_per_wg: u32 = wg_size / (common::SYNAPSES_PER_DENDRITE_PROXIMAL);
		let syns_per_cell_l2: u32 = common::SYNAPSES_PER_CELL_PROXIMAL_LOG2;
		//let dens_per_wg: u32 = 1;

		print!("\nNew Column Synapses with: height: {}, syns_per_row: {}, src_rows_len: {}", height, syns_per_row, src_rows_len);

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
		//kern_cycle.new_arg_envoy(&aux.ints_0);
		//kern_cycle.new_arg_envoy(&aux.ints_1);
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
		let len = self.width * self.per_cell * self.height as u32;
		let mut rng = rand::weak_rng();
		let ei_start = 0usize;
		let ei_end = ei_start + len as usize;
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
