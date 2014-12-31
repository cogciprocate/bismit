use common;
use ocl;
use cortical_component::{ CorticalComponent };

use std::num;
use std::rand;
use std::rand::distributions::{ Normal, IndependentSample, Range };
use std::num::NumCast;

pub struct Axons {
	pub target_cells: CorticalComponent<ocl::cl_uchar>,
	pub target_cell_synapses: CorticalComponent<ocl::cl_uchar>,
}
impl Axons {
	pub fn new(size: uint, ocl: &ocl::Ocl) -> Axons {
		let mut target_cells = CorticalComponent::<ocl::cl_uchar>::new(size, 0u8, ocl);
		let mut target_cell_synapses = CorticalComponent::<ocl::cl_uchar>::new(size, 0u8, ocl);

		let mut rng = rand::task_rng();

		let normal = Normal::new(128f64, 128f64);
		
		for i in range(0u, target_cells.vec.len()) {
			let val = normal.ind_sample(&mut rng) as u8;
			let cell = num::cast(val).unwrap();
			target_cells.vec[i] = cell;
			
		}

		let rng_range = Range::new(0u8, 255u8);

		for i in range(0u, target_cell_synapses.vec.len()) {
			//target_cell_synapses.vec[i] = rng_range.ind_sample(&mut rng);
			target_cell_synapses.vec[i] = 255u8;
		}
		
		target_cells.write();
		target_cell_synapses.write();

		Axons {
			target_cells: target_cells,
			target_cell_synapses: target_cell_synapses,
		}
	}
}


pub struct Dendrites {
	pub thresholds: CorticalComponent<ocl::cl_uchar>,
	pub synapse_states: CorticalComponent<ocl::cl_ushort>,
}
impl Dendrites {
	pub fn new(size: uint, ocl: &ocl::Ocl) -> Dendrites {
		Dendrites {
			thresholds: CorticalComponent::<ocl::cl_uchar>::new(size, 16u8, ocl),
			synapse_states: CorticalComponent::<ocl::cl_ushort>::new(size, 0u16, ocl),
		}
	}
}


pub struct Synapses {
	pub strengths: CorticalComponent<ocl::cl_uchar>,
}
impl Synapses {
	pub fn new(size: uint, ocl: &ocl::Ocl) -> Synapses {

		Synapses {
			strengths: CorticalComponent::<ocl::cl_uchar>::new(size, 16u8, ocl),
		}
	}
}
