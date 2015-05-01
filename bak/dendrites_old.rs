use common;
use ocl::{ self, Ocl };
use envoy::{ Envoy };
use cortical_areas::{ CorticalAreas, Width };
use cortical_regions::{ CorticalRegion, CorticalRegionType };
use protocell::{ CellKind, Protocell, DendriteKind };
use synapses::{ Synapses };
use axons::{ Axons };

use num;
use rand;
use std::mem;
use rand::distributions::{ Normal, IndependentSample, Range };
use rand::{ ThreadRng };
use num::{ NumCast, Integer, FromPrimitive };
use std::default::{ Default };
use std::fmt::{ Display };

pub struct Dendrites {
	depth: u8,
	width: u32,
	per_cell: u32,
	den_type: DendriteKind,
	pub thresholds: Envoy<ocl::cl_uchar>,
	pub states: Envoy<ocl::cl_uchar>,
	//pub health: 
	pub syns: Synapses,
}

impl Dendrites {
	pub fn new(
					width: u32, 
					depth: u8, 
					den_type: DendriteKind, 
					per_cell: u32, 
					region: &CorticalRegion, 
					ocl: &Ocl
	) -> Dendrites {
		let width_dens = width * per_cell;

		Dendrites {
			depth: depth,
			width: width,
			per_cell: per_cell,
			den_type: den_type,
			thresholds: Envoy::<ocl::cl_uchar>::new(width_dens, depth, common::DENDRITE_INITIAL_THRESHOLD_PROXIMAL, ocl),
			states: Envoy::<ocl::cl_uchar>::new(width_dens, depth, common::STATE_ZERO, ocl),
			syns: Synapses::new(width, depth, per_cell * common::SYNAPSES_PER_DENDRITE_DISTAL, den_type, region, ocl),
		}
	}


	pub fn cycle(&self, axns: &Axons, ocl: &Ocl) {
		self.syns.cycle(axns, ocl);

		let len_dens: usize = self.depth as usize * self.width as usize * self.per_cell as usize;

		let boost_log2: u8 = if self.den_type == DendriteKind::Distal {
			common::DST_DEN_BOOST_LOG2
		} else {
			common::PRX_DEN_BOOST_LOG2
		};

		let kern = ocl::new_kernel(ocl.program, "dens_cycle");

		ocl::set_kernel_arg(0, self.syns.states.buf, kern);
		ocl::set_kernel_arg(1, self.thresholds.buf, kern);
		ocl::set_kernel_arg(2, self.states.buf, kern);
		ocl::set_kernel_arg(3, boost_log2, kern);

		ocl::enqueue_kernel(ocl.command_queue, kern, len_dens);

	}
}