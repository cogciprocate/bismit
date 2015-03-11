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
	kern_cycle: ocl::Kernel,
	pub id_vals: Envoy<ocl::cl_uchar>,
	//pub winner_vals: Envoy<ocl::cl_char>,
}

impl AspinyStellate {
	pub fn new(col_width: u32, height: u8, region: &CorticalRegion, cols: &Columns, ocl: &Ocl) -> AspinyStellate {

		let width = (col_width >> common::ASPINY_SPAN_LOG2) + (1 << common::ASPINY_SPAN_LOG2);

		let id_vals = Envoy::<ocl::cl_uchar>::new(width, height, 0u8, ocl);
		//let winner_vals = Envoy::<ocl::cl_char>::new(width, height, 0i8, ocl);

		let mut kern_cycle = ocl.new_kernel("aspiny_cycle", 
			WorkSize::TwoDim(height as usize, width as usize));
		kern_cycle.arg(&cols.states);
		kern_cycle.arg(&id_vals);
			//.arg(&winner_vals)


		AspinyStellate {
			width: width,
			height: height,
			kern_cycle: kern_cycle,
			id_vals: id_vals,
			//winner_vals: winner_vals,

		}
	}

	pub fn cycle(&self) {
		self.kern_cycle.enqueue();
	}

	 
}
