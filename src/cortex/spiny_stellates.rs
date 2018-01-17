// #![allow(unused_imports)]

// use std::collections::BTreeMap;
use rand::Rng;
use cmn::{self, CmnResult, CorticalDims};
use map::{AreaMap};
use ocl::{Kernel, ProQue, Buffer, Event, SpatialDims};
use map::{CellScheme, ExecutionGraph, CommandRelations,
    CorticalBuffer, LayerAddress, LayerTags, CommandUid};
use cortex::{Dendrites, AxonSpace, CorticalAreaSettings, DataCellLayer, ControlCellLayers,
    Tufts};


const PRNT: bool = false;
const TUFT_COUNT: usize = 1;


#[derive(Debug)]
pub struct SpinyStellateLayer {
    layer_name: String,
    // layer_id: usize,
    layer_addr: LayerAddress,
    layer_tags: LayerTags,
    dims: CorticalDims,
    cell_scheme: CellScheme,
    axn_slc_ids: Vec<u8>,
    // base_axn_slc: u8,
    lyr_axn_idz: u32,
    kern_cycle: Kernel,
    kern_mtp: Kernel,
    energies: Buffer<u8>,
    activities: Buffer<u8>,
    pub dens: Dendrites,
    rng: cmn::XorShiftRng,
    cycle_exe_cmd_uid: Option<CommandUid>,
    cycle_exe_cmd_idx: Option<usize>,
    mtp_exe_cmd_uid: Option<CommandUid>,
    mtp_exe_cmd_idx: Option<usize>,
    settings: CorticalAreaSettings,
    control_lyr_idxs: Vec<(LayerAddress, usize)>,
}

