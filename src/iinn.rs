// use num;
// use std::ops;
// use rand;
// use std::mem;
// use rand::distributions::{ Normal, IndependentSample, Range };
// use rand::{ ThreadRng };
// use num::{ Integer };
// use std::default::{ Default };
// use std::fmt::{ Display };

use cmn::{ self, CorticalDimensions };
use map::{ AreaMap };
use ocl::{ self, OclProgQueue, WorkSize, Envoy };
use proto::{ /*ProtoLayerMap, RegionKind, ProtoAreaMaps, ProtocellKind,*/ Protocell, /*DendriteKind*/ };
// use synapses::{ Synapses };
// use dendrites::{ Dendrites };
use axon_space::{ AxonSpace };
// use minicolumns::{ Minicolumns };
// use cortical_area:: { Aux };



pub struct InhibitoryInterneuronNetwork {
	layer_name: &'static str,
	pub dims: CorticalDimensions,
	protocell: Protocell,
	//kern_cycle_pre: ocl::Kernel,
	//kern_cycle_wins: ocl::Kernel,
	//kern_cycle_post: ocl::Kernel,
	//kern_post_inhib: ocl::Kernel,

	kern_inhib_simple: ocl::Kernel,
	kern_inhib_passthrough: ocl::Kernel,

	pub spi_ids: Envoy<ocl::cl_uchar>,
	pub wins: Envoy<ocl::cl_uchar>,
	pub states: Envoy<ocl::cl_uchar>,
	
}

impl InhibitoryInterneuronNetwork {
	pub fn new(layer_name: &'static str, dims: CorticalDimensions, protocell: Protocell, area_map: &AreaMap, src_soma: &Envoy<u8>, src_base_axn_slc: u8, axns: &AxonSpace, /*aux: &Aux,*/ ocl: &OclProgQueue) -> InhibitoryInterneuronNetwork {

		//let dims.width = col_dims.width >> cmn::ASPINY_SPAN_LOG2;

		//let dims = col_dims;

		//let padding = cmn::ASPINY_SPAN;
		//let padding = 0;

		// let spi_ids = Envoy::<ocl::cl_uchar>::with_padding(dims, 0u8, ocl, padding);
		// let wins = Envoy::<ocl::cl_uchar>::with_padding(dims, 0u8, ocl, padding);
		// let states = Envoy::<ocl::cl_uchar>::with_padding(dims, cmn::STATE_ZERO, ocl, padding);

		let spi_ids = Envoy::<ocl::cl_uchar>::new(dims, cmn::STATE_ZERO, ocl);
		let wins = Envoy::<ocl::cl_uchar>::new(dims, cmn::STATE_ZERO, ocl);
		let states = Envoy::<ocl::cl_uchar>::new(dims, cmn::STATE_ZERO, ocl);


		let kern_inhib_simple = ocl.new_kernel("inhib_simple".to_string(),
			WorkSize::ThreeDim(dims.depth() as usize, dims.v_size() as usize, dims.u_size() as usize))
			.lws(WorkSize::ThreeDim(1, 8, 8 as usize))
			.arg_env(&src_soma)
			.arg_scl(src_base_axn_slc)
			// .arg_env_named("aux_ints_0", None)
			// .arg_env_named("aux_ints_1", None)
			.arg_env(&axns.states)
		;

		let kern_inhib_passthrough = ocl.new_kernel("inhib_passthrough".to_string(),
			WorkSize::ThreeDim(dims.depth() as usize, dims.v_size() as usize, dims.u_size() as usize))
			//.lws(WorkSize::ThreeDim(1, 8, 8 as usize))
			.arg_env(&src_soma)
			.arg_scl(src_base_axn_slc)
			.arg_env(&axns.states)
		;

		InhibitoryInterneuronNetwork {
			layer_name: layer_name,
			dims: dims,
			protocell: protocell,
			//kern_cycle_pre: kern_cycle_pre,
			//kern_cycle_wins: kern_cycle_wins,
			//kern_cycle_post: kern_cycle_post,
			//kern_post_inhib: kern_post_inhib,

			kern_inhib_simple: kern_inhib_simple,
			kern_inhib_passthrough: kern_inhib_passthrough,

			spi_ids: spi_ids,
			wins: wins,
			states: states,
		}
	}

	pub fn cycle(&mut self, bypass: bool) {
		// self.kern_cycle_pre.enqueue(); 


		// for i in 0..1 { // <<<<< (was 0..8)
		//  	self.kern_cycle_wins.enqueue(); 
		// }

		// self.kern_cycle_post.enqueue();
		// self.kern_post_inhib.enqueue();
		if bypass {
			self.kern_inhib_passthrough.enqueue();
		} else {
			self.kern_inhib_simple.enqueue();
		}
	}

}
