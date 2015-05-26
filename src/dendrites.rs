use cmn;
use ocl::{ self, Ocl, WorkSize, Envoy, CorticalDimensions };
use proto::areas::{ ProtoAreas };
use proto::cell::{ CellKind, Protocell, DendriteKind };
use proto::regions::{ ProtoRegion, ProtoRegionKind };
use synapses::{ Synapses };
use axons::{ Axons };
use cortical_area:: { Aux };

use num;
use rand;
use std::mem;
use rand::distributions::{ Normal, IndependentSample, Range };
use rand::{ ThreadRng };
use num::{ Integer };
use std::default::{ Default };
use std::fmt::{ Display };

pub struct Dendrites {
	dims: CorticalDimensions,
	//per_cell_l2: u32,
	den_kind: DendriteKind,
	cell_kind: CellKind,
	kern_cycle: ocl::Kernel,
	pub thresholds: Envoy<ocl::cl_uchar>,
	pub states_raw: Envoy<ocl::cl_uchar>,
	pub states: Envoy<ocl::cl_uchar>,
	pub energies: Envoy<ocl::cl_uchar>,
	pub syns: Synapses,
}

impl Dendrites {
	pub fn new(
					dims: CorticalDimensions,
					//per_cell_l2: u32,
					den_kind: DendriteKind, 
					cell_kind: CellKind,
					region: &ProtoRegion,
					axons: &Axons,
					aux: &Aux,
					ocl: &Ocl
	) -> Dendrites {
		//println!("\n### Test D1 ###");
		//let width_dens = dims.width << per_cell_l2;

		//let dims = cel_dims.clone_with_pcl2(per_cell_l2);


		let (den_threshold, syns_per_den_l2, den_kernel) = match den_kind {
			DendriteKind::Distal => (
				cmn::DENDRITE_INITIAL_THRESHOLD_DISTAL,
				cmn::SYNAPSES_PER_DENDRITE_DISTAL_LOG2, 
				"den_cycle"
			),
			DendriteKind::Proximal => (
				cmn::DENDRITE_INITIAL_THRESHOLD_PROXIMAL,
				cmn::SYNAPSES_PER_DENDRITE_PROXIMAL_LOG2, 
				"den_cycle"
			),
		};


		print!("\n##### New {:?} Dendrites with dims: {:?}", den_kind, dims);

		let states = Envoy::<ocl::cl_uchar>::new(dims, cmn::STATE_ZERO, ocl);
		let states_raw = Envoy::<ocl::cl_uchar>::new(dims, cmn::STATE_ZERO, ocl);

		let syns_dims = dims.clone_with_pcl2((dims.per_cel_l2() + syns_per_den_l2 as i8));
		let syns = Synapses::new(syns_dims, syns_per_den_l2, den_kind, cell_kind, region, axons, aux, ocl);

		let energies = Envoy::<ocl::cl_uchar>::new(dims, 255, ocl);


		//println!("\nsyns_per_den_l2 = {}", syns_per_den_l2);
		let kern_cycle = ocl.new_kernel(den_kernel, WorkSize::TwoDim(dims.depth() as usize, dims.per_slice() as usize))
			.arg_env(&syns.states)
			.arg_env(&syns.strengths)
			.arg_scl(syns_per_den_l2 as u32)
			.arg_scl(den_threshold)
			.arg_env(&energies)
			.arg_env(&states_raw)
			.arg_env(&states)
		;
		
		Dendrites {
			dims: dims,
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

	pub fn regrow(&mut self, region: &ProtoRegion) {
		self.syns.regrow(region);
	}

	pub fn confab(&mut self) {
		self.thresholds.read();
		self.states_raw.read();
		self.states.read();
		self.syns.confab();
	} 
}
