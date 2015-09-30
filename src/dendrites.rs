use num;
use rand;
use std::mem;
use rand::distributions::{ Normal, IndependentSample, Range };
use rand::{ ThreadRng };
use num::{ Integer };
use std::default::{ Default };
use std::fmt::{ Display };

use cmn::{ self, CorticalDimensions };
use map::{ AreaMap };
use ocl::{ self, OclProgQueue, WorkSize, Envoy };
use proto::{ ProtoLayerMap, RegionKind, ProtoAreaMaps, ProtocellKind, Protocell, DendriteKind };
use synapses::{ Synapses };
use axons::{ Axons };
use cortical_area:: { Aux };



pub struct Dendrites {
	layer_name: &'static str,
	dims: CorticalDimensions,
	//protocell: Protocell,
	//per_cell_l2: u32,
	den_kind: DendriteKind,
	cell_kind: ProtocellKind,
	kern_cycle: ocl::Kernel,
	pub thresholds: Envoy<ocl::cl_uchar>,
	pub states_raw: Envoy<ocl::cl_uchar>,
	pub states: Envoy<ocl::cl_uchar>,
	pub energies: Envoy<ocl::cl_uchar>,
	pub syns: Synapses,
}

impl Dendrites {
	pub fn new(
					layer_name: &'static str,
					dims: CorticalDimensions,
					//src_tufts: Vec<Vec<&'static str>>,
					protocell: Protocell,
					den_kind: DendriteKind, 
					cell_kind: ProtocellKind,
					area_map: &AreaMap,
					axons: &Axons,
					aux: &Aux,
					ocl: &OclProgQueue
	) -> Dendrites {
		//println!("\n### Test D1 ###");
		//let width_dens = dims.width << per_cell_l2;
		assert!(dims.per_tuft_l2() as u8 == protocell.dens_per_tuft_l2);

		//let dims = cel_dims.clone_with_ptl2(per_cell_l2);

		let syns_per_den_l2 = protocell.syns_per_den_l2;
		let den_threshold = protocell.den_thresh_init.unwrap_or(1);

		/*let (den_threshold, den_kernel) = match den_kind {
			DendriteKind::Distal => (
				protocell.den_thresh_init.unwrap_or(1),
				//cmn::SYNAPSES_PER_DENDRITE_DISTAL_LOG2, 
				"den_cycle"
			),
			DendriteKind::Proximal => (
				protocell.den_thresh_init.unwrap_or(1),
				//cmn::SYNAPSES_PER_DENDRITE_PROXIMAL_LOG2, 
				
			),
		};*/



		let states = Envoy::<ocl::cl_uchar>::new(dims, cmn::STATE_ZERO, ocl);
		let states_raw = Envoy::<ocl::cl_uchar>::new(dims, cmn::STATE_ZERO, ocl);
		let energies = Envoy::<ocl::cl_uchar>::new(dims, 255, ocl);

		println!("            DENDRITES::NEW(): '{}': dendrites with: dims:{:?}, len:{}", layer_name, dims, states.len());

		let syns_dims = dims.clone_with_ptl2((dims.per_tuft_l2() + syns_per_den_l2 as i8));
		let syns = Synapses::new(layer_name, syns_dims, protocell.clone(), den_kind, cell_kind, area_map, axons, aux, ocl);


		let kern_cycle = ocl.new_kernel("den_cycle".to_string(), WorkSize::OneDim(states.len()))
			.arg_env(&syns.states)
			.arg_env(&syns.strengths)
			.arg_scl(syns_per_den_l2)
			.arg_scl(den_threshold)
			.arg_env(&energies)
			.arg_env(&states_raw)
			//.arg_env(&aux.ints_0)
			.arg_env(&states)
		;

		/*let kern_cycle = ocl.new_kernel("den_cycle_old", WorkSize::TwoDim(dims.depth() as usize, dims.per_slc() as usize))
			.arg_env(&syns.states)
			.arg_env(&syns.strengths)
			.arg_scl(syns_per_den_l2)
			.arg_scl(den_threshold)
			.arg_env(&energies)
			.arg_env(&states_raw)
			//.arg_env(&aux.ints_0)
			.arg_env(&states)
		;*/
		
		Dendrites {
			layer_name: layer_name,
			dims: dims,
			//protocell: protocell,
			//per_cell_l2: per_cell_l2,
			den_kind: den_kind,
			cell_kind: cell_kind,
			kern_cycle: kern_cycle,
			thresholds: Envoy::<ocl::cl_uchar>::new(dims, 1, ocl),
			states_raw: states_raw,
			states: states,
			energies: energies,
			syns: syns,
		}
	}


	pub fn cycle(&self) {
		self.syns.cycle();

		self.kern_cycle.enqueue();
	}

	// FOR TESTING PURPOSES
	pub fn cycle_self_only(&self) {
		self.kern_cycle.enqueue();
	}

	pub fn regrow(&mut self) {
		self.syns.regrow();
	}

	pub fn confab(&mut self) {
		self.thresholds.read();
		self.states_raw.read();
		self.states.read();
		self.syns.confab();
	} 

	pub fn dims(&self) -> &CorticalDimensions {
		&self.dims
	}

}
