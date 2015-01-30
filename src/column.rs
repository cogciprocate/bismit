use common;
use ocl;
use envoy::{ Envoy };

use std::num;
use std::rand;
use std::rand::distributions::{ Normal, IndependentSample, Range };
use std::num::{ NumCast, UnsignedInt, Int };
use std::fmt::{ Show, String };
use std::default::{ Default };


pub struct Columns {
	//pub states: Envoy<ocl::cl_int>,
	pub axons: Axons,
	pub somata: Somata,
	pub dendrites: Dendrites,
	pub synapses: Synapses,
}
impl Columns {
	pub fn new(hcols: usize, ocl: &ocl::Ocl) -> Columns {
		Columns {
			//states: Envoy::<ocl::cl_int>::new(common::COLUMNS_PER_SEGMENT, 0u32, ocl),
			axons:	Axons::new(common::COLUMN_AXONS_PER_SEGMENT, ocl),
			somata: Somata::new(common::COLUMNS_PER_SEGMENT, ocl),
			dendrites: Dendrites::new(common::COLUMN_DENDRITES_PER_SEGMENT, ocl),
			synapses: Synapses::new(common::COLUMN_SYNAPSES_PER_SEGMENT, ocl),
		}
	}
}

pub struct Axons {
	pub target_column_somata: Envoy<ocl::cl_short>,
	pub target_column_synapses: Envoy<ocl::cl_char>,
}

impl Axons {
	pub fn new(size: usize, ocl: &ocl::Ocl) -> Axons {

		let target_column_somata = Envoy::<ocl::cl_short>::new(size, 0i16, ocl);
		let target_column_synapses = Envoy::<ocl::cl_char>::new(size, 0i8, ocl);

		//Axons::init(&mut target_column_somata, &mut target_column_synapses, ocl);

		Axons {
			target_column_somata: target_column_somata,
			target_column_synapses: target_column_synapses,
		}
	}

	pub fn init(
				target_column_somata: &mut Envoy<ocl::cl_short>, 
				target_column_synapses: &mut Envoy<ocl::cl_char>,
				//	&mut self,
				ocl: &ocl::Ocl,

	) {

		assert!(target_column_somata.len() == target_column_synapses.len(), "Arrays must be of equal length.");

		let len = target_column_somata.len();

		let source_vec = common::shuffled_vec(len, 0u32);

		for i in range(0, len) {
			let bod_addr = source_vec[i] >> 8;
			let syn_addr = source_vec[i] - (bod_addr << 8);

			target_column_somata.vec[i] = num::cast(bod_addr).expect("column::Axons::init(), target_cell_somata");
			target_column_synapses.vec[i] = num::cast(syn_addr).expect("column::Axons::init(), target_cell_synapses");

		}
		
		target_column_somata.write();
		target_column_synapses.write();

	}
}


pub struct Somata {	
	pub states: Envoy<ocl::cl_char>,
}

impl Somata {
	pub fn new(size: usize, ocl: &ocl::Ocl) -> Somata {
		Somata { states: Envoy::<ocl::cl_char>::new(size, 0i8, ocl), }
	}
}


pub struct Dendrites {
	pub values: Envoy<ocl::cl_char>,
	pub thresholds: Envoy<ocl::cl_char>,
}
impl Dendrites {
	pub fn new(size: usize, ocl: &ocl::Ocl) -> Dendrites {
		Dendrites {
			values: Envoy::<ocl::cl_char>::new(size, 0i8, ocl),
			thresholds: Envoy::<ocl::cl_char>::new(size, common::DENDRITE_INITIAL_THRESHOLD, ocl),
		}
	}
}


pub struct Synapses {
	pub values: Envoy<ocl::cl_char>,
	pub strengths: Envoy<ocl::cl_char>,
}
impl Synapses {
	pub fn new(size: usize, ocl: &ocl::Ocl) -> Synapses {
		Synapses {
			values: Envoy::<ocl::cl_char>::new(size, 0i8, ocl),
			strengths: Envoy::<ocl::cl_char>::new(size, common::SYNAPSE_STRENGTH_ZERO, ocl),
		}
	}

}
