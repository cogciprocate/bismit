use common;
use ocl;
use envoy::{ Envoy };

use std::num;
use std::rand;
use std::rand::distributions::{ Normal, IndependentSample, Range };
use std::num::{ NumCast, Int, FromPrimitive };
use std::default::{ Default };
use std::fmt::{ Display };


pub struct Cells {
	//pub states: Envoy<ocl::cl_uint>,
	pub axons: Axons,
	pub somata: Somata,
	pub dendrites: Dendrites,
	pub synapses: Synapses,
}
impl Cells {
	pub fn new(hcols: usize, ocl: &ocl::Ocl) -> Cells {
		Cells {
			//states: Envoy::<ocl::cl_uint>::new(common::CELLS_PER_SEGMENT, 0u32, ocl),
			axons:	Axons::new(common::CELL_AXONS_PER_SEGMENT, ocl),
			somata: Somata::new(common::CELLS_PER_SEGMENT, ocl),
			dendrites: Dendrites::new(common::CELL_DENDRITES_PER_SEGMENT, ocl),
			synapses: Synapses::new(common::CELL_SYNAPSES_PER_SEGMENT, ocl),
		}
	}
}


pub struct Axons {
	pub target_cell_somata: Envoy<ocl::cl_ushort>,
	pub target_cell_synapses: Envoy<ocl::cl_uchar>,
}
impl Axons {
	pub fn new(size: usize, ocl: &ocl::Ocl) -> Axons {
		let mut target_cell_somata = Envoy::<ocl::cl_ushort>::new(size, 0u16, ocl);
		let mut target_cell_synapses = Envoy::<ocl::cl_uchar>::new(size, 0u8, ocl);

		//init_axon(&mut target_cell_somata, &mut target_cell_synapses);

		Axons::init(&mut target_cell_somata, &mut target_cell_synapses, ocl);

		Axons {
			target_cell_somata: target_cell_somata,
			target_cell_synapses: target_cell_synapses,
		}
	}

		// ************ REWRITE SHUFFLING FUNCTION TO DISTRIBUTE USING A NORMAL DISTRIBUTION

	pub fn init(
				target_cell_somata: &mut Envoy<ocl::cl_ushort>, 
				target_cell_synapses: &mut Envoy<ocl::cl_uchar>,
				ocl: &ocl::Ocl,
	) {

		//println!("cell::Axons init with {} len.", common::CELL_AXONS_PER_SEGMENT);
		assert!(target_cell_somata.len() == target_cell_synapses.len(), "Arrays must be of equal length.");

		let len = target_cell_somata.len();

				// ************ REWRITE SHUFFLING FUNCTION TO DISTRIBUTE USING A NORMAL DISTRIBUTION
		let source_vec = common::shuffled_vec(len, 0u32);

		for i in range(0, len) {
			let som_addr = source_vec[i] >> 8;
			let syn_addr = source_vec[i] - (som_addr << 8);

			target_cell_somata.vec[i] = num::cast(som_addr).unwrap();
			target_cell_synapses.vec[i] = num::cast(syn_addr).unwrap();

		}

		target_cell_somata.write();
		target_cell_synapses.write();


	}
}


pub struct Somata {	
	pub states: Envoy<ocl::cl_uchar>,
}

impl Somata {
	pub fn new(size: usize, ocl: &ocl::Ocl) -> Somata {
		Somata { states: Envoy::<ocl::cl_uchar>::new(size, 0u8, ocl), }
	}
}


pub struct Dendrites {
	pub thresholds: Envoy<ocl::cl_uchar>,
	pub values: Envoy<ocl::cl_uchar>,
}
impl Dendrites {
	pub fn new(size: usize, ocl: &ocl::Ocl) -> Dendrites {
		Dendrites {
			thresholds: Envoy::<ocl::cl_uchar>::new(size, common::DENDRITE_INITIAL_THRESHOLD, ocl),
			values: Envoy::<ocl::cl_uchar>::new(size, 0u8, ocl),
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


/*

pub fn init_axon<T: Clone + NumCast + Int + Default + Display + FromPrimitive>(target_cell_somata: &mut Envoy<T>, target_cell_synapses: &mut Envoy<T>) {
	let mut rng = rand::thread_rng();

	let normal = Normal::new(128f64, 128f64);
	
	for i in range(0, target_cell_somata.vec.len()) {
		let val = normal.ind_sample(&mut rng);
		let cell = num::cast(val).unwrap();
		target_cell_somata.vec[i] = cell;
		
	}

	let rng_range = Range::new(0u8, 255u8);

	for i in range(0, target_cell_synapses.vec.len()) {
		target_cell_synapses.vec[i] = num::cast(rng_range.ind_sample(&mut rng)).unwrap();
	}
	
	target_cell_somata.write();
	target_cell_synapses.write();

}

*/
