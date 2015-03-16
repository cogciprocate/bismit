use common;
use ocl::{ self, Ocl, WorkSize };
use envoy::{ Envoy };
use cortical_areas::{ CorticalAreas, Width };
use cortical_regions::{ CorticalRegion, CorticalRegionType };
use protocell::{ CellKind, Protocell, DendriteKind };
use synapses::{ Synapses };
use dendrites::{ Dendrites };
use axons::{ Axons };
use columns::{ Columns }; 

use std::num;
use std::ops;
use std::rand;
use std::mem;
use std::rand::distributions::{ Normal, IndependentSample, Range };
use std::rand::{ ThreadRng };
use std::num::{ NumCast, Int, FromPrimitive };
use std::default::{ Default };
use std::fmt::{ Display };


pub struct AspinyStellate {
	width: u32,
	height: u8,
	kern_cycle_1: ocl::Kernel,
	pub ids: Envoy<ocl::cl_uchar>,
	pub states: Envoy<ocl::cl_uchar>,
}

impl AspinyStellate {
	pub fn new(col_width: u32, height: u8, region: &CorticalRegion, cols: &Columns, ocl: &Ocl) -> AspinyStellate {

		let width = (col_width >> common::ASPINY_SPAN_LOG2) + common::ASPINY_SPAN;

		let width_no_ofs = width - common::ASPINY_SPAN;

		let ids = Envoy::<ocl::cl_uchar>::new(width, height, 0u8, ocl);
		let states = Envoy::<ocl::cl_uchar>::new(width, height, common::STATE_ZERO, ocl);

		let mut kern_cycle_1 = ocl.new_kernel("aspiny_cycle", 
			WorkSize::TwoDim(height as usize, col_width as usize));
		kern_cycle_1.new_arg_envoy(&cols.states);
		kern_cycle_1.new_arg_envoy(&ids);
		kern_cycle_1.new_arg_envoy(&states);


		AspinyStellate {
			width: width,
			height: height,
			kern_cycle_1: kern_cycle_1,
			ids: ids,
			states: states,
		}
	}

	pub fn cycle(&self) {
		self.kern_cycle_1.enqueue();
	}

	 
}
