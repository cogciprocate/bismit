use std::ops;
use std::mem;
use rand::distributions::{ Normal, IndependentSample, Range };
use rand::{ self, ThreadRng, Rng };
use num::{ self, Integer };
use std::default::{ Default };
use std::fmt::{ Display };

use cmn;
use ocl::{ self, Ocl, WorkSize, Envoy, CorticalDimensions };
use proto::areas::{ Protoareas };
use proto::regions::{ Protoregion, ProtoregionKind };
use proto::layer:: { Protolayer };
use proto::cell::{ ProtocellKind, Protocell, DendriteKind };
use synapses::{ Synapses };
use dendrites::{ Dendrites };
use axons::{ Axons };
use cortical_area:: { Aux };
use iinn:: { InhibitoryInterneuronNetwork };
use pyramidals::{ PyramidalCellularLayer };
use minicolumns::{ Minicolumns };




/*	Minicolumns (aka. Columns)
	- TODO:
		- Reorganization to:
			- Minicolumns
				- SpinyStellateCellularLayer
					- Dendrite

*/
pub struct SpinyStellateCellularLayer {
	layer_name: &'static str,
	dims: CorticalDimensions,
	protocell: Protocell,
	axn_base_slice: u8,
	axn_idz: u32,
	//kern_cycle: ocl::Kernel,
	//kern_post_inhib: ocl::Kernel,
	//kern_output: ocl::Kernel,
	kern_ltp: ocl::Kernel,
	rng: rand::XorShiftRng,
	//regrow_counter: usize,	// SLATED FOR REMOVAL
	//pub states: Envoy<ocl::cl_uchar>,
	//pub states_raw: Envoy<ocl::cl_uchar>,
	//pub cels_status: Envoy<ocl::cl_uchar>,
	//pub best_pyr_den_states: Envoy<ocl::cl_uchar>,
	//pub iinn: InhibitoryInterneuronNetwork,
	//pub syns: ColumnSynapses,
	pub dens: Dendrites,
	//pub syns: Synapses,
}

// pyrs: &PyramidalCellularLayer,
impl SpinyStellateCellularLayer {
	pub fn new(layer_name: &'static str, dims: CorticalDimensions, protocell: Protocell, region: &Protoregion, axns: &Axons, aux: &Aux, ocl: &Ocl) -> SpinyStellateCellularLayer {
		//let layer = region.col_input_layer().expect("spiny_stellates::SpinyStellateCellularLayer::new()");
		//let depth: u8 = layer.depth();

		let axn_base_slices = region.slice_ids(vec![layer_name]);
		let axn_base_slice = axn_base_slices[0];
		let axn_idz = cmn::axn_idx_2d(axn_base_slice, dims.columns(), region.hrz_demarc());

		let syns_per_cel_l2: u8 = protocell.syns_per_den_l2 + protocell.dens_per_cel_l2;

		//let pyr_depth = region.depth_cell_kind(&ProtocellKind::Pyramidal);

		//let pyr_axn_base_slice = region.base_slice_cell_kind(&ProtocellKind::Pyramidal); // SHOULD BE SPECIFIC LAYER(S)  

		//let states = Envoy::<ocl::cl_uchar>::new(dims, cmn::STATE_ZERO, ocl);
		//let states_raw = Envoy::<ocl::cl_uchar>::new(dims, cmn::STATE_ZERO, ocl);
		print!("\n      SPINY_STELLATE::NEW(): dims: {:?}", dims);

		let dens_dims = dims.clone_with_pcl2(protocell.dens_per_cel_l2 as i8);
		let dens = Dendrites::new(dens_dims, protocell.clone(), DendriteKind::Proximal, ProtocellKind::SpinyStellate, region, axns, aux, ocl);

		//let cels_status = Envoy::<ocl::cl_uchar>::new(dims, cmn::STATE_ZERO, ocl);
		//let best_pyr_den_states = Envoy::<ocl::cl_uchar>::new(dims, cmn::STATE_ZERO, ocl);
		//let iinn = InhibitoryInterneuronNetwork::new(dims, region, &dens.states, ocl);

		/*let syns = Synapses::new(dims, syns_per_cel_l2, syns_per_cel_l2, DendriteKind::Proximal, 
			ProtocellKind::SpinyStellateCellularLayer, region, axns, aux, ocl);*/



		/*let kern_cycle = ocl.new_kernel("den_cycle", WorkSize::TwoDim(depth as usize, dims.columns() as usize))
			.arg_env(&dens.syns.states)
			.arg_env(&dens.syns.strengths)
			.arg_scl(syns_per_cel_l2)
			.arg_scl(cmn::DENDRITE_INITIAL_THRESHOLD_PROXIMAL)
			.arg_env(&states_raw)
			.arg_env(&states)
		;*/

		/*let kern_post_inhib = ocl.new_kernel("sst_post_inhib_unoptd", WorkSize::TwoDim(dims.depth() as usize, dims.columns() as usize))
			.arg_env(&iinn.spi_ids)
			.arg_env(&iinn.states)
			.arg_env(&iinn.wins)
			.arg_scl(layer.base_slice_pos() as u32)
			.arg_env(&dens.states)
			.arg_env(&axns.states)
		;*/
		assert!(dims.columns() % cmn::MINIMUM_WORKGROUP_SIZE == 0);
		let cels_per_grp: u32 = dims.columns() / cmn::MINIMUM_WORKGROUP_SIZE;

		println!("\n##### SPINY_STELLATES: cels_per_grp: {}, syns_per_cel_l2: {}, axn_idz: {} ",
			 cels_per_grp, syns_per_cel_l2, axn_idz);

		let kern_ltp = ocl.new_kernel("sst_ltp", WorkSize::TwoDim(dims.depth() as usize, cmn::MINIMUM_WORKGROUP_SIZE as usize))
		//let kern_ltp = ocl.new_kernel("sst_ltp", WorkSize::TwoDim(dims.depth() as usize, iinn.dims.per_slice() as usize))
			.arg_env(&axns.states)
			.arg_env(&dens.syns.states)
			.arg_scl(axn_idz)
			.arg_scl(syns_per_cel_l2)
			.arg_scl(cels_per_grp)
			.arg_scl_named::<u32>("rnd", None)
			.arg_env(&aux.ints_0)
			.arg_env(&dens.syns.strengths)
			//.arg_env(&axns.states)
		;




		/*let kern_ltp_old = ocl.new_kernel("sst_ltp_old", WorkSize::TwoDim(dims.depth() as usize, 16 as usize)) // ***** FIX
		//let kern_ltp = ocl.new_kernel("sst_ltp", WorkSize::TwoDim(dims.depth() as usize, iinn.dims.per_slice() as usize))
			.arg_env(&dens.syns.states)
			.arg_env(&dens.syns.states)
			.arg_env(&dens.syns.states)
			.arg_scl(syns_per_cel_l2 as u32)
			.arg_scl_named::<u32>("rnd", None)
			//.arg_env(&aux.ints_0)
			.arg_env(&dens.syns.strengths)
			//.arg_env(&axns.states)
		;*/


		//println!("\n***Test");

		SpinyStellateCellularLayer {
			layer_name: layer_name,
			dims: dims,
			protocell: protocell,
			axn_base_slice: axn_base_slice,
			axn_idz: axn_idz,
			//kern_cycle: kern_cycle,
			//kern_post_inhib: kern_post_inhib,
			//kern_output: kern_output,
			kern_ltp: kern_ltp,
			rng: rand::weak_rng(),
			//regrow_counter: 0usize,
			//states_raw: states_raw,
			//states: states,
			//cels_status: cels_status,
			//best_pyr_den_states: best_pyr_den_states,
			//iinn: iinn,
			dens: dens,
		}
	}

