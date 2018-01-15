use cmn::{self, CmnResult, CorticalDims, XorShiftRng, DEN_BASAL_PROXIMAL_FLAG,
    DEN_BASAL_DISTAL_FLAG, DEN_APICAL_DISTAL_FLAG};
use ocl::{ProQue, SpatialDims, Buffer, Kernel, Result as OclResult, Event};
use ocl::traits::OclPrm;
use map::{AreaMap, CellScheme, ExecutionGraph, CommandRelations,
    CorticalBuffer, LayerAddress, LayerTags, CommandUid, DendriteClass, DendriteKind};
use cortex::{Dendrites, AxonSpace, CorticalAreaSettings, DataCellLayer, ControlCellLayers,
    Tufts};

const PRNT: bool = false;


#[derive(Debug)]
pub struct PyramidalLayer {
    layer_name: String,
    layer_addr: LayerAddress,
    layer_tags: LayerTags,
    dims: CorticalDims,
    tft_count: usize,
    cell_scheme: CellScheme,
    pyr_cycle_kernel: Kernel,
    axn_slc_ids: Vec<u8>,
    pyr_lyr_axn_idz: u32,
    rng: XorShiftRng,

    states: Buffer<u8>,
    // TODO: Remove:
    best_den_states_raw: Buffer<u8>,
    flag_sets: Buffer<u8>,
    energies: Buffer<u8>,
    activities: Buffer<u8>,

    tfts: Tufts,

    cycle_exe_cmd_uid: Option<CommandUid>,
    cycle_exe_cmd_idx: Option<usize>,
    settings: CorticalAreaSettings,
    control_lyr_idxs: Vec<(LayerAddress, usize)>,
}

