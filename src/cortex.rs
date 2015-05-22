use ocl::{ self, Ocl };
use cmn;
use ocl::{ Envoy };
use chord::{ Chord };
use region_cells:: { self, RegionCells };
use proto::regions::{ self, ProtoRegion, ProtoRegions, ProtoRegionKind };
use proto::areas::{ self, ProtoAreas, ProtoArea, Width, AddNew };
use proto::layer as layer;
use proto::layer::{ ProtoLayer };
	use proto::layer::ProtoLayerKind::{ Cellular, Axonal };
	use proto::layer::AxonKind::{ Spatial, Horizontal };
use proto::cell::{ CellKind, Protocell, DendriteKind, CellFlags };

use rand;
use rand::distributions::{IndependentSample, Range};
use std::ptr;
use num;
use std::collections::{ HashMap };
use time;

	/* Eventually move define_*() to a config file or some such */
pub fn define_protoregions() -> ProtoRegions {
	let mut cort_regs: ProtoRegions = ProtoRegions::new();

	let mut sen = ProtoRegion::new(ProtoRegionKind::Sensory)
		//.layer("test_noise", 1, layer::DEFAULT, Axonal(Spatial))

		.layer("thal", 1, layer::DEFAULT, Axonal(Spatial))

		.layer("out", 1, layer::COLUMN_OUTPUT, Axonal(Spatial))

		.layer("iv", 1, layer::COLUMN_INPUT, Protocell::new_spiny_stellate(vec!["thal", "thal", "thal", "motor"]))  // , "motor"

		//.layer("iii", 1, layer::DEFAULT, Protocell::new_pyramidal(vec!["iii", "iii", "iii", "iii", "motor"]))
		.layer("iii", 4, layer::DEFAULT, Protocell::new_pyramidal(vec!["iii"]))

		//.layer("temp_padding", 2, layer::DEFAULT, Axonal(Horizontal))
		.layer("motor", 1, layer::DEFAULT, Axonal(Horizontal))

	;

	sen.freeze();
	cort_regs.add(sen);
	cort_regs
}

pub fn define_areas() -> ProtoAreas {
	let mut protoareas  = HashMap::new();
	let mut curr_offset: u32 = 128;

	curr_offset += protoareas.add_new("v1", ProtoArea { 
			width: cmn::SENSORY_CHORD_WIDTH, offset: curr_offset, cort_reg_type: ProtoRegionKind::Sensory });

	protoareas
}


pub struct Cortex {
	pub region_cells: RegionCells,
	pub protoregions: ProtoRegions,
	pub areas: ProtoAreas,
	pub ocl: ocl::Ocl,
}

impl Cortex {
	pub fn new() -> Cortex {
		print!("\nInitializing Cortex... ");
		let time_start = time::get_time();		

		let protoregions = define_protoregions();
		let areas = define_areas();

		let hrz_demarc = protoregions[&ProtoRegionKind::Sensory].hrz_demarc();
		let hrz_demarc_opt = ocl::BuildOption::new("HORIZONTAL_AXON_ROW_DEMARCATION", hrz_demarc as i32);
		let build_options = cmn::build_options().add(hrz_demarc_opt);

		let ocl: ocl::Ocl = ocl::Ocl::new(build_options);

		// FOR EACH REGION...
		let mut region_cells: region_cells::RegionCells = {
			let ref region = &protoregions[&ProtoRegionKind::Sensory];
			RegionCells::new(region, &areas, &ocl)
		};

		let time_complete = time::get_time() - time_start;
		println!("\n\n... Cortex initialized in: {}.{} sec.", time_complete.num_seconds(), time_complete.num_milliseconds());

		Cortex {
			region_cells: region_cells,
			protoregions: protoregions,
			areas: areas,
			ocl: ocl,
		}
	}


	pub fn sense(&mut self, sgmt_idx: usize, layer_target: &'static str, chord: &Chord) {
		let mut vec: Vec<ocl::cl_uchar> = chord.unfold();

		self.sense_vec(sgmt_idx, layer_target, &vec);
	}

