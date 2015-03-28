use common;
use ocl::{ self, Ocl, WorkSize };
use envoy::{ Envoy };
use cortical_areas::{ CorticalAreas, Width };
use cortical_regions::{ CorticalRegion, CorticalRegionKind };
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
	depth: u8,
	kern_cycle_pre: ocl::Kernel,
	kern_cycle_wins: ocl::Kernel,
	kern_cycle_post: ocl::Kernel,
	pub ids: Envoy<ocl::cl_uchar>,
	pub wins: Envoy<ocl::cl_uchar>,
	pub states: Envoy<ocl::cl_uchar>,
	
}

impl AspinyStellate {
	pub fn new(col_width: u32, depth: u8, region: &CorticalRegion, src_states: &Envoy<ocl::cl_uchar>, ocl: &Ocl) -> AspinyStellate {

		let width = col_width >> common::ASPINY_SPAN_LOG2;

		let padding = common::ASPINY_SPAN;

		let ids = Envoy::<ocl::cl_uchar>::with_padding(padding, width, depth, 0u8, ocl);
		let wins = Envoy::<ocl::cl_uchar>::with_padding(padding, width, depth, 0u8, ocl);
		let states = Envoy::<ocl::cl_uchar>::with_padding(padding, width, depth, common::STATE_ZERO, ocl);

		let mut kern_cycle_pre = ocl.new_kernel("aspiny_cycle_pre", 
			WorkSize::TwoDim(depth as usize, width as usize))
			.arg_env(&src_states)
			.arg_env(&states)
			.arg_env(&ids)
		;

		let mut kern_cycle_wins = ocl.new_kernel("aspiny_cycle_wins", 
			WorkSize::TwoDim(depth as usize, width as usize))
			.arg_env(&states)
			//.arg_env(&ids)
			.arg_env(&wins)
		;

		let mut kern_cycle_post = ocl.new_kernel("aspiny_cycle_post", 
			WorkSize::TwoDim(depth as usize, width as usize))
			.arg_env(&wins)
			//.arg_env(&ids)
			.arg_env(&states)
		;


		AspinyStellate {
			width: width,
			depth: depth,
			kern_cycle_pre: kern_cycle_pre,
			kern_cycle_wins: kern_cycle_wins,
			kern_cycle_post: kern_cycle_post,
			ids: ids,
			wins: wins,
			states: states,
		}
	}

	pub fn cycle(&mut self) {
		let mut event = self.kern_cycle_pre.enqueue();

		//println!("\n### New aspiny.cycle() iteration: ###");

		for i in range(0, 8) {
			//event = self.cycle_wins(event);
			self.kern_cycle_wins.enqueue();
			//print!("\nasps.wins:");
			//self.wins.print_simple();
		}

		self.kern_cycle_post.enqueue();
	}

}
