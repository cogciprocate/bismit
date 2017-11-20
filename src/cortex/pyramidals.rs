use rand::Rng;
use cmn::{self, CmnResult, CorticalDims, XorShiftRng};
use ocl::{ProQue, SpatialDims, Buffer, Kernel, Result as OclResult, Event};
use std::collections::BTreeMap;
use ocl::traits::OclPrm;
use map::{AreaMap, CellScheme, DendriteKind, ExecutionGraph, CommandRelations,
    CorticalBuffer, LayerAddress, LayerTags, CommandUid};
use cortex::{Dendrites, AxonSpace, CorticalAreaSettings, DataCellLayer, ControlCellLayer};

const PRINT_DEBUG: bool = false;


#[derive(Debug)]
pub struct PyramidalLayer {
    layer_name: String,
    // layer_id: usize,
    layer_addr: LayerAddress,
    layer_tags: LayerTags,
    dims: CorticalDims,
    tft_count: usize,
    cell_scheme: CellScheme,
    pyr_tft_ltp_kernels: Vec<Kernel>,
    pyr_tft_cycle_kernels: Vec<Kernel>,
    pyr_cycle_kernel: Kernel,
    axn_slc_ids: Vec<u8>,
    // base_axn_slc: u8,
    pyr_lyr_axn_idz: u32,
    rng: XorShiftRng,

    states: Buffer<u8>,
    best_den_states_raw: Buffer<u8>,
    flag_sets: Buffer<u8>,
    tft_best_den_ids: Buffer<u8>,
    tft_best_den_states_raw: Buffer<u8>,
    tft_best_den_states: Buffer<u8>,
    energies: Buffer<u8>,
    activities: Buffer<u8>,
    pub dens: Dendrites,

    tft_cycle_exe_cmd_uids: Vec<CommandUid>,
    tft_cycle_exe_cmd_idxs: Vec<usize>,
    tft_ltp_exe_cmd_uids: Vec<CommandUid>,
    tft_ltp_exe_cmd_idxs: Vec<usize>,
    cycle_exe_cmd_uid: Option<CommandUid>,
    cycle_exe_cmd_idx: Option<usize>,
    settings: CorticalAreaSettings,
    control_lyr_idxs: Vec<(LayerAddress, usize)>,
}

