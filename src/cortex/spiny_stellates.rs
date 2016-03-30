use std::ops;
use rand::{self, Rng};

use cmn::{self, CorticalDims};
use map::{AreaMap};
use ocl::{Kernel, ProQue, SpatialDims, Buffer, EventList};
use proto::{CellKind, Protocell, DendriteKind};
use cortex::{Dendrites, AxonSpace};


pub struct SpinyStellateLayer {
    layer_name: &'static str,
    dims: CorticalDims,
    // protocell: Protocell,
    base_axn_slc: u8,
    lyr_axn_idz: u32,
    kern_ltp: Kernel,
    rng: rand::XorShiftRng,
    pub dens: Dendrites,
}

impl SpinyStellateLayer {
    pub fn new(layer_name: &'static str, dims: CorticalDims, protocell: Protocell, area_map: &AreaMap, 
                axns: &AxonSpace, ocl_pq: &ProQue
    ) -> SpinyStellateLayer {
        let base_axn_slcs = area_map.layer_slc_ids(vec![layer_name]);
        let base_axn_slc = base_axn_slcs[0];
        let lyr_axn_idz = area_map.axn_idz(base_axn_slc);

        let syns_per_tuft_l2: u8 = protocell.syns_per_den_l2 + protocell.dens_per_tuft_l2;

        println!("{mt}{mt}SPINYSTELLATES::NEW(): base_axn_slc: {}, lyr_axn_idz: {}, dims: {:?}", 
            base_axn_slc, lyr_axn_idz, dims, mt = cmn::MT);

        let dens_dims = dims.clone_with_ptl2(protocell.dens_per_tuft_l2 as i8);
        let dens = Dendrites::new(layer_name, dens_dims, protocell.clone(), DendriteKind::Proximal, 
            CellKind::SpinyStellate, area_map, axns, ocl_pq);
        let grp_count = cmn::OPENCL_MINIMUM_WORKGROUP_SIZE;
        let cels_per_grp = dims.per_subgrp(grp_count).expect("SpinyStellateLayer::new()");

        let kern_ltp = ocl_pq.create_kernel("sst_ltp").expect("[FIXME]: HANDLE ME")
            // .expect("SpinyStellateLayer::new()")
            .gws(SpatialDims::Two(dims.tfts_per_cel() as usize, grp_count as usize))
            .arg_buf(&axns.states)
            .arg_buf(dens.syns().states())
            .arg_scl(lyr_axn_idz)
            .arg_scl(cels_per_grp)
            .arg_scl(syns_per_tuft_l2)
            .arg_scl_named::<u32>("rnd", None)
            // .arg_buf_named("aux_ints_0", None)
            // .arg_buf_named("aux_ints_1", None)
            .arg_buf(dens.syns().strengths());

        SpinyStellateLayer {
            layer_name: layer_name,
            dims: dims,
            // protocell: protocell,
            base_axn_slc: base_axn_slc,
            lyr_axn_idz: lyr_axn_idz,
            kern_ltp: kern_ltp,
            rng: rand::weak_rng(),
            dens: dens,
        }
    }

    #[inline]
    pub fn cycle(&self, wait_events: Option<&EventList>) {
        self.dens.cycle(wait_events);
    }


    #[inline]
    pub fn learn(&mut self) {
        let rnd = self.rng.gen::<u32>();
        self.kern_ltp.set_arg_scl_named("rnd", rnd).unwrap();
        self.kern_ltp.enq().expect("[FIXME]: HANDLE ME!");
    }

    #[inline]
    pub fn regrow(&mut self) {
        self.dens.regrow();
    }

    // #[inline]
    // pub fn confab(&mut self) {
    //     self.dens.confab();
    // } 

    #[inline]
    pub fn soma(&self) -> &Buffer<u8> {
        self.dens.states()
    }

    // #[inline]
    // pub fn soma_mut(&mut self) -> &mut Buffer<u8> {
    //     &mut self.dens.states
    // }

    #[inline]
    pub fn dims(&self) -> &CorticalDims {
        &self.dims
    }    

    #[inline]
    pub fn base_axn_slc(&self) -> u8 {
        self.base_axn_slc
    }

    #[inline]
    pub fn layer_name(&self) -> &'static str {
        self.layer_name
    }

    // pub fn print_cel(&mut self, cel_idx: usize) {
    //     let emsg = "SpinyStellateLayer::print()";

    //     let cel_syn_idz = (cel_idx << self.dens.syns().dims().per_tft_l2_left()) as usize;
    //     let per_cel = self.dens.syns().dims().per_cel() as usize;
    //     let cel_syn_range = cel_syn_idz..(cel_syn_idz + per_cel);

    //     println!("\ncell.state[{}]: {}", cel_idx, self.dens.states[cel_idx]);

    //     println!("cell.syns.states[{:?}]: ", cel_syn_range.clone()); 
    //     self.dens.syns_mut().states.print(1, None, Some(cel_syn_range.clone()), false);
    //     // cmn::fmt::print_slice(&self.dens.syns_mut().states.vec()[..], 1, None, 
    //     //     Some(cel_syn_range.clone()), false);

    //     println!("cell.syns.strengths[{:?}]: ", cel_syn_range.clone()); 
    //     self.dens.syns_mut().strengths.print(1, None, Some(cel_syn_range.clone()), false);
    //     // cmn::fmt::print_slice(&self.dens.syns_mut().strengths.vec()[..], 1, None, 
    //     //     Some(cel_syn_range.clone()), false);

    //     println!("cell.syns.src_col_v_offs[{:?}]: ", cel_syn_range.clone()); 
    //     self.dens.syns_mut().src_col_v_offs.print(1, None, Some(cel_syn_range.clone()), false);
    //     // cmn::fmt::print_slice(&self.dens.syns_mut().src_col_v_offs.vec()[..], 1, None, 
    //         // Some(cel_syn_range.clone()), false);

    //     println!("cell.syns.src_col_u_offs[{:?}]: ", cel_syn_range.clone()); 
    //     self.dens.syns_mut().src_col_u_offs.print(1, None, Some(cel_syn_range.clone()), false);
    //     // cmn::fmt::print_slice(&self.dens.syns_mut().src_col_u_offs.vec()[..], 1, None, 
    //     //     Some(cel_syn_range.clone()), false);
    // }

    #[inline]
    pub fn dens(&self) -> &Dendrites {
        &self.dens
    }

    #[inline]
    pub fn dens_mut(&mut self) -> &mut Dendrites {
        &mut self.dens
    }

    #[inline]
    pub fn axn_range(&self) -> ops::Range<usize> {
        let ssts_axn_idn = self.lyr_axn_idz + (self.dims.per_slc());
        self.lyr_axn_idz as usize..ssts_axn_idn as usize
    }
}