	/* WRITE_VEC(): 
			TODO: VALIDATE "layer_target, OTHERWISE: 
				- thread '<main>' panicked at '[protoregions::ProtoRegion::index(): 
				invalid layer name: "pre_thal"]', src/protoregions.rs:339
					- Just have row_ids return an option<u8>
	*/
	pub fn write_vec(&mut self, sgmt_idx: usize, layer_target: &'static str, vec: &Vec<ocl::cl_uchar>) {
		let ref region = self.protoregions[&ProtoRegionKind::Sensory];
		let axn_row = region.row_ids(vec!(layer_target))[0];
		let buffer_offset = cmn::axn_idx_2d(axn_row, self.region_cells.axns.width, region.hrz_demarc()) as usize;
		//let buffer_offset = cmn::AXONS_MARGIN + (axn_row as usize * self.region_cells.axns.width as usize);

		ocl::enqueue_write_buffer(&vec, self.region_cells.axns.states.buf, self.ocl.command_queue, buffer_offset);
	}


	pub fn sense_vec(&mut self, sgmt_idx: usize, layer_target: &'static str, vec: &Vec<ocl::cl_uchar>) {
		self.write_vec(sgmt_idx, layer_target, vec);
		self.cycle();
	}

	pub fn cycle(&mut self) {
		let ref region = &self.protoregions[&ProtoRegionKind::Sensory];
		self.region_cells.cycle(region);
	}

	pub fn release_components(&mut self) {
		print!("Releasing OCL Components...");
		self.ocl.release_components();
	}
}


pub struct CorticalDimensions {
	depth_axn_rows: u8,
	depth_cell_rows: u8,
	width_cols: u32,
	width_dens: u32,
	width_syns: u32,
	width_offset_margin_axns: u32,
	initial_cellular_axn: u32,
}









/*	fn cycle_syns(&self) {

		let width: u32 = self.areas.width(ProtoRegionKind::Sensory);
		let depth_total: u8 = self.protoregions.depth_total(ProtoRegionKind::Sensory);
		let (_, depth_cellular) = self.protoregions.depth(ProtoRegionKind::Sensory);
		let len: u32 = width * depth_total as u32;

		let test_envoy = Envoy::<ocl::cl_int>::new(width, depth_total, 0, &self.ocl);

		//println!("cycle_cel_syns running with width = {}, depth = {}", width, depth_total);

		let kern = ocl::new_kernel(self.ocl.program, "cycle_syns");
		ocl::set_kernel_arg(0, self.region_cells.axns.states.buf, kern);
		ocl::set_kernel_arg(1, self.region_cells.dst_dens.syns.axn_row_ids.buf, kern);
		ocl::set_kernel_arg(2, self.region_cells.dst_dens.syns.axn_col_offs.buf, kern);
		ocl::set_kernel_arg(3, self.region_cells.dst_dens.syns.strengths.buf, kern);
		ocl::set_kernel_arg(4, self.region_cells.dst_dens.syns.states.buf, kern);

		//println!("depth_total: {}, depth_cellular: {}, width_syn_row: {}", depth_total, depth_cellular, width_syn_row);

		let gws = (depth_cellular as usize, width as usize, cmn::SYNAPSES_PER_CELL);

		//println!("gws: {:?}", gws);

		ocl::enqueue_3d_kernel(kern, self.ocl.command_queue, &gws);

	}*/

/*	fn cycle_dens(&self) {

		let width: u32 = self.areas.width(ProtoRegionKind::Sensory);
		let (_, depth_cellular) = self.protoregions.depth(ProtoRegionKind::Sensory);

		let width_dens: usize = width as usize * cmn::DENDRITES_PER_CELL * depth_cellular as usize;

		let kern = ocl::new_kernel(self.ocl.program, "cycle_dens");

		ocl::set_kernel_arg(0, self.region_cells.dst_dens.syns.states.buf, kern);
		ocl::set_kernel_arg(1, self.region_cells.dst_dens.thresholds.buf, kern);
		ocl::set_kernel_arg(2, self.region_cells.dst_dens.states.buf, kern);

		ocl::enqueue_kernel(kern, self.ocl.command_queue, width_dens);

	}*/

/*	fn cycle_axns(&self) {
		let width: u32 = self.areas.width(ProtoRegionKind::Sensory);
		let (depth_noncellular, depth_cellular) = self.protoregions.depth(ProtoRegionKind::Sensory);

		let kern = ocl::new_kernel(self.ocl.program, "cycle_axns");
		ocl::set_kernel_arg(0, self.region_cells.dst_dens.states.buf, kern);
		ocl::set_kernel_arg(1, self.region_cells.axns.states.buf, kern);
		ocl::set_kernel_arg(2, depth_noncellular as u32, kern);

		let gws = (depth_cellular as usize, width as usize);

		ocl::enqueue_2d_kernel(kern, self.ocl.command_queue, &gws);

	}*/