	pub fn cycle(&mut self, ltp: bool) {
		self.dens.cycle();
	}


	pub fn ltp(&mut self) {
		//print!("[R:{}]", self.rng.gen::<i32>());
		self.kern_ltp.set_arg_scl_named("rnd", self.rng.gen::<u32>());
		self.kern_ltp.enqueue();
	}

	pub fn regrow(&mut self) {
		self.dens.regrow();
	}

	pub fn confab(&mut self) {
		self.dens.confab();
	} 

	pub fn soma(&self) -> &Envoy<u8> {
		&self.dens.states
	}

	pub fn soma_mut(&mut self) -> &mut Envoy<u8> {
		&mut self.dens.states
	}

	pub fn dims(&self) -> &CorticalDimensions {
		&self.dims
	}

	pub fn axn_range(&self) -> (usize, usize) {
		let ssts_axn_idn = self.axn_idz + (self.dims.per_slice());

		(self.axn_idz as usize, ssts_axn_idn as usize)
	}

	pub fn print_cel(&mut self, cel_idx: usize) {
		let emsg = "SpinyStellateCellularLayer::print()";

		let cel_syn_idz = (cel_idx << self.dens.syns.dims().per_cel_l2_left().expect(emsg)) as usize;
		let per_cel = self.dens.syns.dims().per_cel().expect(emsg) as usize;
		let cel_syn_range = cel_syn_idz..(cel_syn_idz + per_cel);

		println!("\ncell.state[{}]: {}", cel_idx, self.dens.states[cel_idx]);

		print!("\ncell.syns.states[{:?}]: ", cel_syn_range.clone()); 
		cmn::print_vec_simple(&self.dens.syns.states.vec[cel_syn_range.clone()]);

		print!("\ncell.syns.strengths[{:?}]: ", cel_syn_range.clone()); 
		cmn::print_vec_simple(&self.dens.syns.strengths.vec[cel_syn_range.clone()]);

		print!("\ncell.syns.src_col_xy_offs[{:?}]: ", cel_syn_range.clone()); 
		cmn::print_vec_simple(&self.dens.syns.src_col_xy_offs.vec[cel_syn_range.clone()]);
	}
}
