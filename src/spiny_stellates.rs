use std::ops;
// use std::mem;
use rand::distributions::{ /*Normal, IndependentSample,*/ Range };
use rand::{ self, /*ThreadRng,*/ Rng };
// use num::{ self, Integer };
// use std::default::{ Default };
// use std::fmt::{ Display };

use cmn::{ self, CorticalDimensions };
use map::{ AreaMap };
use ocl::{ self, OclProgQueue, WorkSize, Envoy };
use proto::{ /*ProtoLayerMap, RegionKind, ProtoAreaMaps,*/ ProtocellKind, Protocell, DendriteKind };
// use synapses::{ Synapses };
use dendrites::{ Dendrites };
use axon_space::{ AxonSpace };
use cortical_area:: { Aux };
// use iinn:: { InhibitoryInterneuronNetwork };
// use pyramidals::{ PyramidalLayer };
// use minicolumns::{ Minicolumns };



pub struct SpinyStellateLayer {
	layer_name: &'static str,
	dims: CorticalDimensions,
	protocell: Protocell,
	axn_slc_base: u8,
	lyr_axn_idz: u32,
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

// pyrs: &PyramidalLayer,
impl SpinyStellateLayer {
	pub fn new(layer_name: &'static str, dims: CorticalDimensions, protocell: Protocell, area_map: &AreaMap, 
				axns: &AxonSpace, aux: &Aux, ocl: &OclProgQueue
	) -> SpinyStellateLayer {
		//let layer = area_map.proto_layer_map().spt_asc_layer().expect("spiny_stellates::SpinyStellateLayer::new()");
		//let depth: u8 = layer.depth();

		let axn_slc_bases = area_map.proto_layer_map().slc_ids(vec![layer_name]);
		let axn_slc_base = axn_slc_bases[0];
		//let lyr_axn_idz = cmn::axn_idz_2d(axn_slc_base, dims.columns(), area_map.proto_layer_map().hrz_demarc());
		let lyr_axn_idz = area_map.axn_idz(axn_slc_base);

		let syns_per_tuft_l2: u8 = protocell.syns_per_den_l2 + protocell.dens_per_tuft_l2;

		//let pyr_depth = area_map.proto_layer_map().depth_cell_kind(&ProtocellKind::Pyramidal);

		//let pyr_axn_slc_base = area_map.proto_layer_map().base_slc_cell_kind(&ProtocellKind::Pyramidal); // SHOULD BE SPECIFIC LAYER(S)  

		//let states = Envoy::<ocl::cl_uchar>::new(dims, cmn::STATE_ZERO, ocl);
		//let states_raw = Envoy::<ocl::cl_uchar>::new(dims, cmn::STATE_ZERO, ocl);
		println!("      SPINYSTELLATES::NEW(): axn_slc_base: {}, lyr_axn_idz: {}, dims: {:?}", axn_slc_base, lyr_axn_idz, dims);

		let dens_dims = dims.clone_with_ptl2(protocell.dens_per_tuft_l2 as i8);
		let dens = Dendrites::new(layer_name, dens_dims, protocell.clone(), DendriteKind::Distal, ProtocellKind::SpinyStellate, area_map, axns, aux, ocl);

		//let cels_status = Envoy::<ocl::cl_uchar>::new(dims, cmn::STATE_ZERO, ocl);
		//let best_pyr_den_states = Envoy::<ocl::cl_uchar>::new(dims, cmn::STATE_ZERO, ocl);
		//let iinn = InhibitoryInterneuronNetwork::new(dims, area_map.proto_layer_map(), &dens.states, ocl);

		/*let syns = Synapses::new(dims, syns_per_tuft_l2, syns_per_tuft_l2, DendriteKind::Proximal, 
			ProtocellKind::SpinyStellateLayer, area_map.proto_layer_map(), axns, aux, ocl);*/


		//assert!(dims.columns() % cmn::MINIMUM_WORKGROUP_SIZE == 0);
		//let cels_per_tuft: u32 = dims.columns() / cmn::MINIMUM_WORKGROUP_SIZE;

		//let work_size = dims.len() / cmn::AXON_BUF__FER_SIZE as usize;

		//println!("\n##### SPINY_STELLATES: cels_per_tuft: {}, syns_per_tuft_l2: {}, lyr_axn_idz: {} ", cels_per_tuft, syns_per_tuft_l2, lyr_axn_idz);

		let grp_count = cmn::OPENCL_MINIMUM_WORKGROUP_SIZE;
		let cols_per_grp = dims.cols_per_subgrp(grp_count).unwrap();
			//.unwrap_or_else(|s: &'static str| panic!(s));

		let kern_ltp = ocl.new_kernel("sst_ltp".to_string(), 
			//WorkSize::TwoDim(dims.depth() as usize, cmn::MINIMUM_WORKGROUP_SIZE as usize))
			WorkSize::TwoDim(dims.tufts_per_cel() as usize, grp_count as usize))
		//let kern_ltp = ocl.new_kernel("sst_ltp", WorkSize::TwoDim(dims.depth() as usize, iinn.dims.per_slc() as usize))
			.arg_env(&axns.states)
			.arg_env(&dens.syns().states)
			.arg_scl(lyr_axn_idz)
			.arg_scl(cols_per_grp)
			.arg_scl(syns_per_tuft_l2)
			//.arg_scl(cels_per_tuft)
			.arg_scl_named::<u32>("rnd", None)
			.arg_env(&aux.ints_0)
			.arg_env(&dens.syns().strengths)
		;




		/*let kern_ltp_old = ocl.new_kernel("sst_ltp_old", WorkSize::TwoDim(dims.depth() as usize, 16 as usize)) // ***** FIX
		//let kern_ltp = ocl.new_kernel("sst_ltp", WorkSize::TwoDim(dims.depth() as usize, iinn.dims.per_slc() as usize))
			.arg_env(&dens.syns().states)
			.arg_env(&dens.syns().states)
			.arg_env(&dens.syns().states)
			.arg_scl(syns_per_tuft_l2 as u32)
			.arg_scl_named::<u32>("rnd", None)
			//.arg_env(&aux.ints_0)
			.arg_env(&dens.syns().strengths)
			//.arg_env(&axns.states)
		;*/


		//println!("\n***Test");

		SpinyStellateLayer {
			layer_name: layer_name,
			dims: dims,
			protocell: protocell,
			axn_slc_base: axn_slc_base,
			lyr_axn_idz: lyr_axn_idz,
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

	pub fn cycle(&mut self) {
		self.dens.cycle();
	}


	pub fn learn(&mut self) {
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

	pub fn axn_slc_base(&self) -> u8 {
		self.axn_slc_base
	}

	pub fn layer_name(&self) -> &'static str {
		self.layer_name
	}

	pub fn print_cel(&mut self, cel_idx: usize) {
		let emsg = "SpinyStellateLayer::print()";

		let cel_syn_idz = (cel_idx << self.dens.syns().dims().per_tuft_l2_left()) as usize;
		let per_cel = self.dens.syns().dims().per_cel() as usize;
		let cel_syn_range = cel_syn_idz..(cel_syn_idz + per_cel);

		println!("\ncell.state[{}]: {}", cel_idx, self.dens.states[cel_idx]);

		println!("cell.syns.states[{:?}]: ", cel_syn_range.clone()); 
		cmn::print_vec_simple(&self.dens.syns().states.vec()[cel_syn_range.clone()]);

		println!("cell.syns.strengths[{:?}]: ", cel_syn_range.clone()); 
		cmn::print_vec_simple(&self.dens.syns().strengths.vec()[cel_syn_range.clone()]);

		println!("cell.syns.src_col_v_offs[{:?}]: ", cel_syn_range.clone()); 
		cmn::print_vec_simple(&self.dens.syns().src_col_v_offs.vec()[cel_syn_range.clone()]);
	}

	pub fn dens(&self) -> &Dendrites {
		&self.dens
	}

	pub fn dens_mut(&mut self) -> &mut Dendrites {
		&mut self.dens
	}

	pub fn axn_range(&self) -> ops::Range<usize> {
		let ssts_axn_idn = self.lyr_axn_idz + (self.dims.per_slc());
		self.lyr_axn_idz as usize..ssts_axn_idn as usize
	}
}
