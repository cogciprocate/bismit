use common;
use ocl::{ self, Ocl, WorkSize };
use ocl::{ Envoy };
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
use rand::{ ThreadRng, Rng };
use num::{ Integer };
use std::default::{ Default };
use std::fmt::{ Display };


pub struct Pyramidals {
	depth: u8,
	width: u32,
	kern_learn: ocl::Kernel,
	kern_cycle: ocl::Kernel,
	kern_activate: ocl::Kernel,
	//kern_axn_cycle: ocl::Kernel,
	axn_row_base: u8,
	//den_prox_row: u8, 
	rng: rand::XorShiftRng,
	regrow_counter: usize,
	pub depols: Envoy<ocl::cl_uchar>,
	pub best_den_ids: Envoy<ocl::cl_uchar>,
	pub dens: Dendrites,
}

impl Pyramidals {
	pub fn new(width: u32, region: &CorticalRegion, axons: &Axons, aux: &Aux, ocl: &Ocl) -> Pyramidals {

		let axn_row_base = region.base_row_cell_kind(&CellKind::Pyramidals);
		let depth: u8 = region.depth_cell_kind(&CellKind::Pyramidals);
		let dens_per_cel_l2 = common::DENDRITES_PER_CELL_DISTAL_LOG2;
		let syns_per_cel_l2 = common::SYNAPSES_PER_DENDRITE_DISTAL_LOG2;
		//let col_input_layer = region.col_input_layer().expect("Pyramidals::new()");
		//let den_prox_row = region.row_ids(vec![col_input_layer.name])[0];
		
		//print!("\n### Pyramidals: Proximal Dendrite Row: {}", den_prox_row);

		let depols = Envoy::<ocl::cl_uchar>::new(width, depth, common::STATE_ZERO, ocl);

		let best_den_ids = Envoy::<ocl::cl_uchar>::new(width, depth, common::STATE_ZERO, ocl);

		let dens = Dendrites::new(width, depth, DendriteKind::Distal, CellKind::Pyramidals, dens_per_cel_l2, region, axons, aux, ocl);

		
		let kern_cycle = ocl.new_kernel("pyr_cycle", 
			WorkSize::TwoDim(depth as usize, width as usize))
			.arg_env(&dens.states)
			//.arg_scl(axn_row_base)
			.arg_env(&best_den_ids)
			.arg_env(&depols) 		// v.N1
			//.arg_env(&axons.states)
		;

		/*let kern_axn_cycle = ocl.new_kernel("pyr_axn_cycle", 
			WorkSize::TwoDim(depth as usize, width as usize))
			.arg_scl(axn_row_base)
			.arg_env(&depols)
			.arg_env(&axons.states)
		;*/

		let kern_activate = ocl.new_kernel("pyr_activate", 
			WorkSize::TwoDim(depth as usize, width as usize))
		;


		assert!(width % common::MINIMUM_WORKGROUP_SIZE == 0);
		let cels_per_grp: u32 = width / common::MINIMUM_WORKGROUP_SIZE;
		let axn_idx_base: u32 = (axn_row_base as u32 * width) + common::SYNAPSE_REACH;
		//println!("\n### PYRAMIDAL AXON IDX BASE: {} ###", axn_idx_base);

		let kern_learn = ocl.new_kernel("pyrs_learn_unoptd", 
			WorkSize::TwoDim(depth as usize, common::MINIMUM_WORKGROUP_SIZE as usize))
			.arg_env(&axons.states)
			//.arg_env(&depols)
			.arg_env(&best_den_ids)
			.arg_env(&dens.states)
			.arg_env(&dens.syns.states)
			.arg_scl(axn_idx_base)
			.arg_scl(syns_per_cel_l2)
			.arg_scl(dens_per_cel_l2)
			.arg_scl(cels_per_grp)
			.arg_scl_named(0u32, "rnd")
			//.arg_env(&aux.ints_1)
			.arg_env(&dens.syns.strengths)
			//.arg_env(&axons.states)
		;


		Pyramidals {
			depth: depth,
			width: width,
			kern_learn: kern_learn,
			kern_cycle: kern_cycle,
			kern_activate: kern_activate,
			//kern_axn_cycle: kern_axn_cycle,
			axn_row_base: axn_row_base,
			//den_prox_row: den_prox_row,
			rng: rand::weak_rng(),
			regrow_counter: 0usize,
			depols: depols,
			best_den_ids: best_den_ids,
			dens: dens,
		}
	}

	pub fn init_kernels(&mut self, cols: &Columns, axns: &Axons, aux: &Aux) {
		self.kern_activate.new_arg_envoy(&cols.states);
		self.kern_activate.new_arg_envoy(&cols.cels_status);
		self.kern_activate.new_arg_scalar(self.axn_row_base);
		self.kern_activate.new_arg_envoy(&aux.ints_0);
		self.kern_activate.new_arg_envoy(&self.depols);	
		self.kern_activate.new_arg_envoy(&axns.states);
	}

	pub fn activate(&mut self) {
		self.kern_activate.enqueue();

		self.kern_learn.set_named_arg("rnd", self.rng.gen::<u32>());
		self.kern_learn.enqueue();

		self.regrow_counter += 1;

		if self.regrow_counter >= common::SYNAPSE_DECAY_INTERVAL {
			self.dens.regrow();
			self.regrow_counter = 0;
		}
	}

	pub fn cycle(&self) {
		self.dens.cycle();
		self.kern_cycle.enqueue();
		//self.kern_axn_cycle.enqueue();
	}

	pub fn axn_output_range(&self) -> (usize, usize) {
		let start = (self.axn_row_base as usize * self.width as usize) + common::SYNAPSE_REACH as usize;
		(start, start + ((self.width * self.depth as u32) - 1) as usize)
	}

	pub fn confab(&mut self) {
		self.depols.read();
		self.best_den_ids.read();
	} 

	pub fn width(&self) -> u32 {
		self.width
	}
	
}