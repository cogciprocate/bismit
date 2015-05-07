use cmn;
use ocl::{ self, Ocl, WorkSize };
use ocl::{ Envoy };
use cortical_areas::{ CorticalAreas, Width };
use protoregions::{ CorticalRegion, CorticalRegionKind };
use protocell::{ CellKind, Protocell, DendriteKind };
use synapses::{ Synapses };
use dendrites::{ Dendrites };
use axons::{ Axons };
use columns::{ Columns }; 

use num;
use std::ops;
use rand;
use std::mem;
use rand::distributions::{ Normal, IndependentSample, Range };
use rand::{ ThreadRng };
use num::{ Integer };
use std::default::{ Default };
use std::fmt::{ Display };


pub struct PeakColumn {
	width: u32,
	depth: u8,
	kern_cycle_pre: ocl::Kernel,
	kern_cycle_wins: ocl::Kernel,
	kern_cycle_post: ocl::Kernel,
	pub col_ids: Envoy<ocl::cl_uchar>,
	pub wins: Envoy<ocl::cl_uchar>,
	pub states: Envoy<ocl::cl_uchar>,
	
}

impl PeakColumn {
	pub fn new(col_width: u32, depth: u8, region: &CorticalRegion, src_states: &Envoy<ocl::cl_uchar>, ocl: &Ocl) -> PeakColumn {

		let width = col_width >> cmn::ASPINY_SPAN_LOG2;

		let padding = cmn::ASPINY_SPAN;

		let col_ids = Envoy::<ocl::cl_uchar>::with_padding(padding, width, depth, 0u8, ocl);
		let wins = Envoy::<ocl::cl_uchar>::with_padding(padding, width, depth, 0u8, ocl);
		let states = Envoy::<ocl::cl_uchar>::with_padding(padding, width, depth, cmn::STATE_ZERO, ocl);

		let mut kern_cycle_pre = ocl.new_kernel("peak_col_cycle_pre", 
			WorkSize::TwoDim(depth as usize, width as usize))
			.arg_env(&src_states)
			.arg_env(&states)
			.arg_env(&col_ids)
		;

		let mut kern_cycle_wins = ocl.new_kernel("peak_col_cycle_wins", 
			WorkSize::TwoDim(depth as usize, width as usize))
			.arg_env(&states)
			//.arg_env(&col_ids)
			.arg_env(&wins)
		;

		let mut kern_cycle_post = ocl.new_kernel("peak_col_cycle_post", 
			WorkSize::TwoDim(depth as usize, width as usize))
			.arg_env(&wins)
			//.arg_env(&col_ids)
			.arg_env(&states)
		;


		PeakColumn {
			width: width,
			depth: depth,
			kern_cycle_pre: kern_cycle_pre,
			kern_cycle_wins: kern_cycle_wins,
			kern_cycle_post: kern_cycle_post,
			col_ids: col_ids,
			wins: wins,
			states: states,
		}
	}

	pub fn cycle(&mut self) {
		let mut event = self.kern_cycle_pre.enqueue();

		//println!("\n### New aspiny.cycle() iteration: ###");

		for i in 0..8 {
			self.kern_cycle_wins.enqueue();
		}

		self.kern_cycle_post.enqueue();
	}

	pub fn width(&self) -> u32 {
		self.width
	}

}
