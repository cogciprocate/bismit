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
	axn_base_slc: u8,
	axn_idz: u32,
	//den_prox_slc: u8, 
	rng: rand::XorShiftRng,
	den_tufts_per_cel: u32,
	//regrow_counter: usize,
	pub preds: Envoy<ocl::cl_uchar>,
	pub best_den_ids: Envoy<ocl::cl_uchar>,
	pub best_den_states: Envoy<ocl::cl_uchar>,
	//pub best2_den_ids: Envoy<ocl::cl_uchar>,		// <<<<< SLATED FOR REMOVAL
	//pub best2_den_states: Envoy<ocl::cl_uchar>,		// <<<<< SLATED FOR REMOVAL
	//pub prev_best_den_ids: Envoy<ocl::cl_uchar>,
	pub flag_sets: Envoy<ocl::cl_uchar>,
	pub energies: Envoy<ocl::cl_uchar>, // <<<<< SLATED FOR REMOVAL
	//den_tufts: Vec<Box<Dendrites>>,
	pub dens: Dendrites,
}
// protocell: &Protocell,
impl PyramidalCellularLayer {
	pub fn new(layer_name: &'static str, mut dims: CorticalDimensions, protocell: Protocell, region: &Protoregion, axons: &Axons, aux: &Aux, ocl: &Ocl
	) -> PyramidalCellularLayer {

		let axn_base_slcs = region.slc_ids(vec![layer_name]);
		let axn_base_slc = axn_base_slcs[0];
		let axn_idz = cmn::axn_idx_2d(axn_base_slc, dims.columns(), region.hrz_demarc());

		//dims.depth() = region.depth_cell_kind(&ProtocellKind::Pyramidal);
		//let dens_per_tuft_l2 = cmn::DENDRITES_PER_CELL_DISTAL_LOG2; // SET IN PROTOAREA
		//let syns_per_tuft_l2 = cmn::SYNAPSES_PER_DENDRITE_DISTAL_LOG2; // SET IN PROTOAREA
		//let spt_asc_layer = region.spt_asc_layer().expect("PyramidalCellularLayer::new()");
		//let den_prox_slc = region.slc_ids(vec![spt_asc_layer.name])[0];
		
		//print!("\n### PyramidalCellularLayer: Proximal Dendrite Row: {}", den_prox_slc);
		print!("\n      PYRAMIDALS::NEW(): layer: '{}' dims: {:?}, axn_base_slc: {}", layer_name, dims, axn_base_slc);

		let preds = Envoy::<ocl::cl_uchar>::new(dims, cmn::STATE_ZERO, ocl);

		let best_den_ids = Envoy::<ocl::cl_uchar>::new(dims, cmn::STATE_ZERO, ocl);
		let best_den_states = Envoy::<ocl::cl_uchar>::new(dims, cmn::STATE_ZERO, ocl);
		//let best2_den_ids = Envoy::<ocl::cl_uchar>::new(dims, cmn::STATE_ZERO, ocl);		// <<<<< SLATED FOR REMOVAL
		//let best2_den_states = Envoy::<ocl::cl_uchar>::new(dims, cmn::STATE_ZERO, ocl);		// <<<<< SLATED FOR REMOVAL
		let prev_best_den_ids = Envoy::<ocl::cl_uchar>::new(dims, cmn::STATE_ZERO, ocl);
		let flag_sets = Envoy::<ocl::cl_uchar>::new(dims, cmn::STATE_ZERO, ocl);
		let energies = Envoy::<ocl::cl_uchar>::new(dims, 255, ocl);


		let dens_per_tuft_l2 = protocell.dens_per_tuft_l2;
		let syns_per_den_l2 = protocell.syns_per_den_l2;
		let syns_per_tuft_l2 = dens_per_tuft_l2 + syns_per_den_l2;

		let den_tufts_per_cel = region[&layer_name].dst_src_tufts_len();

		let den_tuft_dims = dims.clone_with_pgl2(dens_per_tuft_l2 as i8).groups(den_tufts_per_cel);

		let dens = Dendrites::new(layer_name, den_tuft_dims, protocell.clone(), DendriteKind::Distal, ProtocellKind::Pyramidal, region, axons, aux, ocl);

		//let mut den_tufts = Vec::with_capacity(src_tufts.len());

		/*for tuft in src_tufts {
			den_tufts.push(Box::new(Dendrites::new(layer_name, dens_dims, protocell.clone(), DendriteKind::Distal, ProtocellKind::Pyramidal, region, axons, aux, ocl)));
		}*/

		
		let kern_cycle = ocl.new_kernel("pyr_cycle", 
			WorkSize::OneDim(dims.depth() as usize * dims.columns() as usize))
			.arg_env(&dens.states)
			.arg_env(&dens.states_raw)
			//.arg_scl(axn_base_slc)
			.arg_scl(den_tufts_per_cel)
			.arg_scl(dens_per_tuft_l2)
			//.arg_env(&energies)
			.arg_env(&best_den_ids)
			.arg_env(&best_den_states)
			//.arg_env(&best2_den_ids)				// <<<<< SLATED FOR REMOVAL
			//.arg_env(&best2_den_states)			// <<<<< SLATED FOR REMOVAL
			.arg_env(&preds) 
			//.arg_env(&axons.states)
		;

		/*let kern_axn_cycle = ocl.new_kernel("pyr_axn_cycle", 
			WorkSize::TwoDim(dims.depth() as usize, dims.width as usize))
			.arg_scl(axn_base_slc)
			.arg_env(&preds)
			.arg_env(&axons.states)
		;*/

		let kern_activate = ocl.new_kernel("pyr_activate",		 // <<<<< MOVE TO MCOL
			WorkSize::TwoDim(dims.depth() as usize, dims.columns() as usize));


		assert!(dims.columns() % cmn::MINIMUM_WORKGROUP_SIZE == 0);
		let cels_per_wi: u32 = dims.per_slc() / cmn::MINIMUM_WORKGROUP_SIZE;
		let axn_idx_base: u32 = (axn_base_slc as u32 * dims.columns()) + cmn::SYNAPSE_REACH_LIN; // NEEDS UPDATE TO NEW SYSTEM
		//println!("\n### PYRAMIDAL AXON IDX BASE: {} ###", axn_idx_base);
		assert!(axn_idx_base == axn_idz);

		let kern_ltp = ocl.new_kernel("pyrs_ltp_unoptd", 
			WorkSize::TwoDim(dims.depth() as usize, cmn::MINIMUM_WORKGROUP_SIZE as usize))
			.arg_env(&axons.states)
			.arg_env(&preds)
			.arg_env(&best_den_ids)
			//.arg_env(&best2_den_ids) // <<<<< SLATED FOR REMOVAL
			.arg_env(&dens.states)
			.arg_env(&dens.syns.states)
			.arg_scl(axn_idx_base)
			.arg_scl(syns_per_den_l2 as u32)
			.arg_scl(dens_per_tuft_l2 as u32)
			.arg_scl(cels_per_wi)
			.arg_scl_named::<u32>("rnd", None)		
			.arg_env(&dens.syns.flag_sets)
			.arg_env(&flag_sets)
			//.arg_env(&prev_best_den_ids)
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
			axn_base_slc: axn_base_slc,
			axn_idz: axn_idz,
			//den_prox_slc: den_prox_slc,
			rng: rand::weak_rng(),
			den_tufts_per_cel: den_tufts_per_cel,
			//regrow_counter: 0usize,
			preds: preds,
			best_den_ids: best_den_ids,
			best_den_states: best_den_states,
			//best2_den_ids: best2_den_ids,			// <<<<< SLATED FOR REMOVAL
			//best2_den_states: best2_den_states,		// <<<<< SLATED FOR REMOVAL
			//prev_best_den_ids: prev_best_den_ids,
			flag_sets: flag_sets,
			energies: energies,
			dens: dens,
			//den_tufts: den_tufts,
		}
	}

	// <<<<< MOVE TO MCOL >>>>>
	pub fn init_kernels(&mut self, mcols: &Minicolumns, ssts: &Box<SpinyStellateCellularLayer>, axns: &Axons, aux: &Aux) {
		let (ssts_axn_idz, _) = ssts.axn_range();
		//println!("\n##### Pyramidals::init_kernels(): ssts_axn_idz: {}", ssts_axn_idz as u32);

		//self.kern_activate.new_arg_envoy(Some(&ssts.soma()));
		self.kern_activate.new_arg_envoy(Some(&mcols.cels_status));
		self.kern_activate.new_arg_envoy(Some(&mcols.best_pyr_den_states));
		self.kern_activate.new_arg_envoy(Some(&self.best_den_ids));
		self.kern_activate.new_arg_envoy(Some(&self.dens.states));

		self.kern_activate.new_arg_scalar(Some(ssts_axn_idz as u32));
		self.kern_activate.new_arg_scalar(Some(self.axn_base_slc));
		self.kern_activate.new_arg_scalar(Some(self.protocell.dens_per_tuft_l2));

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
		self.dens_mut().regrow();
	}

	pub fn cycle(&mut self) {
		//self.activate(ltp);
		self.dens_mut().cycle();
		self.kern_cycle.enqueue();
	}

	pub fn confab(&mut self) {
		self.preds.read();
		self.best_den_ids.read();
		self.best_den_states.read();
		//self.best2_den_ids.read();		// <<<<< SLATED FOR REMOVAL
		//self.best2_den_states.read();		// <<<<< SLATED FOR REMOVAL
		self.flag_sets.read();
		self.energies.read();

		self.dens_mut().confab();
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
		let start = (self.axn_base_slc as usize * self.dims.columns() as usize) + cmn::SYNAPSE_REACH_LIN as usize;
		(start, start + ((self.dims.columns() * self.dims.depth() as u32) - 1) as usize)
	}*/

	pub fn dims(&self) -> &CorticalDimensions {
		&self.dims
	}

	pub fn axn_range(&self) -> (usize, usize) {
		let ssts_axn_idn = self.axn_idz + (self.dims.per_slc());

		(self.axn_idz as usize, ssts_axn_idn as usize)
	}

	pub fn axn_base_slc(&self) -> u8 {
		self.axn_base_slc
	}

	pub fn layer_name(&self) -> &'static str {
		self.layer_name
	}


	pub fn print_cel(&mut self, cel_idx: usize) {
		let emsg = "PyramidalCellularLayer::print_cel()";

		self.confab();

		let cel_den_idz = (cel_idx << self.dens_mut().dims().per_tuft_l2_left()) as usize;
		let cel_syn_idz = (cel_idx << self.dens_mut().syns.dims().per_tuft_l2_left()) as usize;

		let dens_per_tuft = self.dens_mut().dims().per_cel() as usize;
		let syns_per_tuft = self.dens_mut().syns.dims().per_cel() as usize;

		let cel_den_range = cel_den_idz..(cel_den_idz + dens_per_tuft);
		let cel_syn_range = cel_syn_idz..(cel_syn_idz + syns_per_tuft);

		print!("\nPrinting Pyramidal Cell:");
		print!("\n   preds[{}]: {}", cel_idx, self.preds[cel_idx]);
		print!("\n   best_den_ids[{}]: {}", cel_idx, self.best_den_ids[cel_idx]);
		print!("\n   best_den_states[{}]: {}", cel_idx, self.best_den_states[cel_idx]);
		//print!("\n   best2_den_ids[{}]: {}", cel_idx, self.best2_den_ids[cel_idx]);			// <<<<< SLATED FOR REMOVAL
		//print!("\n   best2_den_states[{}]: {}", cel_idx, self.best2_den_states[cel_idx]);	// <<<<< SLATED FOR REMOVAL
		print!("\n   flag_sets[{}]: {}", cel_idx, self.flag_sets[cel_idx]);
		print!("\n   energies[{}]: {}", cel_idx, self.energies[cel_idx]);

		print!("\n");

		print!("\ndens.states[{:?}]: ", cel_den_range.clone()); 
		cmn::print_vec_simple(&self.dens_mut().states.vec[cel_den_range.clone()]);

		print!("\ndens.syns.states[{:?}]: ", cel_syn_range.clone()); 
		cmn::print_vec_simple(&self.dens_mut().syns.states.vec[cel_syn_range.clone()]);

		print!("\ndens.syns.strengths[{:?}]: ", cel_syn_range.clone()); 
		cmn::print_vec_simple(&self.dens_mut().syns.strengths.vec[cel_syn_range.clone()]);

		print!("\ndens.src_col_xy_offs[{:?}]: ", cel_syn_range.clone()); 
		cmn::print_vec_simple(&self.dens_mut().syns.src_col_xy_offs.vec[cel_syn_range.clone()]);
	}

	pub fn set_all_to_zero(&mut self) {
		self.preds.set_all_to(0);
		self.best_den_ids.set_all_to(0);
		self.best_den_states.set_all_to(0);
		//self.best2_den_ids.set_all_to(0);			// <<<<< SLATED FOR REMOVAL
		//self.best2_den_states.set_all_to(0);		// <<<<< SLATED FOR REMOVAL
		self.flag_sets.set_all_to(0);
		self.energies.set_all_to(0);
	}

	pub fn dens(&self) -> &Dendrites {
		&self.dens
	}

	pub fn dens_mut(&mut self) -> &mut Dendrites {
		&mut self.dens
	}
}
