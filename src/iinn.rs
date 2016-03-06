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
use ocl::{ Kernel, ProQue, SpatialDims, Buffer };
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

    kern_inhib_simple: Kernel,
    kern_inhib_passthrough: Kernel,

    pub spi_ids: Buffer<u8>,
    pub wins: Buffer<u8>,
    pub states: Buffer<u8>,
    
}

impl InhibitoryInterneuronNetwork {
    pub fn new(layer_name: &'static str, dims: CorticalDims, protocell: Protocell, area_map: &AreaMap, src_soma: &Buffer<u8>, src_base_axn_slc: u8, axns: &AxonSpace, /*aux: &Aux,*/ ocl_pq: &ProQue) -> InhibitoryInterneuronNetwork {

        //let dims.width = col_dims.width >> cmn::ASPINY_SPAN_LOG2;

        //let dims = col_dims;

        //let padding = cmn::ASPINY_SPAN;
        //let padding = 0;

        // let spi_ids = Buffer::<u8>::with_padding(dims, 0u8, ocl, padding);
        // let wins = Buffer::<u8>::with_padding(dims, 0u8, ocl, padding);
        // let states = Buffer::<u8>::with_padding(dims, cmn::STATE_ZERO, ocl, padding);

        let spi_ids = Buffer::<u8>::newer_new(ocl_pq.queue(), None, &dims, None).unwrap();
        let wins = Buffer::<u8>::newer_new(ocl_pq.queue(), None, &dims, None).unwrap();
        let states = Buffer::<u8>::newer_new(ocl_pq.queue(), None, &dims, None).unwrap();


        let kern_inhib_simple = ocl_pq.create_kernel("inhib_simple")
            // .expect("InhibitoryInterneuronNetwork::new()")
            .gws(SpatialDims::Three(dims.depth() as usize, dims.v_size() as usize, 
                dims.u_size() as usize))
            .lws(SpatialDims::Three(1, 8, 8 as usize))
            .arg_buf(&src_soma)
            .arg_scl(src_base_axn_slc)
            // .arg_buf_named("aux_ints_0", None)
            // .arg_buf_named("aux_ints_1", None)
            .arg_buf(&axns.states);

        let kern_inhib_passthrough = ocl_pq.create_kernel("inhib_passthrough")
            // .expect("InhibitoryInterneuronNetwork::new()")
            //.lws(SpatialDims::Three(1, 8, 8 as usize))
            .gws(SpatialDims::Three(dims.depth() as usize, dims.v_size() as usize, 
                dims.u_size() as usize))
            .arg_buf(&src_soma)
            .arg_scl(src_base_axn_slc)
            .arg_buf(&axns.states);

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
        // self.kern_cycle_pre.enqueue(); 


        // for i in 0..1 { // <<<<< (was 0..8)
        //      self.kern_cycle_wins.enqueue(); 
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