impl PyramidalLayer {
    pub fn new<S: Into<String>>(
            layer_name: S,
            layer_id: usize,
            dims: CorticalDims,
            cell_scheme: CellScheme,
            area_map: &AreaMap,
            axons: &AxonSpace,
            ocl_pq: &ProQue,
            settings: CorticalAreaSettings,
            exe_graph: &mut ExecutionGraph)
            -> CmnResult<PyramidalLayer> {
        let layer_name = layer_name.into();
        let layer_addr = LayerAddress::new(area_map.area_id(), layer_id);
        // [FIXME]: Convert arg to layer_id:
        let axn_slc_ids = area_map.layer_slc_ids(&[layer_name.to_owned()]);
        let base_axn_slc = axn_slc_ids[0];
        let pyr_lyr_axn_idz = area_map.axn_idz(base_axn_slc);

        let tft_count = cell_scheme.tft_count();

        let cel_count = dims.to_len();
        let celtft_count = cel_count * tft_count;

        let states = Buffer::<u8>::builder().queue(ocl_pq.queue().clone()).len([cel_count]).fill_val(0).build()?;
        let best_den_states_raw = Buffer::<u8>::builder().queue(ocl_pq.queue().clone()).len([cel_count]).fill_val(0).build()?;
        let flag_sets = Buffer::<u8>::builder().queue(ocl_pq.queue().clone()).len([cel_count]).fill_val(0).build()?;
        let energies = Buffer::builder().queue(ocl_pq.queue().clone()).len(cel_count).fill_val(0).build()?;
        let activities = Buffer::builder().queue(ocl_pq.queue().clone()).len(cel_count).fill_val(0).build()?;

        println!("{mt}{mt}PYRAMIDALS::NEW(): \
            layer: '{}', base_axn_slc: {}, pyr_lyr_axn_idz: {}, tft_count: {}, \
            cel_count: {}, celtft_count: {}, \n{mt}{mt}{mt}dims: {:?}.",
            layer_name, base_axn_slc, pyr_lyr_axn_idz, tft_count,
            states.len(), celtft_count, dims, mt = cmn::MT);

        let tfts = Tufts::new(layer_name.clone(), layer_addr, dims, cell_scheme.clone(),
            area_map, axons, &axn_slc_ids, pyr_lyr_axn_idz, &states,
            &flag_sets, ocl_pq, settings.clone(), exe_graph)?;

        let mut enabled_tft_flags = 0u8;
        let mut enabled_tft_ttl = 0;
        let mut bsl_prx_tft_id = None;
        let mut bsl_dst_tft_id = None;
        let mut apc_dst_tft_id = None;

        for tft_scheme in cell_scheme.tft_schemes() {
            assert!(tft_scheme.tft_id() <= 255);
            match tft_scheme.den_class() {
                DendriteClass::Basal => {
                    match tft_scheme.den_kind() {
                        DendriteKind::Proximal => {
                            assert!(bsl_prx_tft_id.is_none());
                            bsl_prx_tft_id = Some(tft_scheme.tft_id() as u8);
                            enabled_tft_flags |= DEN_BASAL_PROXIMAL_FLAG;
                            enabled_tft_ttl += 1;
                            if PRNT { println!("{mt}{mt}{mt} Basal Proximal Enabled", mt = cmn::MT); }
                        },
                        DendriteKind::Distal => {
                            assert!(bsl_dst_tft_id.is_none());
                            bsl_dst_tft_id = Some(tft_scheme.tft_id() as u8);
                            enabled_tft_flags |= DEN_BASAL_DISTAL_FLAG;
                            enabled_tft_ttl += 1;
                            if PRNT { println!("{mt}{mt}{mt} Basal Distal Enabled", mt = cmn::MT); }
                        },
                        _ => unimplemented!(),
                    }
                }
                DendriteClass::Apical => {
                    match tft_scheme.den_kind() {
                        DendriteKind::Distal => {
                            assert!(apc_dst_tft_id.is_none());
                            apc_dst_tft_id = Some(tft_scheme.tft_id() as u8);
                            enabled_tft_flags |= DEN_APICAL_DISTAL_FLAG;
                            enabled_tft_ttl += 1;
                            if PRNT { println!("{mt}{mt}{mt} Basal Apical Enabled", mt = cmn::MT); }
                        },
                        _ => unimplemented!(),
                    }
                }
                _ => unimplemented!(),
            }
        }

        assert!(enabled_tft_ttl == tft_count);

        if PRNT { println!("{mt}{mt}{mt} enabled_tft_flags: {:08b} ", enabled_tft_flags, mt = cmn::MT); }


        //=============================================================================
        //=============================================================================
        //=============================================================================

        // let kern_name = "pyr_cycle_old";
        // let pyr_cycle_kernel = ocl_pq.create_kernel(kern_name)?
        //     .gws(SpatialDims::One(cel_count))
        //     .arg_buf(tfts.best_den_ids())
        //     .arg_buf(tfts.best_den_states_raw())
        //     .arg_buf(tfts.best_den_states())
        //     .arg_scl(tft_count as u32)
        //     .arg_buf(&best_den_states_raw)
        //     .arg_buf(&states)
        //     .arg_buf_named("aux_ints_0", None::<Buffer<i32>>)
        //     .arg_buf_named("aux_ints_1", None::<Buffer<i32>>)
        // ;

        let kern_name = "pyr_cycle";
        let pyr_cycle_kernel = ocl_pq.create_kernel(kern_name)?
            .gws(SpatialDims::One(cel_count))
            // .arg_buf(tfts.best_den_ids())
            .arg_buf(tfts.best_den_states_raw())
            // .arg_buf(tfts.best_den_states())
            .arg_buf(tfts.states())
            .arg_scl(tft_count as u8)
            .arg_scl(enabled_tft_flags)
            .arg_scl(bsl_prx_tft_id.unwrap_or(0))
            .arg_scl(bsl_dst_tft_id.unwrap_or(0))
            .arg_scl(apc_dst_tft_id.unwrap_or(0))
            .arg_buf(&best_den_states_raw)
            .arg_buf_named("aux_ints_0", None::<Buffer<i32>>)
            .arg_buf_named("aux_ints_1", None::<Buffer<i32>>)
            .arg_buf(&states)
        ;

        let mut cycle_cmd_srcs: Vec<CorticalBuffer> = Vec::with_capacity(3 * tft_count);

        for tft_id in 0..tft_count {
            cycle_cmd_srcs.push(CorticalBuffer::data_soma_tft(tfts.best_den_ids(), layer_addr, tft_id));
            cycle_cmd_srcs.push(CorticalBuffer::data_soma_tft(tfts.best_den_states_raw(), layer_addr, tft_id));
            cycle_cmd_srcs.push(CorticalBuffer::data_soma_tft(tfts.best_den_states(), layer_addr, tft_id));
        }

        let cycle_exe_cmd_uid = if !settings.disable_pyrs {
            Some(exe_graph.add_command(CommandRelations::cortical_kernel(
                kern_name, cycle_cmd_srcs,
                vec![CorticalBuffer::data_soma_lyr(&states, layer_addr),
                    CorticalBuffer::data_soma_lyr(&best_den_states_raw, layer_addr)] ))?)
        } else {
            None
        };

        //=============================================================================
        //=============================================================================
        //=============================================================================

        Ok(PyramidalLayer {
            layer_name: layer_name,
            layer_addr: layer_addr,
            layer_tags: area_map.layer_map().layer_info(layer_id).unwrap().layer_tags(),
            dims: dims,
            tft_count: tft_count,
            cell_scheme: cell_scheme,
            pyr_cycle_kernel: pyr_cycle_kernel,
            axn_slc_ids: axn_slc_ids,
            pyr_lyr_axn_idz: pyr_lyr_axn_idz,
            rng: cmn::weak_rng(),
            states: states,
            best_den_states_raw: best_den_states_raw,
            flag_sets: flag_sets,
            energies,
            activities,
            tfts,

            cycle_exe_cmd_uid,
            cycle_exe_cmd_idx: None,
            settings,
            control_lyr_idxs: Vec::with_capacity(4),
        })
    }

