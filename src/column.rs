use common;
use ocl;
use envoy::{ Envoy };

use std::num;
use std::rand;
use std::rand::distributions::{ Normal, IndependentSample, Range };
use std::num::{ NumCast, UnsignedInt, Int };
use std::fmt::{ Show, String };
use std::default::{ Default };

pub struct Bodies {
	pub states: Envoy<ocl::cl_uchar>,
}

impl Bodies {
	pub fn new(size: usize, ocl: &ocl::Ocl) -> Bodies {
		Bodies { states: Envoy::<ocl::cl_uchar>::new(size, 0u8, ocl), }
	}
}


pub struct Axons {
	pub target_column_bodies: Envoy<ocl::cl_ushort>,
	pub target_column_synapses: Envoy<ocl::cl_uchar>,
}

impl Axons {
	pub fn new(size: usize, ocl: &ocl::Ocl) -> Axons {

		let mut target_column_bodies = Envoy::<ocl::cl_ushort>::new(size, 0u16, ocl);
		let mut target_column_synapses = Envoy::<ocl::cl_uchar>::new(size, 0u8, ocl);

		Axons::init(&mut target_column_bodies, &mut target_column_synapses, ocl);

		Axons {
			target_column_bodies: target_column_bodies,
			target_column_synapses: target_column_synapses,
		}
	}

	pub fn init(
				target_column_bodies: &mut Envoy<ocl::cl_ushort>, 
				target_column_synapses: &mut Envoy<ocl::cl_uchar>,
				//	&mut self,
				ocl: &ocl::Ocl,

	) {

		assert!(target_column_bodies.len() == target_column_synapses.len(), "Arrays must be of equal length.");

		let len = target_column_bodies.len();

		let source_vec = common::shuffled_vec(len, 0u32);

		for i in range(0, len) {
			let bod_addr = source_vec[i] >> 8;
			let syn_addr = source_vec[i] - (bod_addr << 8);

			target_column_bodies.vec[i] = num::cast(bod_addr).unwrap();
			target_column_synapses.vec[i] = num::cast(syn_addr).unwrap();

		}
		
		target_column_bodies.write();
		target_column_synapses.write();

	}
}



pub struct Dendrites {
	pub values: Envoy<ocl::cl_uchar>,
	pub thresholds: Envoy<ocl::cl_uchar>,
}
impl Dendrites {
	pub fn new(size: usize, ocl: &ocl::Ocl) -> Dendrites {
		Dendrites {
			values: Envoy::<ocl::cl_uchar>::new(size, 0u8, ocl),
			thresholds: Envoy::<ocl::cl_uchar>::new(size, common::DENDRITE_INITIAL_THRESHOLD, ocl),
		}
	}
}


pub struct Synapses {
	pub values: Envoy<ocl::cl_uchar>,
	pub strengths: Envoy<ocl::cl_uchar>,
}
impl Synapses {
	pub fn new(size: usize, ocl: &ocl::Ocl) -> Synapses {
		Synapses {
			values: Envoy::<ocl::cl_uchar>::new(size, 0u8, ocl),
			strengths: Envoy::<ocl::cl_uchar>::new(size, common::SYNAPSE_WEIGHT_ZERO, ocl),
		}
	}

}
