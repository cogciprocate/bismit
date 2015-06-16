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
use iinn::{ InhibitoryInterneuronNetwork };
use minicolumns::{ Minicolumns };
use axons::{ Axons };
use spiny_stellates::{ SpinyStellateCellularLayer };



/* PyramidalCellularLayer
	flag_sets: 0b10000000 (0x80) -> previously active

*/
pub struct PyramidalCellularLayer {
	layer_name: &'static str,
	dims: CorticalDimensions,
	protocell: Protocell,
	kern_ltp: ocl::Kernel,
	kern_cycle: ocl::Kernel,
	kern_activate: ocl::Kernel,		// <<<<< MOVE TO MCOL
	//kern_axn_cycle: ocl::Kernel,
	axn_base_slice: u8,
	axn_idz: u32,
	//den_prox_slice: u8, 
	rng: rand::XorShiftRng,
	//regrow_counter: usize,
	pub preds: Envoy<ocl::cl_uchar>,
	pub best1_den_ids: Envoy<ocl::cl_uchar>,
	pub best1_den_states: Envoy<ocl::cl_uchar>,
	pub best2_den_ids: Envoy<ocl::cl_uchar>,
	pub best2_den_states: Envoy<ocl::cl_uchar>,
	//pub prev_best1_den_ids: Envoy<ocl::cl_uchar>,
	pub flag_sets: Envoy<ocl::cl_uchar>,
	pub energies: Envoy<ocl::cl_uchar>, // <<<<< SLATED FOR REMOVAL
	pub dens: Dendrites,
}
// protocell: &Protocell,
impl PyramidalCellularLayer {
	pub fn new(layer_name: &'static str, mut dims: CorticalDimensions, protocell: Protocell, region: &Protoregion, axons: &Axons, aux: &Aux, ocl: &Ocl
	) -> PyramidalCellularLayer {

		let axn_base_slices = region.slice_ids(vec![layer_name]);
		let axn_base_slice = axn_base_slices[0];
		let axn_idz = cmn::axn_idx_2d(axn_base_slice, dims.columns(), region.hrz_demarc());

		let dens_per_cel_l2 = protocell.dens_per_cel_l2;
		let syns_per_den_l2 = protocell.syns_per_den_l2;
		let syns_per_cel_l2 = dens_per_cel_l2 + syns_per_den_l2;


		//dims.depth() = region.depth_cell_kind(&ProtocellKind::Pyramidal);
		//let dens_per_cel_l2 = cmn::DENDRITES_PER_CELL_DISTAL_LOG2; // SET IN PROTOAREA
		//let syns_per_cel_l2 = cmn::SYNAPSES_PER_DENDRITE_DISTAL_LOG2; // SET IN PROTOAREA
		//let spt_asc_layer = region.spt_asc_layer().expect("PyramidalCellularLayer::new()");
		//let den_prox_slice = region.slice_ids(vec![spt_asc_layer.name])[0];
		
		//print!("\n### PyramidalCellularLayer: Proximal Dendrite Row: {}", den_prox_slice);
		print!("\n      PYRAMIDALS::NEW(): dims: {:?}, axn_base_slice: {}", dims, axn_base_slice);

		let preds = Envoy::<ocl::cl_uchar>::new(dims, cmn::STATE_ZERO, ocl);

		let best1_den_ids = Envoy::<ocl::cl_uchar>::new(dims, cmn::STATE_ZERO, ocl);
		let best1_den_states = Envoy::<ocl::cl_uchar>::new(dims, cmn::STATE_ZERO, ocl);
		let best2_den_ids = Envoy::<ocl::cl_uchar>::new(dims, cmn::STATE_ZERO, ocl);
		let best2_den_states = Envoy::<ocl::cl_uchar>::new(dims, cmn::STATE_ZERO, ocl);
		let prev_best1_den_ids = Envoy::<ocl::cl_uchar>::new(dims, cmn::STATE_ZERO, ocl);
		let flag_sets = Envoy::<ocl::cl_uchar>::new(dims, cmn::STATE_ZERO, ocl);
		let energies = Envoy::<ocl::cl_uchar>::new(dims, 255, ocl);

		let dens_dims = dims.clone_with_pcl2(dens_per_cel_l2 as i8);
		let dens = Dendrites::new(dens_dims, protocell.clone(), DendriteKind::Distal, ProtocellKind::Pyramidal, region, axons, aux, ocl);

		
		let kern_cycle = ocl.new_kernel("pyr_cycle_working", 
			WorkSize::TwoDim(dims.depth() as usize, dims.columns() as usize))
			.arg_env(&dens.states)
			.arg_env(&dens.states_raw)
			//.arg_scl(axn_base_slice)
			.arg_scl(dens_per_cel_l2)
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
			.arg_scl(axn_base_slice)
			.arg_env(&preds)
			.arg_env(&axons.states)
		;*/

		let kern_activate = ocl.new_kernel("pyr_activate",		 // <<<<< MOVE TO MCOL
			WorkSize::TwoDim(dims.depth() as usize, dims.columns() as usize))
		;


		assert!(dims.columns() % cmn::MINIMUM_WORKGROUP_SIZE == 0);
		let cels_per_wi: u32 = dims.columns() / cmn::MINIMUM_WORKGROUP_SIZE;
		let axn_idx_base: u32 = (axn_base_slice as u32 * dims.columns()) + cmn::SYNAPSE_REACH_LIN; // NEEDS UPDATE TO NEW SYSTEM
		//println!("\n### PYRAMIDAL AXON IDX BASE: {} ###", axn_idx_base);
		assert!(axn_idx_base == axn_idz);

		let kern_ltp = ocl.new_kernel("pyrs_ltp_unoptd", 
			WorkSize::TwoDim(dims.depth() as usize, cmn::MINIMUM_WORKGROUP_SIZE as usize))
			.arg_env(&axons.states)
			.arg_env(&preds)
			.arg_env(&best1_den_ids)
			.arg_env(&best2_den_ids) // <<<<< SLATED FOR REMOVAL
			.arg_env(&dens.states)
			.arg_env(&dens.syns.states)
			.arg_scl(axn_idx_base)
			.arg_scl(syns_per_den_l2 as u32)
			.arg_scl(dens_per_cel_l2 as u32)
			.arg_scl(cels_per_wi)
			.arg_scl_named::<u32>("rnd", None)			
			.arg_env(&dens.syns.flag_sets)
			.arg_env(&flag_sets)
			//.arg_env(&prev_best1_den_ids)
			//.arg_env(&aux.ints_0)
			//.arg_env(&aux.ints_1)
			.arg_env(&dens.syns.strengths)
			//.arg_env(&axons.states)
		;


		PyramidalCellularLayer {
			layer_name: layer_name,
			dims: dims,
			protocell: protocell,
			kern_ltp: kern_ltp,
			kern_cycle: kern_cycle,
			kern_activate: kern_activate,		// <<<<< MOVE TO MCOL
			//kern_axn_cycle: kern_axn_cycle,
			axn_base_slice: axn_base_slice,
			axn_idz: axn_idz,
			//den_prox_slice: den_prox_slice,
			rng: rand::weak_rng(),
			//regrow_counter: 0usize,
			preds: preds,
			best1_den_ids: best1_den_ids,
			best1_den_states: best1_den_states,
			best2_den_ids: best2_den_ids,
			best2_den_states: best2_den_states,
			//prev_best1_den_ids: prev_best1_den_ids,
			flag_sets: flag_sets,
			energies: energies,
			dens: dens,
		}
	}