    /// Sets the execution order for learning kernels.
    ///
    /// Currently only learns for tufts containing distal dendrites.
    pub fn set_exe_order_learn(&mut self, exe_graph: &mut ExecutionGraph) -> CmnResult<()> {
        self.tfts.set_exe_order_learn(exe_graph)
    }

    /// Sets the execution order for cycle kernels.
    pub fn set_exe_order_cycle(&mut self, control_layers: &mut ControlCellLayers,
            exe_graph: &mut ExecutionGraph) -> CmnResult<()> {
        // Determine which control layers apply to this layer and add to list:
        if self.control_lyr_idxs.is_empty() {
            for (&cl_idx, cl) in control_layers.iter() {
                if cl.host_layer_addr() == self.layer_addr {
                    self.control_lyr_idxs.push(cl_idx);
                }
            }
        }
        if !self.settings.disable_pyrs {
            // Control layers pre:
            for cl_idx in self.control_lyr_idxs.iter() {
                control_layers.get_mut(cl_idx).unwrap().set_exe_order_pre(exe_graph, self.layer_addr)?;
            }

            self.tfts.set_exe_order_cycle(control_layers, exe_graph)?;

            // Somata:
            if let Some(cycle_cmd_uid) = self.cycle_exe_cmd_uid {
                self.cycle_exe_cmd_idx = Some(exe_graph.order_command(cycle_cmd_uid)?);
            }

            // Control layers post:
            for cl_idx in self.control_lyr_idxs.iter() {
                control_layers.get_mut(cl_idx).unwrap().set_exe_order_post(exe_graph, self.layer_addr)?;
            }

            // Learning:
            self.set_exe_order_learn(exe_graph)?;
        }
        Ok(())
    }

    // <<<<< TODO: DEPRICATE >>>>>
    pub fn set_arg_buf_named<T: OclPrm>(&mut self, name: &'static str, env: &Buffer<T>)
            -> OclResult<()> {
        let using_aux_cycle = true;
        let using_aux_learning = true;

        self.tfts.set_arg_buf_named(name, env, using_aux_cycle, using_aux_learning)?;

        if using_aux_cycle {
            self.pyr_cycle_kernel.set_arg_buf_named(name, Some(env))?;
        }

        Ok(())
    }

    #[inline] pub fn layer_id(&self) -> usize { self.layer_addr.layer_id() }
    #[inline] pub fn layer_addr(&self) -> LayerAddress { self.layer_addr }
    #[inline] pub fn layer_tags(&self) -> LayerTags { self.layer_tags }
    #[inline] pub fn states(&self) -> &Buffer<u8> { &self.states }
    #[inline] pub fn best_den_states_raw(&self) -> &Buffer<u8> { &self.best_den_states_raw }
    #[inline] pub fn flag_sets(&self) -> &Buffer<u8> { &self.flag_sets }
    #[inline] pub fn tfts(&self) -> &Tufts { &self.tfts }
}

impl DataCellLayer for PyramidalLayer {
    /// Enqueues learning kernels.
    ///
    /// Only learns for tufts containing distal dendrites.
    #[inline]
    fn learn(&mut self, exe_graph: &mut ExecutionGraph) -> CmnResult <()> {
        self.tfts.learn(exe_graph)
    }

    /// Prunes and regrows synapses.
    #[inline]
    fn regrow(&mut self) {
        if PRNT { printlnc!(yellow: "Pyrs: Regrowing dens..."); }
        self.dens_mut().regrow();
    }

    /// Enqueues cycle kernels.
    fn cycle(&mut self, control_layers: &mut ControlCellLayers, exe_graph: &mut ExecutionGraph)
            -> CmnResult<()> {
        // Control Pre:
        for lyr_idx in self.control_lyr_idxs.iter() {
            if PRNT { printlnc!(royal_blue: "    Pyrs: Pre-cycling control layer: [{:?}]...", lyr_idx); }
            control_layers.get_mut(lyr_idx).unwrap().cycle_pre(exe_graph, self.layer_addr)?;
        }

        // [DEBUG]: TEMPORARY:
        if PRNT { self.pyr_cycle_kernel.default_queue().unwrap().finish().unwrap(); }

        self.tfts.cycle(control_layers, exe_graph)?;

        if PRNT { printlnc!(yellow: "Pyrs: Cycling cell soma..."); }

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
            if PRNT { printlnc!(royal_blue: "    Ssts: Post-cycling control layer: [{:?}]...", lyr_idx); }
            control_layers.get_mut(lyr_idx).unwrap().cycle_post(exe_graph, self.layer_addr)?;
        }

