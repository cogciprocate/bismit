#![allow(unused_imports)]

use rand::Rng;
use cmn::{self, CmnResult, CorticalDims, XorShiftRng};
use ocl::{ProQue, SpatialDims, Buffer, Kernel, Result as OclResult, Event};
use std::collections::BTreeMap;
use ocl::traits::OclPrm;
use map::{AreaMap, CellScheme, DendriteKind, ExecutionGraph, CommandRelations,
    CorticalBuffer, LayerAddress, LayerTags, CommandUid};
use cortex::{Dendrites, AxonSpace, CorticalAreaSettings, DataCellLayer, ControlCellLayers};

const PRINT_DEBUG: bool = false;

#[derive(Debug)]
pub struct Tufts {
    layer_name: String,
    layer_addr: LayerAddress,
    dims: CorticalDims,
    tft_count: usize,
    ltp_kernels: Vec<Kernel>,
    cycle_kernels: Vec<Kernel>,

    best_den_ids: Buffer<u8>,
    best_den_states_raw: Buffer<u8>,
    best_den_states: Buffer<u8>,
    states: Buffer<u8>,

    cycle_exe_cmd_uids: Vec<CommandUid>,
    cycle_exe_cmd_idxs: Vec<usize>,
    ltp_exe_cmd_uids: Vec<CommandUid>,
    ltp_exe_cmd_idxs: Vec<usize>,

    pub dens: Dendrites,
    settings: CorticalAreaSettings,
    rng: XorShiftRng,
}