impl SpinyStellateLayer {
    pub fn new<S: Into<String>>(layer_name: S, layer_id: usize, dims: CorticalDims, cell_scheme: CellScheme,
            area_map: &AreaMap, axons: &AxonSpace, ocl_pq: &ProQue,
            settings: CorticalAreaSettings, exe_graph: &mut ExecutionGraph,
    ) -> CmnResult<SpinyStellateLayer> {
        let layer_name = layer_name.into();
        let layer_addr = LayerAddress::new(area_map.area_id(), layer_id);
        let axn_slc_ids = area_map.layer_slc_ids(&[layer_name.to_owned()]);
        let base_axn_slc = axn_slc_ids[0];
        let lyr_axn_idz = area_map.axn_idz(base_axn_slc);

        let tft_count = cell_scheme.tft_schemes().len();
        // Redesign kernel before changing the 1 tuft limitation:
        assert![tft_count == TUFT_COUNT];
        let ssc_tft_id = 0;

        let syns_per_tuft_l2: u8 = {
            let tft_scheme = &cell_scheme.tft_schemes()[ssc_tft_id];
            tft_scheme.syns_per_den_l2() + tft_scheme.dens_per_tft_l2()
        };

        let energies = Buffer::builder().queue(ocl_pq.queue().clone()).len(dims).fill_val(0).build()?;
        let activities = Buffer::builder().queue(ocl_pq.queue().clone()).len(dims).fill_val(0).build()?;

        println!("{mt}{mt}SPINYSTELLATES::NEW(): base_axn_slc: {}, lyr_axn_idz: {}, dims: {:?}",
            base_axn_slc, lyr_axn_idz, dims, mt = cmn::MT);

        // let dens_dims = dims.clone_with_ptl2(cell_scheme.dens_per_tft_l2 as i8);
        let dens = try!(Dendrites::new(layer_name.clone(), layer_id, dims, cell_scheme.clone(),
            area_map, axons, ocl_pq,
            settings.disable_sscs, exe_graph));
        let _grp_count = cmn::OPENCL_MINIMUM_WORKGROUP_SIZE;
        let _cels_per_grp = dims.per_subgrp(_grp_count).expect("SpinyStellateLayer::new()");

        /*=============================================================================
        ===============================================================================
        =============================================================================*/

        let kern_name = "ssc_cycle";
        let kern_cycle = ocl_pq.create_kernel(kern_name)?
            // .gws(dims)
            .gws(SpatialDims::Three(dims.depth() as usize, dims.v_size() as usize, dims.u_size() as usize))
            .arg_buf(&energies)
            .arg_buf(dens.states());

        // let mut cycle_cmd_srcs = Vec::with_capacity(2);
        // // cycle_cmd_srcs.push(CorticalBuffer::data_syn_tft(dens.syns().states(), layer_addr, ssc_tft_id));
        // cycle_cmd_srcs.push(CorticalBuffer::data_soma_tft(&energies, layer_addr, ssc_tft_id));

        let cycle_exe_cmd_uid = if settings.disable_sscs {
            None
        } else {
            Some(exe_graph.add_command(CommandRelations::cortical_kernel(kern_name,
                vec![CorticalBuffer::data_soma_tft(&energies, layer_addr, ssc_tft_id)],
                vec![CorticalBuffer::data_den_tft(dens.states(), layer_addr, ssc_tft_id)]) )?)
        };

        let kern_name = "ssc_mtp_simple";
        let kern_mtp = ocl_pq.create_kernel(kern_name)?
            // .gws(dims)
            .gws(SpatialDims::Two(1, dims.cells() as usize))
            .arg_buf(axons.states())
            .arg_buf(dens.syns().states())
            .arg_scl(lyr_axn_idz)
            // .arg_scl(cels_per_grp)
            .arg_scl(syns_per_tuft_l2)
            // CURRENTLY UNUSED:
            .arg_scl_named::<u32>("rnd", None)
            // .arg_buf_named("aux_ints_0", None)
            // .arg_buf_named("aux_ints_1", None)
            .arg_buf(dens.syns().strengths());

        ////// KEEP ME:
            // let kern_name = "ssc_mtp";
            // let kern_mtp = ocl_pq.create_kernel(kern_name)?
            //     // .expect("SpinyStellateLayer::new()")
            //     .gws(SpatialDims::Two(tft_count, grp_count as usize))
            //     .arg_buf(axons.states())
            //     .arg_buf(dens.syns().states())
            //     .arg_scl(lyr_axn_idz)
            //     .arg_scl(_cels_per_grp)
            //     .arg_scl(syns_per_tuft_l2)
            //     .arg_scl_named::<u32>("rnd", None)
            //     // .arg_buf_named("aux_ints_0", None)
            //     // .arg_buf_named("aux_ints_1", None)
            //     .arg_buf(dens.syns().strengths());
        ///////


        // Set up execution command:
        let mut mtp_cmd_srcs: Vec<CorticalBuffer> = axn_slc_ids.iter()
            .map(|&slc_id|
                CorticalBuffer::axon_slice(&axons.states(), layer_addr.area_id(), slc_id))
            .collect();

        mtp_cmd_srcs.push(CorticalBuffer::data_syn_tft(dens.syns().states(), layer_addr, ssc_tft_id));

        let mtp_exe_cmd_uid = if settings.disable_sscs | settings.disable_learning {
            None
        } else {
            Some(exe_graph.add_command(CommandRelations::cortical_kernel(kern_name, mtp_cmd_srcs,
                vec![CorticalBuffer::data_syn_tft(dens.syns().strengths(), layer_addr, ssc_tft_id)]))?)
        };

        /*=============================================================================
        ===============================================================================
        =============================================================================*/

        Ok(SpinyStellateLayer {
            layer_name: layer_name,
            // layer_id: layer_id,
            layer_addr,
            layer_tags: area_map.layer_map().layer_info(layer_id).unwrap().layer_tags(),
            dims: dims,
            cell_scheme: cell_scheme,
            axn_slc_ids: axn_slc_ids,
            // base_axn_slc: base_axn_slc,
            lyr_axn_idz: lyr_axn_idz,
            kern_cycle: kern_cycle,
            kern_mtp: kern_mtp,
            energies,
            activities,
            rng: cmn::weak_rng(),
            dens: dens,
            cycle_exe_cmd_uid,
            cycle_exe_cmd_idx: None,
            mtp_exe_cmd_uid,
            mtp_exe_cmd_idx: None,
            settings,
            control_lyr_idxs: Vec::with_capacity(4),
        })
    }

    pub fn set_exe_order_cycle(&mut self, control_layers: &mut ControlCellLayers,
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

        if !self.settings.disable_sscs {
            // Control layers pre-cycle:
            for cl_idx in self.control_lyr_idxs.iter() {
                control_layers.get_mut(cl_idx).unwrap().set_exe_order_pre(exe_graph, self.layer_addr)?;
            }

            // Dendrites:
            self.dens.set_exe_order(exe_graph)?;

            // Soma:
            if let Some(cycle_cmd_uid) = self.cycle_exe_cmd_uid {
                self.cycle_exe_cmd_idx = Some(exe_graph.order_command(cycle_cmd_uid)?);
            }

            // Control layers post-cycle:
            for cl_idx in self.control_lyr_idxs.iter() {
                control_layers.get_mut(cl_idx).unwrap().set_exe_order_post(exe_graph, self.layer_addr)?;
            }
        }

        Ok(())
    }

