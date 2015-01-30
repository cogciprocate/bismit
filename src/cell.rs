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
	//pub states: Envoy<ocl::cl_int>,
	pub axons: Axons,
	pub somata: Somata,
	pub dendrites: Dendrites,
	pub synapses: Synapses,
}
impl Cells {
	pub fn new(ocl: &ocl::Ocl) -> Cells {

		Cells {
			//states: Envoy::<ocl::cl_int>::new(common::CELLS_PER_SEGMENT, 0u32, ocl),
			axons:	Axons::new(common::CELL_AXONS_PER_SEGMENT, ocl),
			somata: Somata::new(common::CELLS_PER_SEGMENT, ocl),
			dendrites: Dendrites::new(common::CELL_DENDRITES_PER_SEGMENT, ocl),
			synapses: Synapses::new(common::CELL_SYNAPSES_PER_SEGMENT, ocl),
		}
	}
}


pub struct Axons {
	pub states: Envoy<ocl::cl_char>,
}
impl Axons {
	pub fn new(size: usize, ocl: &ocl::Ocl) -> Axons {

		let actual_axons = size / common::AXONS_PER_NEURON;		// one per cell;

		assert!(actual_axons == common::CELLS_PER_SEGMENT, "Each cell in segment must have 1 axon.");

		//init_axon(&mut target_cell_somata, &mut target_cell_synapses);

		//Axons::init(&mut target_cell_somata, &mut target_cell_synapses, ocl);

		Axons {
			states: Envoy::<ocl::cl_char>::new(actual_axons, 0i8, ocl),
		}
	}

		// ************ REWRITE SHUFFLING FUNCTION TO DISTRIBUTE USING A NORMAL DISTRIBUTION

	pub fn init(
				target_cell_somata: &mut Envoy<ocl::cl_short>, 
				target_cell_synapses: &mut Envoy<ocl::cl_char>,
				ocl: &ocl::Ocl,
	) {

		//println!("cell::Axons init with {} len.", common::CELL_AXONS_PER_SEGMENT);
		assert!(target_cell_somata.len() == target_cell_synapses.len(), "Arrays must be of equal length.");

		let len = target_cell_somata.len();

				// ************ REWRITE SHUFFLING FUNCTION TO DISTRIBUTE USING A NORMAL DISTRIBUTION
		let source_vec = common::shuffled_vec(len, 0i32);

		for i in range(0, len) {
			let som_addr = source_vec[i] >> 8;
			let syn_addr = source_vec[i] - (som_addr << 8);

			//target_cell_somata.vec[i] = num::cast(som_addr).expect("cell::Axons::init(), target_cell_somata");
			//target_cell_synapses.vec[i] = num::cast(syn_addr).expect("cell::Axons::init(), target_cell_synapses");

		}

		target_cell_somata.write();
		target_cell_synapses.write();


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
	pub thresholds: Envoy<ocl::cl_char>,
	pub values: Envoy<ocl::cl_char>,
}
impl Dendrites {
	pub fn new(size: usize, ocl: &ocl::Ocl) -> Dendrites {
		Dendrites {
			thresholds: Envoy::<ocl::cl_char>::new(size, common::DENDRITE_INITIAL_THRESHOLD, ocl),
			values: Envoy::<ocl::cl_char>::new(size, 0i8, ocl),
		}
	}
}


pub struct Synapses {
	pub values: Envoy<ocl::cl_char>,
	pub strengths: Envoy<ocl::cl_char>,
	pub source_addrs: Envoy<ocl::cl_short>,
}
impl Synapses {
	pub fn new(size: usize, ocl: &ocl::Ocl) -> Synapses {
		let input_size = 1024i16;

		let mut source_addrs = Envoy::<ocl::cl_short>::new(size, 0i16, ocl);


		Synapses::init(&mut source_addrs, input_size);

		Synapses {
			values: Envoy::<ocl::cl_char>::new(size, 0i8, ocl),
			strengths: Envoy::<ocl::cl_char>::new(size, common::SYNAPSE_STRENGTH_ZERO, ocl),
			source_addrs: source_addrs,
		}
	}

	fn init(mut source_addrs: &mut Envoy<ocl::cl_short>, input_size: i16) {
		let len = source_addrs.vec.len();
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
					source_addrs.vec[offset + i] = input_range.ind_sample(&mut rng);
				}
		 	} else {
		 		let source_size = 256u16;		// NUMBER OF SOURCE CELLS READABLE FROM WITHIN LAYER (should be 256 later on)
				let source_range = Range::new(0, source_size);

				for i in range(0, syn_per_layer) {
					//source_addrs.vec[offset + i] = source_range.ind_sample(&mut rng);
				}
		 	}

			current_layer += 1;
		}


		
		source_addrs.write();

		/*
		println!("Printing Sources: (input_size: {})", input_size);
		source_addrs.print(1);
		*/
	}
}


/*

pub fn init_axon<T: Clone + NumCast + Int + Default + Display + FromPrimitive>(target_cell_somata: &mut Envoy<T>, target_cell_synapses: &mut Envoy<T>) {
	let mut rng = rand::thread_rng();

	let normal = Normal::new(128f64, 128f64);
	
	for i in range(0, target_cell_somata.vec.len()) {
		let val = normal.ind_sample(&mut rng);
		let cell = num::cast(val).expect();
		target_cell_somata.vec[i] = cell;
		
	}

	let rng_range = Range::new(0u8, 255u8);

	for i in range(0, target_cell_synapses.vec.len()) {
		target_cell_synapses.vec[i] = num::cast(rng_range.ind_sample(&mut rng)).expect();
	}
	
	target_cell_somata.write();
	target_cell_synapses.write();

}

*/
