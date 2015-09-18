use num;
use std::ops;
use rand;
use std::mem;
use rand::distributions::{ Normal, IndependentSample, Range };
use rand::{ ThreadRng };
use num::{ Integer };
use std::default::{ Default };
use std::fmt::{ Display };

use cmn;
use ocl::{ self, OclProgQueue, WorkSize, Envoy, CorticalDimensions };
use proto::areas::{ Protoareas };
use proto::regions::{ Protoregion, ProtoregionKind };
use proto::cell::{ ProtocellKind, Protocell, DendriteKind };
use synapses::{ Synapses };
use dendrites::{ Dendrites };
use axons::{ Axons };
use minicolumns::{ Minicolumns };
use cortical_area:: { Aux };



pub struct InhibitoryInterneuronNetworkNew {
	layer_name: &'static str,
	pub dims: CorticalDimensions,
	protocell: Protocell,
	kern_cycle_pre: ocl::Kernel,
	kern_cycle_wins: ocl::Kernel,
	kern_cycle_post: ocl::Kernel,
	kern_post_inhib: ocl::Kernel,
	pub spi_ids: Envoy<ocl::cl_uchar>,
	pub wins: Envoy<ocl::cl_uchar>,
	pub states: Envoy<ocl::cl_uchar>,
}

impl InhibitoryInterneuronNetworkNew {
	pub fn new(layer_name: &'static str, col_dims: CorticalDimensions, protocell: Protocell, region: &Protoregion, src_soma: &Envoy<u8>, src_axn_base_slc: u8, axns: &Axons, ocl: &OclProgQueue) -> InhibitoryInterneuronNetworkNew {

		//let dims.width = col_dims.width >> cmn::ASPINY_SPAN_LOG2;

		let dims = col_dims.clone_with_ptl2(0 - cmn::ASPINY_SPAN_LOG2 as i8);

		let padding = cmn::ASPINY_SPAN;

		let spi_ids = Envoy::<ocl::cl_uchar>::with_padding(padding, dims, 0u8, ocl);
		let wins = Envoy::<ocl::cl_uchar>::with_padding(padding, dims, 0u8, ocl);
		let states = Envoy::<ocl::cl_uchar>::with_padding(padding, dims, cmn::STATE_ZERO, ocl);

		let kern_inhib_1 = ocl.new_kernel("inhib_1",
			WorkSize::TwoDim(dims.depth() as usize, dims.per_slc() as usize))
			.arg_env(&src_soma)
		;


		let kern_cycle_pre = ocl.new_kernel("peak_sst_cycle_pre", 
			WorkSize::TwoDim(dims.depth() as usize, dims.per_slc() as usize))
			.arg_env(&src_soma)
			.arg_env(&states)
			.arg_env(&spi_ids)
		;

		let kern_cycle_wins = ocl.new_kernel("peak_sst_cycle_wins", 
			WorkSize::TwoDim(dims.depth() as usize, dims.per_slc() as usize))
			.arg_env(&states)
			//.arg_env(&spi_ids)
			.arg_env(&wins)
		;

		let kern_cycle_post = ocl.new_kernel("peak_sst_cycle_post", 
			WorkSize::TwoDim(dims.depth() as usize, dims.per_slc() as usize))
			.arg_env(&wins)
			//.arg_env(&spi_ids)
			.arg_env(&states)
		;

		let kern_post_inhib = ocl.new_kernel("sst_post_inhib_unoptd", WorkSize::TwoDim(dims.depth() as usize, dims.columns() as usize))
			.arg_env(&spi_ids)
			.arg_env(&states)
			.arg_env(&wins)
			.arg_scl(src_axn_base_slc)
			.arg_env(src_soma)
			.arg_env(&axns.states)
		;


		InhibitoryInterneuronNetworkNew {
			layer_name: layer_name,
			dims: dims,
			protocell: protocell,
			kern_cycle_pre: kern_cycle_pre,
			kern_cycle_wins: kern_cycle_wins,
			kern_cycle_post: kern_cycle_post,
			kern_post_inhib: kern_post_inhib,
			spi_ids: spi_ids,
			wins: wins,
			states: states,
		}
	}

	pub fn cycle(&mut self) {
		self.kern_cycle_pre.enqueue(); 
		//let mut event = self.kern_cycle_pre.enqueue();

		//println!("\n### New aspiny.cycle() iteration: ###");

		for i in 0..4 { // <<<<< (was 0..8)
			self.kern_cycle_wins.enqueue(); 
		}

		self.kern_cycle_post.enqueue();
		self.kern_post_inhib.enqueue();
	}

}
