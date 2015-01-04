use ocl;
use common;
use cortical_component::{ CorticalComponent };
use chord::{ Chord };
use neurons_column;
use neurons_cell;
//use std;
//use std::io;
use std::rand;
use std::rand::distributions::{IndependentSample, Range};
use std::ptr;

use time;

pub struct Columns {
	pub states: CorticalComponent<ocl::cl_uint>,
	pub axons: neurons_column::Axons,
	pub dendrites: neurons_column::Dendrites,
	pub synapses: neurons_column::Synapses,
}
impl Columns {
	pub fn new(hcols: uint, ocl: &ocl::Ocl) -> Columns {
		Columns {
			states: CorticalComponent::<ocl::cl_uint>::new(common::COLUMNS_PER_SEGMENT, 0u32, ocl),
			axons:	neurons_column::Axons::new(common::COLUMN_AXONS_PER_SEGMENT, ocl),
			dendrites: neurons_column::Dendrites::new(common::COLUMN_DENDRITES_PER_SEGMENT, ocl),
			synapses: neurons_column::Synapses::new(common::COLUMN_SYNAPSES_PER_SEGMENT, ocl),
		}
	}
}


pub struct Cells {
	pub states: CorticalComponent<ocl::cl_uint>,
	pub axons: neurons_cell::Axons,
	pub dendrites: neurons_cell::Dendrites,
	pub synapses: neurons_cell::Synapses,
}
impl Cells {
	pub fn new(hcols: uint, ocl: &ocl::Ocl) -> Cells {
		Cells {
			states: CorticalComponent::<ocl::cl_uint>::new(common::CELLS_PER_SEGMENT, 0u32, ocl),
			axons:	neurons_cell::Axons::new(common::CELL_AXONS_PER_SEGMENT, ocl),
			dendrites: neurons_cell::Dendrites::new(common::CELL_DENDRITES_PER_SEGMENT, ocl),
			synapses: neurons_cell::Synapses::new(common::CELL_SYNAPSES_PER_SEGMENT, ocl),
		}
	}
}

pub struct HyperColumns {
	pub qty: uint,
	pub states: CorticalComponent<ocl::cl_uint>,
}
impl HyperColumns {
	pub fn new(qty: uint, ocl: &ocl::Ocl) -> HyperColumns {
		HyperColumns {
			qty: common::HYPERCOLUMNS_PER_SEGMENT,
			states: CorticalComponent::<ocl::cl_uint>::new(common::HYPERCOLUMNS_PER_SEGMENT, 0u32, ocl),
		}
	}
}


pub struct CortexSegment {
	pub hypercolumns: HyperColumns,
	pub columns: Columns,
	pub cells: Cells,
}
impl CortexSegment {
	pub fn new(hcols: uint, ocl: &ocl::Ocl) -> CortexSegment {
		CortexSegment {
			hypercolumns: HyperColumns::new(hcols, ocl),
			columns: Columns::new(hcols, ocl),
			cells: Cells::new(hcols, ocl),
		}
	}
}

pub struct SensorySegment {
	pub targets: CorticalComponent<ocl::cl_ushort>,
	pub values: CorticalComponent<ocl::cl_uchar>,
	pub context: ocl::cl_context,
	pub command_queue: ocl::cl_command_queue,
	pub kernel: ocl::cl_kernel,
}
impl SensorySegment {
	pub fn new(width: uint, ocl: &ocl::Ocl) -> SensorySegment {

		let sense_kernel_name = "sense";

		let targets = CorticalComponent::<ocl::cl_ushort>::new(width, 0u16, ocl);
		let values = CorticalComponent::<ocl::cl_uchar>::new(width, 0u8, ocl);
		let kernel = ocl::new_kernel(ocl.program, sense_kernel_name);

		ocl::set_kernel_arg(0, values.buff, kernel);

		SensorySegment { 
			targets : targets,
			values : values,
			context: ocl.context,
			command_queue: ocl.command_queue,
			kernel: kernel,
		}
	}

	pub fn sense(&mut self, chord: &Chord) {

		chord.unfold_into(&mut self.values.vec);
		self.values.write();


		ocl::enqueue_kernel(self.kernel, self.command_queue, self.values.vec.len());
	}
}

pub struct MotorSegment {
	pub targets: CorticalComponent<ocl::cl_ushort>,
	pub values: CorticalComponent<ocl::cl_uchar>,
}
impl MotorSegment {
	pub fn new(width: uint, ocl: &ocl::Ocl) -> MotorSegment {
		MotorSegment { 
			targets : CorticalComponent::<ocl::cl_ushort>::new(width, 0u16, ocl),
			values : CorticalComponent::<ocl::cl_uchar>::new(width, 0u8, ocl),
		}
	}
}


pub struct Cortex {
	pub ocl: ocl::Ocl,
	pub cortex_segments: Vec<CortexSegment>,
	pub sensory_segments: Vec<SensorySegment>,
	pub motor_segments: Vec<MotorSegment>,
	
}

impl Cortex {
	pub fn new() -> Cortex {
		println!("Initializing Cortex...");
		let time_start = time::get_time().sec;

		//println!("Timer started...");

		let ocl = ocl::Ocl::new();

		let mut cs = Vec::with_capacity(common::CORTICAL_SEGMENTS_TOTAL);
		for i in range(0u, common::CORTICAL_SEGMENTS_TOTAL) {
			cs.push(CortexSegment::new(common::HYPERCOLUMNS_PER_SEGMENT, &ocl));
		}

		let mut ss = Vec::with_capacity(common::SENSORY_SEGMENTS_TOTAL);
		for i in range(0u, common::SENSORY_SEGMENTS_TOTAL) {
			ss.push(SensorySegment::new(common::SENSORY_CHORD_WIDTH, &ocl));
		}

		let mut ms = Vec::with_capacity(common::MOTOR_SEGMENTS_TOTAL);
		for i in range(0u, common::MOTOR_SEGMENTS_TOTAL) {
			ms.push(MotorSegment::new(common::MOTOR_CHORD_WIDTH, &ocl));
		}
 
		let time_finish = time::get_time().sec;

		println!("Cortex initialized in: {} sec.", time_finish - time_start);

		Cortex {
			sensory_segments: ss,
			cortex_segments: cs,
			motor_segments: ms,
			ocl: ocl,
		}
	}

	pub fn sense(&mut self, sgmt_idx: uint, chord: &Chord) {

		self.sensory_segments[sgmt_idx].sense(chord);

	}

	pub fn release_components(&mut self) {
		println!("Releasing OCL Components...");
		self.ocl.release_components();
	}
}


