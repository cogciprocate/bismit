use ocl;
use common;
use cortex::{ HyperColumns };
use envoy::{ Envoy };
use sensory_segment::{ SensorySegment };
use chord::{ Chord };
use column;
use column::{ Columns };
use cell::{ Cells };

use rand;
use rand::distributions::{IndependentSample, Range};
use std::ptr;
use num;

use time;

pub struct CorticalSegment {
	//pub hypercolumns: HyperColumns,
	//pub columns: Columns,
	pub cells: Cells,
	pub ocl: ocl::Ocl,
	//pub input_source_len: i16,		// CHANGE TO BUFFER
}

impl  CorticalSegment {
	pub fn new(hcols: usize, ocl: &ocl::Ocl) -> CorticalSegment {

		CorticalSegment {
			//hypercolumns: HyperColumns::new(hcols, ocl),
			//columns: Columns::new(hcols, ocl),
			cells: Cells::new(ocl),
			ocl: ocl.clone(),
			//input_source_len: input_source_len,		// CHANGE TO BUFFER
		}
	}

	pub fn cycle(&self, input_source: &Envoy<ocl::cl_char>) {
		//self.cycle_col_dens();
		//self.cycle_col_soms();

	}


	/*
	*	struct CcssParams<'a> {
	*		input_source: &'a Envoy<ocl::cl_char>, 
	*		seq_iters: u32,
	*		syn_offset_factor: u32, 
	*		syn_offset_start: u32,
	*		src_offset_factor: u32,
	*		src_offset_start: u32,
	*		gid_offset_factor: u32, 
	*		boost_factor: u8
	*	}
	*/


	/*
	fn cycle_col_dens(&self) {
		let kern = ocl::new_kernel(self.ocl.program, "cycle_col_dens");

		ocl::set_kernel_arg(0, self.columns.synapses.values.buf, kern);
		ocl::set_kernel_arg(1, self.columns.synapses.strengths.buf, kern);
		ocl::set_kernel_arg(2, self.columns.dendrites.thresholds.buf, kern);
		ocl::set_kernel_arg(3, self.columns.dendrites.values.buf, kern);

		ocl::enqueue_kernel(kern, self.ocl.command_queue, common::COLUMN_DENDRITES_PER_SEGMENT);
	}

	fn cycle_col_soms(&self) {
		let kern = ocl::new_kernel(self.ocl.program, "cycle_col_soms");

		ocl::set_kernel_arg(0, self.columns.dendrites.values.buf, kern);
		ocl::set_kernel_arg(1, self.columns.somata.states.buf, kern);
		ocl::set_kernel_arg(2, self.cells.somata.states.buf, kern);

		ocl::enqueue_kernel(kern, self.ocl.command_queue, common::COLUMNS_PER_SEGMENT);
	}
	*/

}




/*
		for i in range(0u32, layers_total) {



			if i < 4 {
				
				
			} else if i < 12 { 
				
				
				assert!(i > 0);
				syn_offset = i * syn_per_layer;
				src_offset_start = ((i - 1) * syn_per_layer) - 128;
				gid_offset_factor = 1u32;
				boost_factor = 16;

				//print!("[syn_offset: {}, src_offset_start: {}, gid_offset_factor: {}]", syn_offset, src_offset_start, gid_offset_factor);

				assert!(src_offset_start >= 128u32 && (src_offset_start as usize) < (self.cells.synapses.values.vec.len() - 128));

				ocl::set_kernel_arg(0, self.cells.synapses.values.buf, kern);
				ocl::set_kernel_arg(4, src_offset_start, kern);
				ocl::set_kernel_arg(5, syn_offset, kern);
				ocl::set_kernel_arg(6, gid_offset_factor, kern);
				ocl::set_kernel_arg(7, boost_factor, kern);
				

			} 
			 else if i < layers_total { 
				
				assert!(i > 0);
				syn_offset = i * syn_per_layer;
				src_offset_start = ((i - 1) * syn_per_layer) - 128;
				gid_offset_factor = 1u32;
				boost_factor = 64;

				//print!("[syn_offset: {}, src_offset_start: {}, gid_offset_factor: {}]", syn_offset, src_offset_start, gid_offset_factor);

				assert!(src_offset_start >= 128u32 && (src_offset_start as usize) < (self.cells.synapses.values.vec.len() - 128));

				ocl::set_kernel_arg(0, self.cells.synapses.values.buf, kern);
				ocl::set_kernel_arg(4, src_offset_start, kern);
				ocl::set_kernel_arg(5, syn_offset, kern);
				ocl::set_kernel_arg(6, gid_offset_factor, kern);
				ocl::set_kernel_arg(7, boost_factor, kern);
				
			} 
			
			

			ocl::enqueue_kernel(kern, self.ocl.command_queue, common::SYNAPSES_PER_LAYER);
		}
		*/
