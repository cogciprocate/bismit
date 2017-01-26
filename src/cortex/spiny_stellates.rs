use std::ops;
use rand::{self, Rng};

use cmn::{self, CmnResult, CorticalDims};
use map::{AreaMap};
use ocl::{Kernel, ProQue, SpatialDims, Buffer};
use ocl::core::ClWaitList;
use map::{CellKind, CellScheme, DendriteKind, ExecutionGraph, ExecutionCommand,
    CorticalBuffer, LayerAddress};
use cortex::{Dendrites, AxonSpace};


const TUFT_COUNT: usize = 1;


pub struct SpinyStellateLayer {
    layer_name: &'static str,
    layer_id: usize,
    dims: CorticalDims,
    // cell_scheme: CellScheme,
    axn_slc_ids: Vec<u8>,
    // base_axn_slc: u8,
    lyr_axn_idz: u32,
    kern_ltp: Kernel,
    rng: rand::XorShiftRng,
    pub dens: Dendrites,
    ltp_exe_cmd_idx: usize,
}

impl SpinyStellateLayer {
    pub fn new(layer_name: &'static str, layer_id: usize, dims: CorticalDims, cell_scheme: CellScheme,
            area_map: &AreaMap, axons: &AxonSpace, ocl_pq: &ProQue, exe_graph: &mut ExecutionGraph,
    ) -> CmnResult<SpinyStellateLayer> {
        let layer_addr = LayerAddress::new(area_map.area_id(), layer_id);
        let axn_slc_ids = area_map.layer_slc_ids(&[layer_name.to_owned()]);
        let base_axn_slc = axn_slc_ids[0];
        let lyr_axn_idz = area_map.axn_idz(base_axn_slc);

        let tft_count = cell_scheme.tft_schemes().len();
        // Redesign kernel before changing the 1 tuft limitation:
        assert![tft_count == TUFT_COUNT];
        let sst_tft_id = 0;
        let tft_scheme = &cell_scheme.tft_schemes()[sst_tft_id];

        let syns_per_tuft_l2: u8 = tft_scheme.syns_per_den_l2() + tft_scheme.dens_per_tft_l2();

        println!("{mt}{mt}SPINYSTELLATES::NEW(): base_axn_slc: {}, lyr_axn_idz: {}, dims: {:?}",
            base_axn_slc, lyr_axn_idz, dims, mt = cmn::MT);

        // let dens_dims = dims.clone_with_ptl2(cell_scheme.dens_per_tft_l2 as i8);
        let dens = try!(Dendrites::new(layer_name, layer_id, dims, cell_scheme.clone(),
            DendriteKind::Proximal, CellKind::SpinyStellate, area_map, axons, ocl_pq, exe_graph));
        let grp_count = cmn::OPENCL_MINIMUM_WORKGROUP_SIZE;
        let cels_per_grp = dims.per_subgrp(grp_count).expect("SpinyStellateLayer::new()");

        let kern_ltp = ocl_pq.create_kernel("sst_ltp").expect("[FIXME]: HANDLE ME")
            // .expect("SpinyStellateLayer::new()")
            .gws(SpatialDims::Two(tft_count, grp_count as usize))
            .arg_buf(&axons.states)
            .arg_buf(dens.syns().states())
            .arg_scl(lyr_axn_idz)
            .arg_scl(cels_per_grp)
            .arg_scl(syns_per_tuft_l2)
            .arg_scl_named::<u32>("rnd", None)
            // .arg_buf_named("aux_ints_0", None)
            // .arg_buf_named("aux_ints_1", None)
            .arg_buf(dens.syns().strengths());

        // Set up execution command:
        let mut ltp_cmd_srcs: Vec<CorticalBuffer> = axn_slc_ids.iter()
            .map(|&slc_id|
                CorticalBuffer::axon_slice(&axons.states, layer_addr.area_id(), slc_id))
            .collect();

        ltp_cmd_srcs.push(CorticalBuffer::data_syn_tft(dens.syns().states(), layer_addr, sst_tft_id));

        let ltp_exe_cmd_idx = exe_graph.add_command(ExecutionCommand::cortical_kernel(
            ltp_cmd_srcs,
            vec![CorticalBuffer::data_syn_tft(dens.syns().strengths(), layer_addr, sst_tft_id)]
        ))?;

        Ok(SpinyStellateLayer {
            layer_name: layer_name,
            layer_id: layer_id,
            dims: dims,
            // cell_scheme: cell_scheme,
            axn_slc_ids: axn_slc_ids,
            // base_axn_slc: base_axn_slc,
            lyr_axn_idz: lyr_axn_idz,
            kern_ltp: kern_ltp,
            rng: rand::weak_rng(),
            dens: dens,
            ltp_exe_cmd_idx: ltp_exe_cmd_idx,
        })
    }

    pub fn set_exe_order(&self, exe_graph: &mut ExecutionGraph) -> CmnResult<()> {
        self.dens.set_exe_order(exe_graph)?;
        exe_graph.order_next(self.ltp_exe_cmd_idx)?;
        Ok(())
    }

    #[inline]
    pub fn cycle(&self, wait_events: Option<&ClWaitList>) {
        self.dens.cycle(wait_events);
    }


    #[inline]
    pub fn learn(&mut self) {
        let rnd = self.rng.gen::<u32>();
        self.kern_ltp.set_arg_scl_named("rnd", rnd).unwrap();
        self.kern_ltp.enq().expect("[FIXME]: HANDLE ME!");
    }

    #[inline] pub fn regrow(&mut self) {
        self.dens.regrow();
    }

    #[inline]
    pub fn axn_range(&self) -> ops::Range<usize> {
        let ssts_axn_idn = self.lyr_axn_idz + (self.dims.cells());
        self.lyr_axn_idz as usize..ssts_axn_idn as usize
    }

    #[inline] pub fn soma(&self) -> &Buffer<u8> { self.dens.states() }
    #[inline] pub fn dims(&self) -> &CorticalDims { &self.dims }
    #[inline] pub fn axn_slc_ids(&self) -> &[u8] { self.axn_slc_ids.as_slice() }
    #[inline] pub fn base_axn_slc(&self) -> u8 { self.axn_slc_ids[0] }
    #[inline] pub fn tft_count(&self) -> usize { TUFT_COUNT }
    #[inline] pub fn layer_name(&self) -> &'static str { self.layer_name }
    #[inline] pub fn layer_id(&self) -> usize { self.layer_id }
    #[inline] pub fn dens(&self) -> &Dendrites { &self.dens }
    #[inline] pub fn dens_mut(&mut self) -> &mut Dendrites { &mut self.dens }
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