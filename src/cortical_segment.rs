use ocl;
use common;
use cortex::{ HyperColumns };
use envoy::{ Envoy };
use sensory_segment::{ SensorySegment };
use chord::{ Chord };
use column;
use column::{ Columns };
use cell::{ Cells };

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
		self.cycle_col_dens();
		self.cycle_col_soms();
		self.cycle_cel_syns();
		self.cycle_cel_dens();
		self.cycle_cel_soms();

	}

	fn cycle_col_dens(&self) {
		let kern = ocl::new_kernel(self.ocl.program, "cycle_dens");

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

	fn cycle_cel_syns(&self) {
		let kern = ocl::new_kernel(self.ocl.program, "cycle_cel_syns");

		let dup_factor = self.cells.synapses.values.vec.len() / self.cells.somata.states.vec.len();
		let dup_factor_shift: ocl::cl_uchar = common::int_hb_log2(dup_factor);		// (8)

		/*
		println!("ocl::set_kernel_arg(0, self.cells.somata.states.buf, kern): {}", self.cells.somata.states.vec.len()) ;
		println!("ocl::set_kernel_arg(1, self.cells.synapses.values.buf, kern): {}", self.cells.synapses.values.vec.len());
		println!("ocl::set_kernel_arg(2, self.cells.axons.target_cell_somata.buf, kern): {}", self.cells.axons.target_cell_somata.vec.len());
		println!("ocl::set_kernel_arg(3, self.cells.axons.target_cell_synapses.buf, kern): {}", self.cells.axons.target_cell_synapses.vec.len());
		println!("ocl::set_kernel_arg(4, dup_factor_shift, kern): {}", dup_factor_shift) ;
		*/

		ocl::set_kernel_arg(0, self.cells.somata.states.buf, kern);
		ocl::set_kernel_arg(1, self.cells.synapses.values.buf, kern);
		ocl::set_kernel_arg(2, self.cells.axons.target_cell_somata.buf, kern);
		ocl::set_kernel_arg(3, self.cells.axons.target_cell_synapses.buf, kern);
		ocl::set_kernel_arg(4, dup_factor_shift, kern);

		ocl::enqueue_kernel(kern, self.ocl.command_queue, common::CELL_SYNAPSES_PER_SEGMENT);
	}

	fn cycle_cel_dens(&self) {
		let kern = ocl::new_kernel(self.ocl.program, "cycle_dens");

		ocl::set_kernel_arg(0, self.cells.synapses.values.buf, kern);
		ocl::set_kernel_arg(1, self.cells.synapses.strengths.buf, kern);
		ocl::set_kernel_arg(2, self.cells.dendrites.thresholds.buf, kern);
		ocl::set_kernel_arg(3, self.cells.dendrites.values.buf, kern);

		ocl::enqueue_kernel(kern, self.ocl.command_queue, common::CELL_DENDRITES_PER_SEGMENT);

	}

	fn cycle_cel_soms(&self) {
		let kern = ocl::new_kernel(self.ocl.program, "cycle_cel_soms");

		ocl::set_kernel_arg(0, self.cells.dendrites.values.buf, kern);
		ocl::set_kernel_arg(1, self.cells.somata.states.buf, kern);

		ocl::enqueue_kernel(kern, self.ocl.command_queue, common::COLUMNS_PER_SEGMENT);

	}
}
