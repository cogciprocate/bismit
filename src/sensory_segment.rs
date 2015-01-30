extern crate libc;

use ocl;
use common;
use envoy::{ Envoy };
use column;
use column::{ Columns };
use chord::{ Chord };

use std::rand;
use std::rand::distributions::{IndependentSample, Range};
use std::num;
use std::mem;
use std::num::{ Int };


pub struct SensorySegment {
	pub values: Envoy<ocl::cl_char>,
	//pub target_addresses: column::Axons,
	//pub target_segment_idx: usize,
	//pub context: ocl::cl_context,
	//pub command_queue: ocl::cl_command_queue,
	//pub sense_kernel: ocl::cl_kernel,


}
impl SensorySegment {
	pub fn new(width: usize, ocl: &ocl::Ocl) -> SensorySegment {

		let values = Envoy::<ocl::cl_char>::new(width, 0i8, ocl);
		//let target_addresses = column::Axons::new(common::COLUMN_SYNAPSES_PER_SEGMENT, ocl);


		//let kern = ocl::new_kernel(ocl.program, "sense");
		//let dup_factor: ocl::cl_uint = num::FromPrimitive::from_uint(target_segment.columns.synapses.values.len() / values.len()).expect();


		//let dup_factor_shift: ocl::cl_uchar = common::int_hb_log2(dup_factor);
		

		//let tmp_out = Envoy::new(common::COLUMN_SYNAPSES_PER_SEGMENT, 0u32, ocl);

	
		/*
		ocl::set_kernel_arg(0, values.buf, kern);
		ocl::set_kernel_arg(1, target_segment.columns.synapses.values.buf, kern);
		ocl::set_kernel_arg(2, target_addresses.target_column_somata.buf, kern);
		ocl::set_kernel_arg(3, target_addresses.target_column_synapses.buf, kern);
		ocl::set_kernel_arg(4, dup_factor_shift, kern);
		*/
		

		SensorySegment {
			values: values,
			//target_addresses: target_addresses,
			//target_segment_idx: target_segment_idx,
			//context: ocl.context,
			//command_queue: ocl.command_queue,
			//sense_kernel: kern,

	
		}
	}

	pub fn sense(&mut self, chord: &Chord) {
		chord.unfold_into(&mut self.values.vec);
		self.values.write();


		//ocl::enqueue_kernel(self.sense_kernel, self.command_queue, common::COLUMN_SYNAPSES_PER_SEGMENT);

	}

}