impl Tufts {
    pub fn new<S: Into<String>>(
            layer_name: S,
            layer_addr: LayerAddress,
            dims: CorticalDims,
            cell_scheme: CellScheme,
            den_kind: DendriteKind,
            area_map: &AreaMap,
            axons: &AxonSpace,
            cel_axn_slc_ids: &[u8],
            cel_lyr_axn_idz: u32,
            cel_states: &Buffer<u8>,
            cel_flag_sets: &Buffer<u8>,
            ocl_pq: &ProQue,
            settings: CorticalAreaSettings,
            exe_graph: &mut ExecutionGraph)
            -> CmnResult<Tufts> {
        let layer_name = layer_name.into();
        let tft_count = cell_scheme.tft_count();
        let cel_count = dims.to_len();
        let celtft_count = cel_count * tft_count;

        let best_den_ids = Buffer::<u8>::builder().queue(ocl_pq.queue().clone()).dims([celtft_count]).fill_val(0).build()?;
        let best_den_states_raw = Buffer::<u8>::builder().queue(ocl_pq.queue().clone()).dims([celtft_count]).fill_val(0).build()?;
        let best_den_states = Buffer::<u8>::builder().queue(ocl_pq.queue().clone()).dims([celtft_count]).fill_val(0).build()?;
        let states = Buffer::<u8>::builder().queue(ocl_pq.queue().clone()).dims([celtft_count]).fill_val(0).build()?;

        let dens = Dendrites::new(layer_name.clone(), layer_addr.layer_id(), dims, cell_scheme.clone(),
            den_kind, area_map, axons, ocl_pq, settings.disable_pyrs, exe_graph)?;

        let mut ltp_kernels = Vec::with_capacity(tft_count);
        let mut cycle_kernels = Vec::with_capacity(tft_count);
        let mut cycle_exe_cmd_uids = Vec::with_capacity(tft_count);
        let cycle_exe_cmd_idxs = Vec::with_capacity(tft_count);
        let mut ltp_exe_cmd_uids = Vec::with_capacity(tft_count);
        let ltp_exe_cmd_idxs = Vec::with_capacity(tft_count);
        let mut den_count_ttl = 0u32;
        let mut syn_count_ttl = 0u32;

        for (tft_id, tft_scheme) in cell_scheme.tft_schemes().iter().enumerate() {
            let dens_per_tft_l2 = tft_scheme.dens_per_tft_l2();
            let syns_per_den_l2 = tft_scheme.syns_per_den_l2();
            let syns_per_tft_l2 = dens_per_tft_l2 + syns_per_den_l2;
            let tft_cel_idz = tft_id as u32 * dims.cells();

            // Dendrites:
            let tft_den_idz = den_count_ttl;
            let tft_den_count = dims.cells() << dens_per_tft_l2;
            den_count_ttl += tft_den_count;

            // Synapses:
            let tft_syn_idz = syn_count_ttl;
            let tft_syn_count = dims.cells() << syns_per_tft_l2;
            syn_count_ttl += tft_syn_count;

            /*=============================================================================
            ===============================================================================
            =============================================================================*/

            let kern_name = "tft_cycle";
            cycle_kernels.push(ocl_pq.create_kernel(kern_name)?
                .gws(SpatialDims::One(cel_count))
                .arg_buf(dens.states_raw())
                .arg_buf(dens.states())
                .arg_scl(tft_cel_idz)
                .arg_scl(tft_den_idz)
                .arg_scl(dens_per_tft_l2)
                .arg_buf(&best_den_ids)
                .arg_buf(&best_den_states_raw)
                .arg_buf(&best_den_states)
                .arg_buf_named("aux_ints_0", None::<Buffer<i32>>)
                .arg_buf_named("aux_ints_1", None::<Buffer<i32>>)
                .arg_buf(&states)
            );

            if !settings.disable_pyrs {
                cycle_exe_cmd_uids.push(exe_graph.add_command(CommandRelations::cortical_kernel(
                    kern_name,
                    vec![
                        CorticalBuffer::data_den_tft(dens.states_raw(), layer_addr, tft_id),
                        CorticalBuffer::data_den_tft(dens.states(), layer_addr, tft_id)
                    ],
                    vec![
                        CorticalBuffer::data_soma_tft(&best_den_ids, layer_addr, tft_id),
                        CorticalBuffer::data_soma_tft(&best_den_states_raw, layer_addr, tft_id),
                        CorticalBuffer::data_soma_tft(&best_den_states, layer_addr, tft_id),
                    ]
                ))?);
            };

            /*=============================================================================
            ===============================================================================
            =============================================================================*/

            // let syns_per_tftsec = dens.syns().syns_per_tftsec();
            // let cel_grp_count = cmn::OPENCL_MINIMUM_WORKGROUP_SIZE;
            let cel_grp_count = 64;
            let cels_per_cel_grp = dims.per_subgrp(cel_grp_count)?;
            let learning_rate_l2i = 0i32;

            let kern_name = "tft_ltp";
            ltp_kernels.push(ocl_pq.create_kernel(kern_name)?
                .gws(SpatialDims::One(cel_grp_count as usize))
                .arg_buf(axons.states())
                .arg_buf(cel_states)
                .arg_buf(&best_den_ids)
                .arg_buf(&best_den_states_raw)
                .arg_buf(dens.states())
                .arg_buf(dens.syns().states())
                // .arg_scl(tfts_per_cel as u32)
                .arg_scl(tft_cel_idz)
                .arg_scl(tft_den_idz)
                .arg_scl(tft_syn_idz)
                .arg_scl(dens_per_tft_l2 as u32)
                .arg_scl(syns_per_den_l2 as u32)
                .arg_scl(syns_per_tft_l2 as u32)
                .arg_scl(cels_per_cel_grp)
                .arg_scl(cel_lyr_axn_idz)
                .arg_scl_named::<i32>("lr_l2i", Some(learning_rate_l2i))
                .arg_scl_named::<i32>("rnd", None)
                .arg_buf(dens.syns().flag_sets())
                .arg_buf(cel_flag_sets)
                .arg_buf_named("aux_ints_0", None::<Buffer<i32>>)
                .arg_buf_named("aux_ints_1", None::<Buffer<i32>>)
                .arg_buf(dens.syns().strengths())
            );

            let mut ltp_cmd_srcs: Vec<CorticalBuffer> = cel_axn_slc_ids.iter()
                .map(|&slc_id|
                    CorticalBuffer::axon_slice(&axons.states(), layer_addr.area_id(), slc_id))
                .collect();

            ltp_cmd_srcs.push(CorticalBuffer::data_soma_lyr(&cel_states, layer_addr));
            ltp_cmd_srcs.push(CorticalBuffer::data_soma_tft(&best_den_ids, layer_addr, tft_id));
            ltp_cmd_srcs.push(CorticalBuffer::data_soma_tft(&best_den_states_raw, layer_addr, tft_id));
            ltp_cmd_srcs.push(CorticalBuffer::data_den_tft(dens.states(), layer_addr, tft_id));
            ltp_cmd_srcs.push(CorticalBuffer::data_syn_tft(dens.syns().states(), layer_addr, tft_id));

            if !settings.disable_learning & !settings.disable_pyrs {
                ltp_exe_cmd_uids.push(exe_graph.add_command(CommandRelations::cortical_kernel(
                    kern_name, ltp_cmd_srcs,
                    vec![
                        CorticalBuffer::data_syn_tft(dens.syns().flag_sets(), layer_addr, tft_id),
                        CorticalBuffer::data_soma_tft(&cel_flag_sets, layer_addr, tft_id),
                        CorticalBuffer::data_syn_tft(dens.syns().strengths(), layer_addr, tft_id),
                    ]
                ))?);
            }
        }


        assert!(den_count_ttl == dens.count());
        assert!(syn_count_ttl == dens.syns().count());

        Ok(Tufts {
            layer_name,
            layer_addr,
            dims,
            tft_count,

            best_den_ids,
            best_den_states_raw,
            best_den_states,
            states,

            ltp_kernels,
            cycle_kernels,

            cycle_exe_cmd_uids,
            cycle_exe_cmd_idxs,
            ltp_exe_cmd_uids,
            ltp_exe_cmd_idxs,

            dens,
            settings,
            rng: cmn::weak_rng(),
        })
    }

