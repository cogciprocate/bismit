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
	height: u8,
	width: u32,
	per_cell: u32,
	den_type: DendriteKind,
	kern_cycle: ocl::Kernel,
	pub thresholds: Envoy<ocl::cl_uchar>,
	pub states: Envoy<ocl::cl_uchar>,
	pub syns: Synapses,
}

impl Dendrites {
	pub fn new(
					width: u32, 
					height: u8, 
					den_type: DendriteKind, 
					per_cell: u32, 
					region: &CorticalRegion,
					axons: &Axons,
					ocl: &Ocl
	) -> Dendrites {
		let width_dens = width * per_cell;


		let per_den = match den_type {
			DendriteKind::Distal => common::SYNAPSES_PER_DENDRITE_DISTAL,
			DendriteKind::Proximal => common::SYNAPSES_PER_DENDRITE_PROXIMAL,
		};

		let states = Envoy::<ocl::cl_uchar>::new(width_dens, height, common::STATE_ZERO, ocl);

		let syns = Synapses::new(width, height, per_cell * per_den, den_type, region, axons, ocl);

		let kern_cycle = ocl.new_kernel("dens_cycle", WorkSize::TwoDim(height as usize, width as usize))
			.arg_env(&syns.states)
			.arg_scl(common::SYNAPSES_PER_DENDRITE_DISTAL_LOG2)
			.arg_env(&states);

		Dendrites {
			height: height,
			width: width,
			per_cell: per_cell,
			den_type: den_type,
			kern_cycle: kern_cycle,
			thresholds: Envoy::<ocl::cl_uchar>::new(width_dens, height, common::DENDRITE_INITIAL_THRESHOLD, ocl),
			states: states,
			syns: syns,
		}
	}


	pub fn cycle(&self) {
		self.syns.cycle();

		self.kern_cycle.enqueue();
	}
}
