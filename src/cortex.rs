use ocl;
use common;
use envoy::{ Envoy };
use sensory_segment::{ SensorySegment };
use chord::{ Chord };
use cell:: { self, Cells };

use std::rand;
use std::rand::distributions::{IndependentSample, Range};
use std::ptr;
use std::num;
use time;

pub struct HyperColumns {
	pub qty: usize,
	pub states: Envoy<ocl::cl_int>,
}
impl HyperColumns {
	pub fn new(qty: usize, ocl: &ocl::Ocl) -> HyperColumns {
		HyperColumns {
			qty: common::HYPERCOLUMNS_PER_SEGMENT,
			states: Envoy::<ocl::cl_int>::new(common::HYPERCOLUMNS_PER_SEGMENT, 0i32, ocl),
		}
	}
}


pub struct MotorSegment {
	pub targets: Envoy<ocl::cl_short>,
	pub values: Envoy<ocl::cl_char>,
}
impl MotorSegment {
	pub fn new(width: usize, ocl: &ocl::Ocl) -> MotorSegment {
		MotorSegment { 
			targets : Envoy::<ocl::cl_short>::new(width, 0i16, ocl),
			values : Envoy::<ocl::cl_char>::new(width, 0i8, ocl),
		}
	}
}


pub struct Cortex {
	pub cells: Cells,
	pub ocl: ocl::Ocl,
	pub sensory_segments: Vec<SensorySegment>,
}

impl Cortex {
	pub fn new() -> Cortex {
		println!("Initializing Cortex... ");
		let time_start = time::get_time().sec;

		let ocl: ocl::Ocl = ocl::Ocl::new();

		assert!(common::SENSORY_SEGMENTS_TOTAL <= common::CORTICAL_SEGMENTS_TOTAL);
		let mut ss = Vec::with_capacity(common::SENSORY_SEGMENTS_TOTAL);
		for i in range(0, common::SENSORY_SEGMENTS_TOTAL) {
			ss.push(SensorySegment::new(common::SENSORY_CHORD_WIDTH, &ocl));
		}

		let time_finish = time::get_time().sec;

		println!(" ...initialized in: {} sec.", time_finish - time_start);

		Cortex {
			cells: Cells::new(&ocl),
			sensory_segments: ss,
			ocl: ocl,
		}
	}

	pub fn sense(&mut self, sgmt_idx: usize, chord: &Chord) {

		self.sensory_segments[sgmt_idx].sense(chord);

		self.cycle_cel_syns(&self.sensory_segments[sgmt_idx].values);
		self.cycle_cel_dens();
		self.cycle_cel_axons();

	}

	fn cycle_cel_syns(&self, input_source: &Envoy<ocl::cl_char>,) {
		
		let layers_total: u32 = num::cast(common::LAYERS_PER_SEGMENT).expect("cycle_cel_syns, layers_total");
		let syn_per_layer: u32 = num::cast(common::SYNAPSES_PER_LAYER).expect("cycle_cel_syns, syn_per_layer");
		let axons_per_layer: u32 = num::cast(common::COLUMNS_PER_SEGMENT).expect("cycle_cel_syns, axons_per_layer");


		let il: i16 = num::cast(input_source.vec.len()).expect("cycle_cel_syns, il");
		//assert!(il == self.input_source_len, "Input vector size must equal self.input_source_len");

		let kern = ocl::new_kernel(self.ocl.program, "cycle_cel_syns");
		ocl::set_kernel_arg(1, self.cells.synapses.source_addrs.buf, kern);
		ocl::set_kernel_arg(2, self.cells.synapses.strengths.buf, kern);
		ocl::set_kernel_arg(3, self.cells.synapses.values.buf, kern);


		let cp = CcssParams { 
			input_source: input_source, 
			seq_iters: 4,
			syn_offset_factor: syn_per_layer, 
			syn_offset_start: 0,
			src_offset_factor: 0,
			src_offset_start: 0,
			gid_offset_factor: 0, 
			boost_factor: 1,
		 };
		self.cycle_cel_syn_seq(cp, kern);

		/*
		let cp = CcssParams { 
			input_source: &self.cells.axons.states, 
			seq_iters: 0,
			syn_offset_factor: syn_per_layer, 
			syn_offset_start: syn_per_layer * 4,
			src_offset_factor: axons_per_layer,
			src_offset_start: (axons_per_layer * 4) - 128,
			gid_offset_factor: 1, 
			boost_factor: 16,
		 };
		self.cycle_cel_syn_seq(cp, kern);
		*/

	}

	fn cycle_cel_syn_seq( 
					&self, 
					params: CcssParams,
					kern: ocl::cl_kernel,
	) {
		let mut syn_offset = params.syn_offset_start;
		let mut src_offset = params.src_offset_start;

		assert!(
			((params.src_offset_factor + (src_offset * params.seq_iters)) as usize) 
			< (params.input_source.vec.len() - 128)
		);

		for i in range(0, params.seq_iters) {
			
			println!("[syn_offset: {}, src_offset: {}, gid_offset_factor: {}, source.len: {}]", syn_offset, src_offset, params.gid_offset_factor, params.input_source.vec.len());

			ocl::set_kernel_arg(0, params.input_source.buf, kern);
			ocl::set_kernel_arg(4, src_offset, kern);
			ocl::set_kernel_arg(5, syn_offset, kern);
			ocl::set_kernel_arg(6, params.gid_offset_factor, kern);
			ocl::set_kernel_arg(7, params.boost_factor, kern);

			ocl::enqueue_kernel(kern, self.ocl.command_queue, common::SYNAPSES_PER_LAYER);

			syn_offset += params.syn_offset_factor;
			src_offset += params.src_offset_factor;
		}

	}

	fn cycle_cel_dens(&self) {
		let kern = ocl::new_kernel(self.ocl.program, "cycle_cel_dens");

		ocl::set_kernel_arg(0, self.cells.synapses.values.buf, kern);
		ocl::set_kernel_arg(1, self.cells.dendrites.thresholds.buf, kern);
		ocl::set_kernel_arg(2, self.cells.dendrites.values.buf, kern);

		ocl::enqueue_kernel(kern, self.ocl.command_queue, common::CELL_DENDRITES_PER_SEGMENT);

	}

	fn cycle_cel_axons(&self) {
		let kern = ocl::new_kernel(self.ocl.program, "cycle_cel_axons");

		ocl::set_kernel_arg(0, self.cells.dendrites.values.buf, kern);
		ocl::set_kernel_arg(1, self.cells.axons.states.buf, kern);

		ocl::enqueue_kernel(kern, self.ocl.command_queue, common::CELLS_PER_SEGMENT);

	}


	pub fn release_components(&mut self) {
		println!("\nReleasing OCL Components...");
		self.ocl.release_components();
	}
}


struct CcssParams<'a> {
	input_source: &'a Envoy<ocl::cl_char>, 
	seq_iters: u32,
	syn_offset_factor: u32, 
	syn_offset_start: u32,
	src_offset_factor: u32,
	src_offset_start: u32,
	gid_offset_factor: u32, 
	boost_factor: u8
}