    pub fn set_exe_order_learn(&mut self, exe_graph: &mut ExecutionGraph) -> CmnResult<()> {
        if !self.settings.disable_pyrs && !self.settings.disable_learning {
            // Clear old ltp cmd idxs:
            self.ltp_exe_cmd_idxs.clear();

            // Learning:
            for &cmd_uid in self.ltp_exe_cmd_uids.iter() {
                self.ltp_exe_cmd_idxs.push(exe_graph.order_command(cmd_uid)?);
            }
        }
        Ok(())
    }

    pub fn set_exe_order_cycle(&mut self, _control_layers: &mut ControlCellLayers,
            exe_graph: &mut ExecutionGraph) -> CmnResult<()> {
        if !self.settings.disable_pyrs {
            // Clear old cycle cmd idxs:
            self.cycle_exe_cmd_idxs.clear();

            // Dendrites:
            self.dens.set_exe_order(exe_graph)?;

            // Tufts:
            for &cmd_uid in self.cycle_exe_cmd_uids.iter() {
                self.cycle_exe_cmd_idxs.push(exe_graph.order_command(cmd_uid)?);
            }
        }
        Ok(())
    }


    // <<<<< TODO: DEPRICATE >>>>>
    pub fn set_arg_buf_named<T: OclPrm>(&mut self, name: &'static str, env: &Buffer<T>,
            using_aux_cycle: bool, using_aux_learning: bool) -> OclResult<()> {
        for (cycle_kern, ltp_kern) in self.cycle_kernels.iter_mut()
                .zip(self.ltp_kernels.iter_mut())
        {
            if using_aux_cycle {
                try!(cycle_kern.set_arg_buf_named(name, Some(env)));
            }

            if using_aux_learning {
                try!(ltp_kern.set_arg_buf_named(name, Some(env)));
            }
        }

        Ok(())
    }


    #[inline]
    pub fn learn(&mut self, exe_graph: &mut ExecutionGraph) -> CmnResult <()> {
        if PRINT_DEBUG { printlnc!(yellow: "  Tfts: Performing learning for layer: '{}'...", self.layer_name); }

        for (ltp_kernel, &cmd_idx) in self.ltp_kernels.iter_mut().zip(self.ltp_exe_cmd_idxs.iter()) {
            if PRINT_DEBUG { printlnc!(yellow: "  Tfts:   Setting scalar to a random value..."); }

            ltp_kernel.set_arg_scl_named("rnd", self.rng.gen::<i32>()).expect("PyramidalLayer::learn()");

            if PRINT_DEBUG { printlnc!(yellow: "  Tfts:   Enqueuing kern_ltp..."); }

            let mut event = Event::empty();
            unsafe { ltp_kernel.cmd().ewait(exe_graph.get_req_events(cmd_idx).unwrap()).enew(&mut event).enq()?; }
            exe_graph.set_cmd_event(cmd_idx, Some(event))?;
            if PRINT_DEBUG { ltp_kernel.default_queue().unwrap().finish().unwrap(); }
            if PRINT_DEBUG { printlnc!(yellow: "  Tfts: Learning complete for layer: '{}'.", self.layer_name); }
        }

        Ok(())
    }

    pub fn cycle(&mut self, _control_layers: &mut ControlCellLayers,
            exe_graph: &mut ExecutionGraph) -> CmnResult<()> {
        // Dens:
        if PRINT_DEBUG { printlnc!(yellow: "  Tfts: Cycling dens..."); }
        self.dens.cycle(exe_graph)?;

        // Tufts:
        for (tft_id, (cycle_kernel, &cmd_idx)) in self.cycle_kernels.iter()
                .zip(self.cycle_exe_cmd_idxs.iter()).enumerate()
        {
            if PRINT_DEBUG { printlnc!(yellow: "  Tfts: Enqueuing cycle kernels for tft: {}...", tft_id); }

            let mut event = Event::empty();
            unsafe { cycle_kernel.cmd().ewait(exe_graph.get_req_events(cmd_idx)?).enew(&mut event).enq()?; }
            exe_graph.set_cmd_event(cmd_idx, Some(event))?;

            // [DEBUG]: TEMPORARY:
            if PRINT_DEBUG { cycle_kernel.default_queue().unwrap().finish().unwrap(); }
        }

        if PRINT_DEBUG { printlnc!(yellow: "  Tfts: Cycling complete for layer: '{}'.", self.layer_name); }

        Ok(())
    }

    #[inline] pub fn layer_name<'s>(&'s self) -> &'s str { &self.layer_name }
    #[inline] pub fn layer_addr(&self) -> LayerAddress { self.layer_addr }
    #[inline] pub fn dims(&self) -> &CorticalDims { &self.dims }
    #[inline] pub fn states(&self) -> &Buffer<u8> { &self.states }
    #[inline] pub fn best_den_ids(&self) -> &Buffer<u8> { &self.best_den_ids }
    #[inline] pub fn best_den_states_raw(&self) -> &Buffer<u8> { &self.best_den_states_raw }
    #[inline] pub fn best_den_states(&self) -> &Buffer<u8> { &self.best_den_states }
    #[inline] pub fn dens(&self) -> &Dendrites { &self.dens }
    #[inline] pub fn dens_mut(&mut self) -> &mut Dendrites { &mut self.dens }
}