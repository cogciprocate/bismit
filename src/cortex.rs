use ocl;
use common;
use cortical_component::{ CorticalComponent };
use neurons_column;
use neurons_cell;
//use std;
//use std::io;
use std::rand;
use std::rand::distributions::{IndependentSample, Range};
use std::ptr;

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

pub struct Cortex {
	pub ocl: ocl::Ocl,
	pub cortex_segments: Vec<CortexSegment>,
	pub sensory_segments: Vec<(CorticalComponent<ocl::cl_ushort>, CorticalComponent<ocl::cl_uchar>)>,
	// ADD ME:  pub motor_segments: Vec<MotorSegment>,

	
}

impl Cortex {
	pub fn new() -> Cortex {
		println!("Initializing Cortex...");

		let ocl = ocl::Ocl::new();

		let mut cs = Vec::with_capacity(common::CORTICAL_SEGMENTS_TOTAL);
		for i in range(0u, common::CORTICAL_SEGMENTS_TOTAL) {
			cs.push(CortexSegment::new(common::HYPERCOLUMNS_PER_SEGMENT, &ocl));
		}

		let mut ss = Vec::with_capacity(common::SENSORY_SEGMENTS_TOTAL);
		for i in range(0u, common::SENSORY_SEGMENTS_TOTAL) {
			let col = CorticalComponent::<ocl::cl_ushort>::new(common::SENSORY_CHORD_WIDTH, 0u16, &ocl);
			let syn = CorticalComponent::<ocl::cl_uchar>::new(common::SENSORY_CHORD_WIDTH, 0u8, &ocl);
			ss.push((col, syn));
		}

		Cortex {
			sensory_segments: ss,
			cortex_segments: cs,
			ocl: ocl,
		}
	}

	pub fn release_components(&mut self) {
		println!("Releasing OCL Components...")
		self.ocl.release_components();
	}
}


