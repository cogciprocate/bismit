use cmn;
use ocl::{ self, Ocl, WorkSize, Envoy, CorticalDimensions };
use proto::areas::{ ProtoAreas };
use proto::regions::{ ProtoRegion, ProtoRegionKind };
use proto::cell::{ CellKind, Protocell, DendriteKind };
use synapses::{ Synapses };
use dendrites::{ Dendrites };
use axons::{ Axons };
use minicolumns::{ MiniColumns };
use cortical_area:: { Aux };

use num;
use std::ops;
use rand;
use std::mem;
use rand::distributions::{ Normal, IndependentSample, Range };
use rand::{ ThreadRng };
use num::{ Integer };
use std::default::{ Default };
use std::fmt::{ Display };


pub struct PeakColumns {
	pub dims: CorticalDimensions,
	kern_cycle_pre: ocl::Kernel,
	kern_cycle_wins: ocl::Kernel,
	kern_cycle_post: ocl::Kernel,
	pub spi_ids: Envoy<ocl::cl_uchar>,
	pub wins: Envoy<ocl::cl_uchar>,
	pub states: Envoy<ocl::cl_uchar>,
	
}

impl PeakColumns {
	pub fn new(col_dims: CorticalDimensions, region: &ProtoRegion, src_states: &Envoy<ocl::cl_uchar>, ocl: &Ocl) -> PeakColumns {

		//let dims.width = col_dims.width >> cmn::ASPINY_SPAN_LOG2;

		let dims = CorticalDimensions::new(col_dims.width() >> cmn::ASPINY_SPAN_LOG2, col_dims.height(), col_dims.depth(), 0);

		let padding = cmn::ASPINY_SPAN;

		let spi_ids = Envoy::<ocl::cl_uchar>::with_padding(padding, dims, 0u8, ocl);
		let wins = Envoy::<ocl::cl_uchar>::with_padding(padding, dims, 0u8, ocl);
		let states = Envoy::<ocl::cl_uchar>::with_padding(padding, dims, cmn::STATE_ZERO, ocl);

		let mut kern_cycle_pre = ocl.new_kernel("peak_spi_cycle_pre", 
			WorkSize::TwoDim(dims.depth() as usize, dims.per_slice() as usize))
			.arg_env(&src_states)
			.arg_env(&states)
			.arg_env(&spi_ids)
		;

		let mut kern_cycle_wins = ocl.new_kernel("peak_spi_cycle_wins", 
			WorkSize::TwoDim(dims.depth() as usize, dims.per_slice() as usize))
			.arg_env(&states)
			//.arg_env(&spi_ids)
			.arg_env(&wins)
		;

		let mut kern_cycle_post = ocl.new_kernel("peak_spi_cycle_post", 
			WorkSize::TwoDim(dims.depth() as usize, dims.per_slice() as usize))
			.arg_env(&wins)
			//.arg_env(&spi_ids)
			.arg_env(&states)
		;


		PeakColumns {
			dims: dims,
			kern_cycle_pre: kern_cycle_pre,
			kern_cycle_wins: kern_cycle_wins,
			kern_cycle_post: kern_cycle_post,
			spi_ids: spi_ids,
			wins: wins,
			states: states,
		}
	}

	pub fn cycle(&mut self) {
		self.kern_cycle_pre.enqueue(); 
		//let mut event = self.kern_cycle_pre.enqueue();

		//println!("\n### New aspiny.cycle() iteration: ###");

		for i in 0..4 { // ***** (was 0..8)
			self.kern_cycle_wins.enqueue(); 
		}

		self.kern_cycle_post.enqueue();
	}

}
