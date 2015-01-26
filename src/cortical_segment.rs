use ocl;
use common;
use cortex::{ HyperColumns, Columns, Cells };
use envoy::{ Envoy };
use sensory_segment::{ SensorySegment };
use chord::{ Chord };
use column;
use cell;

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

	}
}