	// <<<<< MOVE TO MCOL >>>>>
	pub fn init_kernels(&mut self, mcols: &Minicolumns, ssts: &Box<SpinyStellateCellularLayer>, axns: &Axons, aux: &Aux) {
		let (ssts_axn_idz, _) = ssts.axn_range();
		//println!("\n##### Pyramidals::init_kernels(): ssts_axn_idz: {}", ssts_axn_idz as u32);

		//self.kern_activate.new_arg_envoy(Some(&ssts.soma()));
		self.kern_activate.new_arg_envoy(Some(&mcols.cels_status));
		self.kern_activate.new_arg_envoy(Some(&mcols.best_pyr_den_states));
		self.kern_activate.new_arg_envoy(Some(&self.best1_den_ids));
		self.kern_activate.new_arg_envoy(Some(&self.dens.states));

		self.kern_activate.new_arg_scalar(Some(ssts_axn_idz as u32));
		self.kern_activate.new_arg_scalar(Some(self.axn_base_slice));
		self.kern_activate.new_arg_scalar(Some(self.protocell.dens_per_cel_l2));

		//self.kern_activate.new_arg_envoy(&self.energies);
		self.kern_activate.new_arg_envoy(Some(&self.flag_sets));
		self.kern_activate.new_arg_envoy(Some(&self.preds));	
		//self.kern_activate.new_arg_envoy(Some(&aux.ints_0));
		self.kern_activate.new_arg_envoy(Some(&axns.states));
	}

	pub fn activate(&mut self) {
		self.kern_activate.enqueue(); 	// <<<<< MOVE TO MCOL

		/*if ltp { 
			self.ltp(); 
		}*/
	}

