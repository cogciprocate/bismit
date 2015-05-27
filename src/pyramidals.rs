use num;
use rand;
use std::mem;
use rand::distributions::{ Normal, IndependentSample, Range };
use rand::{ ThreadRng, Rng };
use num::{ Integer };
use std::default::{ Default };
use std::fmt::{ Display };

use cmn;
use ocl::{ self, Ocl, WorkSize, Envoy, CorticalDimensions };
use proto::areas::{ Protoareas };
use proto::regions::{ Protoregion, ProtoregionKind };
use proto::cell::{ ProtocellKind, Protocell, DendriteKind };
use synapses::{ Synapses };
use dendrites::{ Dendrites };
use cortical_area:: { Aux };
use peak_column::{ PeakColumns };
use minicolumns::{ Minicolumns };
use axons::{ Axons };
use spiny_stellates::{ SpinyStellateCellularLayer };



/* PyramidalCellularLayer
	flag_sets: 0b10000000 (0x80) -> previously active

*/
pub struct PyramidalCellularLayer {
	name: &'static str,
	dims: CorticalDimensions,
	kern_ltp: ocl::Kernel,
	kern_cycle: ocl::Kernel,
	kern_activate: ocl::Kernel,
	//kern_axn_cycle: ocl::Kernel,
	axn_slice_base: u8,
	//den_prox_slice: u8, 
	rng: rand::XorShiftRng,
	//regrow_counter: usize,
	pub preds: Envoy<ocl::cl_uchar>,
	pub best1_den_ids: Envoy<ocl::cl_uchar>,
	pub best1_den_states: Envoy<ocl::cl_uchar>,
	pub best2_den_ids: Envoy<ocl::cl_uchar>,
	pub best2_den_states: Envoy<ocl::cl_uchar>,
	pub prev_best1_den_ids: Envoy<ocl::cl_uchar>,
	pub flag_sets: Envoy<ocl::cl_uchar>,
	pub energies: Envoy<ocl::cl_uchar>,
	pub dens: Dendrites,
}
// protocell: &Protocell,
impl PyramidalCellularLayer {
	pub fn new(name: &'static str, mut dims: CorticalDimensions, region: &Protoregion, 
					axons: &Axons, aux: &Aux, ocl: &Ocl
	) -> PyramidalCellularLayer {

		let axn_slice_base = region.base_slice_cell_kind(&ProtocellKind::Pyramidal);
		//dims.depth() = region.depth_cell_kind(&ProtocellKind::Pyramidal);
		let dens_per_cel_l2 = cmn::DENDRITES_PER_CELL_DISTAL_LOG2; // SET IN PROTOAREA
		let syns_per_cel_l2 = cmn::SYNAPSES_PER_DENDRITE_DISTAL_LOG2; // SET IN PROTOAREA
		//let col_input_layer = region.col_input_layer().expect("PyramidalCellularLayer::new()");
		//let den_prox_slice = region.slice_ids(vec![col_input_layer.name])[0];
		
		//print!("\n### PyramidalCellularLayer: Proximal Dendrite Row: {}", den_prox_slice);
		print!("\n##### PYRAMIDAL dims: {:?}", dims);

		let preds = Envoy::<ocl::cl_uchar>::new(dims, cmn::STATE_ZERO, ocl);

		let best1_den_ids = Envoy::<ocl::cl_uchar>::new(dims, cmn::STATE_ZERO, ocl);
		let best1_den_states = Envoy::<ocl::cl_uchar>::new(dims, cmn::STATE_ZERO, ocl);
		let best2_den_ids = Envoy::<ocl::cl_uchar>::new(dims, cmn::STATE_ZERO, ocl);
		let best2_den_states = Envoy::<ocl::cl_uchar>::new(dims, cmn::STATE_ZERO, ocl);
		let prev_best1_den_ids = Envoy::<ocl::cl_uchar>::new(dims, cmn::STATE_ZERO, ocl);
		let flag_sets = Envoy::<ocl::cl_uchar>::new(dims, cmn::STATE_ZERO, ocl);
		let energies = Envoy::<ocl::cl_uchar>::new(dims, 255, ocl);

		let dens_dims = dims.clone_with_pcl2(dens_per_cel_l2 as i8);
		let dens = Dendrites::new(dens_dims, DendriteKind::Distal, ProtocellKind::Pyramidal, region, axons, aux, ocl);

		
		let kern_cycle = ocl.new_kernel("pyr_cycle", 
			WorkSize::TwoDim(dims.depth() as usize, dims.columns() as usize))
			.arg_env(&dens.states)
			.arg_env(&dens.states_raw)
			//.arg_scl(axn_slice_base)
			.arg_env(&energies)
			.arg_env(&best1_den_ids)
			.arg_env(&best1_den_states)
			.arg_env(&best2_den_ids)
			.arg_env(&best2_den_states)
			.arg_env(&preds) 		// v.N1
			//.arg_env(&axons.states)
		;

		/*let kern_axn_cycle = ocl.new_kernel("pyr_axn_cycle", 
			WorkSize::TwoDim(dims.depth() as usize, dims.width as usize))
			.arg_scl(axn_slice_base)
			.arg_env(&preds)
			.arg_env(&axons.states)
		;*/

		let kern_activate = ocl.new_kernel("pyr_activate", 
			WorkSize::TwoDim(dims.depth() as usize, dims.columns() as usize))
		;


		assert!(dims.columns() % cmn::MINIMUM_WORKGROUP_SIZE == 0);
		let cels_per_wi: u32 = dims.columns() / cmn::MINIMUM_WORKGROUP_SIZE;
		let axn_idx_base: u32 = (axn_slice_base as u32 * dims.columns()) + cmn::SYNAPSE_REACH_LIN;
		//println!("\n### PYRAMIDAL AXON IDX BASE: {} ###", axn_idx_base);

		let kern_ltp = ocl.new_kernel("pyrs_ltp_unoptd", 
			WorkSize::TwoDim(dims.depth() as usize, cmn::MINIMUM_WORKGROUP_SIZE as usize))
			.arg_env(&axons.states)
			.arg_env(&preds)
			.arg_env(&best1_den_ids)
			.arg_env(&best2_den_ids) // ***** SLATED FOR REMOVAL
			.arg_env(&dens.states)
			.arg_env(&dens.syns.states)
			.arg_scl(axn_idx_base)
			.arg_scl(syns_per_cel_l2 as u32)
			.arg_scl(dens_per_cel_l2 as u32)
			.arg_scl(cels_per_wi)
			.arg_scl_named(0u32, "rnd")
			//.arg_env(&aux.ints_1)
			.arg_env(&dens.syns.flag_sets)
			.arg_env(&flag_sets)
			//.arg_env(&prev_best1_den_ids)
			.arg_env(&dens.syns.strengths)
			//.arg_env(&axons.states)
		;


		PyramidalCellularLayer {
			name: name,
			dims: dims,
			kern_ltp: kern_ltp,
			kern_cycle: kern_cycle,
			kern_activate: kern_activate,
			//kern_axn_cycle: kern_axn_cycle,
			axn_slice_base: axn_slice_base,
			//den_prox_slice: den_prox_slice,
			rng: rand::weak_rng(),
			//regrow_counter: 0usize,
			preds: preds,
			best1_den_ids: best1_den_ids,
			best1_den_states: best1_den_states,
			best2_den_ids: best2_den_ids,
			best2_den_states: best2_den_states,
			prev_best1_den_ids: prev_best1_den_ids,
			flag_sets: flag_sets,
			energies: energies,
			dens: dens,
		}
	}

