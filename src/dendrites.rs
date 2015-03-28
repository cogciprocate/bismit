use common;
use ocl::{ self, Ocl, WorkSize, };
use envoy::{ Envoy };
use cortical_areas::{ CorticalAreas, Width };
use cortical_regions::{ CorticalRegion, CorticalRegionKind };
use protocell::{ CellKind, Protocell, DendriteKind };
use synapses::{ Synapses };
use axons::{ Axons };

use std::num;
use std::rand;
use std::mem;
use std::rand::distributions::{ Normal, IndependentSample, Range };
use std::rand::{ ThreadRng };
use std::num::{ NumCast, Int, FromPrimitive };
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
					region: &CorticalRegion,
					axons: &Axons,
					ocl: &Ocl
	) -> Dendrites {
		let width_dens = width << per_cell_l2;


		let (per_den_l2, den_kernel) = match den_kind {
			DendriteKind::Distal => (common::SYNAPSES_PER_DENDRITE_DISTAL_LOG2, "den_dist_cycle"),
			DendriteKind::Proximal => (common::SYNAPSES_PER_DENDRITE_PROXIMAL_LOG2, "den_prox_cycle"),
		};

		let states = Envoy::<ocl::cl_uchar>::new(width_dens, depth, common::STATE_ZERO, ocl);

		let syns = Synapses::new(width, depth, per_cell_l2 + per_den_l2, den_kind, cell_kind, region, axons, ocl);

		let kern_cycle = ocl.new_kernel(den_kernel, WorkSize::TwoDim(depth as usize, width_dens as usize))
			.arg_env(&syns.states)
			.arg_scl(per_cell_l2)
			.arg_env(&states);

		Dendrites {
			depth: depth,
			width: width,
			per_cell_l2: per_cell_l2,
			den_kind: den_kind,
			cell_kind: cell_kind,
			kern_cycle: kern_cycle,
			thresholds: Envoy::<ocl::cl_uchar>::new(width_dens, depth, 1, ocl),
			states: states,
			syns: syns,
		}
	}


	pub fn cycle(&self) {
		self.syns.cycle();

		self.kern_cycle.enqueue();
	}
}
