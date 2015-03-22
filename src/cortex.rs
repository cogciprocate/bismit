use ocl::{ self, Ocl };
use common;
use envoy::{ Envoy };
use chord::{ Chord };
use cells:: { self, Cells };
use cortical_regions::{ self, CorticalRegion, CorticalRegions, CorticalRegionKind };
use cortical_areas::{ self, CorticalAreas, CorticalArea, Width, AddNew };
use cortical_region_layer as layer;
use cortical_region_layer::{ CorticalRegionLayer };
use protocell::{ CellKind, Protocell, DendriteKind, CellFlags };


use std::rand;
use std::rand::distributions::{IndependentSample, Range};
use std::ptr;
use std::num;
use std::collections::{ HashMap };
use time;


/* Eventually move define_*() to a config file or some such */
pub fn define_regions() -> CorticalRegions {
	let mut cort_regs: CorticalRegions = CorticalRegions::new();
	let mut sen = CorticalRegion::new(CorticalRegionKind::Sensory)
		.layer("pre_thal", 1, layer::DEFAULT, None)
		.layer("thal", 1, layer::DEFAULT, None)
		.layer("post_thal", 1, layer::DEFAULT, None)
		//.layer("test_2", 1, None);
		//.layer("inhib_tmp", 1, None);
		//.layer("inhib_tmp_2", 1, None);
		//.layer("test_3", 1, None);

		.layer("iv", 1, layer::COLUMN_INPUT, Protocell::new_spiny_stellate(vec!["thal"]))
		//.layer("iv-b", 1, layer::DEFAULT, Protocell::new_pyramidal(vec!["iv"], "iv"));
		.layer("iii", 4, layer::DEFAULT, Protocell::new_pyramidal(vec!["iv"], "iv"))
		//.layer("ii", 1, layer::DEFAULT, Protocell::new_pyramidal(vec!["ii"], "iv"))
	;

	sen.finalize();

	cort_regs.add(sen);

	cort_regs
}

pub fn define_areas() -> CorticalAreas {
	let mut cortical_areas  = HashMap::new();
	let mut curr_offset: u32 = 128;

	curr_offset += cortical_areas.add_new("v1", CorticalArea { width: common::SENSORY_CHORD_WIDTH, offset: curr_offset, cort_reg_type: CorticalRegionKind::Sensory });

	cortical_areas
}


pub struct Cortex {
	pub cells: Cells,
	pub regions: CorticalRegions,
	pub areas: CorticalAreas,
	pub ocl: ocl::Ocl,
}

impl Cortex {
	pub fn new() -> Cortex {
		print!("\nInitializing Cortex... ");
		let build_options: String = common::build_options();
		//let build_options: String = common::CL_BUILD_OPTIONS.to_string();
		let time_start = time::get_time();
		let ocl: ocl::Ocl = ocl::Ocl::new(build_options);
		let regions = define_regions();
		let areas = define_areas();

		// FOR EACH REGION...
		let mut cells: cells::Cells = {
			let ref region = &regions[CorticalRegionKind::Sensory];
			Cells::new(region, &areas, &ocl)
		};

		let time_complete = time::get_time() - time_start;
		print!("\n... Cortex initialized in: {}.{} sec.", time_complete.num_seconds(), time_complete.num_milliseconds());

		Cortex {
			cells: cells,
			regions: regions,
			areas: areas,
			ocl: ocl,
		}
	}


	pub fn sense(&mut self, sgmt_idx: usize, layer_target: &'static str, chord: &Chord) {

		let mut vec: Vec<ocl::cl_uchar> = Vec::with_capacity(chord.width as usize);
		chord.unfold_into(&mut vec, 0);
		self.sense_vec(sgmt_idx, layer_target, &vec);

	}

