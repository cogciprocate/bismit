#![allow(unused_imports)]

use rand::Rng;
use cmn::{self, CmnResult, CorticalDims, XorShiftRng};
use ocl::{ProQue, SpatialDims, Buffer, Kernel, Result as OclResult, Event};
use std::collections::BTreeMap;
use ocl::traits::OclPrm;
use map::{AreaMap, CellScheme, DendriteClass, DendriteKind, ExecutionGraph, CommandRelations,
    CorticalBuffer, LayerAddress, LayerTags, CommandUid};
use cortex::{Dendrites, AxonSpace, CorticalAreaSettings, DataCellLayer, ControlCellLayers};

const PRNT: bool = false;

#[derive(Debug)]
pub struct Tufts {
    layer_name: String,
    layer_addr: LayerAddress,
    dims: CorticalDims,
    tft_count: usize,

    prev_best_den_ids: Buffer<u8>,
    prev_best_den_states_raw: Buffer<u8>,
    prev_best_den_states: Buffer<u8>,
    prev_states: Buffer<u8>,
    best_den_ids: Buffer<u8>,
    best_den_states_raw: Buffer<u8>,
    best_den_states: Buffer<u8>,
    states: Buffer<u8>,

    mtp_kernels: Vec<Kernel>,
    cycle_kernels: Vec<Kernel>,
    cycle_exe_cmd_uids: Vec<CommandUid>,
    cycle_exe_cmd_idxs: Vec<usize>,
    mtp_exe_cmd_uids: Vec<CommandUid>,
    mtp_exe_cmd_idxs: Vec<usize>,

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

        let prev_best_den_ids = Buffer::<u8>::builder().queue(ocl_pq.queue().clone()).len([celtft_count]).fill_val(0).build()?;
        let prev_best_den_states_raw = Buffer::<u8>::builder().queue(ocl_pq.queue().clone()).len([celtft_count]).fill_val(0).build()?;
        let prev_best_den_states = Buffer::<u8>::builder().queue(ocl_pq.queue().clone()).len([celtft_count]).fill_val(0).build()?;
        let prev_states = Buffer::<u8>::builder().queue(ocl_pq.queue().clone()).len([celtft_count]).fill_val(0).build()?;
        let best_den_ids = Buffer::<u8>::builder().queue(ocl_pq.queue().clone()).len([celtft_count]).fill_val(0).build()?;
        let best_den_states_raw = Buffer::<u8>::builder().queue(ocl_pq.queue().clone()).len([celtft_count]).fill_val(0).build()?;
        let best_den_states = Buffer::<u8>::builder().queue(ocl_pq.queue().clone()).len([celtft_count]).fill_val(0).build()?;
        let states = Buffer::<u8>::builder().queue(ocl_pq.queue().clone()).len([celtft_count]).fill_val(0).build()?;

        let dens = Dendrites::new(layer_name.clone(), layer_addr.layer_id(), dims, cell_scheme.clone(),
            area_map, axons, ocl_pq, settings.disable_pyrs, exe_graph)?;

        let mut mtp_kernels = Vec::with_capacity(tft_count);
        let mut cycle_kernels = Vec::with_capacity(tft_count);
        let mut cycle_exe_cmd_uids = Vec::with_capacity(tft_count);
        let cycle_exe_cmd_idxs = Vec::with_capacity(tft_count);
        let mut mtp_exe_cmd_uids = Vec::with_capacity(tft_count);
        let mtp_exe_cmd_idxs = Vec::with_capacity(tft_count);
        // let mut den_kinds = Vec::with_capacity(tft_count);
        let mut den_count_ttl = 0u32;
        let mut syn_count_ttl = 0u32;

