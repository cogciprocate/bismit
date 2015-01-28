use ocl;
use common;
use envoy::{ Envoy };
use sensory_segment::{ SensorySegment };
use cortical_segment::{ CorticalSegment };
use chord::{ Chord };
use column;
use cell;

use std::rand;
use std::rand::distributions::{IndependentSample, Range};
use std::ptr;
use time;

pub struct HyperColumns {
	pub qty: usize,
	pub states: Envoy<ocl::cl_uint>,
}
impl HyperColumns {
	pub fn new(qty: usize, ocl: &ocl::Ocl) -> HyperColumns {
		HyperColumns {
			qty: common::HYPERCOLUMNS_PER_SEGMENT,
			states: Envoy::<ocl::cl_uint>::new(common::HYPERCOLUMNS_PER_SEGMENT, 0u32, ocl),
		}
	}
}


pub struct MotorSegment {
	pub targets: Envoy<ocl::cl_ushort>,
	pub values: Envoy<ocl::cl_uchar>,
}
impl MotorSegment {
	pub fn new(width: usize, ocl: &ocl::Ocl) -> MotorSegment {
		MotorSegment { 
			targets : Envoy::<ocl::cl_ushort>::new(width, 0u16, ocl),
			values : Envoy::<ocl::cl_uchar>::new(width, 0u8, ocl),
		}
	}
}


pub struct Cortex {
	pub ocl: ocl::Ocl,
	pub cortical_segments: Vec<CorticalSegment>,
	pub sensory_segments: Vec<SensorySegment>,
	pub motor_segments: Vec<MotorSegment>,
	
}

impl Cortex {
	pub fn new() -> Cortex {
		println!("Initializing Cortex...");
		let time_start = time::get_time().sec;

		let ocl: ocl::Ocl = ocl::Ocl::new();

		let mut cs = Vec::with_capacity(common::CORTICAL_SEGMENTS_TOTAL);
		for i in range(0, common::CORTICAL_SEGMENTS_TOTAL) {
			cs.push(CorticalSegment::new(common::HYPERCOLUMNS_PER_SEGMENT, &ocl));
		}

		assert!(common::SENSORY_SEGMENTS_TOTAL <= common::CORTICAL_SEGMENTS_TOTAL);
		let mut ss = Vec::with_capacity(common::SENSORY_SEGMENTS_TOTAL);
		for i in range(0, common::SENSORY_SEGMENTS_TOTAL) {
			ss.push(SensorySegment::new(common::SENSORY_CHORD_WIDTH, &cs[i], i, &ocl));
		}

		let mut ms = Vec::with_capacity(common::MOTOR_SEGMENTS_TOTAL);
		for i in range(0, common::MOTOR_SEGMENTS_TOTAL) {
			ms.push(MotorSegment::new(common::MOTOR_CHORD_WIDTH, &ocl));
		}
 
		let time_finish = time::get_time().sec;

		println!("Cortex initialized in: {} sec.", time_finish - time_start);

		Cortex {
			sensory_segments: ss,
			cortical_segments: cs,
			motor_segments: ms,
			ocl: ocl,
		}
	}

	pub fn sense(&mut self, sgmt_idx: usize, chord: &Chord) {

		self.sensory_segments[sgmt_idx].sense(chord, &mut self.cortical_segments[sgmt_idx]);
		self.cortical_segments[self.sensory_segments[sgmt_idx].target_segment_idx].cycle();

	}

	pub fn release_components(&mut self) {
		println!("Releasing OCL Components...");
		self.ocl.release_components();
	}
}


