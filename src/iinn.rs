// use num;
// use std::ops;
// use rand;
// use std::mem;
// use rand::distributions::{ Normal, IndependentSample, Range };
// use rand::{ ThreadRng };
// use num::{ Integer };
// use std::default::{ Default };
// use std::fmt::{ Display };

use cmn::{ CorticalDims };
use map::{ AreaMap };
use ocl::{ self, ProQue, WorkSize, Buffer };
use proto::{ /*ProtolayerMap, LayerMapKind, ProtoareaMaps, CellKind,*/ Protocell, /*DendriteKind*/ };
// use synapses::{ Synapses };
// use dendrites::{ Dendrites };
use axon_space::{ AxonSpace };
// use minicolumns::{ Minicolumns };
// use cortical_area:: { Aux };



pub struct InhibitoryInterneuronNetwork {
	layer_name: &'static str,
	pub dims: CorticalDims,
	protocell: Protocell,
	//kern_cycle_pre: ocl::Kernel,
	//kern_cycle_wins: ocl::Kernel,
	//kern_cycle_post: ocl::Kernel,
	//kern_post_inhib: ocl::Kernel,

	kern_inhib_simple: ocl::Kernel,
	kern_inhib_passthrough: ocl::Kernel,

	pub spi_ids: Buffer<ocl::cl_uchar>,
	pub wins: Buffer<ocl::cl_uchar>,
	pub states: Buffer<ocl::cl_uchar>,
	
}

impl InhibitoryInterneuronNetwork {
	pub fn new(layer_name: &'static str, dims: CorticalDims, protocell: Protocell, area_map: &AreaMap, src_soma: &Buffer<u8>, src_base_axn_slc: u8, axns: &AxonSpace, /*aux: &Aux,*/ ocl_pq: &ProQue) -> InhibitoryInterneuronNetwork {

		//let dims.width = col_dims.width >> cmn::ASPINY_SPAN_LOG2;

		//let dims = col_dims;

		//let padding = cmn::ASPINY_SPAN;
		//let padding = 0;

		// let spi_ids = Buffer::<ocl::cl_uchar>::with_padding(dims, 0u8, ocl, padding);
		// let wins = Buffer::<ocl::cl_uchar>::with_padding(dims, 0u8, ocl, padding);
		// let states = Buffer::<ocl::cl_uchar>::with_padding(dims, cmn::STATE_ZERO, ocl, padding);

		let spi_ids = Buffer::<ocl::cl_uchar>::with_vec(dims, ocl_pq.queue());
		let wins = Buffer::<ocl::cl_uchar>::with_vec(dims, ocl_pq.queue());
		let states = Buffer::<ocl::cl_uchar>::with_vec(dims, ocl_pq.queue());


		let kern_inhib_simple = ocl_pq.create_kernel("inhib_simple",
			WorkSize::ThreeDims(dims.depth() as usize, dims.v_size() as usize, dims.u_size() as usize))
			.lws(WorkSize::ThreeDims(1, 8, 8 as usize))
			.arg_buf(&src_soma)
			.arg_scl(src_base_axn_slc)
			// .arg_buf_named("aux_ints_0", None)
			// .arg_buf_named("aux_ints_1", None)
			.arg_buf(&axns.states)
		;

		let kern_inhib_passthrough = ocl_pq.create_kernel("inhib_passthrough",
			WorkSize::ThreeDims(dims.depth() as usize, dims.v_size() as usize, dims.u_size() as usize))
			//.lws(WorkSize::ThreeDims(1, 8, 8 as usize))
			.arg_buf(&src_soma)
			.arg_scl(src_base_axn_slc)
			.arg_buf(&axns.states)
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

	#[inline]
	pub fn cycle(&mut self, bypass: bool) {
		// self.kern_cycle_pre.enqueue(None, None); 


		// for i in 0..1 { // <<<<< (was 0..8)
		//  	self.kern_cycle_wins.enqueue(None, None); 
		// }

		// self.kern_cycle_post.enqueue(None, None);
		// self.kern_post_inhib.enqueue(None, None);
		if bypass {
			self.kern_inhib_passthrough.enqueue(None, None);
		} else {
			self.kern_inhib_simple.enqueue(None, None);
		}
	}

}
