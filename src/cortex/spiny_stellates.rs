// #![allow(unused_imports)]

use rand::{self, Rng};
use cmn::{self, CmnResult, CorticalDims};
use map::{AreaMap};
use ocl::{Kernel, ProQue, Buffer, Event};
use map::{CellScheme, DendriteKind, ExecutionGraph, ExecutionCommand,
    CorticalBuffer, LayerAddress, LayerTags};
use cortex::{Dendrites, AxonSpace, CorticalAreaSettings, DataCellLayer, ControlCellLayer};


const PRINT_DEBUG: bool = false;
const TUFT_COUNT: usize = 1;


pub struct SpinyStellateLayer {
    layer_name: &'static str,
    // layer_id: usize,
    layer_addr: LayerAddress,
    layer_tags: LayerTags,
    dims: CorticalDims,
    cell_scheme: CellScheme,
    axn_slc_ids: Vec<u8>,
    // base_axn_slc: u8,
    lyr_axn_idz: u32,
    kern_ltp: Kernel,
    rng: rand::XorShiftRng,
    pub dens: Dendrites,
    ltp_exe_cmd_idx: Option<usize>,
    _settings: CorticalAreaSettings,
    control_lyr_idxs: Vec<usize>,
}

impl SpinyStellateLayer {
    pub fn new(layer_name: &'static str, layer_id: usize, dims: CorticalDims, cell_scheme: CellScheme,
            area_map: &AreaMap, axons: &AxonSpace, ocl_pq: &ProQue,
            settings: CorticalAreaSettings, exe_graph: &mut ExecutionGraph,
    ) -> CmnResult<SpinyStellateLayer> {
        let layer_addr = LayerAddress::new(area_map.area_id(), layer_id);
        let axn_slc_ids = area_map.layer_slc_ids(&[layer_name.to_owned()]);
        let base_axn_slc = axn_slc_ids[0];
        let lyr_axn_idz = area_map.axn_idz(base_axn_slc);

        let tft_count = cell_scheme.tft_schemes().len();
        // Redesign kernel before changing the 1 tuft limitation:
        assert![tft_count == TUFT_COUNT];
        let sst_tft_id = 0;

        let syns_per_tuft_l2: u8 = {
            let tft_scheme = &cell_scheme.tft_schemes()[sst_tft_id];
            tft_scheme.syns_per_den_l2() + tft_scheme.dens_per_tft_l2()
        };

        println!("{mt}{mt}SPINYSTELLATES::NEW(): base_axn_slc: {}, lyr_axn_idz: {}, dims: {:?}",
            base_axn_slc, lyr_axn_idz, dims, mt = cmn::MT);

        // let dens_dims = dims.clone_with_ptl2(cell_scheme.dens_per_tft_l2 as i8);
        let dens = try!(Dendrites::new(layer_name, layer_id, dims, cell_scheme.clone(),
            DendriteKind::Proximal, /*DataCellKind::SpinyStellate,*/ area_map, axons, ocl_pq,
            settings.disable_ssts, exe_graph));
        let _grp_count = cmn::OPENCL_MINIMUM_WORKGROUP_SIZE;
        let _cels_per_grp = dims.per_subgrp(_grp_count).expect("SpinyStellateLayer::new()");

        /*=============================================================================
        ===============================================================================
        =============================================================================*/

        let kern_name = "sst_ltp_simple";
        let kern_ltp = ocl_pq.create_kernel(kern_name)?
            // .expect("SpinyStellateLayer::new()")
            .gws(dims)
            .arg_buf(axons.states())
            .arg_buf(dens.syns().states())
            .arg_scl(lyr_axn_idz)
            // .arg_scl(cels_per_grp)
            .arg_scl(syns_per_tuft_l2)
            .arg_scl_named::<u32>("rnd", None)
            // .arg_buf_named("aux_ints_0", None)
            // .arg_buf_named("aux_ints_1", None)
            .arg_buf(dens.syns().strengths());

        // let kern_name = "sst_ltp";
        // let kern_ltp = ocl_pq.create_kernel(kern_name)?
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

        // Set up execution command:
        let mut ltp_cmd_srcs: Vec<CorticalBuffer> = axn_slc_ids.iter()
            .map(|&slc_id|
                CorticalBuffer::axon_slice(&axons.states(), layer_addr.area_id(), slc_id))
            .collect();

        ltp_cmd_srcs.push(CorticalBuffer::data_syn_tft(dens.syns().states(), layer_addr, sst_tft_id));

        let ltp_exe_cmd_idx = if settings.disable_learning {
            None
        } else {
            Some(exe_graph.add_command(ExecutionCommand::cortical_kernel(kern_name, ltp_cmd_srcs,
                vec![CorticalBuffer::data_syn_tft(dens.syns().strengths(), layer_addr, sst_tft_id)]))?)
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
            kern_ltp: kern_ltp,
            rng: rand::weak_rng(),
            dens: dens,
            ltp_exe_cmd_idx: ltp_exe_cmd_idx,
            _settings: settings,
            control_lyr_idxs: Vec::with_capacity(4),
        })
    }

    pub fn set_exe_order_cycle(&mut self, control_layers: &[Box<ControlCellLayer>],
            exe_graph: &mut ExecutionGraph) -> CmnResult<()>
    {
        for (cl_idx, cl) in control_layers.iter().enumerate() {
            if cl.host_layer_addr() == self.layer_addr {
                self.control_lyr_idxs.push(cl_idx);
            }
        }

        for lyr in self.control_lyr_idxs.iter().map(|&idx| &control_layers[idx]) {
            lyr.set_exe_order_pre(exe_graph, self.layer_addr)?;
        }
        self.dens.set_exe_order(exe_graph)?;
        for lyr in self.control_lyr_idxs.iter().map(|&idx| &control_layers[idx]) {
            lyr.set_exe_order_post(exe_graph, self.layer_addr)?;
        }
        Ok(())
    }

    pub fn set_exe_order_learn(&self, exe_graph: &mut ExecutionGraph) -> CmnResult<()> {
        if let Some(cmd_idx) = self.ltp_exe_cmd_idx {
            exe_graph.order_next(cmd_idx)?;
        }
        Ok(())
    }

    #[inline]
    pub fn cycle(&mut self, control_layers: &[Box<ControlCellLayer>], exe_graph: &mut ExecutionGraph)
            -> CmnResult<()>
    {
        if PRINT_DEBUG { printlnc!(royal_blue: "Ssts: Cycling layer: '{}'...", self.layer_name); }
        for lyr in self.control_lyr_idxs.iter().map(|&idx| &control_layers[idx]) {
            lyr.cycle_pre(exe_graph, self.layer_addr)?;
        }
        self.dens.cycle(exe_graph)?;
        for lyr in self.control_lyr_idxs.iter().map(|&idx| &control_layers[idx]) {
            lyr.cycle_post(exe_graph, self.layer_addr)?;
        }
        if PRINT_DEBUG { printlnc!(royal_blue: "Ssts: Cycling complete for layer: '{}'.", self.layer_name); }
        Ok(())
    }


    #[inline]
    pub fn learn(&mut self, exe_graph: &mut ExecutionGraph) -> CmnResult<()> {
        if let Some(cmd_idx) = self.ltp_exe_cmd_idx {
            if PRINT_DEBUG { printlnc!(royal_blue: "Ssts: Performing learning for layer: '{}'...", self.layer_name); }
            let rnd = self.rng.gen::<u32>();
            self.kern_ltp.set_arg_scl_named("rnd", rnd).unwrap();

            let mut event = Event::empty();
            self.kern_ltp.cmd().ewait(exe_graph.get_req_events(cmd_idx)?).enew(&mut event).enq()?;
            exe_graph.set_cmd_event(cmd_idx, Some(event))?;
            if PRINT_DEBUG { printlnc!(royal_blue: "Ssts: Learning complete for layer: '{}'.", self.layer_name); }
        }
        Ok(())
    }

    #[inline] pub fn regrow(&mut self) {
        self.dens.regrow();
    }

    #[inline]
    pub fn axn_range(&self) -> (usize, usize) {
        let ssts_axn_idn = self.lyr_axn_idz + (self.dims.cells());
        (self.lyr_axn_idz as usize, ssts_axn_idn as usize)
    }

    #[inline] pub fn layer_name(&self) -> &'static str { self.layer_name }
    #[inline] pub fn layer_tags(&self) -> LayerTags { self.layer_tags }
    #[inline] pub fn layer_addr(&self) -> LayerAddress { self.layer_addr }
    #[inline] pub fn soma(&self) -> &Buffer<u8> { self.dens.states() }
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
    fn cycle(&mut self, control_layers: &[Box<ControlCellLayer>], exe_graph: &mut ExecutionGraph)
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

    #[inline] fn layer_name(&self) -> &'static str { self.layer_name }
    #[inline] fn layer_addr(&self) -> LayerAddress { self.layer_addr }
    #[inline] fn soma(&self) -> &Buffer<u8> { self.dens.states() }
    #[inline] fn soma_mut(&mut self) -> &mut Buffer<u8> { self.dens.states_mut() }
    #[inline] fn dims(&self) -> &CorticalDims { &self.dims }
    #[inline] fn axn_slc_ids(&self) -> &[u8] { self.axn_slc_ids.as_slice() }
    #[inline] fn base_axn_slc(&self) -> u8 { self.axn_slc_ids[0] }
    #[inline] fn tft_count(&self) -> usize { TUFT_COUNT }
    #[inline] fn cell_scheme(&self) -> &CellScheme { &self.cell_scheme }
    #[inline] fn dens(&self) -> &Dendrites { &self.dens }
    #[inline] fn dens_mut(&mut self) -> &mut Dendrites { &mut self.dens }
}



#[cfg(test)]
pub mod tests {
    use std::ops::{Range};
    use rand::{XorShiftRng, Rng};
    use rand::distributions::{IndependentSample, Range as RandRange};
    // use ocl::util;
    use cmn;
    use cortex::{SpinyStellateLayer, DendritesTest, DataCellLayerTest, CelCoords};

    impl DataCellLayerTest for SpinyStellateLayer {
        fn cycle_solo(&self) {
            // self.dens.syns().cycle_solo();
            self.dens.cycle_solo();
        }

        fn learn_solo(&mut self) {
            self.kern_ltp.default_queue().unwrap().finish().unwrap();
            let rnd = self.rng.gen::<u32>();
            self.kern_ltp.set_arg_scl_named("rnd", rnd).unwrap();

            self.kern_ltp.cmd().enq()
                .expect("<SpinyStellateLayer as DataCellLayerTest>::learn_solo [1]");

            self.kern_ltp.default_queue().unwrap().finish().unwrap();
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