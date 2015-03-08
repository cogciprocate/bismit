use common;
use ocl::{ self, Ocl };
use envoy::{ Envoy };
use cortical_areas::{ CorticalAreas, Width };
use cortical_regions::{ CorticalRegion, CorticalRegionType };
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
	pub thresholds: Envoy<ocl::cl_char>,
	pub states: Envoy<ocl::cl_char>,
	//pub health: 
	pub syns: Synapses,
}

impl Dendrites {
	pub fn new(
					width: u32, 
					height: u8, 
					den_type: DendriteKind, 
					per_cell: u32, 
					region: &CorticalRegion, 
					ocl: &Ocl
	) -> Dendrites {
		let width_dens = width * per_cell;

		Dendrites {
			height: height,
			width: width,
			per_cell: per_cell,
			den_type: den_type,
			thresholds: Envoy::<ocl::cl_char>::new(width_dens, height, common::DENDRITE_INITIAL_THRESHOLD, ocl),
			states: Envoy::<ocl::cl_char>::new(width_dens, height, 0i8, ocl),
			syns: Synapses::new(width, height, per_cell * common::SYNAPSES_PER_DENDRITE_DISTAL, den_type, region, ocl),
		}
	}

	pub fn cycle(&self, axns: &Axons, ocl: &Ocl) {
		self.syns.cycle(axns, ocl);

		let len_dens: usize = self.height as usize * self.width as usize * self.per_cell as usize;

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
