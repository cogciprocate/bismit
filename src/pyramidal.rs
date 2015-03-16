use common;
use ocl::{ self, Ocl, WorkSize };
use envoy::{ Envoy };
use cortical_areas::{ CorticalAreas, Width };
use cortical_regions::{ CorticalRegion, CorticalRegionType };
use protocell::{ CellKind, Protocell, DendriteKind };
use synapses::{ Synapses };
use dendrites::{ Dendrites };
use cells::{ Somata, Aux };
use aspiny::{ AspinyStellate };
use columns::{ Columns };
use axons::{ Axons };


use std::num;
use std::rand;
use std::mem;
use std::rand::distributions::{ Normal, IndependentSample, Range };
use std::rand::{ ThreadRng };
use std::num::{ NumCast, Int, FromPrimitive };
use std::default::{ Default };
use std::fmt::{ Display };


pub struct Pyramidal {
	height: u8,
	width: u32,
	pub states: Envoy<ocl::cl_uchar>,
	pub dens: Dendrites,
}

impl Pyramidal {
	pub fn new(width: u32, height: u8, region: &CorticalRegion, ocl: &Ocl) -> Pyramidal {

		Pyramidal {
			height: height,
			width: width,
			states: Envoy::<ocl::cl_uchar>::new(width, height, common::STATE_ZERO, ocl),
			dens: Dendrites::new(width, height, DendriteKind::Distal, common::DENDRITES_PER_CELL_DISTAL, region, ocl),
		}
	}
}