	pub fn learn(&mut self) {
		self.kern_ltp.set_arg_scl_named("rnd", self.rng.gen::<u32>());
		self.kern_ltp.enqueue();
	}

	pub fn regrow(&mut self) {
		self.dens.regrow();
	}

	pub fn cycle(&mut self) {
		//self.activate(ltp);
		self.dens.cycle();
		self.kern_cycle.enqueue();
	}

	pub fn confab(&mut self) {
		self.preds.read();
		self.best1_den_ids.read();
		self.best1_den_states.read();
		self.best2_den_ids.read();
		self.best2_den_states.read();
		self.flag_sets.read();
		self.energies.read();

		self.dens.confab();
	}

	pub fn soma(&self) -> &Envoy<u8> {
		&self.preds
	}

	pub fn soma_mut(&mut self) -> &mut Envoy<u8> {
		&mut self.preds
	}

	// CYCLE_SELF_ONLY(): USED BY TESTS
	pub fn cycle_self_only(&self) {
		self.kern_cycle.enqueue();
	}

	// AXN_OUTPUT_RANGE(): USED BY OUTPUT_CZAR (DEBUGGING/TESTING)
/*	pub fn axn_output_range(&self) -> (usize, usize) {
		let start = (self.axn_base_slice as usize * self.dims.columns() as usize) + cmn::SYNAPSE_REACH_LIN as usize;
		(start, start + ((self.dims.columns() * self.dims.depth() as u32) - 1) as usize)
	}*/

	pub fn dims(&self) -> &CorticalDimensions {
		&self.dims
	}

	pub fn axn_range(&self) -> (usize, usize) {
		let ssts_axn_idn = self.axn_idz + (self.dims.per_slice());

		(self.axn_idz as usize, ssts_axn_idn as usize)
	}

	pub fn axn_base_slice(&self) -> u8 {
		self.axn_base_slice
	}

	pub fn layer_name(&self) -> &'static str {
		self.layer_name
	}


	pub fn print_cel(&mut self, cel_idx: usize) {
		let emsg = "PyramidalCellularLayer::print_cel()";

		self.confab();

		let cel_den_idz = (cel_idx << self.dens.dims().per_cel_l2_left().expect(emsg)) as usize;
		let cel_syn_idz = (cel_idx << self.dens.syns.dims().per_cel_l2_left().expect(emsg)) as usize;

		let dens_per_cel = self.dens.dims().per_cel().expect(emsg) as usize;
		let syns_per_cel = self.dens.syns.dims().per_cel().expect(emsg) as usize;

		let cel_den_range = cel_den_idz..(cel_den_idz + dens_per_cel);
		let cel_syn_range = cel_syn_idz..(cel_syn_idz + syns_per_cel);

		print!("\nPrinting Pyramidal Cell:");
		print!("\n   preds[{}]: {}", cel_idx, self.preds[cel_idx]);
		print!("\n   best1_den_ids[{}]: {}", cel_idx, self.best1_den_ids[cel_idx]);
		print!("\n   best1_den_states[{}]: {}", cel_idx, self.best1_den_states[cel_idx]);
		print!("\n   best2_den_ids[{}]: {}", cel_idx, self.best2_den_ids[cel_idx]);
		print!("\n   best2_den_states[{}]: {}", cel_idx, self.best2_den_states[cel_idx]);
		print!("\n   flag_sets[{}]: {}", cel_idx, self.flag_sets[cel_idx]);
		print!("\n   energies[{}]: {}", cel_idx, self.energies[cel_idx]);

		print!("\n");

		print!("\ndens.states[{:?}]: ", cel_den_range.clone()); 
		cmn::print_vec_simple(&self.dens.states.vec[cel_den_range.clone()]);

		print!("\ndens.syns.states[{:?}]: ", cel_syn_range.clone()); 
		cmn::print_vec_simple(&self.dens.syns.states.vec[cel_syn_range.clone()]);

		print!("\ndens.syns.strengths[{:?}]: ", cel_syn_range.clone()); 
		cmn::print_vec_simple(&self.dens.syns.strengths.vec[cel_syn_range.clone()]);

		print!("\ndens.src_col_xy_offs[{:?}]: ", cel_syn_range.clone()); 
		cmn::print_vec_simple(&self.dens.syns.src_col_xy_offs.vec[cel_syn_range.clone()]);
	}

	pub fn set_all_to_zero(&mut self) {
		self.preds.set_all_to(0);
		self.best1_den_ids.set_all_to(0);
		self.best1_den_states.set_all_to(0);
		self.best2_den_ids.set_all_to(0);
		self.best2_den_states.set_all_to(0);
		self.flag_sets.set_all_to(0);
		self.energies.set_all_to(0);
	}

}
