use ocl;
use common;
use cortical_component::{ CorticalComponent };
use sensory_segment::{ SensorySegment };
use chord::{ Chord };
use column_neurons;
use cell_neurons;
//use std;
//use std::io;
use std::rand;
use std::rand::distributions::{IndependentSample, Range};
use std::ptr;

use std::time;

pub struct Columns {
	pub states: CorticalComponent<ocl::cl_uint>,
	pub axons: column_neurons::Axons,
	pub dendrites: column_neurons::Dendrites,
	pub synapses: column_neurons::Synapses,
}
impl Columns {
	pub fn new(hcols: usize, ocl: &ocl::Ocl) -> Columns {
		Columns {
			states: CorticalComponent::<ocl::cl_uint>::new(common::COLUMNS_PER_SEGMENT, 0u32, ocl),
			axons:	column_neurons::Axons::new(common::COLUMN_AXONS_PER_SEGMENT, ocl),
			dendrites: column_neurons::Dendrites::new(common::COLUMN_DENDRITES_PER_SEGMENT, ocl),
			synapses: column_neurons::Synapses::new(common::COLUMN_SYNAPSES_PER_SEGMENT, ocl),
		}
	}
}


pub struct Cells {
	pub states: CorticalComponent<ocl::cl_uint>,
	pub axons: cell_neurons::Axons,
	pub dendrites: cell_neurons::Dendrites,
	pub synapses: cell_neurons::Synapses,
}
impl Cells {
	pub fn new(hcols: usize, ocl: &ocl::Ocl) -> Cells {
		Cells {
			states: CorticalComponent::<ocl::cl_uint>::new(common::CELLS_PER_SEGMENT, 0u32, ocl),
			axons:	cell_neurons::Axons::new(common::CELL_AXONS_PER_SEGMENT, ocl),
			dendrites: cell_neurons::Dendrites::new(common::CELL_DENDRITES_PER_SEGMENT, ocl),
			synapses: cell_neurons::Synapses::new(common::CELL_SYNAPSES_PER_SEGMENT, ocl),
		}
	}
}

pub struct HyperColumns {
	pub qty: usize,
	pub states: CorticalComponent<ocl::cl_uint>,
}
impl HyperColumns {
	pub fn new(qty: usize, ocl: &ocl::Ocl) -> HyperColumns {
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
	pub fn new(hcols: usize, ocl: &ocl::Ocl) -> CortexSegment {
		CortexSegment {
			hypercolumns: HyperColumns::new(hcols, ocl),
			columns: Columns::new(hcols, ocl),
			cells: Cells::new(hcols, ocl),
		}
	}
}

pub struct MotorSegment {
	pub targets: CorticalComponent<ocl::cl_ushort>,
	pub values: CorticalComponent<ocl::cl_uchar>,
}
impl MotorSegment {
	pub fn new(width: usize, ocl: &ocl::Ocl) -> MotorSegment {
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
		let time_start = 0u32;			// time::get_time().sec;

		//println!("Timer started...");

		let ocl = ocl::Ocl::new();

		let mut cs = Vec::with_capacity(common::CORTICAL_SEGMENTS_TOTAL);
		for i in range(0, common::CORTICAL_SEGMENTS_TOTAL) {
			cs.push(CortexSegment::new(common::HYPERCOLUMNS_PER_SEGMENT, &ocl));
		}

		assert!(common::SENSORY_SEGMENTS_TOTAL <= common::CORTICAL_SEGMENTS_TOTAL);
		let mut ss = Vec::with_capacity(common::SENSORY_SEGMENTS_TOTAL);
		for i in range(0, common::SENSORY_SEGMENTS_TOTAL) {
			ss.push(SensorySegment::new(common::SENSORY_CHORD_WIDTH, &cs[i].columns.synapses, &ocl));
		}

		let mut ms = Vec::with_capacity(common::MOTOR_SEGMENTS_TOTAL);
		for i in range(0, common::MOTOR_SEGMENTS_TOTAL) {
			ms.push(MotorSegment::new(common::MOTOR_CHORD_WIDTH, &ocl));
		}
 
		let time_finish = 0u32;		// time::get_time().sec;

		println!("Cortex initialized in: {} sec.", time_finish - time_start);

		Cortex {
			sensory_segments: ss,
			cortex_segments: cs,
			motor_segments: ms,
			ocl: ocl,
		}
	}

	pub fn sense(&mut self, sgmt_idx: usize, chord: &Chord) {

		self.sensory_segments[sgmt_idx].sense(chord);

	}

	pub fn release_components(&mut self) {
		println!("Releasing OCL Components...");
		self.ocl.release_components();
	}
}


