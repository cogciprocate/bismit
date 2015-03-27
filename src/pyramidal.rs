use common;
use ocl::{ self, Ocl, WorkSize };
use envoy::{ Envoy };
use cortical_areas::{ CorticalAreas, Width };
use cortical_regions::{ CorticalRegion, CorticalRegionKind };
use protocell::{ CellKind, Protocell, DendriteKind };
use synapses::{ Synapses };
use dendrites::{ Dendrites };
use cells::{ Aux };
use aspiny::{ AspinyStellate };
use columns::{ Columns };
use axons::{ Axons };


use std::num;
use std::rand;
use std::mem;
use std::rand::distributions::{ Normal, IndependentSample, Range };
use std::rand::{ ThreadRng };
use std::num::{ NumCast, Int, FromPrimitive };
use std::default::{ Default };
use std::fmt::{ Display };


pub struct Pyramidal {
	height: u8,
	width: u32,
	kern_cycle_dens: ocl::Kernel,
	//den_prox_row: u8, 
	pub states: Envoy<ocl::cl_uchar>,
	pub dens: Dendrites,
}

impl Pyramidal {
	pub fn new(width: u32, region: &CorticalRegion, axons: &Axons, ocl: &Ocl) -> Pyramidal {

		let axn_row_offset = region.base_row_cell_kind(&CellKind::Pyramidal);
		let height: u8 = region.height_cell_kind(&CellKind::Pyramidal);
		//let col_input_layer = region.col_input_layer().expect("Pyramidal::new()");
		//let den_prox_row = region.row_ids(vec![col_input_layer.name])[0];
		
		//print!("\n### Pyramidal: Proximal Dendrite Row: {}", den_prox_row);

		let states = Envoy::<ocl::cl_uchar>::new(width, height, common::STATE_ZERO, ocl);

		let dens = Dendrites::new(width, height, DendriteKind::Distal, CellKind::Pyramidal, common::DENDRITES_PER_CELL_DISTAL_LOG2, region, axons, ocl);

		let kern_cycle_dens = ocl.new_kernel("pyr_cycle_dens", 
			WorkSize::TwoDim(height as usize, width as usize))
			.arg_env(&dens.states)
			.arg_scl(axn_row_offset)
			.arg_env(&states)
		;


		Pyramidal {
			height: height,
			width: width,
			kern_cycle_dens: kern_cycle_dens,
			//den_prox_row: den_prox_row,
			states: states,
			dens: dens,
		}
	}

	pub fn cycle(&self) {
		self.dens.cycle();
		self.kern_cycle_dens.enqueue();

	}
}
