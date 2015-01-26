use common;
use ocl;
use envoy::{ Envoy };

use std::num;
use std::rand;
use std::rand::distributions::{ Normal, IndependentSample, Range };
use std::num::{ NumCast, Int, FromPrimitive };
use std::default::{ Default };
use std::fmt::{ Display };

pub struct Axons {
	pub target_cells: Envoy<ocl::cl_uchar>,
	pub target_cell_synapses: Envoy<ocl::cl_uchar>,
}
impl Axons {
	pub fn new(size: usize, ocl: &ocl::Ocl) -> Axons {
		let mut target_cells = Envoy::<ocl::cl_uchar>::new(size, 0u8, ocl);
		let mut target_cell_synapses = Envoy::<ocl::cl_uchar>::new(size, 0u8, ocl);

		init_axon(&mut target_cells, &mut target_cell_synapses);

		Axons {
			target_cells: target_cells,
			target_cell_synapses: target_cell_synapses,
		}
	}
}


pub fn init_axon<T: Clone + NumCast + Int + Default + Display + FromPrimitive>(target_cells: &mut Envoy<T>, target_cell_synapses: &mut Envoy<T>) {
	let mut rng = rand::thread_rng();

	let normal = Normal::new(128f64, 128f64);
	
	for i in range(0, target_cells.vec.len()) {
		let val = normal.ind_sample(&mut rng);
		let cell = num::cast(val).unwrap();
		target_cells.vec[i] = cell;
		
	}

	let rng_range = Range::new(0u8, 255u8);

	for i in range(0, target_cell_synapses.vec.len()) {
		target_cell_synapses.vec[i] = num::cast(rng_range.ind_sample(&mut rng)).unwrap();
	}
	
	target_cells.write();
	target_cell_synapses.write();

}


pub struct Dendrites {
	pub thresholds: Envoy<ocl::cl_uchar>,
	pub synapse_states: Envoy<ocl::cl_ushort>,
}
impl Dendrites {
	pub fn new(size: usize, ocl: &ocl::Ocl) -> Dendrites {
		Dendrites {
			thresholds: Envoy::<ocl::cl_uchar>::new(size, 16u8, ocl),
			synapse_states: Envoy::<ocl::cl_ushort>::new(size, 0u16, ocl),
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
			strengths: Envoy::<ocl::cl_uchar>::new(size, common::DENDRITE_INITIAL_THRESHOLD, ocl),
		}
	}
}