	pub fn init_kernels(&mut self, mcols: &Minicolumns, ssts: &SpinyStellateCellularLayer, axns: &Axons, aux: &Aux) {
		self.kern_activate.new_arg_envoy(&ssts.dens.states);
		self.kern_activate.new_arg_envoy(&mcols.cels_status);
		self.kern_activate.new_arg_envoy(&mcols.best_col_den_states);
		self.kern_activate.new_arg_envoy(&self.best1_den_ids);
		self.kern_activate.new_arg_envoy(&self.dens.states);
		self.kern_activate.new_arg_scalar(self.axn_slice_base);
		//self.kern_activate.new_arg_envoy(&aux.ints_0);
		//self.kern_activate.new_arg_envoy(&self.energies);
		self.kern_activate.new_arg_envoy(&self.flag_sets);
		self.kern_activate.new_arg_envoy(&self.preds);	
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

	pub fn regrow(&mut self, region: &Protoregion) {

		self.dens.regrow(region);
	}

	pub fn cycle(&self) {
		self.dens.cycle();
		self.kern_cycle.enqueue();
	}

	pub fn axn_output_range(&self) -> (usize, usize) {
		let start = (self.axn_slice_base as usize * self.dims.columns() as usize) + cmn::SYNAPSE_REACH_LIN as usize;
		(start, start + ((self.dims.columns() * self.dims.depth() as u32) - 1) as usize)
	}

	pub fn confab(&mut self) {
		self.preds.read();
		self.best1_den_ids.read();
		self.dens.confab();
	} 

}
