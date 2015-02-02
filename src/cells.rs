use common;
use ocl;
use envoy::{ Envoy };

use std::num;
use std::rand;
use std::mem;
use std::rand::distributions::{ Normal, IndependentSample, Range };
use std::num::{ NumCast, Int, FromPrimitive };
use std::default::{ Default };
use std::fmt::{ Display };


pub struct Cells {
	//pub axons: Axons,
	pub somata: Somata,
	pub dendrites: Dendrites,
	pub synapses: Synapses,
}
impl Cells {
	pub fn new(len: usize, ocl: &ocl::Ocl) -> Cells {

		Cells {
			//axons:	Axons::new(common::CELL_AXONS_PER_SEGMENT, ocl),
			somata: Somata::new(len, ocl),
			dendrites: Dendrites::new(len * common::DENDRITES_PER_NEURON, ocl),
			synapses: Synapses::new(len * common::SYNAPSES_PER_NEURON, ocl),
		}
	}
}


/*pub struct Axons {
	pub states: Envoy<ocl::cl_char>,
}
impl Axons {
	pub fn new(size: usize, ocl: &ocl::Ocl) -> Axons {

		Axons {
			states: Envoy::<ocl::cl_char>::new(actual_axons, 0i8, ocl),
		}
	}
}*/


pub struct Somata {	
	pub states: Envoy<ocl::cl_char>,
}

impl Somata {
	pub fn new(size: usize, ocl: &ocl::Ocl) -> Somata {
		Somata { states: Envoy::<ocl::cl_char>::new(size, 0i8, ocl), }
	}
}


pub struct Dendrites {
	pub thresholds: Envoy<ocl::cl_char>,
	pub states: Envoy<ocl::cl_char>,
}
impl Dendrites {
	pub fn new(size: usize, ocl: &ocl::Ocl) -> Dendrites {
		Dendrites {
			thresholds: Envoy::<ocl::cl_char>::new(size, common::DENDRITE_INITIAL_THRESHOLD, ocl),
			states: Envoy::<ocl::cl_char>::new(size, 0i8, ocl),
		}
	}
}


pub struct Synapses {
	pub states: Envoy<ocl::cl_char>,
	pub strengths: Envoy<ocl::cl_char>,
	//pub src_idxs: Envoy<ocl::cl_short>,
	pub axon_levs: Envoy<ocl::cl_uchar>,
	pub axon_idxs: Envoy<ocl::cl_char>,
}
impl Synapses {
	pub fn new(size: usize, ocl: &ocl::Ocl) -> Synapses {
		let input_size = 1024i16;

		//let mut src_idxs = Envoy::<ocl::cl_short>::new(size, 0i16, ocl);
		let mut axon_levs = Envoy::<ocl::cl_uchar>::new(size, 0, ocl);
		let mut axon_idxs = Envoy::<ocl::cl_char>::new(size, 0, ocl);


		//Synapses::init(&mut axon_idxs, input_size);

		Synapses {
			states: Envoy::<ocl::cl_char>::new(size, 0i8, ocl),
			strengths: Envoy::<ocl::cl_char>::new(size, common::SYNAPSE_STRENGTH_ZERO, ocl),
			axon_levs: axon_levs,
			axon_idxs: axon_idxs,
			//axon_idxs: axon_idxs,

		}
	}

	fn init(mut axon_idxs: &mut Envoy<ocl::cl_short>, input_size: i16) {
		let len = axon_idxs.vec.len();
		let syn_per_layer = common::SYNAPSES_PER_LAYER;
		let mut offset: usize = 0;
		let mut current_layer: usize = 0;

		let mut rng = rand::thread_rng();

		//println!("common::SYNAPSES_PER_LAYER: {}", common::SYNAPSES_PER_LAYER);

		while current_layer < 16 {
		 	offset = current_layer * syn_per_layer;
		 	//println!("current_layer: {}", current_layer);

		 	if current_layer < 4 {
		 		let input_range = Range::new(0, input_size);

				for i in range(0, syn_per_layer) {
					axon_idxs.vec[offset + i] = input_range.ind_sample(&mut rng);
				}
		 	} else {
		 		let source_size = 256u16;		// NUMBER OF SOURCE CELLS READABLE FROM WITHIN LAYER (should be 256 later on)
				let source_range = Range::new(0, source_size);

				for i in range(0, syn_per_layer) {
					//axon_idxs.vec[offset + i] = source_range.ind_sample(&mut rng);
				}
		 	}

			current_layer += 1;
		}


		
		axon_idxs.write();

		/*
		println!("Printing Sources: (input_size: {})", input_size);
		axon_idxs.print(1);
		*/
	}
}
