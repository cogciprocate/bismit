use ocl;
use common;
use envoy::{ Envoy };
use thalamus::{ Thalamus, SensorySegment };
use chord::{ Chord };
use cells:: { self, Cells };
use cortical_regions::{ self, CorticalRegion, CorticalRegions, CorticalRegionType };
use cortical_areas::{ self, CorticalAreas, Width };
//use axn_space::{ AxonSpace };
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
	pub sensory_segs: Thalamus,
	//pub axn_space: AxonSpace,
	//pub layers: Vec<CorticalLayer>,
	pub regions: CorticalRegions,
	pub areas: CorticalAreas,
	//pub syn_segs: SynSegs,
	//pub segs: HashMap<&'static str, CortSeg>,
	pub ocl: ocl::Ocl,
}

impl Cortex {
	pub fn new() -> Cortex {
		println!("Initializing Cortex... ");
		let time_start = time::get_time();
		let ocl: ocl::Ocl = ocl::Ocl::new();
		let regions = cortical_regions::define();
		let areas = cortical_areas::define();
		//let axn_space = AxonSpace::new();
		let cells = Cells::new(&regions, &areas, &ocl);
		//let mut syn_segs = SynSegs::new(&cells.synapses);


		/***	Sensory Segments 	***/
		let mut ss = Vec::with_capacity(common::SENSORY_SEGMENTS_TOTAL);
		ss.push(SensorySegment::new(num::cast(common::SENSORY_CHORD_WIDTH).unwrap(), &ocl));

		
		let time_complete = time::get_time() - time_start;
		println!("\n ...initialized in: {}.{} sec. ======", time_complete.num_seconds(), time_complete.num_milliseconds());

		Cortex {
			cells: cells,
			sensory_segs: ss,
			//axn_space: axn_space,
			regions: regions,
			areas: areas,
			//syn_segs: syn_segs,
			//segs: HashMap::new(),
			ocl: ocl,
		}
	}


	pub fn sense(&mut self, sgmt_idx: usize, chord: &Chord) {
		let sensory_area = "v1";

		let mut glimpse: Vec<i8> = Vec::with_capacity(common::SENSORY_CHORD_WIDTH);
		chord.unfold_into(&mut glimpse, 0);
		ocl::enqueue_write_buffer(&glimpse, self.cells.axns.states.buf, self.ocl.command_queue, common::AXONS_MARGIN);


		//self.sensory_segs[sgmt_idx].sense(chord);

		//self.values.write();		*******


		self.cycle_syns();
		self.cycle_dens();
		self.cycle_axns();

	}

	fn cycle_syns(&self) {

		let width: u32 = self.areas.width(CorticalRegionType::Sensory);
		let height_total: u8 = self.regions.height_total(CorticalRegionType::Sensory);
		let (_, height_cellular) = self.regions.height(CorticalRegionType::Sensory);
		let len: u32 = width * height_total as u32;

		//let width_syn_row: u32 = width as u32 * num::cast(common::SYNAPSES_PER_NEURON).expect("cortex::Cortex::cycle_syns()");
		//let width_axn_row: u32 = width;

		let test_envoy = Envoy::<ocl::cl_int>::new(width, height_total, 0, &self.ocl);

		//println!("cycle_cel_syns running with width = {}, height = {}", width, height_total);

		let kern = ocl::new_kernel(self.ocl.program, "cycle_syns");
		ocl::set_kernel_arg(0, self.cells.axns.states.buf, kern);
		ocl::set_kernel_arg(1, self.cells.synapses.axn_row_ids.buf, kern);
		ocl::set_kernel_arg(2, self.cells.synapses.axn_col_offs.buf, kern);
		ocl::set_kernel_arg(3, self.cells.synapses.strengths.buf, kern);
		ocl::set_kernel_arg(4, self.cells.synapses.states.buf, kern);
		//ocl::set_kernel_arg(5, width, kern);

		//println!("height_total: {}, height_cellular: {}, width_syn_row: {}", height_total, height_cellular, width_syn_row);


		//let gws = height_cellular as u32 * width_syn_row as u32;
		//let dim_info = vec![height_total as u32, width_syn_row as u32];
		let gws = (height_cellular as usize, width as usize, common::SYNAPSES_PER_NEURON);

		println!("gws: {:?}", gws);

		//ocl::enqueue_kernel(kern, self.ocl.command_queue, gws);

			// ADD 3RD DIMENSION (SYNAPSE)

		ocl::enqueue_3d_kernel(kern, self.ocl.command_queue, &gws);

	}