	pub fn sense_vec_no_cycle(&mut self, sgmt_idx: usize, layer_target: &'static str, vec: &Vec<ocl::cl_uchar>) {

		let axn_row = self.regions[CorticalRegionKind::Sensory].row_ids(vec!(layer_target))[0];

		let buffer_offset = common::AXONS_MARGIN + (axn_row as usize * self.cells.axns.width as usize);

		ocl::enqueue_write_buffer(&vec, self.cells.axns.states.buf, self.ocl.command_queue, buffer_offset);

	}


	pub fn sense_vec(&mut self, sgmt_idx: usize, layer_target: &'static str, vec: &Vec<ocl::cl_uchar>) {

		let axn_row = self.regions[CorticalRegionKind::Sensory].row_ids(vec!(layer_target))[0];

		let buffer_offset = common::AXONS_MARGIN + (axn_row as usize * self.cells.axns.width as usize);

		ocl::enqueue_write_buffer(&vec, self.cells.axns.states.buf, self.ocl.command_queue, buffer_offset);

		self.cells.cycle();
	}


	pub fn release_components(&mut self) {
		print!("Releasing OCL Components...");
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


/*	fn cycle_syns(&self) {

		let width: u32 = self.areas.width(CorticalRegionKind::Sensory);
		let height_total: u8 = self.regions.height_total(CorticalRegionKind::Sensory);
		let (_, height_cellular) = self.regions.height(CorticalRegionKind::Sensory);
		let len: u32 = width * height_total as u32;

		let test_envoy = Envoy::<ocl::cl_int>::new(width, height_total, 0, &self.ocl);

		//println!("cycle_cel_syns running with width = {}, height = {}", width, height_total);

		let kern = ocl::new_kernel(self.ocl.program, "cycle_syns");
		ocl::set_kernel_arg(0, self.cells.axns.states.buf, kern);
		ocl::set_kernel_arg(1, self.cells.dst_dens.syns.axn_row_ids.buf, kern);
		ocl::set_kernel_arg(2, self.cells.dst_dens.syns.axn_col_offs.buf, kern);
		ocl::set_kernel_arg(3, self.cells.dst_dens.syns.strengths.buf, kern);
		ocl::set_kernel_arg(4, self.cells.dst_dens.syns.states.buf, kern);

		//println!("height_total: {}, height_cellular: {}, width_syn_row: {}", height_total, height_cellular, width_syn_row);

		let gws = (height_cellular as usize, width as usize, common::SYNAPSES_PER_CELL);

		//println!("gws: {:?}", gws);

		ocl::enqueue_3d_kernel(kern, self.ocl.command_queue, &gws);

	}*/

/*	fn cycle_dens(&self) {

		let width: u32 = self.areas.width(CorticalRegionKind::Sensory);
		let (_, height_cellular) = self.regions.height(CorticalRegionKind::Sensory);

		let width_dens: usize = width as usize * common::DENDRITES_PER_CELL * height_cellular as usize;

		let kern = ocl::new_kernel(self.ocl.program, "cycle_dens");

		ocl::set_kernel_arg(0, self.cells.dst_dens.syns.states.buf, kern);
		ocl::set_kernel_arg(1, self.cells.dst_dens.thresholds.buf, kern);
		ocl::set_kernel_arg(2, self.cells.dst_dens.states.buf, kern);

		ocl::enqueue_kernel(kern, self.ocl.command_queue, width_dens);

	}*/

/*	fn cycle_axns(&self) {
		let width: u32 = self.areas.width(CorticalRegionKind::Sensory);
		let (height_noncellular, height_cellular) = self.regions.height(CorticalRegionKind::Sensory);

		let kern = ocl::new_kernel(self.ocl.program, "cycle_axns");
		ocl::set_kernel_arg(0, self.cells.dst_dens.states.buf, kern);
		ocl::set_kernel_arg(1, self.cells.axns.states.buf, kern);
		ocl::set_kernel_arg(2, height_noncellular as u32, kern);

		let gws = (height_cellular as usize, width as usize);

		ocl::enqueue_2d_kernel(kern, self.ocl.command_queue, &gws);

	}*/
