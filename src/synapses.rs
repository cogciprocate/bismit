use common;
use ocl::{ self, Ocl, WorkSize };
use envoy::{ Envoy };
use cortical_areas::{ CorticalAreas, Width };
use cortical_regions::{ CorticalRegion, CorticalRegionKind };
use cortical_region_layer::{ CorticalRegionLayer };
use protocell::{ CellKind, Protocell, DendriteKind };
use dendrites::{ Dendrites };
use axons::{ Axons };

use std::num;
use std::rand;
use std::mem;
use std::rand::distributions::{ Normal, IndependentSample, Range };
use std::rand::{ ThreadRng };
use std::num::{ NumCast, Int, FromPrimitive };
use std::default::{ Default };
use std::fmt::{ Display };

pub struct Synapses {
	depth: u8,
	width: u32,
	per_cell_l2: u32,
	den_kind: DendriteKind,
	cell_kind: CellKind,
	since_decay: usize,
	kern_cycle: ocl::Kernel,
	pub states: Envoy<ocl::cl_uchar>,
	pub strengths: Envoy<ocl::cl_char>,
	pub src_row_ids: Envoy<ocl::cl_uchar>,
	pub src_col_offs: Envoy<ocl::cl_char>,
}

impl Synapses {
	pub fn new(width: u32, depth: u8, per_cell_l2: u32, den_kind: DendriteKind, cell_kind: CellKind, 
					region: &CorticalRegion, axons: &Axons, ocl: &Ocl) -> Synapses {

		let syns_per_row = width << per_cell_l2;

		let wg_size = common::SYNAPSES_WORKGROUP_SIZE;
		//print!("\nNew {:?} Synapses with: depth: {}, width: {}, per_cell_l2: {}, syns_per_row(row area): {}", den_kind, depth, width, per_cell_l2, syns_per_row);

		let states = Envoy::<ocl::cl_uchar>::new(syns_per_row, depth, 0, ocl);
		let strengths = Envoy::<ocl::cl_char>::new(syns_per_row, depth, 0, ocl);
		let mut src_row_ids = Envoy::<ocl::cl_uchar>::new(syns_per_row, depth, 0, ocl);
		//let mut src_col_offs = Envoy::<ocl::cl_char>::new(syns_per_row, depth, 0, ocl);
		let mut src_col_offs = Envoy::<ocl::cl_char>::shuffled(syns_per_row, depth, -128, 127, ocl);

		let mut kern_cycle = ocl.new_kernel("syns_cycle", 
			WorkSize::TwoDim(depth as usize, width as usize))
			.lws(WorkSize::TwoDim(1 as usize, wg_size as usize))
			.arg_env(&axons.states)
			.arg_env(&src_col_offs)
			.arg_env(&src_row_ids)
			.arg_scl(per_cell_l2)
			//.arg_env(&aux.ints_0)
			//.arg_env(&aux.ints_1)
			.arg_env(&states)
		;

		let mut syns = Synapses {
			width: width,
			depth: depth,
			per_cell_l2: per_cell_l2,
			den_kind: den_kind,
			cell_kind: cell_kind,
			since_decay: 0,
			kern_cycle: kern_cycle,
			states: states,
			strengths: strengths,
			src_row_ids: src_row_ids,
			src_col_offs: src_col_offs,
		};

		syns.init(region);

		syns
	}

	fn init(&mut self, region: &CorticalRegion) {
		assert!(
			(self.src_col_offs.width() == self.src_row_ids.width()) 
			&& ((self.src_row_ids.width() == (self.width << self.per_cell_l2))), 
			"[cells::Synapses::init(): width mismatch]"
		);

		let syns_per_row = self.width << self.per_cell_l2;
		let mut rng = rand::weak_rng();

		/* LOOP THROUGH ALL LAYERS */
		for (&layer_name, layer) in region.layers().iter() {
			let src_row_ids: Vec<u8> = match layer.cell {
				Some(ref cell) => {
					if cell.cell_kind == self.cell_kind {
						region.src_row_ids(layer_name, self.den_kind)
					} else {
						continue
					}
				},
				_ => continue,
			};

			let row_ids = region.row_ids(vec!(layer_name));
			let src_row_ids_len: usize = src_row_ids.len();

			assert!(src_row_ids_len > 0, "Synapses must have at least one source row");

			let kind_base_row_pos = layer.kind_base_row_pos;
			let src_row_idx_range: Range<usize> = Range::new(0, src_row_ids_len);
			
			assert!(src_row_ids_len <= (1 << self.per_cell_l2) as usize, "cells::Synapses::init(): Number of source rows must not exceed number of synapses per cell.");

			print!("\nLayer: \"{}\" ({:?}): row_ids: {:?}, src_row_ids: {:?}", layer_name, self.den_kind, row_ids, src_row_ids);
			
			/* LOOP THROUGH ROWS (WITHIN LAYER) */
			for row_pos in kind_base_row_pos..(kind_base_row_pos + layer.depth) {
				let ei_start = syns_per_row as usize * row_pos as usize;
				let ei_end = ei_start + syns_per_row as usize;
				print!("\n	Row {}: ei_start: {}, ei_end: {}, src_ids: {:?}", row_pos, ei_start, ei_end, src_row_ids);

				/* LOOP THROUGH ENVOY VECTOR ELEMENTS (WITHIN ROW) */
				for ref i in ei_start..ei_end {
					self.src_row_ids[i] = src_row_ids[src_row_idx_range.ind_sample(&mut rng)];
				}
			}
		}

		self.strengths.write();
		self.src_col_offs.write();
		self.src_row_ids.write();		
	}

	pub fn cycle(&self) {
		self.kern_cycle.enqueue();
	}

}
