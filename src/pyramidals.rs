use cmn;
use ocl::{ self, Ocl, WorkSize };
use ocl::{ Envoy };
use proto::areas::{ ProtoAreas, Width };
use proto::regions::{ ProtoRegion, ProtoRegionKind };
use proto::cell::{ CellKind, Protocell, DendriteKind };
use synapses::{ Synapses };
use dendrites::{ Dendrites };
use region_cells::{ Aux };
use peak_column::{ PeakColumn };
use minicolumns::{ MiniColumns };
use axons::{ Axons };


use num;
use rand;
use std::mem;
use rand::distributions::{ Normal, IndependentSample, Range };
use rand::{ ThreadRng, Rng };
use num::{ Integer };
use std::default::{ Default };
use std::fmt::{ Display };


/* Pyramidal
	flag_sets: 0b10000000 (0x80) -> previously active

*/
pub struct Pyramidal {
	depth: u8,
	width: u32,
	kern_ltp: ocl::Kernel,
	kern_cycle: ocl::Kernel,
	kern_activate: ocl::Kernel,
	//kern_axn_cycle: ocl::Kernel,
	axn_row_base: u8,
	//den_prox_row: u8, 
	rng: rand::XorShiftRng,
	//regrow_counter: usize,
	pub depols: Envoy<ocl::cl_uchar>,
	pub best_den_ids: Envoy<ocl::cl_uchar>,
	pub best_den_states: Envoy<ocl::cl_uchar>,
	pub prev_lrnd_den_ids: Envoy<ocl::cl_uchar>,
	pub flag_sets: Envoy<ocl::cl_uchar>,
	pub dens: Dendrites,
}

impl Pyramidal {
	pub fn new(width: u32, region: &ProtoRegion, axons: &Axons, aux: &Aux, ocl: &Ocl) -> Pyramidal {

		let axn_row_base = region.base_row_cell_kind(&CellKind::Pyramidal);
		let depth: u8 = region.depth_cell_kind(&CellKind::Pyramidal);
		let dens_per_cel_l2 = cmn::DENDRITES_PER_CELL_DISTAL_LOG2;
		let syns_per_cel_l2 = cmn::SYNAPSES_PER_DENDRITE_DISTAL_LOG2;
		//let col_input_layer = region.col_input_layer().expect("Pyramidal::new()");
		//let den_prox_row = region.row_ids(vec![col_input_layer.name])[0];
		
		//print!("\n### Pyramidal: Proximal Dendrite Row: {}", den_prox_row);

		let depols = Envoy::<ocl::cl_uchar>::new(width, depth, cmn::STATE_ZERO, ocl);

		let best_den_ids = Envoy::<ocl::cl_uchar>::new(width, depth, cmn::STATE_ZERO, ocl);
		let best_den_states = Envoy::<ocl::cl_uchar>::new(width, depth, cmn::STATE_ZERO, ocl);
		let prev_lrnd_den_ids = Envoy::<ocl::cl_uchar>::new(width, depth, cmn::STATE_ZERO, ocl);
		let flag_sets = Envoy::<ocl::cl_uchar>::new(width, depth, cmn::STATE_ZERO, ocl);

		let dens = Dendrites::new(width, depth, DendriteKind::Distal, CellKind::Pyramidal, dens_per_cel_l2, region, axons, aux, ocl);

		
		let kern_cycle = ocl.new_kernel("pyr_cycle", 
			WorkSize::TwoDim(depth as usize, width as usize))
			.arg_env(&dens.states)
			//.arg_scl(axn_row_base)
			.arg_env(&best_den_ids)
			.arg_env(&best_den_states)
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


		assert!(width % cmn::MINIMUM_WORKGROUP_SIZE == 0);
		let cels_per_wi: u32 = width / cmn::MINIMUM_WORKGROUP_SIZE;
		let axn_idx_base: u32 = (axn_row_base as u32 * width) + cmn::SYNAPSE_REACH;
		//println!("\n### PYRAMIDAL AXON IDX BASE: {} ###", axn_idx_base);

		let kern_ltp = ocl.new_kernel("pyrs_ltp_unoptd", 
			WorkSize::TwoDim(depth as usize, cmn::MINIMUM_WORKGROUP_SIZE as usize))
			.arg_env(&axons.states)
			//.arg_env(&depols)
			.arg_env(&best_den_ids)
			.arg_env(&dens.states)
			.arg_env(&dens.syns.states)
			.arg_scl(axn_idx_base)
			.arg_scl(syns_per_cel_l2)
			.arg_scl(dens_per_cel_l2)
			.arg_scl(cels_per_wi)
			.arg_scl_named(0u32, "rnd")
			//.arg_env(&aux.ints_1)
			.arg_env(&dens.syns.flag_sets)
			.arg_env(&flag_sets)
			.arg_env(&prev_lrnd_den_ids)
			.arg_env(&dens.syns.strengths)
			//.arg_env(&axons.states)
		;


		Pyramidal {
			depth: depth,
			width: width,
			kern_ltp: kern_ltp,
			kern_cycle: kern_cycle,
			kern_activate: kern_activate,
			//kern_axn_cycle: kern_axn_cycle,
			axn_row_base: axn_row_base,
			//den_prox_row: den_prox_row,
			rng: rand::weak_rng(),
			//regrow_counter: 0usize,
			depols: depols,
			best_den_ids: best_den_ids,
			best_den_states: best_den_states,
			prev_lrnd_den_ids: prev_lrnd_den_ids,
			flag_sets: flag_sets,
			dens: dens,
		}
	}

	pub fn init_kernels(&mut self, cols: &MiniColumns, axns: &Axons, aux: &Aux) {
		self.kern_activate.new_arg_envoy(&cols.states);
		self.kern_activate.new_arg_envoy(&cols.cels_status);
		self.kern_activate.new_arg_envoy(&cols.best_col_den_states);
		self.kern_activate.new_arg_envoy(&self.best_den_ids);
		self.kern_activate.new_arg_envoy(&self.dens.states);
		self.kern_activate.new_arg_scalar(self.axn_row_base);
		//self.kern_activate.new_arg_envoy(&aux.ints_0);
		self.kern_activate.new_arg_envoy(&self.flag_sets);
		self.kern_activate.new_arg_envoy(&self.depols);	
		self.kern_activate.new_arg_envoy(&axns.states);
	}

	pub fn activate(&mut self, ltp: bool) {
		self.kern_activate.enqueue();

		if ltp { 
			self.ltp(); 
		}
	}

	pub fn ltp(&mut self) {
		self.kern_ltp.set_named_arg("rnd", self.rng.gen::<u32>());
		self.kern_ltp.enqueue();
	}

	pub fn regrow(&mut self, region: &ProtoRegion) {

		/*self.regrow_counter += 1;

		if self.regrow_counter >= cmn::SYNAPSE_DECAY_INTERVAL {
			
			self.regrow_counter = 0;
		}*/


		self.dens.regrow(region);
	}

	pub fn cycle(&self) {
		self.dens.cycle();
		self.kern_cycle.enqueue();
			//self.kern_axn_cycle.enqueue();
	}

	pub fn axn_output_range(&self) -> (usize, usize) {
		let start = (self.axn_row_base as usize * self.width as usize) + cmn::SYNAPSE_REACH as usize;
		(start, start + ((self.width * self.depth as u32) - 1) as usize)
	}

	pub fn confab(&mut self) {
		self.depols.read();
		self.best_den_ids.read();
		self.dens.confab();
	} 

	pub fn width(&self) -> u32 {
		self.width
	}
}