        for (tft_id, tft_scheme) in cell_scheme.tft_schemes().iter().enumerate() {
            // den_kinds.push((tft_scheme.den_class(), tft_scheme.den_kind()));

            let dens_per_tft = tft_scheme.dens_per_tft();
            let syns_per_den = tft_scheme.syns_per_den();
            let syns_per_tft = dens_per_tft * syns_per_den;
            let tft_cel_idz = tft_id as u32 * dims.cells();

            // Dendrites:
            let tft_den_idz = den_count_ttl;
            let tft_den_count = dims.cells() * dens_per_tft;
            den_count_ttl += tft_den_count;

            // Synapses:
            let tft_syn_idz = syn_count_ttl;
            let tft_syn_count = dims.cells() * syns_per_tft;
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
                .arg_scl(dens_per_tft)
                .arg_scl(tft_scheme.max_active_dens_l2())
                .arg_buf(&prev_best_den_ids)
                .arg_buf(&prev_best_den_states_raw)
                .arg_buf(&prev_best_den_states)
                .arg_buf(&prev_states)
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
                        CorticalBuffer::data_tft(&best_den_ids, layer_addr, tft_id),
                        CorticalBuffer::data_tft(&best_den_states_raw, layer_addr, tft_id),
                        CorticalBuffer::data_tft(&best_den_states, layer_addr, tft_id),
                        CorticalBuffer::data_tft(&prev_best_den_ids, layer_addr, tft_id),
                        CorticalBuffer::data_tft(&prev_best_den_states_raw, layer_addr, tft_id),
                        CorticalBuffer::data_tft(&prev_best_den_states, layer_addr, tft_id),
                    ]
                ))?);
            };

            /*=============================================================================
            ===============================================================================
            =============================================================================*/

            if !settings.disable_learning & !settings.disable_pyrs {
                match tft_scheme.den_kind() {
                    DendriteKind::Distal => {
                        // let syns_per_tftsec = dens.syns().syns_per_tftsec();
                        // let cel_grp_count = cmn::OPENCL_MINIMUM_WORKGROUP_SIZE;
                        let cel_grp_count = 64;
                        let cels_per_cel_grp = dims.per_subgrp(cel_grp_count)?;
                        let potentiation_rate_l2i = 0i32;
                        let depression_rate_l2i = 3i32;

                        let kern_name = "tft_dst_mtp";
                        mtp_kernels.push(ocl_pq.create_kernel(kern_name)?
                            .gws(SpatialDims::One(cel_grp_count as usize))
                            .arg_buf(axons.states())
                            .arg_buf(cel_states)
                            .arg_buf(&prev_best_den_ids)
                            .arg_buf(&prev_best_den_states_raw)
                            .arg_buf(dens.states())
                            .arg_buf(dens.syns().states())
                            // .arg_scl(tfts_per_cel as u32)
                            .arg_scl(tft_cel_idz)
                            .arg_scl(tft_den_idz)
                            .arg_scl(tft_syn_idz)
                            .arg_scl(dens_per_tft)
                            .arg_scl(syns_per_den)
                            .arg_scl(syns_per_tft)
                            .arg_scl(cels_per_cel_grp)
                            .arg_scl(cel_lyr_axn_idz)
                            .arg_scl_named::<i32>("pr_l2i", Some(potentiation_rate_l2i))
                            .arg_scl_named::<i32>("dr_l2i", Some(depression_rate_l2i))
                            .arg_scl_named::<i32>("rnd", None)
                            .arg_buf(dens.syns().flag_sets())
                            .arg_buf(cel_flag_sets)
                            .arg_buf_named("aux_ints_0", None::<Buffer<i32>>)
                            .arg_buf_named("aux_ints_1", None::<Buffer<i32>>)
                            .arg_buf(dens.syns().strengths())
                        );

                        let mut mtp_cmd_srcs: Vec<CorticalBuffer> = cel_axn_slc_ids.iter()
                            .map(|&slc_id|
                                CorticalBuffer::axon_slice(&axons.states(), layer_addr.area_id(), slc_id))
                            .collect();

                        mtp_cmd_srcs.push(CorticalBuffer::data_soma_lyr(&cel_states, layer_addr));
                        mtp_cmd_srcs.push(CorticalBuffer::data_tft(&prev_best_den_ids, layer_addr, tft_id));
                        mtp_cmd_srcs.push(CorticalBuffer::data_tft(&prev_best_den_states_raw, layer_addr, tft_id));
                        mtp_cmd_srcs.push(CorticalBuffer::data_den_tft(dens.states(), layer_addr, tft_id));
                        mtp_cmd_srcs.push(CorticalBuffer::data_syn_tft(dens.syns().states(), layer_addr, tft_id));

                        mtp_exe_cmd_uids.push(exe_graph.add_command(CommandRelations::cortical_kernel(
                            kern_name, mtp_cmd_srcs,
                            vec![
                                CorticalBuffer::data_syn_tft(dens.syns().flag_sets(), layer_addr, tft_id),
                                CorticalBuffer::data_tft(&cel_flag_sets, layer_addr, tft_id),
                                CorticalBuffer::data_syn_tft(dens.syns().strengths(), layer_addr, tft_id),
                            ]
                        ))?);

                        println!("\n\n###### Adding distal mtp kernel cmd_uid\n");
                    },
                    _ => (),
                }
            }
        }

        assert!(den_count_ttl == dens.count());
        assert!(syn_count_ttl == dens.syns().count());

        Ok(Tufts {
            layer_name,
            layer_addr,
            dims,
            tft_count,

            prev_best_den_ids,
            prev_best_den_states_raw,
            prev_best_den_states,
            prev_states,
            best_den_ids,
            best_den_states_raw,
            best_den_states,
            states,

            mtp_kernels,
            cycle_kernels,
            cycle_exe_cmd_uids,
            cycle_exe_cmd_idxs,
            mtp_exe_cmd_uids,
            mtp_exe_cmd_idxs,

            dens,
            settings,
            rng: cmn::weak_rng(),
        })
    }

    /// Sets the execution order for learning kernels.
    ///
    /// Only learns for tufts containing distal dendrites.
    pub fn set_exe_order_learn(&mut self, exe_graph: &mut ExecutionGraph) -> CmnResult<()> {
        if !self.settings.disable_pyrs && !self.settings.disable_learning {
            // Clear old mtp cmd idxs:
            self.mtp_exe_cmd_idxs.clear();

            // Learning:
            for &cmd_uid in self.mtp_exe_cmd_uids.iter() {
                self.mtp_exe_cmd_idxs.push(exe_graph.order_command(cmd_uid)?);
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
        for (cycle_kern, mtp_kern) in self.cycle_kernels.iter_mut()
                .zip(self.mtp_kernels.iter_mut())
        {
            if using_aux_cycle {
                try!(cycle_kern.set_arg_buf_named(name, Some(env)));
            }

            if using_aux_learning {
                try!(mtp_kern.set_arg_buf_named(name, Some(env)));
            }
        }

        Ok(())
    }


    /// Enqueues learning kernels.
    ///
    /// Only learns for tufts containing distal dendrites.
    pub fn learn(&mut self, exe_graph: &mut ExecutionGraph) -> CmnResult <()> {
        if PRNT { printlnc!(yellow: "  Tfts: Performing learning for layer: '{}'...",
            self.layer_name); }

        for (mtp_kernel, &cmd_idx) in self.mtp_kernels.iter_mut()
                .zip(self.mtp_exe_cmd_idxs.iter()) {
            if PRNT { printlnc!(yellow: "  Tfts: Setting scalar to a random value..."); }

            mtp_kernel.set_arg_scl_named("rnd", self.rng.gen::<i32>())
                .expect("PyramidalLayer::learn()");

            if PRNT { printlnc!(yellow: "  Tfts: Enqueuing kern_mtp..."); }

            let mut event = Event::empty();
            unsafe {
                mtp_kernel.cmd()
                    .ewait(exe_graph.get_req_events(cmd_idx).unwrap())
                    .enew(&mut event)
                    .enq()?;
            }
            exe_graph.set_cmd_event(cmd_idx, Some(event))?;
            if PRNT {
                mtp_kernel.default_queue().unwrap().finish().unwrap();
                printlnc!(yellow: "  Tfts: Learning complete for layer: '{}'.",
                    self.layer_name);
            }
        }

        Ok(())
    }

    pub fn cycle(&mut self, _control_layers: &mut ControlCellLayers,
            exe_graph: &mut ExecutionGraph) -> CmnResult<()> {
        // Dens:
        if PRNT { printlnc!(yellow: "  Tfts: Cycling dens..."); }
        self.dens.cycle(exe_graph)?;

        // Tufts:
        for (tft_id, (cycle_kernel, &cmd_idx)) in self.cycle_kernels.iter()
                .zip(self.cycle_exe_cmd_idxs.iter()).enumerate()
        {
            if PRNT { printlnc!(yellow: "  Tfts: Enqueuing cycle kernels for tft: {}...", tft_id); }

            let mut event = Event::empty();
            unsafe { cycle_kernel.cmd().ewait(exe_graph.get_req_events(cmd_idx)?).enew(&mut event).enq()?; }
            exe_graph.set_cmd_event(cmd_idx, Some(event))?;

            // [DEBUG]: TEMPORARY:
            if PRNT { cycle_kernel.default_queue().unwrap().finish().unwrap(); }
        }

        if PRNT { printlnc!(yellow: "  Tfts: Cycling complete for layer: '{}'.", self.layer_name); }

        Ok(())
    }

    #[inline] pub fn layer_name<'s>(&'s self) -> &'s str { &self.layer_name }
    #[inline] pub fn layer_addr(&self) -> LayerAddress { self.layer_addr }
    #[inline] pub fn dims(&self) -> &CorticalDims { &self.dims }
    #[inline] pub fn prev_states(&self) -> &Buffer<u8> { &self.prev_states }
    #[inline] pub fn prev_best_den_ids(&self) -> &Buffer<u8> { &self.prev_best_den_ids }
    #[inline] pub fn prev_best_den_states_raw(&self) -> &Buffer<u8> { &self.prev_best_den_states_raw }
    #[inline] pub fn prev_best_den_states(&self) -> &Buffer<u8> { &self.prev_best_den_states }
    #[inline] pub fn states(&self) -> &Buffer<u8> { &self.states }
    #[inline] pub fn best_den_ids(&self) -> &Buffer<u8> { &self.best_den_ids }
    #[inline] pub fn best_den_states_raw(&self) -> &Buffer<u8> { &self.best_den_states_raw }
    #[inline] pub fn best_den_states(&self) -> &Buffer<u8> { &self.best_den_states }
    #[inline] pub fn dens(&self) -> &Dendrites { &self.dens }
    #[inline] pub fn dens_mut(&mut self) -> &mut Dendrites { &mut self.dens }
}


// #[cfg(test)]
pub mod tests {
    use std::ops::{Range};
    use rand::{Rng};
    use rand::distributions::{IndependentSample};
    use ocl::util;
    use cmn::{self, XorShiftRng, Range as RandRange};
    use cortex::{PyramidalLayer, DataCellLayer, DataCellLayerTest, CelCoords, Tufts};

    impl Tufts {
        pub fn cycle_solo(&self) {
            for cycle_kern in self.cycle_kernels.iter() {
                cycle_kern.default_queue().unwrap().finish().unwrap();
                unsafe { cycle_kern.cmd().enq().expect("PyramidalLayer::cycle_self_only: tft_cycle_kernels"); }
                cycle_kern.default_queue().unwrap().finish().unwrap();
            }
        }

        pub fn learn_solo(&mut self) {
            for mtp_kernel in self.mtp_kernels.iter_mut() {
                mtp_kernel.default_queue().unwrap().finish().unwrap();

                mtp_kernel.set_arg_scl_named("rnd", self.rng.gen::<i32>())
                    .expect("<PyramidalLayer as DataCellLayerTest>::learn_solo [0]");

                unsafe {
                    mtp_kernel.cmd().enq()
                        .expect("<PyramidalLayer as DataCellLayerTest>::learn_solo [1]");
                }

                mtp_kernel.default_queue().unwrap().finish().unwrap();
            }
        }
    }
}