        self.learn(exe_graph)?;

        // [DEBUG]: TEMPORARY:
        if PRNT { self.pyr_cycle_kernel.default_queue().unwrap().finish().unwrap(); }
        if PRNT { printlnc!(yellow: "Pyrs: Cycling complete for layer: '{}'.", self.layer_name); }

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
    #[inline] fn dens(&self) -> &Dendrites { self.tfts.dens() }
    #[inline] fn dens_mut(&mut self) -> &mut Dendrites { self.tfts.dens_mut() }
}


#[cfg(test)]
pub mod tests {
    use std::ops::{Range};
    // use rand::{Rng};
    use rand::distributions::{IndependentSample};
    use ocl::util;
    use cmn::{self, XorShiftRng, Range as RandRange};
    use cortex::{PyramidalLayer, DataCellLayer, DataCellLayerTest, CelCoords};

    impl DataCellLayerTest for PyramidalLayer {
        fn cycle_solo(&self) {
            self.pyr_cycle_kernel.default_queue().unwrap().finish().unwrap();

            // for cycle_kern in self.tft_cycle_kernels.iter() {
            //     cycle_kern.default_queue().unwrap().finish().unwrap();
            //     unsafe { cycle_kern.cmd().enq().expect("PyramidalLayer::cycle_self_only: tft_cycle_kernels"); }
            //     cycle_kern.default_queue().unwrap().finish().unwrap();
            // }

            self.tfts.cycle_solo();

            unsafe {
                self.pyr_cycle_kernel.cmd().enq()
                    .expect("PyramidalLayer::cycle_self_only: pyr_cycle_kernel");
            }

            self.pyr_cycle_kernel.default_queue().unwrap().finish().unwrap();
        }

        fn learn_solo(&mut self) {
            // for ltp_kernel in self.tft_ltp_kernels.iter_mut() {
            //     ltp_kernel.default_queue().unwrap().finish().unwrap();

            //     ltp_kernel.set_arg_scl_named("rnd", self.rng.gen::<i32>())
            //         .expect("<PyramidalLayer as DataCellLayerTest>::learn_solo [0]");

            //     unsafe {
            //         ltp_kernel.cmd().enq()
            //             .expect("<PyramidalLayer as DataCellLayerTest>::learn_solo [1]");
            //     }

            //     ltp_kernel.default_queue().unwrap().finish().unwrap();
            // }
            self.tfts.learn_solo();
        }

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
            self.tfts.best_den_states_raw().read(&mut vec).enq().unwrap();
            util::print_slice(&vec, 1 << 0, None, idx_range.clone(), false);

            print!("pyramidal.tft_best_den_states: ");
            self.tfts.best_den_states().read(&mut vec).enq().unwrap();
            util::print_slice(&vec, 1 << 0, None, idx_range.clone(), false);

        }

        fn print_all(&self) {
            self.print_range(None);
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
            self.tfts.best_den_ids().default_queue().unwrap().finish().unwrap();
            self.tfts.best_den_states().default_queue().unwrap().finish().unwrap();
            self.tfts.best_den_states_raw().default_queue().unwrap().finish().unwrap();

            self.states.cmd().fill(0, None).enq().unwrap();
            self.flag_sets.cmd().fill(0, None).enq().unwrap();
            self.tfts.best_den_ids().cmd().fill(0, None).enq().unwrap();
            self.tfts.best_den_states().cmd().fill(0, None).enq().unwrap();
            self.tfts.best_den_states_raw().cmd().fill(0, None).enq().unwrap();
            //self.best2_den_ids.cmd().fill(&[0], None).enq().unwrap();            // <<<<< SLATED FOR REMOVAL
            //self.best2_den_states.cmd().fill(&[0], None).enq().unwrap();        // <<<<< SLATED FOR REMOVAL

            // self.energies.cmd().fill(&[0], None).enq().unwrap();                // <<<<< SLATED FOR REMOVAL

            self.states.default_queue().unwrap().finish().unwrap();
            self.flag_sets.default_queue().unwrap().finish().unwrap();
            self.tfts.best_den_ids().default_queue().unwrap().finish().unwrap();
            self.tfts.best_den_states().default_queue().unwrap().finish().unwrap();
            self.tfts.best_den_states_raw().default_queue().unwrap().finish().unwrap();
        }


    }
}