	fn cycle_dens(&self) {

		let width: u32 = self.areas.width(CorticalRegionType::Sensory);
		let (_, height_cellular) = self.regions.height(CorticalRegionType::Sensory);

		let width_dens: usize = width as usize * common::DENDRITES_PER_NEURON * height_cellular as usize;


		let kern = ocl::new_kernel(self.ocl.program, "cycle_dens");

		ocl::set_kernel_arg(0, self.cells.synapses.states.buf, kern);
		ocl::set_kernel_arg(1, self.cells.dendrites.thresholds.buf, kern);
		ocl::set_kernel_arg(2, self.cells.dendrites.states.buf, kern);

		ocl::enqueue_kernel(kern, self.ocl.command_queue, width_dens);

	}

	fn cycle_axns(&self) {
		let width: u32 = self.areas.width(CorticalRegionType::Sensory);
		let (height_antecellular, height_cellular) = self.regions.height(CorticalRegionType::Sensory);
		//let width = self.cells.axns.width;
		//println!("width: {}", width);

		let kern = ocl::new_kernel(self.ocl.program, "cycle_axns");
		ocl::set_kernel_arg(0, self.cells.dendrites.states.buf, kern);
		ocl::set_kernel_arg(1, self.cells.axns.states.buf, kern);
		ocl::set_kernel_arg(2, height_antecellular as u32, kern);

		//ocl::enqueue_kernel(kern, self.ocl.command_queue, width as usize);

		let gws = (height_cellular as usize, width as usize);

		//println!("dim_info: {:?}", dim_info);


			// ADD 3RD DIMENSION (DENDRITE)

		ocl::enqueue_2d_kernel(kern, self.ocl.command_queue, &gws);

	}


	pub fn release_components(&mut self) {
		println!("\nReleasing OCL Components...");
		self.ocl.release_components();
	}
}



pub struct CorticalDimensions {
	height_axn_rows: u8,
	height_cell_rows: u8,
	width_cols: u32,
	width_dens: u32,
	width_syns: u32,
	width_offset_margin_axns: u32,
	initial_cellular_axn: u32,
}



//type CorticalSegment = Vec<&'static str>;








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






/*	fn cycle_cel_syns(&self, input_source: &Envoy<ocl::cl_char>,) {
		
		let layers_total: u32 = num::cast(common::LAYERS_PER_SEGMENT).expect("cycle_cel_syns, layers_total");
		let syn_per_layer: u32 = num::cast(common::SYNAPSES_PER_LAYER).expect("cycle_cel_syns, syn_per_layer");
		let axns_per_layer: u32 = num::cast(common::COLUMNS_PER_SEGMENT).expect("cycle_cel_syns, axns_per_layer");


		let il: i16 = num::cast(input_source.vec.len()).expect("cycle_cel_syns, il");
		//assert!(il == self.input_source_len, "Input vector size must equal self.input_source_len");

		let kern = ocl::new_kernel(self.ocl.program, "cycle_cel_syns");
		ocl::set_kernel_arg(1, self.cells.synapses.axn_idxs.buf, kern);
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

	fn cycle_cel_axns(&self) {
		let kern = ocl::new_kernel(self.ocl.program, "cycle_cel_axns");

		ocl::set_kernel_arg(0, self.cells.dendrites.states.buf, kern);
		//ocl::set_kernel_arg(1, self.cells.axns.states.buf, kern);

		//ocl::enqueue_kernel(kern, self.ocl.command_queue, common::CELLS_PER_SEGMENT);

	}*/





		/*
		let cp = CcssParams { 
			input_source: &self.cells.axns.states, 
			seq_iters: 0,
			syn_offset_factor: syn_per_layer, 
			syn_offset_start: syn_per_layer * 4,
			src_offset_factor: axns_per_layer,
			src_offset_start: (axns_per_layer * 4) - 128,
			gid_offset_factor: 1, 
			boost_factor: 16,
		 };
		self.cycle_cel_syn_seq(cp, kern);


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
		//cortex.axn_space.new_region("visual", 0, 1024, &cortex.segs[0].states);

			/***	Synaptic Segments 	***/
		//cortex.syn_segs.new_segment("visual", 0, common::SYNAPSES_PER_LAYER, SegType::Distal);
	
		/***	Cortical Segments 	***/
		//let mut seg1 = CortSeg::new("visual", common::COLUMNS_PER_SEGMENT);

		//seg1.gen_rows("visual", &mut cortex.axn_space, &mut cortex.syn_segs);



	/*pub fn regions_len(&self, region_type: CorticalRegionType) -> usize {
		let mut len = 0us;
		for (name, cr) in self.regions.iter() {
			if cr.region_type == region_type {
				len += (cr.height as usize) * cr.width;
			}
		}
		len
	}*/
