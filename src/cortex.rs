use ocl;
use common;
use envoy::{ Envoy };
use sensory_seg::{ SensorySegment };
use chord::{ Chord };
use cells:: { self, Cells };
use cortical_regions::{ self, CorticalRegion, CorticalRegions, CorticalRegionType };
use cortical_areas;
//use axon_space::{ AxonSpace };
//use syn_segs::{ SynSegs, SegType };
//use cort_seg::{ CortSeg };

use std::rand;
use std::rand::distributions::{IndependentSample, Range};
use std::ptr;
use std::num;
use std::collections::{ HashMap };
use time;


pub struct Cortex {
	pub cells: Cells,
	pub sensory_segs: Vec<SensorySegment>,
	//pub axon_space: AxonSpace,
	//pub layers: Vec<CorticalLayer>,
	pub regions: CorticalRegions,
	//pub syn_segs: SynSegs,
	//pub segs: HashMap<&'static str, CortSeg>,
	pub ocl: ocl::Ocl,
}

impl Cortex {
	pub fn new() -> Cortex {
		println!("Initializing Cortex... ");
		let time_start = time::get_time().sec;
		let ocl: ocl::Ocl = ocl::Ocl::new();
		let regions = cortical_regions::define();
		let area_width = cortical_areas::define();
		//let mut axon_space = AxonSpace::new();
		//let mut syn_segs = SynSegs::new(&cells.synapses);
		let mut cell_layers = 0us;

		for (region_type, region) in regions.iter() {
			if *region_type == CorticalRegionType::Sensory {
				let (antecell, cell) = region.row_count();
				cell_layers += cell;
				//println!("*** antecell: {}, cell: {} ***", antecell, cell);
			}
		}

		let total_cells = cell_layers * area_width;

		let cells = Cells::new(total_cells, &ocl);


			/***	Sensory Segments 	***/
		let mut ss = Vec::with_capacity(common::SENSORY_SEGMENTS_TOTAL);
		ss.push(SensorySegment::new(common::SENSORY_CHORD_WIDTH, &ocl));

		



		let time_finish = time::get_time().sec;
		println!(" ...initialized in: {} sec.", time_finish - time_start);

		Cortex {
			cells: cells,
			sensory_segs: ss,
			//axon_space: axon_space,
			regions: regions,
			//syn_segs: syn_segs,
			//segs: HashMap::new(),
			ocl: ocl,
		}
	}


	pub fn sense(&mut self, sgmt_idx: usize, chord: &Chord) {
		self.sensory_segs[sgmt_idx].sense(chord);
		//self.cycle_cel_syns(&self.sensory_segs[sgmt_idx].states);
		//self.cycle_cel_dens();
		//self.cycle_cel_axons();

	}

	/*pub fn regions_len(&self, region_type: CorticalRegionType) -> usize {
		let mut len = 0us;
		for (name, cr) in self.regions.iter() {
			if cr.region_type == region_type {
				len += (cr.height as usize) * cr.width;
			}
		}
		len
	}*/


	pub fn release_components(&mut self) {
		println!("\nReleasing OCL Components...");
		self.ocl.release_components();
	}
}


//type CorticalSegment = Vec<&'static str>;






struct CcssParams<'a> {
	input_source: &'a Envoy<ocl::cl_char>, 
	seq_iters: u32,
	syn_offset_factor: u32, 
	syn_offset_start: u32,
	src_offset_factor: u32,
	src_offset_start: u32,
	gid_offset_factor: u32, 
	boost_factor: u8
}






/*
pub struct CorticalRegion {
	region_type: CorticalRegionType,
	name: &'static str,
	width: usize,
	height: ocl::cl_uchar,
	rows: Vec<(&'static str, CorticalCellType)>,
}

impl CorticalRegion {
	fn new(
					region_type: CorticalRegionType,
					name: &'static str,
					width: usize,
	) -> CorticalRegion {
		use self::CorticalCellType::*;

		let mut rows = match region_type {
			CorticalRegionType::Associational => {
				vec![
					("L1", Pyramidal),
					("L2", Pyramidal),
				]
			},
			CorticalRegionType::Sensory => {
				vec![
					("Ultra-Cortical Input", Corticocortical),
					("Motor Command State", Corticocortical),
					("Primary Sensory Input", Thalamocortical),
					("Sub-Cortical Input", Corticocortical),
					("Layer IV", Pyramidal),
					("Layer IV", Pyramidal),
					("Layer IV", Pyramidal),
					("Layer IV", Pyramidal),
					("Layer IV", Pyramidal),

				]
			},
			CorticalRegionType::Motor => {
				vec![
					("L1", Pyramidal),
					("L2", Pyramidal),
				]

			},
		};

		rows.shrink_to_fit();

		CorticalRegion { 
			region_type: region_type,
			name: name,
			width: width,
			height: num::cast(rows.len()).unwrap(),
			rows: rows,
		}
	}
}
*/






