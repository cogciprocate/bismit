use ocl;
use common;
use envoy::{ Envoy };
use chord::{ Chord };
use cells:: { self, Cells };
use cortical_regions::{ self, CorticalRegion, CorticalRegions, CorticalRegionType };
use cortical_areas::{ self, CorticalAreas, Width };

use std::rand;
use std::rand::distributions::{IndependentSample, Range};
use std::ptr;
use std::num;
use std::collections::{ HashMap };
use time;


pub struct Cortex {
	pub cells: Cells,
	//pub sensory_segs: Thalamus,
	pub regions: CorticalRegions,
	pub areas: CorticalAreas,
	pub ocl: ocl::Ocl,
}

impl Cortex {
	pub fn new() -> Cortex {
		println!("Initializing Cortex... ");
		let time_start = time::get_time();
		let ocl: ocl::Ocl = ocl::Ocl::new();
		let regions = cortical_regions::define();
		let areas = cortical_areas::define();
		let cells = Cells::new(&regions, &areas, &ocl);


		/***	Sensory Segments 	***/
		/*let mut ss = Vec::with_capacity(common::SENSORY_SEGMENTS_TOTAL);
		ss.push(SensorySegment::new(num::cast(common::SENSORY_CHORD_WIDTH).unwrap(), &ocl));*/

		
		let time_complete = time::get_time() - time_start;
		println!("\n ...initialized in: {}.{} sec. ======", time_complete.num_seconds(), time_complete.num_milliseconds());

		Cortex {
			cells: cells,
			//sensory_segs: ss,
			regions: regions,
			areas: areas,
			ocl: ocl,
		}
	}


	pub fn sense(&mut self, sgmt_idx: usize, chord: &Chord) {
		let sensory_area = "v1";

		let mut glimpse: Vec<i8> = Vec::with_capacity(common::SENSORY_CHORD_WIDTH);
		chord.unfold_into(&mut glimpse, 0);
		ocl::enqueue_write_buffer(&glimpse, self.cells.axns.states.buf, self.ocl.command_queue, common::AXONS_MARGIN);


		self.cycle_syns();
		self.cycle_dens();
		self.cycle_axns();

	}

	fn cycle_syns(&self) {

		let width: u32 = self.areas.width(CorticalRegionType::Sensory);
		let height_total: u8 = self.regions.height_total(CorticalRegionType::Sensory);
		let (_, height_cellular) = self.regions.height(CorticalRegionType::Sensory);
		let len: u32 = width * height_total as u32;

		let test_envoy = Envoy::<ocl::cl_int>::new(width, height_total, 0, &self.ocl);

		//println!("cycle_cel_syns running with width = {}, height = {}", width, height_total);

		let kern = ocl::new_kernel(self.ocl.program, "cycle_syns");
		ocl::set_kernel_arg(0, self.cells.axns.states.buf, kern);
		ocl::set_kernel_arg(1, self.cells.synapses.axn_row_ids.buf, kern);
		ocl::set_kernel_arg(2, self.cells.synapses.axn_col_offs.buf, kern);
		ocl::set_kernel_arg(3, self.cells.synapses.strengths.buf, kern);
		ocl::set_kernel_arg(4, self.cells.synapses.states.buf, kern);

		//println!("height_total: {}, height_cellular: {}, width_syn_row: {}", height_total, height_cellular, width_syn_row);

		let gws = (height_cellular as usize, width as usize, common::SYNAPSES_PER_NEURON);

		//println!("gws: {:?}", gws);

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

		let kern = ocl::new_kernel(self.ocl.program, "cycle_axns");
		ocl::set_kernel_arg(0, self.cells.dendrites.states.buf, kern);
		ocl::set_kernel_arg(1, self.cells.axns.states.buf, kern);
		ocl::set_kernel_arg(2, height_antecellular as u32, kern);

		let gws = (height_cellular as usize, width as usize);

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
