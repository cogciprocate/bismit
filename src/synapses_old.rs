use common;
use ocl::{ self, Ocl };
use envoy::{ Envoy };
use cortical_areas::{ CorticalAreas, Width };
use cortical_regions::{ CorticalRegion, CorticalRegionType };
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
	per_cell: u32,
	den_type: DendriteKind,
	since_decay: usize,
	pub states: Envoy<ocl::cl_uchar>,
	pub strengths: Envoy<ocl::cl_char>,
	pub axn_row_ids: Envoy<ocl::cl_uchar>,
	pub axn_col_offs: Envoy<ocl::cl_char>,
}

impl Synapses {
	pub fn new(width: u32, depth: u8, per_cell: u32, den_type: DendriteKind, region: &CorticalRegion, ocl: &Ocl) -> Synapses {
		let width_syns = width * per_cell;

		//println!("New {:?} Synapses with: depth: {}, width: {}, per_cell(row depth): {}, width_syns(row area): {}", den_type, depth, width, per_cell, width_syns);

		let mut axn_row_ids = Envoy::<ocl::cl_uchar>::new(width_syns, depth, 0, ocl);
		let mut axn_col_offs = Envoy::<ocl::cl_char>::new(width_syns, depth, 0, ocl);

		let mut syns = Synapses {
			width: width,
			depth: depth,
			per_cell: per_cell,
			den_type: den_type,
			since_decay: 0,
			states: Envoy::<ocl::cl_uchar>::new(width_syns, depth, 0, ocl),
			strengths: Envoy::<ocl::cl_char>::new(width_syns, depth, 0, ocl),
			axn_row_ids: axn_row_ids,
			axn_col_offs: axn_col_offs,
		};

		syns.init(region);

		syns
	}

	fn init(&mut self, region: &CorticalRegion) {
		assert!((self.axn_col_offs.width() == self.axn_row_ids.width()) && ((self.axn_row_ids.width() == (self.width * self.per_cell))), "[cells::Synapses::init(): width mismatch]");

		//let ref region = region[CorticalRegionType::Sensory];
		assert!(region.layers.len() > 0, "cells::Synapses::init(): Region has no layers.");

		let row_len = self.width * self.per_cell;
		let mut rng = rand::weak_rng();

		/* LOOP THROUGH LAYERS */
		for (&ln, l) in region.layers.iter() {
			let src_row_ids: Vec<u8> =	match l.cell {
				Some(_) => {
					region.src_row_ids(ln, self.den_type)
				},
				_ => continue,
			};

			let src_row_ids_len: usize = src_row_ids.len();

			if src_row_ids_len == 0 {
				continue
			}

			let kind_base_row_pos = l.kind_base_row_pos;
			
			let src_row_idx_range: Range<usize> = Range::new(0, src_row_ids_len);
			assert!(src_row_ids_len <= self.per_cell as usize, "cells::Synapses::init(): Number of source rows must not exceed number of synapses per cell.");


			let row_ids = region.row_ids(vec!(ln));

			//println!("Layer: \"{}\" ({:?}): row_ids: {:?}, src_row_ids: {:?}", ln, self.den_type, row_ids, src_row_ids);
			
			/* LOOP THROUGH ROWS OF LIKE KIND (WITHIN LAYER) */
			for row_pos in range(kind_base_row_pos, kind_base_row_pos + l.depth) {
				let ei_start = row_len as usize * row_pos as usize;
				let ei_end = ei_start + row_len as usize;
				//println!("	Row {}: ei_start: {}, ei_end: {}, idx_len: {}", row_pos, ei_start, ei_end, src_row_ids_len);
				let col_off_range: Range<i8> = Range::new(-126, 127);

				/* LOOP THROUGH ENVOY VECTOR ELEMENTS (WITHIN ROW) */

				match self.den_type {

					DendriteKind::Distal => {
						for i in range(ei_start, ei_end) {
							self.strengths[i] = common::DST_SYNAPSE_STRENGTH_DEFAULT;
							self.axn_row_ids[i] = src_row_ids[src_row_idx_range.ind_sample(&mut rng)];
							self.axn_col_offs[i] = col_off_range.ind_sample(&mut rng);
						}
					},

					DendriteKind::Proximal => {
							//	TEMPORARY HOKIE WORKAROUND
							//	REWRITE LOOPING OF ROWS ON A PER-DENDRITE-TYPE BASIS
						if ei_start as usize >= self.strengths.len() {
							break
						}

						let mut syn_pos: usize = 0;

						for i in range(ei_start, ei_end) {

							if syn_pos == self.per_cell as usize {
								syn_pos = 0;
							}

							if syn_pos < src_row_ids_len {
								self.strengths[i] = common::DST_SYNAPSE_STRENGTH_DEFAULT;
								self.axn_row_ids[i] = src_row_ids[syn_pos];
							} else {
								self.strengths[i] = common::DST_SYNAPSE_STRENGTH_DEFAULT;
								self.axn_row_ids[i] = src_row_ids[src_row_idx_range.ind_sample(&mut rng)];
							}

							self.axn_col_offs[i] = 0;

							syn_pos += 1;
						}
					},

				}
			}
		}

		self.strengths.write();
		self.axn_col_offs.write();
		self.axn_row_ids.write();		
	}

	pub fn cycle(&self, axns: &Axons, ocl: &Ocl) {
		//println!("cycle_cel_syns running with width = {}, depth = {}", width, depth_total);

		let kern = ocl::new_kernel(ocl.program, "syns_cycle");

		ocl::set_kernel_arg(0, axns.states.buf, kern);
		ocl::set_kernel_arg(1, self.axn_row_ids.buf, kern);
		ocl::set_kernel_arg(2, self.axn_col_offs.buf, kern);
		ocl::set_kernel_arg(3, self.strengths.buf, kern);
		ocl::set_kernel_arg(4, self.states.buf, kern);

		//println!("depth_total: {}, depth_cellular: {}, width_syn_row: {}", depth_total, depth_cellular, width_syn_row);

		let gws = (self.depth as usize, self.width as usize, self.per_cell as usize);

		//println!("gws: {:?}", gws);

		ocl::enqueue_3d_kernel(ocl.command_queue, kern, None, &gws, None);
	}

	pub fn decay(&mut self, rand_ofs: &mut Envoy<ocl::cl_uchar>, ocl: &Ocl) {
		self.since_decay += 1;

		if self.since_decay >= common::SYNAPSE_DECAY_INTERVAL {
			let kern = ocl::new_kernel(ocl.program, "syns_decay");
			ocl::set_kernel_arg(0, self.strengths.buf, kern);

			let gws = (self.depth as usize, self.width as usize, self.per_cell as usize);
			ocl::enqueue_3d_kernel(ocl.command_queue, kern, None, &gws, None);

			self.regrow(rand_ofs, ocl);
			self.since_decay = 0;
		}

	}

	pub fn regrow(&self, rand_ofs: &mut Envoy<ocl::cl_uchar>, ocl: &Ocl) {

		common::shuffle_vec(&mut rand_ofs.vec);
		rand_ofs.write();
		
		let kern = ocl::new_kernel(ocl.program, "syns_regrow");
		ocl::set_kernel_arg(0, self.strengths.buf, kern);
		ocl::set_kernel_arg(1, rand_ofs.buf, kern);
		ocl::set_kernel_arg(2, self.axn_col_offs.buf, kern);

		let gws = (self.depth as usize, self.width as usize, self.per_cell as usize);
		ocl::enqueue_3d_kernel(ocl.command_queue, kern, None, &gws, None);
	}
}
