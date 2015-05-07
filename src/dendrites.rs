use cmn;
use ocl::{ self, Ocl, WorkSize, };
use ocl::{ Envoy };
use cortical_areas::{ CorticalAreas, Width };
use protocell::{ CellKind, Protocell, DendriteKind };
use protoregions::{ ProtoRegion, ProtoRegionKind };
use synapses::{ Synapses };
use axons::{ Axons };
use cells::{ Aux };

use num;
use rand;
use std::mem;
use rand::distributions::{ Normal, IndependentSample, Range };
use rand::{ ThreadRng };
use num::{ Integer };
use std::default::{ Default };
use std::fmt::{ Display };

pub struct Dendrites {
	depth: u8,
	width: u32,
	per_cell_l2: u32,
	den_kind: DendriteKind,
	cell_kind: CellKind,
	kern_cycle: ocl::Kernel,
	pub thresholds: Envoy<ocl::cl_uchar>,
	pub states_raw: Envoy<ocl::cl_uchar>,
	pub states: Envoy<ocl::cl_uchar>,
	pub syns: Synapses,
}

impl Dendrites {
	pub fn new(
					width: u32, 
					depth: u8, 
					den_kind: DendriteKind, 
					cell_kind: CellKind,
					per_cell_l2: u32, 
					region: &ProtoRegion,
					axons: &Axons,
					aux: &Aux,
					ocl: &Ocl
	) -> Dendrites {
		//println!("\n### Test D1 ###");
		let width_dens = width << per_cell_l2;


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

		let states = Envoy::<ocl::cl_uchar>::new(width_dens, depth, cmn::STATE_ZERO, ocl);

		let states_raw = Envoy::<ocl::cl_uchar>::new(width_dens, depth, cmn::STATE_ZERO, ocl);

		let syns = Synapses::new(width, depth, per_cell_l2 + syns_per_den_l2, syns_per_den_l2, den_kind, cell_kind, region, axons, aux, ocl);


		//println!("\nsyns_per_den_l2 = {}", syns_per_den_l2);
		let kern_cycle = ocl.new_kernel(den_kernel, WorkSize::TwoDim(depth as usize, width_dens as usize))
			.arg_env(&syns.states)
			.arg_env(&syns.strengths)
			.arg_scl(syns_per_den_l2)
			.arg_scl(den_threshold)
			.arg_env(&states_raw)
			.arg_env(&states)
		;
		
		Dendrites {
			depth: depth,
			width: width,
			per_cell_l2: per_cell_l2,
			den_kind: den_kind,
			cell_kind: cell_kind,
			kern_cycle: kern_cycle,
			thresholds: Envoy::<ocl::cl_uchar>::new(width_dens, depth, 1, ocl),
			states_raw: states_raw,
			states: states,
			syns: syns,
		}
	}


	pub fn cycle(&self) {
		self.syns.cycle();

		self.kern_cycle.enqueue();
	}

	pub fn regrow(&mut self) {
		self.syns.regrow();
	}

	pub fn confab(&mut self) {
		self.thresholds.read();
		self.states_raw.read();
		self.states.read();
	} 
}
