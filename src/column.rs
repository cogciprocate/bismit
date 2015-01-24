use common;
use ocl;
use cortical_component::{ CorticalComponent };

use std::num;
use std::rand;
use std::rand::distributions::{ Normal, IndependentSample, Range };
use std::num::{ NumCast, UnsignedInt, Int };
use std::fmt::{ Show, String };
use std::default::{ Default };

pub struct Bodies {
	pub states: CorticalComponent<ocl::cl_uchar>,
}

impl Bodies {
	pub fn new(size: usize, ocl: &ocl::Ocl) -> Bodies {
		Bodies { states: CorticalComponent::<ocl::cl_uchar>::new(size, 0u8, ocl), }
	}
}


pub struct Axons {
	pub target_column_bodies: CorticalComponent<ocl::cl_ushort>,
	pub target_column_synapses: CorticalComponent<ocl::cl_uchar>,
}

impl Axons {
	pub fn new(size: usize, ocl: &ocl::Ocl) -> Axons {

		let mut target_column_bodies = CorticalComponent::<ocl::cl_ushort>::new(size, 0u16, ocl);
		let mut target_column_synapses = CorticalComponent::<ocl::cl_uchar>::new(size, 0u8, ocl);

		Axons::init(&mut target_column_bodies, &mut target_column_synapses, ocl);

		//	println!("[Initializing Axons with size: {}]", size);

		Axons {
			target_column_bodies: target_column_bodies,
			target_column_synapses: target_column_synapses,
		}
	}

	pub fn init(
				target_column_bodies: &mut CorticalComponent<ocl::cl_ushort>, 
				target_column_synapses: &mut CorticalComponent<ocl::cl_uchar>,
				//	&mut self,
				ocl: &ocl::Ocl,

	) {
		/*
			let mut rng = rand::thread_rng();

			let rng_range = Range::new(0u16, (common::COLUMNS_PER_SEGMENT) as u16);

			for i in range(0, target_column_bodies.vec.len()) {
				target_column_bodies.vec[i] = rng_range.ind_sample(&mut rng);
			}

			let rng_range = Range::new(0u16, 256u16);

			for i in range(0, target_column_synapses.vec.len()) {
				target_column_synapses.vec[i] = num::FromPrimitive::from_u16(rng_range.ind_sample(&mut rng)).unwrap();
			}
		*/

		//	common::print_vec(&self.target_column_bodies.vec);
		//	common::print_vec(&target_column_synapses.vec, 1);

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

		/*
			println!("*** Printing New Axon ***");
			println!("** body addresses **");
			println!("* local: *");
			common::print_vec(&target_column_bodies.vec, 10000);
			println!("\n* remote: *");
			target_column_bodies.print(10000);
			println!("\n** synapse addresses **");
			println!("* local: *");
			common::print_vec(&target_column_synapses.vec, 10000);
			println!("\n* remote: *");
			target_column_synapses.print(10000);
			println!("");
		*/

		//	target_column_bodies.print(ocl);
		//	target_column_synapses.print(ocl);

		//	ocl::enqueue_write_buffer(&mut self.target_column_bodies.vec, self.target_column_bodies.buf, ocl.command_queue);

		//	common::print_vec(&self.target_column_bodies.vec);

	}
}



pub struct Dendrites {
	pub values: CorticalComponent<ocl::cl_uchar>,
	pub thresholds: CorticalComponent<ocl::cl_uchar>,
}
impl Dendrites {
	pub fn new(size: usize, ocl: &ocl::Ocl) -> Dendrites {
		Dendrites {
			values: CorticalComponent::<ocl::cl_uchar>::new(size, 0u8, ocl),
			thresholds: CorticalComponent::<ocl::cl_uchar>::new(size, common::DENDRITE_INITIAL_THRESHOLD, ocl),
		}
	}
}


pub struct Synapses {
	pub values: CorticalComponent<ocl::cl_uchar>,
	pub strengths: CorticalComponent<ocl::cl_uchar>,
}
impl Synapses {
	pub fn new(size: usize, ocl: &ocl::Ocl) -> Synapses {
		Synapses {
			values: CorticalComponent::<ocl::cl_uchar>::new(size, 0u8, ocl),
			strengths: CorticalComponent::<ocl::cl_uchar>::new(size, common::SYNAPSE_WEIGHT_ZERO, ocl),
		}
	}

}




//	<T: Int + Num + Primitive + NumCast + Show + UnsignedInt + Default, U: num::NumCast>
