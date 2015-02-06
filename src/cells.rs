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
	pub axns: Axons,
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
			axns:	Axons::new(width, height_total, ocl),
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

		let padding = common::AXONS_MARGIN * 2;

		Axons {
			states: Envoy::<ocl::cl_char>::with_padding(padding, width, height, 0i8, ocl),
			width: width + num::cast(padding).expect("Axons::new()"),
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
		let width = width_cel * num::cast(common::DENDRITES_PER_NEURON).expect("cells::Dendrites::new()");
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
	pub axn_row_ids: Envoy<ocl::cl_uchar>,
	pub axn_col_offs: Envoy<ocl::cl_char>,
}

impl Synapses {
	pub fn new(width_cel: u32, height: u8, regions: &CorticalRegions, ocl: &ocl::Ocl) -> Synapses {
		let width = width_cel * num::cast(common::SYNAPSES_PER_NEURON).expect("cells::Synapses::new()");

		let input_size = 1024i16;	//	TEMPORARY

		//let mut src_idxs = Envoy::<ocl::cl_short>::new(width, height, 0i16, ocl);
		let mut axn_row_ids = Envoy::<ocl::cl_uchar>::new(width, height, 0, ocl);
		let mut axn_col_offs = Envoy::<ocl::cl_char>::new(width, height, 0, ocl);

		let mut synapses = Synapses {
			states: Envoy::<ocl::cl_char>::new(width, height, 0, ocl),
			strengths: Envoy::<ocl::cl_char>::new(width, height, common::SYNAPSE_STRENGTH_ZERO, ocl),
			axn_row_ids: axn_row_ids,
			axn_col_offs: axn_col_offs,
			//axn_col_offs: axn_col_offs,

		};

		synapses.init(input_size, regions);

		synapses
	}

	fn init(&mut self, input_size: i16, regions: &CorticalRegions) {
		//let len = self.axn_col_offs.len();
		assert!(self.axn_col_offs.width() == self.axn_row_ids.width(), "[cells::Synapse::init(): width mismatch]");
		let width = self.axn_col_offs.width();
		//let syn_per_layer = common::SYNAPSES_PER_LAYER;
		//let mut offset: usize = 0;
		//let mut current_layer: usize = 0;

		let mut rng = rand::thread_rng();

		let col_off_range: Range<i8> = Range::new(-126, 127);

		
		/*for axn_idx in self.axn_col_offs.vec.iter_mut() {
			*axn_idx = row_idx_range.ind_sample(&mut rng);
		}*/
		

		let ref r = regions[CorticalRegionType::Sensory];

		for (&ln, l) in r.layers.iter() {
			let row_ids = r.layer_row_ids_ct(ln);
			let src_row_ids: Vec<u8> =	match l.class {
				CorticalLayerClass::Interlaminar(_, _) => {
					r.layer_src_row_ids(ln)
				},
				_ => continue,
			};

			for &ri in row_ids.iter() {

				let src_row_idx_count: u8 = num::cast(src_row_ids.len()).expect("cells::Synapses::init()");
				//println!("Adding Indexes for row_idx: {}: src_row_idx_count: {}, src_row_ids.len(): {}", ri, src_row_idx_count, src_row_ids.len());
				let src_row_idx_range: Range<u8> = Range::new(0, src_row_idx_count);


					//	Envoy Indexes
				let ei_start = width as usize * ri as usize;
				let ei_end = ei_start + width as usize;

				//println!("	ei_start: {}, ei_end: {}", ei_start, ei_end);

				//src_row_ids[src_row_idx_range.ind_sample(&mut rng)];

				for i in range(ei_start, ei_end) {
					self.axn_row_ids[i] = src_row_ids[src_row_idx_range.ind_sample(&mut rng) as usize];
					self.axn_col_offs[i] = col_off_range.ind_sample(&mut rng);
				}


				/*for axn_row_idx in self.axn_row_ids.vec.iter_mut() {
					*axn_row_idx = idx_range.ind_sample(&mut rng);
				}*/
			}


			//println!("Synapses::init(): 	layer name: {}, row_ids: {:?}, src_row_ids: {:?}", ln, row_ids, src_row_ids);
		}




		/*while current_layer < 16 {
		 	offset = current_layer * syn_per_layer;
		 	//println!("current_layer: {}", current_layer);

		 	if current_layer < 4 {
		 		let input_range = Range::new(0, input_size);

				for i in range(0, syn_per_layer) {
					axn_col_offs.vec[offset + i] = input_range.ind_sample(&mut rng);
				}
		 	} else {
		 		let source_size = 256u16;		// NUMBER OF SOURCE CELLS READABLE FROM WITHIN LAYER (should be 256 later on)
				let source_range = Range::new(0, source_size);

				for i in range(0, syn_per_layer) {
					//axn_col_offs.vec[offset + i] = source_range.ind_sample(&mut rng);
				}
		 	}

			current_layer += 1;
		}*/


		
		self.axn_col_offs.write();
		self.axn_row_ids.write();

		/*
		println!("Printing Sources: (input_size: {})", input_size);
		axn_col_offs.print(1);
		*/
	}
}
