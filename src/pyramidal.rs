use common;
use ocl::{ self, Ocl, WorkSize };
use envoy::{ Envoy };
use cortical_areas::{ CorticalAreas, Width };
use cortical_regions::{ CorticalRegion, CorticalRegionKind };
use protocell::{ CellKind, Protocell, DendriteKind };
use synapses::{ Synapses };
use dendrites::{ Dendrites };
use cells::{ Aux };
use peak_column::{ PeakColumn };
use columns::{ Columns };
use axons::{ Axons };


use num;
use rand;
use std::mem;
use rand::distributions::{ Normal, IndependentSample, Range };
use rand::{ ThreadRng };
use num::{ Integer };
use std::default::{ Default };
use std::fmt::{ Display };


pub struct Pyramidal {
	depth: u8,
	width: u32,
	kern_cycle: ocl::Kernel,
	kern_activate: ocl::Kernel,
	//kern_axn_cycle: ocl::Kernel,
	axn_row_base: u8,
	//den_prox_row: u8, 
	pub states: Envoy<ocl::cl_uchar>,
	pub dens: Dendrites,
}

impl Pyramidal {
	pub fn new(width: u32, region: &CorticalRegion, axons: &Axons, ocl: &Ocl) -> Pyramidal {

		let axn_row_base = region.base_row_cell_kind(&CellKind::Pyramidal);
		let depth: u8 = region.depth_cell_kind(&CellKind::Pyramidal);
		//let col_input_layer = region.col_input_layer().expect("Pyramidal::new()");
		//let den_prox_row = region.row_ids(vec![col_input_layer.name])[0];
		
		//print!("\n### Pyramidal: Proximal Dendrite Row: {}", den_prox_row);

		let states = Envoy::<ocl::cl_uchar>::new(width, depth, common::STATE_ZERO, ocl);

		let dens = Dendrites::new(width, depth, DendriteKind::Distal, CellKind::Pyramidal, common::DENDRITES_PER_CELL_DISTAL_LOG2, region, axons, ocl);

		let kern_cycle = ocl.new_kernel("pyr_cycle", 
			WorkSize::TwoDim(depth as usize, width as usize))
			.arg_env(&dens.states)
			.arg_scl(axn_row_base)
			.arg_env(&states)
			.arg_env(&axons.states)
		;

		/*let kern_axn_cycle = ocl.new_kernel("pyr_axn_cycle", 
			WorkSize::TwoDim(depth as usize, width as usize))
			.arg_scl(axn_row_base)
			.arg_env(&states)
			.arg_env(&axons.states)
		;*/

		let kern_activate = ocl.new_kernel("pyr_activate", 
			WorkSize::TwoDim(depth as usize, width as usize))
		;


		Pyramidal {
			depth: depth,
			width: width,
			kern_cycle: kern_cycle,
			kern_activate: kern_activate,
			//kern_axn_cycle: kern_axn_cycle,
			axn_row_base: axn_row_base,
			//den_prox_row: den_prox_row,
			states: states,
			dens: dens,
		}
	}

	pub fn init_kernels(&mut self, cols: &Columns, axns: &Axons) {
		self.kern_activate.new_arg_envoy(&cols.states);
		self.kern_activate.new_arg_envoy(&cols.cel_status);
		//self.kern_activate.new_arg_scalar(self.depth);
		self.kern_activate.new_arg_scalar(self.axn_row_base);
		self.kern_activate.new_arg_envoy(&self.states);
		self.kern_activate.new_arg_envoy(&axns.states);
	}

	pub fn cycle(&self) {
		self.dens.cycle();
		self.kern_cycle.enqueue();
		//self.kern_axn_cycle.enqueue();
	}

	pub fn activate(&self) {
		self.kern_activate.enqueue();
	}
}