/*fn cycle_cel_syns(&self, input_source: &Envoy<ocl::cl_char>,) {
		
		let layers_total: u32 = num::cast(common::LAYERS_PER_SEGMENT).expect("cycle_cel_syns, layers_total");
		let syn_per_layer: u32 = num::cast(common::SYNAPSES_PER_LAYER).expect("cycle_cel_syns, syn_per_layer");
		let axons_per_layer: u32 = num::cast(common::COLUMNS_PER_SEGMENT).expect("cycle_cel_syns, axons_per_layer");


		let il: i16 = num::cast(input_source.vec.len()).expect("cycle_cel_syns, il");
		//assert!(il == self.input_source_len, "Input vector size must equal self.input_source_len");

		let kern = ocl::new_kernel(self.ocl.program, "cycle_cel_syns");
		ocl::set_kernel_arg(1, self.cells.synapses.axon_idxs.buf, kern);
		ocl::set_kernel_arg(2, self.cells.synapses.strengths.buf, kern);
		ocl::set_kernel_arg(3, self.cells.synapses.states.buf, kern);


		let cp = CcssParams { 
			input_source: input_source, 
			seq_iters: 4,
			syn_offset_factor: syn_per_layer, 
			syn_offset_start: 0,
			src_offset_factor: 0,
			src_offset_start: 0,
			gid_offset_factor: 0, 
			boost_factor: 1,
		 };
		self.cycle_cel_syn_seq(cp, kern);


	}

	fn cycle_cel_syn_seq( 
					&self, 
					params: CcssParams,
					kern: ocl::cl_kernel,
	) {
		let mut syn_offset = params.syn_offset_start;
		let mut src_offset = params.src_offset_start;

		assert!(
			((params.src_offset_factor + (src_offset * params.seq_iters)) as usize) 
			< (params.input_source.vec.len() - 128)
		);

		for i in range(0, params.seq_iters) {
			
			println!("[syn_offset: {}, src_offset: {}, gid_offset_factor: {}, source.len: {}]", syn_offset, src_offset, params.gid_offset_factor, params.input_source.vec.len());

			ocl::set_kernel_arg(0, params.input_source.buf, kern);
			ocl::set_kernel_arg(4, src_offset, kern);
			ocl::set_kernel_arg(5, syn_offset, kern);
			ocl::set_kernel_arg(6, params.gid_offset_factor, kern);
			ocl::set_kernel_arg(7, params.boost_factor, kern);

			ocl::enqueue_kernel(kern, self.ocl.command_queue, common::SYNAPSES_PER_LAYER);

			syn_offset += params.syn_offset_factor;
			src_offset += params.src_offset_factor;
		}

	}

	fn cycle_cel_dens(&self) {
		let kern = ocl::new_kernel(self.ocl.program, "cycle_cel_dens");

		ocl::set_kernel_arg(0, self.cells.synapses.states.buf, kern);
		ocl::set_kernel_arg(1, self.cells.dendrites.thresholds.buf, kern);
		ocl::set_kernel_arg(2, self.cells.dendrites.states.buf, kern);

		ocl::enqueue_kernel(kern, self.ocl.command_queue, common::CELL_DENDRITES_PER_SEGMENT);

	}

	fn cycle_cel_axons(&self) {
		let kern = ocl::new_kernel(self.ocl.program, "cycle_cel_axons");

		ocl::set_kernel_arg(0, self.cells.dendrites.states.buf, kern);
		//ocl::set_kernel_arg(1, self.cells.axons.states.buf, kern);

		//ocl::enqueue_kernel(kern, self.ocl.command_queue, common::CELLS_PER_SEGMENT);

	}*/





		/*
		let cp = CcssParams { 
			input_source: &self.cells.axons.states, 
			seq_iters: 0,
			syn_offset_factor: syn_per_layer, 
			syn_offset_start: syn_per_layer * 4,
			src_offset_factor: axons_per_layer,
			src_offset_start: (axons_per_layer * 4) - 128,
			gid_offset_factor: 1, 
			boost_factor: 16,
		 };
		self.cycle_cel_syn_seq(cp, kern);
		*/





/*pub struct HyperColumns {
	pub qty: usize,
	pub states: Envoy<ocl::cl_int>,
}
impl HyperColumns {
	pub fn new(qty: usize, ocl: &ocl::Ocl) -> HyperColumns {
		HyperColumns {
			qty: common::HYPERCOLUMNS_PER_SEGMENT,
			states: Envoy::<ocl::cl_int>::new(common::HYPERCOLUMNS_PER_SEGMENT, 0i32, ocl),
		}
	}
}*/


/*pub struct MotorSegment {
	pub targets: Envoy<ocl::cl_short>,
	pub states: Envoy<ocl::cl_char>,
}
impl MotorSegment {
	pub fn new(width: usize, ocl: &ocl::Ocl) -> MotorSegment {
		MotorSegment { 
			targets : Envoy::<ocl::cl_short>::new(width, 0i16, ocl),
			states : Envoy::<ocl::cl_char>::new(width, 0i8, ocl),
		}
	}
}*/







/*
		cortex.regions.push(
			CorticalRegion {
				name: "v1",
				layers: vec![
					CorticalLayer { 
						vec![

						]
					}
					CorticalLayer { }
				],
			}
		);
*/


		/*cortex.regions.push(CorticalRegion { 
			name: "v1", 
			region_type: CorticalRegionType::Sensory,
			width: 1024,
		});*/


		
			/***	Axon Spaces 	***/
		//cortex.axon_space.new_region("visual", 0, 1024, &cortex.sensory_segs[0].states);

			/***	Synaptic Segments 	***/
		//cortex.syn_segs.new_segment("visual", 0, common::SYNAPSES_PER_LAYER, SegType::Distal);
	
		/***	Cortical Segments 	***/
		//let mut seg1 = CortSeg::new("visual", common::COLUMNS_PER_SEGMENT);

		//seg1.gen_rows("visual", &mut cortex.axon_space, &mut cortex.syn_segs);