    pub fn set_exe_order_learn(&mut self, exe_graph: &mut ExecutionGraph) -> CmnResult<()> {
        if !self.settings.disable_sscs & !self.settings.disable_learning {
            if let Some(cmd_uid) = self.mtp_exe_cmd_uid {
                self.mtp_exe_cmd_idx = Some(exe_graph.order_command(cmd_uid)?);
            }
        }
        Ok(())
    }

    #[inline]
    pub fn cycle(&mut self, control_layers: &mut ControlCellLayers, exe_graph: &mut ExecutionGraph)
            -> CmnResult<()>
    {
        if PRNT { printlnc!(royal_blue: "Ssts: Cycling layer: '{}'...", self.layer_name); }

        // Pre cycle:
        for lyr_idx in self.control_lyr_idxs.iter() {
            if PRNT { printlnc!(royal_blue: "    Ssts: Pre-cycling control layer: [{:?}]...", lyr_idx); }
            control_layers.get_mut(lyr_idx).unwrap().cycle_pre(exe_graph, self.layer_addr)?;
        }

        // Cycle dens:
        self.dens.cycle(exe_graph)?;

        // Cycle soma (currently adds energies to den states):
        if let Some(cycle_cmd_idx) = self.cycle_exe_cmd_idx {
            let mut event = Event::empty();
            unsafe {
                self.kern_cycle.cmd().ewait(exe_graph.get_req_events(cycle_cmd_idx)?)
                    .enew(&mut event).enq()?;
            }
            exe_graph.set_cmd_event(cycle_cmd_idx, Some(event))?;
        }

        // Post cycle:
        for lyr_idx in self.control_lyr_idxs.iter() {
            if PRNT { printlnc!(royal_blue: "    Ssts: Post-cycling control layer: [{:?}]...", lyr_idx); }
            control_layers.get_mut(lyr_idx).unwrap().cycle_post(exe_graph, self.layer_addr)?;
        }

        if PRNT { printlnc!(royal_blue: "Ssts: Cycling complete for layer: '{}'.", self.layer_name); }
        Ok(())
    }


    #[inline]
    pub fn learn(&mut self, exe_graph: &mut ExecutionGraph) -> CmnResult<()> {
        if let Some(cmd_idx) = self.mtp_exe_cmd_idx {
            if PRNT { printlnc!(royal_blue: "Ssts: Performing learning for layer: '{}'...", self.layer_name); }
            let rnd = self.rng.gen::<u32>();
            self.kern_mtp.set_arg_scl_named("rnd", rnd).unwrap();

            let mut event = Event::empty();
            unsafe { self.kern_mtp.cmd().ewait(exe_graph.get_req_events(cmd_idx)?).enew(&mut event).enq()?; }
            exe_graph.set_cmd_event(cmd_idx, Some(event))?;
            if PRNT { printlnc!(royal_blue: "Ssts: Learning complete for layer: '{}'.", self.layer_name); }
        }
        Ok(())
    }

    #[inline] pub fn regrow(&mut self) {
        self.dens.regrow();
    }

    #[inline]
    pub fn axn_range(&self) -> (usize, usize) {
        let sscs_axn_idn = self.lyr_axn_idz + (self.dims.cells());
        (self.lyr_axn_idz as usize, sscs_axn_idn as usize)
    }

    #[inline] pub fn layer_name<'s>(&'s self) -> &'s str { &self.layer_name }
    #[inline] pub fn layer_tags(&self) -> LayerTags { self.layer_tags }
    #[inline] pub fn layer_addr(&self) -> LayerAddress { self.layer_addr }
    #[inline] pub fn soma(&self) -> &Buffer<u8> { self.dens.states() }
    #[inline] pub fn energies(&self) -> &Buffer<u8> { &self.energies }
    #[inline] pub fn activities(&self) -> &Buffer<u8> { &self.activities }
    #[inline] pub fn dims(&self) -> &CorticalDims { &self.dims }
    #[inline] pub fn axn_slc_ids(&self) -> &[u8] { self.axn_slc_ids.as_slice() }
    #[inline] pub fn base_axn_slc(&self) -> u8 { self.axn_slc_ids[0] }
    #[inline] pub fn tft_count(&self) -> usize { TUFT_COUNT }
    #[inline] pub fn dens(&self) -> &Dendrites { &self.dens }
    #[inline] pub fn dens_mut(&mut self) -> &mut Dendrites { &mut self.dens }
}

