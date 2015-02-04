use common;
use ocl;
use envoy::{ Envoy };
use cortical_areas::{ CorticalAreas, Width };
use cortical_regions::{ CorticalRegions, CorticalRegionType, CorticalLayerClass };

use std::num;
use std::rand;
use std::mem;
use std::rand::distributions::{ Normal, IndependentSample, Range };
use std::num::{ NumCast, Int, FromPrimitive };
use std::default::{ Default };
use std::fmt::{ Display };


pub struct Cells {
	pub axons: Axons,
	pub somata: Somata,
	pub dendrites: Dendrites,
	pub synapses: Synapses,
}
impl Cells {
	pub fn new(regions: &CorticalRegions, areas: &CorticalAreas, ocl: &ocl::Ocl) -> Cells {

		let (height_antecellular_rows, height_cellular_rows) = regions.height(CorticalRegionType::Sensory);
		let height_total = height_antecellular_rows + height_cellular_rows;

		let width = areas.width(CorticalRegionType::Sensory);

		//let mut len_cells = (height_cellular_rows as usize) * (width as usize);

		//println!("ante_height: {}, width: {}", height_antecellular_rows, width);
		

		Cells {
			axons:	Axons::new(width, height_total, ocl),
			somata: Somata::new(width, height_cellular_rows, ocl),
			dendrites: Dendrites::new(width, height_cellular_rows, ocl),
			synapses: Synapses::new(width, height_cellular_rows, regions, ocl),
		}
	}
}


pub struct Axons {
	pub states: Envoy<ocl::cl_char>,
	pub width: u32,
	pub height: u8,
}

impl Axons {
	pub fn new(width: u32, height: u8, ocl: &ocl::Ocl) -> Axons {
		//let len = (width as usize * height as usize)+ common::MAX_SYNAPSE_RANGE;

		Axons {
			states: Envoy::<ocl::cl_char>::new(width, height, 0i8, ocl),
			width: width,
			height: height,
		}
	}
}


pub struct Somata {	
	pub states: Envoy<ocl::cl_char>,
}

impl Somata {
	pub fn new(width: u32, height: u8, ocl: &ocl::Ocl) -> Somata {
		Somata { states: Envoy::<ocl::cl_char>::new(width, height, 0i8, ocl), }
	}
}


pub struct Dendrites {
	pub thresholds: Envoy<ocl::cl_char>,
	pub states: Envoy<ocl::cl_char>,
}

impl Dendrites {
	pub fn new(width_cel: u32, height: u8, ocl: &ocl::Ocl) -> Dendrites {
		let width = width_cel * num::cast(common::DENDRITES_PER_NEURON).unwrap();
		Dendrites {
			thresholds: Envoy::<ocl::cl_char>::new(width, height, common::DENDRITE_INITIAL_THRESHOLD, ocl),
			states: Envoy::<ocl::cl_char>::new(width, height, 0i8, ocl),
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
	pub fn new(width_cel: u32, height: u8, regions: &CorticalRegions, ocl: &ocl::Ocl) -> Synapses {
		let width = width_cel * num::cast(common::SYNAPSES_PER_NEURON).unwrap();

		let input_size = 1024i16;	//	TEMPORARY

		//let mut src_idxs = Envoy::<ocl::cl_short>::new(width, height, 0i16, ocl);
		let mut axon_levs = Envoy::<ocl::cl_uchar>::new(width, height, 0, ocl);
		let mut axon_idxs = Envoy::<ocl::cl_char>::new(width, height, 0, ocl);

		let mut synapses = Synapses {
			states: Envoy::<ocl::cl_char>::new(width, height, 0, ocl),
			strengths: Envoy::<ocl::cl_char>::new(width, height, common::SYNAPSE_STRENGTH_ZERO, ocl),
			axon_levs: axon_levs,
			axon_idxs: axon_idxs,
			//axon_idxs: axon_idxs,

		};

		synapses.init(input_size, regions);

		synapses
	}

	fn init(&mut self, input_size: i16, regions: &CorticalRegions) {
		let len = self.axon_idxs.len();
		//let syn_per_layer = common::SYNAPSES_PER_LAYER;
		//let mut offset: usize = 0;
		//let mut current_layer: usize = 0;

		let mut rng = rand::thread_rng();

		let idx_range = Range::new(-126, 127);

		for axon_idx in self.axon_idxs.vec.iter_mut() {
			*axon_idx = idx_range.ind_sample(&mut rng);
		}

		let ref r = regions.hash_map[CorticalRegionType::Sensory];

		for (region_type, region) in regions.hash_map.iter() {
			println!("Synapses::init(): Region: {:?}:", region_type);
			for (&ln, l) in region.layers.iter() {
				let lri = region.layer_row_idxs(ln);
				let mut lsri: Vec<u8> =	match l.class {
					CorticalLayerClass::Interlaminar(_, _) => {
						region.layer_src_row_idxs(ln)
					},
					_ => Vec::with_capacity(0),		//	continue,
				};

				println!("Synapses::init(): 	layer name: {}, row_idxs: {:?}, src_idxs: {:?}", ln, lri, lsri);
			}
		}




		/*while current_layer < 16 {
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
		}*/


		
		self.axon_idxs.write();

		/*
		println!("Printing Sources: (input_size: {})", input_size);
		axon_idxs.print(1);
		*/
	}
}
