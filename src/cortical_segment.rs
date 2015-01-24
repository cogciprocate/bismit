use ocl;
use common;
use cortex::{ HyperColumns, Columns, Cells };
use cortical_component::{ CorticalComponent };
use sensory_segment::{ SensorySegment };
use chord::{ Chord };
use column;
use cell;
//use std;
//use std::io;
use std::rand;
use std::rand::distributions::{IndependentSample, Range};
use std::ptr;

use time;

pub struct CorticalSegment {
	pub hypercolumns: HyperColumns,
	pub columns: Columns,
	pub cells: Cells,
	pub ocl: ocl::Ocl,
}

impl  CorticalSegment {
	pub fn new(hcols: usize, ocl: &ocl::Ocl) -> CorticalSegment {

		/*

		
		let dup_factor: ocl::cl_uint = num::FromPrimitive::from_uint(target_segment.columns.synapses.values.len() / source_values.len()).unwrap();
		let dup_factor_shift: ocl::cl_uchar = common::int_log2(dup_factor);

		let tmp_out = CorticalComponent::new(common::COLUMN_SYNAPSES_PER_SEGMENT, 0u32, ocl);

		let kern = ocl::new_kernel(ocl.program, "sense");

		ocl::set_kernel_arg(0, source_values.buf, kern);
		ocl::set_kernel_arg(1, target_segment.columns.synapses.values.buf, kern);
		ocl::set_kernel_arg(2, target_addresses.target_column_bodies.buf, kern);
		ocl::set_kernel_arg(3, target_addresses.target_column_synapses.buf, kern);
		ocl::set_kernel_arg(4, dup_factor_shift, kern);
		ocl::set_kernel_arg(5, tmp_out.buf, kern);

		*/


		CorticalSegment {
			hypercolumns: HyperColumns::new(hcols, ocl),
			columns: Columns::new(hcols, ocl),
			cells: Cells::new(hcols, ocl),
			ocl: ocl.clone(),
		}
	}

	pub fn cycle(&self) {

		let kern = ocl::new_kernel(self.ocl.program, "cycle_col_dens");

		ocl::set_kernel_arg(0, self.columns.synapses.values.buf, kern);
		ocl::set_kernel_arg(1, self.columns.synapses.strengths.buf, kern);
		ocl::set_kernel_arg(2, self.columns.dendrites.thresholds.buf, kern);
		ocl::set_kernel_arg(3, self.columns.dendrites.values.buf, kern);

		ocl::enqueue_kernel(kern, self.ocl.command_queue, common::COLUMN_DENDRITES_PER_SEGMENT);

		//kern = ocl::new_kernel(self.ocl.program, "cycle_col_cells");

	}
}