impl DataCellLayer for SpinyStellateLayer {
    #[inline]
    fn learn(&mut self, exe_graph: &mut ExecutionGraph) -> CmnResult <()> {
        self.learn(exe_graph)
    }

    #[inline]
    fn cycle(&mut self, control_layers: &mut ControlCellLayers, exe_graph: &mut ExecutionGraph)
            -> CmnResult<()>
    {
        self.cycle(control_layers, exe_graph)
    }

    #[inline]
    fn regrow(&mut self) {
        self.regrow()
    }

    #[inline]
    fn axn_range(&self) -> (usize, usize) {
        self.axn_range()
    }

    #[inline] fn layer_name<'s>(&'s self) -> &'s str { &self.layer_name }
    #[inline] fn layer_addr(&self) -> LayerAddress { self.layer_addr }
    #[inline] fn soma(&self) -> &Buffer<u8> { self.dens.states() }
    #[inline] fn soma_mut(&mut self) -> &mut Buffer<u8> { self.dens.states_mut() }
    #[inline] fn energies(&self) -> &Buffer<u8> { &self.energies }
    #[inline] fn activities(&self) -> &Buffer<u8> { &self.activities }
    #[inline] fn dims(&self) -> &CorticalDims { &self.dims }
    #[inline] fn axn_slc_ids(&self) -> &[u8] { self.axn_slc_ids.as_slice() }
    #[inline] fn base_axn_slc(&self) -> u8 { self.axn_slc_ids[0] }
    #[inline] fn tft_count(&self) -> usize { TUFT_COUNT }
    #[inline] fn cell_scheme(&self) -> &CellScheme { &self.cell_scheme }
    #[inline] fn tufts(&self) -> &Tufts { unimplemented!(); }
    #[inline] fn dens(&self) -> &Dendrites { &self.dens }
    #[inline] fn dens_mut(&mut self) -> &mut Dendrites { &mut self.dens }
}



#[cfg(test)]
pub mod tests {
    use std::ops::{Range};
    use rand::{Rng};
    use rand::distributions::{IndependentSample};
    // use ocl::util;
    use cmn::{self, XorShiftRng, Range as RandRange};
    use cortex::{SpinyStellateLayer, DendritesTest, DataCellLayerTest, CelCoords};

    impl DataCellLayerTest for SpinyStellateLayer {
        fn cycle_solo(&self) {
            // self.dens.syns().cycle_solo();
            self.dens.cycle_solo();
        }

        fn learn_solo(&mut self) {
            self.kern_mtp.default_queue().unwrap().finish().unwrap();
            let rnd = self.rng.gen::<u32>();
            self.kern_mtp.set_arg_scl_named("rnd", rnd).unwrap();

            unsafe {
            self.kern_mtp.cmd().enq()
                .expect("<SpinyStellateLayer as DataCellLayerTest>::learn_solo [1]");
            }

            self.kern_mtp.default_queue().unwrap().finish().unwrap();
        }

        /// Prints a range of pyramidal buffers.
        ///
        //
        ////// Ocl print function signature:
        //
        // ocl::util::print_slice<T: OclScl>(vec: &[T], every: usize, val_range: Option<(T, T)>,
        // idx_range: Option<Range<usize>>, show_zeros: bool)
        //
        fn print_range(&self, _: Option<Range<usize>>, /*print_children: bool*/) {
            // let mut vec = vec![0; self.dens.states().len()];

            // states: Buffer<u8>,
            // flag_sets: Buffer<u8>,
            // pyr_states: Buffer<u8>,
            // tft_best_den_ids: Buffer<u8>,
            // tft_best_den_states_raw: Buffer<u8>,
            // tft_best_den_states: Buffer<u8>,

            // print!("pyramidal.states: ");
            // self.states.read(&mut vec).enq().unwrap();
            // util::print_slice(&vec, 1 << 0, None, idx_range.clone(), false);

            // print!("pyramidal.tft_best_den_states_raw: ");
            // self.tft_best_den_states_raw.read(&mut vec).enq().unwrap();
            // util::print_slice(&vec, 1 << 0, None, idx_range.clone(), false);

            // print!("pyramidal.tft_best_den_states: ");
            // self.tft_best_den_states.read(&mut vec).enq().unwrap();
            // util::print_slice(&vec, 1 << 0, None, idx_range.clone(), false);

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

        fn set_all_to_zero(&mut self) {
            self.dens.states().default_queue().unwrap().finish().unwrap();

            self.dens.states().cmd().fill(0, None).enq().unwrap();

            self.dens.states().default_queue().unwrap().finish().unwrap();
        }
    }
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