impl PyramidalLayer {
    pub fn new<S: Into<String>>(layer_name: S, layer_id: usize, dims: CorticalDims, cell_scheme: CellScheme,
            area_map: &AreaMap, axons: &AxonSpace, ocl_pq: &ProQue,
            settings: CorticalAreaSettings, exe_graph: &mut ExecutionGraph)
            -> CmnResult<PyramidalLayer>
    {
        let layer_name = layer_name.into();
        let layer_addr = LayerAddress::new(area_map.area_id(), layer_id);
        // [FIXME]: Convert to layer_id:
        let axn_slc_ids = area_map.layer_slc_ids(&[layer_name.to_owned()]);
        let base_axn_slc = axn_slc_ids[0];
        let pyr_lyr_axn_idz = area_map.axn_idz(base_axn_slc);

        // let tfts_per_cel = area_map.layer_dst_srcs(layer_name).len() as u32;
        let tft_count = cell_scheme.tft_count();
        // assert!(area_map.layer_dst_srcs(layer_name).len() == tft_count);

        // let best_dens_per_cel = tfts_per_cel;
        // let dims_tft_best_den = dims.clone().with_tfts(tft_count);

        let cel_count = dims.to_len();
        let celtft_count = cel_count * tft_count;

        let states = Buffer::<u8>::builder().queue(ocl_pq.queue().clone()).dims([cel_count]).fill_val(0).build()?;
        let best_den_states_raw = Buffer::<u8>::builder().queue(ocl_pq.queue().clone()).dims([cel_count]).fill_val(0).build()?;
        let flag_sets = Buffer::<u8>::builder().queue(ocl_pq.queue().clone()).dims([cel_count]).fill_val(0).build()?;
        let tft_best_den_ids = Buffer::<u8>::builder().queue(ocl_pq.queue().clone()).dims([celtft_count]).fill_val(0).build()?;
        let tft_best_den_states_raw = Buffer::<u8>::builder().queue(ocl_pq.queue().clone()).dims([celtft_count]).fill_val(0).build()?;
        let tft_best_den_states = Buffer::<u8>::builder().queue(ocl_pq.queue().clone()).dims([celtft_count]).fill_val(0).build()?;
        let energies = Buffer::builder().queue(ocl_pq.queue().clone()).dims(cel_count).fill_val(0).build()?;
        let activities = Buffer::builder().queue(ocl_pq.queue().clone()).dims(cel_count).fill_val(0).build()?;

        println!("{mt}{mt}PYRAMIDALS::NEW(): \
            layer: '{}', base_axn_slc: {}, pyr_lyr_axn_idz: {}, tft_count: {}, \
            len: {}, celtft_count: {}, \n{mt}{mt}{mt}dims: {:?}.",
            layer_name, base_axn_slc, pyr_lyr_axn_idz, tft_count,
            states.len(), tft_best_den_ids.len(), dims, mt = cmn::MT);

        let dens = Dendrites::new(layer_name.clone(), layer_id, dims, cell_scheme.clone(),
            DendriteKind::Distal, /*DataCellKind::Pyramidal,*/ area_map, axons, ocl_pq,
            settings.disable_pyrs, exe_graph)?;

        let mut pyr_tft_ltp_kernels = Vec::with_capacity(tft_count);
        let mut pyr_tft_cycle_kernels = Vec::with_capacity(tft_count);
        let mut tft_cycle_exe_cmd_uids = Vec::with_capacity(tft_count);
        let tft_cycle_exe_cmd_idxs = Vec::with_capacity(tft_count);
        let mut tft_ltp_exe_cmd_uids = Vec::with_capacity(tft_count);
        let tft_ltp_exe_cmd_idxs = Vec::with_capacity(tft_count);
        let mut den_count_ttl = 0u32;
        let mut syn_count_ttl = 0u32;

        /*=============================================================================
        ===============================================================================
        =============================================================================*/

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

            let kern_name = "pyr_tft_cycle";
            pyr_tft_cycle_kernels.push(ocl_pq.create_kernel(kern_name)?
                // .expect("PyramidalLayer::new()")
                .gws(SpatialDims::One(cel_count))
                // .gwo(SpatialDims::One(tft_cel_idz))
                .arg_buf(dens.states_raw())
                .arg_buf(dens.states())
                // .arg_scl(tfts_per_cel)
                // .arg_scl(tft_id)
                .arg_scl(tft_cel_idz)
                .arg_scl(tft_den_idz)
                .arg_scl(dens_per_tft_l2)
                //.arg_buf(&energies) // <<<<< SLATED FOR REMOVAL
                .arg_buf(&tft_best_den_ids)
                .arg_buf(&tft_best_den_states_raw)
                .arg_buf(&tft_best_den_states)
                // .arg_buf(&best_den_states)
                .arg_buf_named("aux_ints_0", None::<Buffer<i32>>)
                .arg_buf_named("aux_ints_1", None::<Buffer<i32>>)
                // .arg_buf(&states)
            );

            if !settings.disable_pyrs {
                tft_cycle_exe_cmd_uids.push(exe_graph.add_command(CommandRelations::cortical_kernel(
                    kern_name,
                    vec![
                        CorticalBuffer::data_den_tft(dens.states_raw(), layer_addr, tft_id),
                        CorticalBuffer::data_den_tft(dens.states(), layer_addr, tft_id)
                    ],
                    vec![
                        CorticalBuffer::data_soma_tft(&tft_best_den_ids, layer_addr, tft_id),
                        CorticalBuffer::data_soma_tft(&tft_best_den_states_raw, layer_addr, tft_id),
                        CorticalBuffer::data_soma_tft(&tft_best_den_states, layer_addr, tft_id),
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

            let kern_name = "pyr_tft_ltp";
            pyr_tft_ltp_kernels.push(ocl_pq.create_kernel(kern_name)?
                // .expect("PyramidalLayer::new()")
                .gws(SpatialDims::One(cel_grp_count as usize))
                .arg_buf(axons.states())
                .arg_buf(&states)
                .arg_buf(&tft_best_den_ids)
                .arg_buf(&tft_best_den_states_raw)
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
                .arg_scl(pyr_lyr_axn_idz)
                .arg_scl_named::<i32>("lr_l2i", Some(learning_rate_l2i))
                .arg_scl_named::<i32>("rnd", None)
                .arg_buf(dens.syns().flag_sets())
                .arg_buf(&flag_sets)
                .arg_buf_named("aux_ints_0", None::<Buffer<i32>>)
                .arg_buf_named("aux_ints_1", None::<Buffer<i32>>)
                .arg_buf(dens.syns().strengths())
            );

            let mut tft_ltp_cmd_srcs: Vec<CorticalBuffer> = axn_slc_ids.iter()
                .map(|&slc_id|
                    CorticalBuffer::axon_slice(&axons.states(), layer_addr.area_id(), slc_id))
                .collect();

            tft_ltp_cmd_srcs.push(CorticalBuffer::data_soma_lyr(&states, layer_addr));
            tft_ltp_cmd_srcs.push(CorticalBuffer::data_soma_tft(&tft_best_den_ids, layer_addr, tft_id));
            tft_ltp_cmd_srcs.push(CorticalBuffer::data_soma_tft(&tft_best_den_states_raw, layer_addr, tft_id));
            tft_ltp_cmd_srcs.push(CorticalBuffer::data_den_tft(dens.states(), layer_addr, tft_id));
            tft_ltp_cmd_srcs.push(CorticalBuffer::data_syn_tft(dens.syns().states(), layer_addr, tft_id));

            if !settings.disable_learning & !settings.disable_pyrs {
                tft_ltp_exe_cmd_uids.push(exe_graph.add_command(CommandRelations::cortical_kernel(
                    kern_name, tft_ltp_cmd_srcs,
                    vec![
                        CorticalBuffer::data_syn_tft(dens.syns().flag_sets(), layer_addr, tft_id),
                        CorticalBuffer::data_soma_tft(&flag_sets, layer_addr, tft_id),
                        CorticalBuffer::data_syn_tft(dens.syns().strengths(), layer_addr, tft_id),
                    ]
                ))?);
            }
        }

        /*=============================================================================
        ===============================================================================
        =============================================================================*/

        let kern_name = "pyr_cycle";
        let pyr_cycle_kernel = ocl_pq.create_kernel(kern_name)?
            .gws(SpatialDims::One(cel_count))
            .arg_buf(&tft_best_den_ids)
            .arg_buf(&tft_best_den_states_raw)
            .arg_buf(&tft_best_den_states)
            .arg_scl(tft_count as u32)
            .arg_buf(&best_den_states_raw)
            .arg_buf(&states)
            .arg_buf_named("aux_ints_0", None::<Buffer<i32>>)
            .arg_buf_named("aux_ints_1", None::<Buffer<i32>>)
        ;

        let mut cycle_cmd_srcs: Vec<CorticalBuffer> = Vec::with_capacity(3 * tft_count);

        for tft_id in 0..tft_count {
            cycle_cmd_srcs.push(CorticalBuffer::data_soma_tft(&tft_best_den_ids, layer_addr, tft_id));
            cycle_cmd_srcs.push(CorticalBuffer::data_soma_tft(&tft_best_den_states_raw, layer_addr, tft_id));
            cycle_cmd_srcs.push(CorticalBuffer::data_soma_tft(&tft_best_den_states, layer_addr, tft_id));
        }

        let cycle_exe_cmd_uid = if !settings.disable_pyrs {
            Some(exe_graph.add_command(CommandRelations::cortical_kernel(
                kern_name, cycle_cmd_srcs,
                vec![CorticalBuffer::data_soma_lyr(&states, layer_addr),
                    CorticalBuffer::data_soma_lyr(&best_den_states_raw, layer_addr)] ))?)
        } else {
            None
        };

        /*=============================================================================
        ===============================================================================
        =============================================================================*/

        assert!(den_count_ttl == dens.count());
        assert!(syn_count_ttl == dens.syns().count());

        Ok(PyramidalLayer {
            layer_name: layer_name,
            // layer_id: layer_id,
            layer_addr: layer_addr,
            layer_tags: area_map.layer_map().layer_info(layer_id).unwrap().layer_tags(),
            dims: dims,
            tft_count: tft_count,
            cell_scheme: cell_scheme,
            pyr_tft_ltp_kernels: pyr_tft_ltp_kernels,
            pyr_tft_cycle_kernels: pyr_tft_cycle_kernels,
            pyr_cycle_kernel: pyr_cycle_kernel,
            axn_slc_ids: axn_slc_ids,
            // base_axn_slc: base_axn_slc,
            pyr_lyr_axn_idz: pyr_lyr_axn_idz,
            rng: cmn::weak_rng(),
            states: states,
            best_den_states_raw: best_den_states_raw,
            flag_sets: flag_sets,
            tft_best_den_ids: tft_best_den_ids,
            tft_best_den_states_raw: tft_best_den_states_raw,
            tft_best_den_states: tft_best_den_states,
            energies,
            activities,
            dens: dens,
            tft_cycle_exe_cmd_uids,
            tft_cycle_exe_cmd_idxs,
            tft_ltp_exe_cmd_uids,
            tft_ltp_exe_cmd_idxs,
            cycle_exe_cmd_uid,
            cycle_exe_cmd_idx: None,
            settings,
            control_lyr_idxs: Vec::with_capacity(4),
        })
    }

    pub fn set_exe_order_learn(&mut self, exe_graph: &mut ExecutionGraph) -> CmnResult<()> {
        if !self.settings.disable_pyrs && !self.settings.disable_learning {
            // Clear old ltp cmd idxs:
            self.tft_ltp_exe_cmd_idxs.clear();

            // Learning:
            for &cmd_uid in self.tft_ltp_exe_cmd_uids.iter() {
                self.tft_ltp_exe_cmd_idxs.push(exe_graph.order_command(cmd_uid)?);
            }
        }
        Ok(())
    }

    pub fn set_exe_order_cycle(&mut self, control_layers: &mut BTreeMap<(LayerAddress, usize), Box<ControlCellLayer>>,
            exe_graph: &mut ExecutionGraph) -> CmnResult<()>
    {
        // Determine which control layers apply to this layer and add to list:
        if self.control_lyr_idxs.is_empty() {
            for (&cl_idx, cl) in control_layers.iter() {
                if cl.host_layer_addr() == self.layer_addr {
                    self.control_lyr_idxs.push(cl_idx);
                }
            }
        }
        if !self.settings.disable_pyrs {
            // Clear old cycle cmd idxs:
            self.tft_cycle_exe_cmd_idxs.clear();

            // Control layers pre:
            for cl_idx in self.control_lyr_idxs.iter() {
                control_layers.get_mut(cl_idx).unwrap().set_exe_order_pre(exe_graph, self.layer_addr)?;
            }

            // Dendrites:
            self.dens.set_exe_order(exe_graph)?;

            // Tufts:
            for &cmd_uid in self.tft_cycle_exe_cmd_uids.iter() {
                self.tft_cycle_exe_cmd_idxs.push(exe_graph.order_command(cmd_uid)?);
            }

            // Somata:
            if let Some(cycle_cmd_uid) = self.cycle_exe_cmd_uid {
                self.cycle_exe_cmd_idx = Some(exe_graph.order_command(cycle_cmd_uid)?);
            }

            // Control layers post:
            for cl_idx in self.control_lyr_idxs.iter() {
                control_layers.get_mut(cl_idx).unwrap().set_exe_order_post(exe_graph, self.layer_addr)?;
            }
        }
        Ok(())
    }

    // pub fn dens_per_tft_l2(&self) -> u8 {
    //     self.dens_per_tft_l2
    // }

    // pub fn syns_per_den_l2(&self) -> u8 {
    //     self.syns_per_den_l2
    // }

    // <<<<< TODO: DEPRICATE >>>>>
    pub fn set_arg_buf_named<T: OclPrm>(&mut self, name: &'static str, env: &Buffer<T>)
            -> OclResult<()>
    {
        let using_aux_cycle = true;
        let using_aux_learning = true;

        for (tft_cycle_kern, ltp_kern) in self.pyr_tft_cycle_kernels.iter_mut()
                .zip(self.pyr_tft_ltp_kernels.iter_mut())
        {
            if using_aux_cycle {
                try!(tft_cycle_kern.set_arg_buf_named(name, Some(env)));
            }

            if using_aux_learning {
                try!(ltp_kern.set_arg_buf_named(name, Some(env)));
            }
        }

        if using_aux_cycle {
            self.pyr_cycle_kernel.set_arg_buf_named(name, Some(env))?;
        }

        Ok(())
    }

    // // USED BY AUX
    // #[inline] pub fn kern_ltp(&mut self) -> &mut Kernel { &mut self.kern_ltp }
    // // USED BY AUX
    // #[inline] pub fn kern_cycle(&mut self) -> &mut Kernel { &mut self.kern_cycle }

    #[inline] pub fn layer_id(&self) -> usize { self.layer_addr.layer_id() }
    #[inline] pub fn layer_addr(&self) -> LayerAddress { self.layer_addr }
    #[inline] pub fn layer_tags(&self) -> LayerTags { self.layer_tags }
    #[inline] pub fn states(&self) -> &Buffer<u8> { &self.states }
    #[inline] pub fn best_den_states_raw(&self) -> &Buffer<u8> { &self.best_den_states_raw }
    #[inline] pub fn flag_sets(&self) -> &Buffer<u8> { &self.flag_sets }
    #[inline] pub fn tft_best_den_ids(&self) -> &Buffer<u8> { &self.tft_best_den_ids }
    #[inline] pub fn tft_best_den_states_raw(&self) -> &Buffer<u8> { &self.tft_best_den_states_raw }
    #[inline] pub fn tft_best_den_states(&self) -> &Buffer<u8> { &self.tft_best_den_states }
}

impl DataCellLayer for PyramidalLayer {
    #[inline]
    fn learn(&mut self, exe_graph: &mut ExecutionGraph) -> CmnResult <()> {
        for (ltp_kernel, &cmd_idx) in self.pyr_tft_ltp_kernels.iter_mut()
                .zip(self.tft_ltp_exe_cmd_idxs.iter())
        {
            if PRINT_DEBUG { printlnc!(yellow: "Pyrs: Performing learning for layer: '{}'...", self.layer_name); }
            if PRINT_DEBUG { printlnc!(yellow: "Pyrs:   Setting scalar to a random value..."); }

            ltp_kernel.set_arg_scl_named("rnd", self.rng.gen::<i32>()).expect("PyramidalLayer::learn()");

            if PRINT_DEBUG { printlnc!(yellow: "Pyrs:   Enqueuing kern_ltp..."); }

            let mut event = Event::empty();
            unsafe { ltp_kernel.cmd().ewait(exe_graph.get_req_events(cmd_idx).unwrap()).enew(&mut event).enq()?; }
            exe_graph.set_cmd_event(cmd_idx, Some(event))?;
            if PRINT_DEBUG { ltp_kernel.default_queue().unwrap().finish().unwrap(); }
            if PRINT_DEBUG { printlnc!(yellow: "Pyrs: Learning complete for layer: '{}'.", self.layer_name); }
        }

        Ok(())
    }

    #[inline]
    fn regrow(&mut self) {
        if PRINT_DEBUG { printlnc!(yellow: "Pyrs: Regrowing dens..."); }
        self.dens_mut().regrow();
    }

    fn cycle(&mut self, control_layers: &mut BTreeMap<(LayerAddress, usize), Box<ControlCellLayer>>,
            exe_graph: &mut ExecutionGraph) -> CmnResult<()>
    {
        // Control Pre:
        for lyr_idx in self.control_lyr_idxs.iter() {
            if PRINT_DEBUG { printlnc!(royal_blue: "    Pyrs: Pre-cycling control layer: [{:?}]...", lyr_idx); }
            control_layers.get_mut(lyr_idx).unwrap().cycle_pre(exe_graph, self.layer_addr)?;
        }

        if PRINT_DEBUG { printlnc!(yellow: "Pyrs: Cycling layer: '{}'...", self.layer_name); }
        if PRINT_DEBUG { printlnc!(yellow: "Pyrs: Cycling dens..."); }
        self.dens.cycle(exe_graph)?;

        // [DEBUG]: TEMPORARY:
        if PRINT_DEBUG { self.pyr_cycle_kernel.default_queue().unwrap().finish().unwrap(); }


        // Tufts:
        for (tft_id, (tft_cycle_kernel, &cmd_idx)) in self.pyr_tft_cycle_kernels.iter()
                .zip(self.tft_cycle_exe_cmd_idxs.iter()).enumerate()
        {
            if PRINT_DEBUG { printlnc!(yellow: "Pyrs: Enqueuing cycle kernels for tft: {}...", tft_id); }

            let mut event = Event::empty();
            unsafe { tft_cycle_kernel.cmd().ewait(exe_graph.get_req_events(cmd_idx)?).enew(&mut event).enq()?; }
            exe_graph.set_cmd_event(cmd_idx, Some(event))?;

            // [DEBUG]: TEMPORARY:
            if PRINT_DEBUG { tft_cycle_kernel.default_queue().unwrap().finish().unwrap(); }
        }

        if PRINT_DEBUG { printlnc!(yellow: "Pyrs: Cycling cell soma..."); }

        // Soma:
        if let Some(cycle_cmd_idx) = self.cycle_exe_cmd_idx {
            let mut event = Event::empty();
            unsafe {
                self.pyr_cycle_kernel.cmd().ewait(exe_graph.get_req_events(cycle_cmd_idx)?)
                    .enew(&mut event).enq()?;
            }
            exe_graph.set_cmd_event(cycle_cmd_idx, Some(event))?;
        }

        // Control Post:
        for lyr_idx in self.control_lyr_idxs.iter() {
            if PRINT_DEBUG { printlnc!(royal_blue: "    Ssts: Post-cycling control layer: [{:?}]...", lyr_idx); }
            control_layers.get_mut(lyr_idx).unwrap().cycle_post(exe_graph, self.layer_addr)?;
        }

        // [DEBUG]: TEMPORARY:
        if PRINT_DEBUG { self.pyr_cycle_kernel.default_queue().unwrap().finish().unwrap(); }
        if PRINT_DEBUG { printlnc!(yellow: "Pyrs: Cycling complete for layer: '{}'.", self.layer_name); }

        Ok(())
    }

    #[inline]
    fn axn_range(&self) -> (usize, usize) {
        let axn_idn = self.pyr_lyr_axn_idz + (self.dims.columns());
        (self.pyr_lyr_axn_idz as usize, axn_idn as usize)
    }

    #[inline] fn layer_name<'s>(&'s self) -> &'s str { &self.layer_name }
    #[inline] fn layer_addr(&self) -> LayerAddress{ self.layer_addr }
    #[inline] fn soma(&self) -> &Buffer<u8> { &self.states }
    #[inline] fn soma_mut(&mut self) -> &mut Buffer<u8> { &mut self.states }
    #[inline] fn energies(&self) -> &Buffer<u8> { &self.energies }
    #[inline] fn activities(&self) -> &Buffer<u8> { &self.activities }
    #[inline] fn dims(&self) -> &CorticalDims { &self.dims }
    #[inline] fn axn_slc_ids(&self) -> &[u8] { self.axn_slc_ids.as_slice() }
    #[inline] fn base_axn_slc(&self) -> u8 { self.axn_slc_ids[0] }
    #[inline] fn tft_count(&self) -> usize { self.tft_count }
    #[inline] fn cell_scheme(&self) -> &CellScheme { &self.cell_scheme }
    #[inline] fn dens(&self) -> &Dendrites { &self.dens }
    #[inline] fn dens_mut(&mut self) -> &mut Dendrites { &mut self.dens }
}


#[cfg(test)]
pub mod tests {
    use std::ops::{Range};
    use rand::{Rng};
    use rand::distributions::{IndependentSample};
    use ocl::util;
    use cmn::{self, XorShiftRng, Range as RandRange};
    use cortex::{PyramidalLayer, DataCellLayer, DataCellLayerTest, CelCoords};

    impl DataCellLayerTest for PyramidalLayer {
        fn cycle_solo(&self) {
            self.pyr_cycle_kernel.default_queue().unwrap().finish().unwrap();

            for cycle_kern in self.pyr_tft_cycle_kernels.iter() {
                cycle_kern.default_queue().unwrap().finish().unwrap();
                unsafe { cycle_kern.cmd().enq().expect("PyramidalLayer::cycle_self_only: pyr_tft_cycle_kernels"); }
                cycle_kern.default_queue().unwrap().finish().unwrap();
            }

            unsafe {
                self.pyr_cycle_kernel.cmd().enq()
                    .expect("PyramidalLayer::cycle_self_only: pyr_cycle_kernel");
            }

            self.pyr_cycle_kernel.default_queue().unwrap().finish().unwrap();
        }

        fn learn_solo(&mut self) {
            for ltp_kernel in self.pyr_tft_ltp_kernels.iter_mut() {
                ltp_kernel.default_queue().unwrap().finish().unwrap();

                ltp_kernel.set_arg_scl_named("rnd", self.rng.gen::<i32>())
                    .expect("<PyramidalLayer as DataCellLayerTest>::learn_solo [0]");

                unsafe {
                    ltp_kernel.cmd().enq()
                        .expect("<PyramidalLayer as DataCellLayerTest>::learn_solo [1]");
                }

                ltp_kernel.default_queue().unwrap().finish().unwrap();
            }
        }

        // fn print_cel(&mut self, cel_idx: usize) {
        //     let emsg = "PyramidalLayer::print_cel()";

        //     self.confab();

        //     let cel_den_idz = (cel_idx << self.dens_mut().dims().per_tft_l2_left()) as usize;
        //     let cel_syn_idz = (cel_idx << self.dens_mut().syns_mut().dims().per_tft_l2_left()) as usize;

        //     let dens_per_tft = self.dens_mut().dims().per_cel() as usize;
        //     let syns_per_tft = self.dens_mut().syns_mut().dims().per_cel() as usize;

        //     let cel_den_range = cel_den_idz..(cel_den_idz + dens_per_tft);
        //     let cel_syn_range = cel_syn_idz..(cel_syn_idz + syns_per_tft);

        //     println!("Printing Pyramidal Cell:");
        //     println!("   states[{}]: {}", cel_idx, self.states[cel_idx]);
        //     println!("   flag_sets[{}]: {}", cel_idx, self.flag_sets[cel_idx]);
        //     println!("   best_den_states[{}]: {}", cel_idx, self.best_den_states[cel_idx]);
        //     println!("   tft_best_den_ids[{}]: {}", cel_idx, self.tft_best_den_ids[cel_idx]);
        //     println!("   tft_best_den_states[{}]: {}", cel_idx, self.tft_best_den_states[cel_idx]);

        //     // println!("   energies[{}]: {}", cel_idx, self.energies[cel_idx]); // <<<<< SLATED FOR REMOVAL

        //     println!("");

        //     println!("dens.states[{:?}]: ", cel_den_range.clone());
        //     self.dens.states.print(1, None, Some(cel_den_range.clone()), false);

        //     println!("dens.syns().states[{:?}]: ", cel_syn_range.clone());
        //     self.dens.syns_mut().states.print(1, None, Some(cel_den_range.clone()), false);

        //     println!("dens.syns().strengths[{:?}]: ", cel_syn_range.clone());
        //     self.dens.syns_mut().strengths.print(1, None, Some(cel_den_range.clone()), false);

        //     println!("dens.src_col_v_offs[{:?}]: ", cel_syn_range.clone());
        //     self.dens.syns_mut().src_col_v_offs.print(1, None, Some(cel_den_range.clone()), false);

        //     println!("dens.src_col_u_offs[{:?}]: ", cel_syn_range.clone());
        //     self.dens.syns_mut().src_col_u_offs.print(1, None, Some(cel_den_range.clone()), false);
        // }


        // // PRINT_ALL(): TODO: [complete] change argument to print dens at some point
        // fn print_range(&mut self, range: Range<usize>, print_children: bool) {
        //     print!("pyrs.states: ");
        //     self.states.print(1, Some((0, 255)), None, false);
        //     print!("pyrs.flag_sets: ");
        //     self.flag_sets.print(1, Some((0, 255)), None, false);
        //     print!("pyrs.best_den_states: ");
        //     self.best_den_states.print(1, Some((0, 255)), None, false);
        //     print!("pyrs.tft_best_den_ids: ");
        //     self.tft_best_den_ids.print(1, Some((0, 255)), None, false);
        //     print!("pyrs.tft_best_den_states: ");
        //     self.tft_best_den_states.print(1, Some((0, 255)), None, false);

        //     // print!("pyrs.energies: ");                            // <<<<< SLATED FOR REMOVAL
        //     // self.energies.print(1, Some((0, 255)), None, false); // <<<<< SLATED FOR REMOVAL


        //     if print_children {
        //         print!("dens.states: ");
        //         // FOR EACH TUFT:
        //             // Calculate range for tuft dens
        //             self.dens.states.print(1, Some((1, 255)), None, false);
        //             // Calculate range for tuft syns
        //             self.dens.syns_mut().print_all();
        //     }
        // }


        /// Prints a range of pyramidal buffers.
        ///
        //
        ////// Ocl print function signature:
        //
        // ocl::util::print_slice<T: OclScl>(vec: &[T], every: usize, val_range: Option<(T, T)>,
        // idx_range: Option<Range<usize>>, show_zeros: bool)
        //
        fn print_range(&self, idx_range: Option<Range<usize>>, /*print_children: bool*/) {
            let mut vec = vec![0; self.states.len()];

            // states: Buffer<u8>,
            // flag_sets: Buffer<u8>,
            // pyr_states: Buffer<u8>,
            // tft_best_den_ids: Buffer<u8>,
            // tft_best_den_states_raw: Buffer<u8>,
            // tft_best_den_states: Buffer<u8>,

            print!("pyramidal.states: ");
            self.states.read(&mut vec).enq().unwrap();
            util::print_slice(&vec, 1 << 0, None, idx_range.clone(), false);

            print!("pyramidal.tft_best_den_states_raw: ");
            self.tft_best_den_states_raw.read(&mut vec).enq().unwrap();
            util::print_slice(&vec, 1 << 0, None, idx_range.clone(), false);

            print!("pyramidal.tft_best_den_states: ");
            self.tft_best_den_states.read(&mut vec).enq().unwrap();
            util::print_slice(&vec, 1 << 0, None, idx_range.clone(), false);

        }

        fn print_all(&self, /*print_children: bool*/) {
            self.print_range(None, /*print_children*/);
        }

        fn rng(&mut self) -> &mut XorShiftRng {
            &mut self.rng
        }

        fn rand_cel_coords(&mut self) -> CelCoords {
            let slc_range = RandRange::new(0, self.dims().depth());
            let v_range = RandRange::new(0, self.dims().v_size());
            let u_range = RandRange::new(0, self.dims().u_size());

            let slc_id_lyr = slc_range.ind_sample(self.rng());
            let v_id = v_range.ind_sample(self.rng());
            let u_id = u_range.ind_sample(self.rng());

            let axn_slc_id = self.base_axn_slc() + slc_id_lyr;

            CelCoords::new(axn_slc_id, slc_id_lyr, v_id, u_id, self.dims().clone())
                //self.tft_count, self.dens_per_tft_l2(), self.syns_per_den_l2()
        }

        fn last_cel_coords(&self) -> CelCoords {
            let slc_id_lyr = self.dims().depth() - 1;
            let v_id = self.dims().v_size() - 1;
            let u_id = self.dims().u_size() - 1;

            let axn_slc_id = self.base_axn_slc() + slc_id_lyr;

            CelCoords::new(axn_slc_id, slc_id_lyr, v_id, u_id, self.dims().clone())
        }


        fn cel_idx(&self, slc_id_lyr: u8, v_id: u32, u_id: u32)-> u32 {
            cmn::cel_idx_3d(self.dims().depth(), slc_id_lyr, self.dims().v_size(), v_id,
                self.dims().u_size(), u_id)
        }

        fn celtft_idx(&self, tft_id: usize, cel_coords: &CelCoords) -> u32 {
            (tft_id as u32 * self.dims.cells()) + cel_coords.idx
        }

        fn set_all_to_zero(&mut self) { // MOVE TO TEST TRAIT IMPL
            self.states.default_queue().unwrap().finish().unwrap();
            self.flag_sets.default_queue().unwrap().finish().unwrap();
            self.tft_best_den_ids.default_queue().unwrap().finish().unwrap();
            self.tft_best_den_states.default_queue().unwrap().finish().unwrap();
            self.tft_best_den_states_raw.default_queue().unwrap().finish().unwrap();

            self.states.cmd().fill(0, None).enq().unwrap();
            self.flag_sets.cmd().fill(0, None).enq().unwrap();
            self.tft_best_den_ids.cmd().fill(0, None).enq().unwrap();
            self.tft_best_den_states.cmd().fill(0, None).enq().unwrap();
            self.tft_best_den_states_raw.cmd().fill(0, None).enq().unwrap();
            //self.best2_den_ids.cmd().fill(&[0], None).enq().unwrap();            // <<<<< SLATED FOR REMOVAL
            //self.best2_den_states.cmd().fill(&[0], None).enq().unwrap();        // <<<<< SLATED FOR REMOVAL

            // self.energies.cmd().fill(&[0], None).enq().unwrap();                // <<<<< SLATED FOR REMOVAL

            self.states.default_queue().unwrap().finish().unwrap();
            self.flag_sets.default_queue().unwrap().finish().unwrap();
            self.tft_best_den_ids.default_queue().unwrap().finish().unwrap();
            self.tft_best_den_states.default_queue().unwrap().finish().unwrap();
            self.tft_best_den_states_raw.default_queue().unwrap().finish().unwrap();
        }

        // fn confab(&mut self) {
        //     self.states.fill_vec();
        //     self.best_den_states.fill_vec();
        //     self.tft_best_den_ids.fill_vec();
        //     self.tft_best_den_states.fill_vec();
        //     self.flag_sets.fill_vec();
        //     // self.energies.fill_vec(); // <<<<< SLATED FOR REMOVAL

        //     self.dens_mut().confab();
        // }
    }
}

