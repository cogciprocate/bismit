use std::collections::HashMap;
use std::ops::Range;
use std::borrow::Borrow;
use ocl::{flags, Device, ProQue, Context, Buffer, Event, Queue};
// use ocl::core::ClWaitList;
use cmn::{self, CmnError, CmnResult, CorticalDims, DataCellLayer};
use map::{self, AreaMap, SliceTractMap, LayerKind, CellKind, InhibitoryCellKind,
    ExecutionGraph, /*AxonDomainRoute,*/};
use ::Thalamus;
use cortex::{AxonSpace, Minicolumns, InhibitoryInterneuronNetwork, PyramidalLayer,
    SpinyStellateLayer};

#[cfg(test)] pub use self::tests::{CorticalAreaTest};

// Out of order asynchronous command queues:
const QUEUE_OUT_OF_ORDER: bool = true;

// GDB debug mode:
const KERNEL_DEBUG_MODE: bool = false;

pub type CorticalAreas = HashMap<&'static str, Box<CorticalArea>>;


/// Cortical area settings.
#[derive(Debug, Clone)]
pub struct CorticalAreaSettings {
    pub bypass_inhib: bool,
    pub bypass_filters: bool,
    pub disable_pyrs: bool,
    pub disable_ssts: bool,
    pub disable_mcols: bool,
    pub disable_regrowth: bool,
    pub disable_learning: bool,
}

impl CorticalAreaSettings {
    pub fn new() -> CorticalAreaSettings {
        CorticalAreaSettings {
            bypass_inhib: false,
            bypass_filters: false,
            disable_pyrs: false,
            disable_ssts: false,
            disable_mcols: false,
            disable_regrowth: false,
            disable_learning: false,
        }
    }
}


/// An area of the cortex.
pub struct CorticalArea {
    area_id: usize,
    name: &'static str,
    dims: CorticalDims,
    area_map: AreaMap,
    axns: AxonSpace,
    mcols: Box<Minicolumns>,
    pyrs_map: HashMap<&'static str, Box<PyramidalLayer>>,
    ssts_map: HashMap<&'static str, Box<SpinyStellateLayer>>,
    iinns: HashMap<&'static str, Box<InhibitoryInterneuronNetwork>>,
    ptal_name: &'static str,    // PRIMARY TEMPORAL ASSOCIATIVE LAYER NAME
    psal_name: &'static str,    // PRIMARY SPATIAL ASSOCIATIVE LAYER NAME
    aux: Aux,
    ocl_pq: ProQue,
    write_queue: Queue,
    read_queue: Queue,
    counter: usize,
    settings: CorticalAreaSettings,
    exe_graph: ExecutionGraph,
}

impl CorticalArea {
    /// Creates a new cortical area.
    ///
    //
    // [TODO]: Break this function up a bit. Probably break the major sections
    // out into new types.
    //
    // The only use for `thal` is currently within `axon_space` for the
    // purpose of getting precise slice ids for layers in other areas for use
    // by the execution graph system.
    //
    pub fn new(area_map: AreaMap, device_idx: usize, ocl_context: &Context,
                    settings: Option<CorticalAreaSettings>, thal: &Thalamus) -> CmnResult<CorticalArea> {
        let emsg = "cortical_area::CorticalArea::new()";
        let area_id = area_map.area_id();
        let area_name = area_map.area_name();

        println!("\n\nCORTICALAREA::NEW(): Creating Cortical Area: \"{}\"...", area_name);

        // Optionally pass `-g` and `-s {cl path}` flags to compiler:
        let build_options = if KERNEL_DEBUG_MODE && cfg!(target_os = "linux") {
            // [TODO]: Add something to identify the platform vendor and match:
            // let debug_opts = format!("-g -s {}", cmn::cl_root_path().join("bismit.cl").to_str());
            let debug_opts = "-g";
            area_map.gen_build_options().cmplr_opt(debug_opts)
        } else {
            area_map.gen_build_options()
        };

        let ocl_pq = ProQue::builder()
            .device(device_idx)
            .context(ocl_context.clone())
            .prog_bldr(build_options)
            // .queue_properties(/*flags::QUEUE_PROFILING_ENABLE |*/ flags::QUEUE_OUT_OF_ORDER_EXEC_MODE_ENABLE)
            .build().expect("CorticalArea::new(): ocl_pq.build(): error");

        let (write_queue, read_queue) = if QUEUE_OUT_OF_ORDER {
            (Queue::new(ocl_context, ocl_pq.device().clone(),
                    Some(/*flags::QUEUE_PROFILING_ENABLE |*/ flags::QUEUE_OUT_OF_ORDER_EXEC_MODE_ENABLE))?,
                Queue::new(ocl_context, ocl_pq.device().clone(),
                    Some(/*flags::QUEUE_PROFILING_ENABLE |*/ flags::QUEUE_OUT_OF_ORDER_EXEC_MODE_ENABLE))?)
        } else {
            (ocl_pq.queue().clone(), ocl_pq.queue().clone())
        };

        let dims = area_map.dims().clone_with_incr(ocl_pq.max_wg_size().unwrap());

        println!("{mt}CORTICALAREA::NEW(): Area \"{}\" details: \
            (u_size: {}, v_size: {}, depth: {}), eff_areas: {:?}, aff_areas: {:?}, \n\
            {mt}{mt}device_idx: [{}], device.name(): {}, device.vendor(): {}",
            area_name, dims.u_size(), dims.v_size(), dims.depth(), area_map.eff_areas(),
            area_map.aff_areas(), device_idx, ocl_pq.device().name().trim(),
            ocl_pq.device().vendor().trim(), mt = cmn::MT);

        let psal_name = area_map.layers().layers_containing_tags(map::SPATIAL_ASSOCIATIVE)[0].name();
        let ptal_name = area_map.layers().layers_containing_tags(map::TEMPORAL_ASSOCIATIVE)[0].name();

        let settings = settings.unwrap_or(CorticalAreaSettings::new());

        /*=============================================================================
        =============================== EXECUTION GRAPH ===============================
        =============================================================================*/

        let mut exe_graph = ExecutionGraph::new();

        /*=============================================================================
        ================================ CELLS & AXONS ================================
        =============================================================================*/

        let mut pyrs_map = HashMap::new();
        let mut ssts_map = HashMap::new();
        let mut iinns = HashMap::new();
        let mut mcols = None;
        let axns = AxonSpace::new(&area_map, &ocl_pq, &write_queue, &mut exe_graph, thal)?;
        // println!("{mt}::NEW(): IO_INFO: {:#?}, Settings: {:#?}", axns.io_info(), settings, mt = cmn::MT);

        /*=============================================================================
        ================================== DATA CELLS =================================
        =============================================================================*/
        // [TODO]: BREAK OFF THIS CODE INTO NEW STRUCT DEF

        for layer in area_map.layers().iter() {
            match layer.kind() {
                &LayerKind::Cellular(ref cell_scheme) => {
                    println!("{mt}::NEW(): making a(n) {:?} layer: '{}' (depth: {})",
                        cell_scheme.cell_kind(), layer.name(), layer.depth(), mt = cmn::MT);

                    match *cell_scheme.cell_kind() {
                        CellKind::Pyramidal => {
                            let pyrs_dims = dims.clone_with_depth(layer.depth());

                            let pyr_lyr = try!(PyramidalLayer::new(layer.name(), layer.layer_id(),
                                pyrs_dims, cell_scheme.clone(), &area_map, &axns, &ocl_pq,
                                &mut exe_graph));

                            pyrs_map.insert(layer.name(), Box::new(pyr_lyr));
                        },
                        CellKind::SpinyStellate => {
                            let ssts_map_dims = dims.clone_with_depth(layer.depth());

                            let sst_lyr = try!(SpinyStellateLayer::new(layer.name(), layer.layer_id(),
                                ssts_map_dims, cell_scheme.clone(), &area_map, &axns, &ocl_pq,
                                &mut exe_graph));

                            ssts_map.insert(layer.name(), Box::new(sst_lyr));
                        },
                        _ => (),
                    }
                },
                _ => (),
            }
        }


        /*=============================================================================
        ================================ CONTROL CELLS ================================
        =============================================================================*/
        // [TODO]: BREAK OFF THIS CODE INTO NEW STRUCT DEF

        for layer in area_map.layers().iter() {
            if let LayerKind::Cellular(ref layer_kind) = *layer.kind() {
                if let CellKind::Inhibitory(ref inh_cell_kind) = *layer_kind.cell_kind() {
                    match *inh_cell_kind {
                        InhibitoryCellKind::BasketSurround { lyr_name: ref src_lyr_name, field_radius: _ } => {
                            let em1 = format!("{}: '{}' is not a valid layer", emsg, src_lyr_name);
                            let src_soma = &ssts_map.get_mut(src_lyr_name.as_str()).expect(&em1);
                            let src_soma_buf = src_soma.soma();

                            let src_slc_ids = area_map.layer_slc_ids(&[src_lyr_name.clone()]);
                            let src_lyr_depth = src_slc_ids.len() as u8;
                            let src_base_axn_slc = src_slc_ids[0];

                            let iinns_dims = dims.clone_with_depth(src_lyr_depth);
                            let iinn_lyr = InhibitoryInterneuronNetwork::new(layer.name(),
                                layer.layer_id(), iinns_dims, layer_kind.clone(),
                                &area_map, src_soma_buf, src_soma.layer_id(), src_base_axn_slc,
                                src_soma.tft_count(),
                                &axns, &ocl_pq, &mut exe_graph)?;

                            iinns.insert(layer.name(), Box::new(iinn_lyr));
                        },
                    }
                }
            }
        }

        for layer in area_map.layers().iter() {
            match layer.kind() {
                &LayerKind::Cellular(ref cell_scheme) => {
                    println!("{mt}::NEW(): making a(n) {:?} layer: '{}' (depth: {})",
                        cell_scheme.cell_kind(), layer.name(), layer.depth(), mt = cmn::MT);

                    match *cell_scheme.cell_kind() {
                        CellKind::Complex => {
                            let mcols_dims = dims.clone_with_depth(1);

                            mcols = Some(Box::new({
                                let ssts = ssts_map.get(psal_name)
                                    .expect(&format!("{}: '{}' is not a valid layer", emsg, psal_name));

                                let pyrs = pyrs_map.get(ptal_name)
                                    .expect(&format!("{}: '{}' is not a valid layer", emsg, ptal_name));

                                let layer_id = layer.layer_id();

                                debug_assert!(area_map.aff_out_slcs().len() > 0, "CorticalArea::new(): \
                                    No afferent output slices found for area: '{}'", area_name);
                                Minicolumns::new(layer_id, mcols_dims, &area_map, &axns, ssts, pyrs,
                                    &ocl_pq, &mut exe_graph)?
                            }));
                        },
                        _ => (),
                    }
                },
                _ => (),
            }
        }

        let mut mcols = mcols.expect("CorticalArea::new(): No Minicolumn layer found!");

        // let mcols_dims = dims.clone_with_depth(1);

        // // <<<<< EVENTUALLY ADD TO CONTROL CELLS (+PROTOCONTROLCELLS) >>>>>
        // let mcols = Box::new({
        //     let em_ssts = format!("{}: '{}' is not a valid layer", emsg, psal_name);
        //     let ssts = ssts_map.get(psal_name).expect(&em_ssts);

        //     let em_pyrs = format!("{}: '{}' is not a valid layer", emsg, ptal_name);
        //     let pyrs = pyrs_map.get(ptal_name).expect(&em_pyrs);

        //     debug_assert!(area_map.aff_out_slcs().len() > 0, "CorticalArea::new(): \
        //         No afferent output slices found for area: '{}'", area_name);
        //     Minicolumns::new(mcols_dims, &area_map, &axns, ssts, pyrs, /*&aux,*/ &ocl_pq,
        //         &mut exe_graph)?
        // });

        /*=============================================================================
        ===================================== AUX =====================================
        =============================================================================*/

        let aux = Aux::new(pyrs_map[ptal_name].dens().syns().len(), &ocl_pq);

        // <<<<< TODO: CLEAN THIS UP >>>>>
        // MAKE ABOVE LIKE BELOW (eliminate set_arg_buf_named() methods and just call directly on buffer)
        // mcols.set_arg_buf_named("aux_ints_0", &aux.ints_0).unwrap();
        pyrs_map.get_mut(ptal_name).unwrap()
            .set_arg_buf_named("aux_ints_0", &aux.ints_0).unwrap();
        pyrs_map.get_mut(ptal_name).unwrap()
            .set_arg_buf_named("aux_ints_1", &aux.ints_1).unwrap();

        pyrs_map.get_mut(ptal_name).unwrap().dens_mut()
            .set_arg_buf_named("aux_ints_0", &aux.ints_0).unwrap();
        pyrs_map.get_mut(ptal_name).unwrap().dens_mut()
            .set_arg_buf_named("aux_ints_1", &aux.ints_1).unwrap();

        pyrs_map.get_mut(ptal_name).unwrap().dens_mut().syns_mut()
            .set_arg_buf_named("aux_ints_0", &aux.ints_0).unwrap();
        pyrs_map.get_mut(ptal_name).unwrap().dens_mut().syns_mut()
            .set_arg_buf_named("aux_ints_1", &aux.ints_1).unwrap();
        // mcols.set_arg_buf_named("aux_ints_1", &aux.ints_0).unwrap();
        // pyrs_map.get_mut(ptal_name).unwrap().kern_ltp()

        //     .set_arg_buf_named("aux_ints_1", Some(&aux.ints_1)).unwrap();
        // pyrs_map.get_mut(ptal_name).unwrap().kern_cycle()
        //     .set_arg_buf_named("aux_ints_1", Some(&aux.ints_1)).unwrap();

        /*=============================================================================
        =================== EXECUTION ORDERING & GRAPH POPULATION =====================
        =============================================================================*/

        // (1.) Axon Intake:
        axns.set_exe_order_intake(&mut exe_graph)?;

        // (2.) SSTs Cycle:
        for sst in ssts_map.values() {
            sst.set_exe_order_cycle(&mut exe_graph)?;
        }

        // (3.) IINNs Cycle:
        for iinn in iinns.values() {
            iinn.set_exe_order(&mut exe_graph)?;
        }

        // (4.) SSTs Learn:
        for sst in ssts_map.values() {
            sst.set_exe_order_learn(&mut exe_graph)?;
        }

        // (5.) MCOLSs Activate:
        mcols.as_mut().set_exe_order_activate(&mut exe_graph)?;

        // (6.) PYRs Learn & Cycle:
        for pyr in pyrs_map.values() {
            pyr.set_exe_order(&mut exe_graph)?;
        }

        // (7.) MCOLs Output:
        mcols.as_mut().set_exe_order_output(&mut exe_graph)?;

        // (9.) Axon Output:
        axns.set_exe_order_output(&mut exe_graph)?;

        exe_graph.populate_requisites();

        /*=============================================================================
        ===============================================================================
        =============================================================================*/

        let cortical_area = CorticalArea {
            area_id: area_id,
            name: area_name,
            dims: dims,
            area_map: area_map,
            ptal_name: ptal_name,
            psal_name: psal_name,
            axns: axns,
            mcols: mcols,
            pyrs_map: pyrs_map,
            ssts_map: ssts_map,
            iinns: iinns,
            aux: aux,
            ocl_pq: ocl_pq,
            write_queue: write_queue,
            read_queue: read_queue,
            counter: 0,
            settings: settings,
            exe_graph: exe_graph,
        };

        Ok(cortical_area)
    }

    /// Cycles the area: running kernels, intaking, and outputting.
    ///
    //
    // [TODO]: ISOLATE LEARNING INTO SEPARATE THREAD
    pub fn cycle(&mut self, thal: &mut Thalamus) -> CmnResult<()> {
        //////
        ////// [REMOVE ME]: TEMPORARY:
            // self.exe_graph._RESET();
        //////
        //////


        // (1.) Axon Intake:
        self.axns.intake(thal, &mut self.exe_graph, self.settings.bypass_filters)?;

        // (2.) SSTs Cycle:
        if !self.settings.disable_ssts {
            // let aff_input_events = { self.axns.io_info().group_events(AxonDomainRoute::Input)
            //     .map(|wl| wl as &ClWaitList) };
            // self.psal().cycle(aff_input_events);
            // self.psal().cycle(&mut self.exe_graph);
            self.ssts_map[self.psal_name].cycle(&mut self.exe_graph)?;
        }

        // (3.) IINNs Cycle:
        self.iinns.get_mut("iv_inhib")
            .ok_or(CmnError::new("cortical_area::CorticalArea::cycle(): Invalid layer."))?
            .cycle(&mut self.exe_graph, self.settings.bypass_inhib)?;

        // (4.) SSTs Learn:
        if !self.settings.disable_ssts && !self.settings.disable_learning {
            self.ssts_map.get_mut(self.psal_name).ok_or("CorticalArea::cycle: PSAL (ssts) not found.")?
                .learn(&mut self.exe_graph)?;
        }

        // (5.) MCOLSs Activate:
        if !self.settings.disable_mcols { self.mcols.activate(&mut self.exe_graph)?; }

        // (6.) PYRs Learn & Cycle:
        if !self.settings.disable_pyrs {
            if !self.settings.disable_learning {
                self.pyrs_map.get_mut(self.ptal_name).ok_or("CorticalArea::cycle: PTAL (pyrs) not found.")?
                    .learn(&mut self.exe_graph)?;
            }
            // let eff_input_events = { self.axns.io_info().group_events(AxonDomainRoute::Input)
            //     .map(|wl| wl as &ClWaitList) };
            // self.ptal().cycle(eff_input_events);
            // self.ptal().cycle(&mut self.exe_graph);
            self.pyrs_map[self.ptal_name].cycle(&mut self.exe_graph)?;
        }

        // (7.) MCOLs Output:
        if !self.settings.disable_mcols {
            // let output_events = { self.axns.io_info_mut().group_events_mut(AxonDomainRoute::Output) };
            // self.mcols.output(output_events);
            self.mcols.output(&mut self.exe_graph)?;
        }

        // (8.) Regrow:
        if !self.settings.disable_regrowth { self.regrow(); }

        // (9.) Axon Output:
        self.axns.output(&self.read_queue, thal, &mut self.exe_graph)?;

        // // Finish queues [SEMI-TEMPORARY]:
        // self.finish_queues();

        Ok(())
    }

    /// Attaches synapses which are below strength threshold to new axons.
    pub fn regrow(&mut self) {
        self.finish_queues();

        if !self.settings.disable_regrowth {
            if self.counter >= cmn::SYNAPSE_REGROWTH_INTERVAL {
                //print!("$");
                self.ssts_map.get_mut(self.psal_name).expect("CorticalArea::regrow").regrow();
                self.pyrs_map.get_mut(self.ptal_name).expect("CorticalArea::regrow").regrow();
                self.counter = 0;
            } else {
                self.counter += 1;
            }
        }
    }

    pub fn finish_queues(&self) {
        self.write_queue.finish();
        self.ocl_pq.queue().finish();
        self.read_queue.finish();
    }

    // /* PIL(): Get Primary Spatial Associative Layer (immutable) */
    // pub fn psal(&self) -> &Box<SpinyStellateLayer> {
    //     let e_string = "cortical_area::CorticalArea::psal(): Primary Spatial Associative Layer: '{}' not found. ";
    //     self.ssts_map.get(self.psal_name).expect(e_string)
    // }

    // /* PIL_MUT(): Get Primary Spatial Associative Layer (mutable) */
    // pub fn psal_mut(&mut self) -> &mut Box<SpinyStellateLayer> {
    //     let e_string = "cortical_area::CorticalArea::psal_mut(): Primary Spatial Associative Layer: '{}' not found. ";
    //     self.ssts_map.get_mut(self.psal_name).expect(e_string)
    // }

    // /* PAL(): Get Primary Temporal Associative Layer (immutable) */
    // pub fn ptal(&self) -> &Box<PyramidalLayer> {
    //     let e_string = "cortical_area::CorticalArea::ptal(): Primary Temporal Associative Layer: '{}' not found. ";
    //     self.pyrs_map.get(self.ptal_name).expect(e_string)
    // }

    // /* PAL_MUT(): Get Primary Temporal Associative Layer (mutable) */
    // pub fn ptal_mut(&mut self) -> &mut Box<PyramidalLayer> {
    //     let e_string = "cortical_area::CorticalArea::ptal_mut(): Primary Temporal Associative Layer: '{}' not found. ";
    //     self.pyrs_map.get_mut(self.ptal_name).expect(e_string)
    // }

    /// [FIXME]: Currnently assuming aff out slice is == 1. Ascertain the
    /// slice range correctly by consulting area_map.layers().
    pub fn sample_aff_out(&self, buf: &mut [u8]) {
        let aff_out_slc = self.mcols.axn_slc_id();
        self.sample_axn_slc_range(aff_out_slc..(aff_out_slc + 1), buf);
    }

    pub fn sample_axn_slc_range<R: Borrow<Range<u8>>>(&self, slc_range: R, buf: &mut [u8])
                -> Event {
        let slc_range = slc_range.borrow();
        assert!(slc_range.len() > 0, "CorticalArea::sample_axn_slc_range(): \
            Invalid slice range: '{:?}'. Slice range length must be at least one.", slc_range);
        let axn_range_start = self.area_map.slices().axn_range(slc_range.start).start;
        let axn_range_end = self.area_map.slices().axn_range(slc_range.end - 1).end;
        let axn_range = axn_range_start..axn_range_end;

        debug_assert!(buf.len() == axn_range.len(), "Sample buffer length ({}) not \
            equal to slice axon length({}). axn_range: {:?}, slc_range: {:?}",
            buf.len(), axn_range.len(), axn_range, slc_range);
        let mut event = Event::empty();

        self.finish_queues();

        self.axns.states().cmd().read(buf).offset(axn_range.start).enew(&mut event).enq().unwrap();
        event
    }

    pub fn sample_axn_space(&self, buf: &mut [u8]) -> Event {
        debug_assert!(buf.len() == self.area_map.slices().axn_count() as usize);
        let mut event = Event::empty();

        self.finish_queues();

        self.axns.states().read(buf).enew(&mut event).enq().expect("[FIXME]: HANDLE ME!");
        event
    }

    #[inline] pub fn mcols(&self) -> &Box<Minicolumns> { &self.mcols }
    #[inline] pub fn mcols_mut(&mut self) -> &mut Box<Minicolumns> { &mut self.mcols }
    #[inline] pub fn axns(&self) -> &AxonSpace { &self.axns }
    #[inline] pub fn dims(&self) -> &CorticalDims { &self.dims }
    #[inline] pub fn psal_name(&self) -> &'static str { self.psal_name }
    #[inline] pub fn ptal_name(&self) -> &'static str { self.ptal_name }
    #[inline] pub fn afferent_target_names(&self) -> &Vec<&'static str> { &self.area_map.aff_areas() }
    #[inline] pub fn efferent_target_names(&self) -> &Vec<&'static str> { &self.area_map.eff_areas() }
    #[inline] pub fn ocl_pq(&self) -> &ProQue { &self.ocl_pq }
    #[inline] pub fn device(&self) -> &Device { &self.ocl_pq.queue().device() }
    #[inline] pub fn axn_tract_map(&self) -> SliceTractMap { self.area_map.slices().tract_map() }
    #[inline] pub fn area_map(&self) -> &AreaMap { &self.area_map }
    #[inline] pub fn area_id(&self) -> usize { self.area_id }
    #[inline] pub fn aux(&self) -> &Aux { &self.aux }
    #[inline] pub fn exe_graph_mut(&mut self) -> &mut ExecutionGraph { &mut self.exe_graph }
}

impl Drop for CorticalArea {
    fn drop(&mut self) {
        print!("Releasing OpenCL components for '{}'... ", self.name);
        // NOW DONE AUTOMATICALLY:
        // self.ocl_pq.release();
        print!("[ Buffers ][ Event Lists ][ Program ][ Command Queue ]");
        print!(" ...complete. \n");
    }
}


const INT_32_MIN: i32 = -2147483648;

pub struct Aux {
    pub ints_0: Buffer<i32>,
    pub ints_1: Buffer<i32>,
    // pub chars_0: Buffer<ocl::i8>,
    // pub chars_1: Buffer<ocl::i8>,
}

impl Aux {
    pub fn new(ptal_syn_len: usize, ocl_pq: &ProQue) -> Aux {
        let int_32_min = INT_32_MIN;

        let ints_0 = Buffer::<i32>::new(ocl_pq.queue().clone(), None, [ptal_syn_len * 4], None).unwrap();
        ints_0.cmd().fill(int_32_min, None).enq().unwrap();
        let ints_1 = Buffer::<i32>::new(ocl_pq.queue().clone(), None, [ptal_syn_len * 4], None).unwrap();
        ints_1.cmd().fill(int_32_min, None).enq().unwrap();

        ocl_pq.queue().finish();

        Aux {
            ints_0: ints_0,
            ints_1: ints_1,
            // chars_0: Buffer::<ocl::i8>::new(dims, 0, ocl),
            // chars_1: Buffer::<ocl::i8>::new(dims, 0, ocl),
        }
    }

    // pub unsafe fn resize(&mut self, new_dims: &CorticalDims, ocl_queue: &Queue) {
    //     let int_32_min = -INT_32_MIN;
    //     self.dims = new_dims.clone();

    //     self.ints_0.resize(&self.dims, ocl_queue);
    //     // self.ints_0.cmd().fill([int_32_min]).enq().unwrap();
    //     self.ints_0.cmd().fill(&[int_32_min], None).enq().unwrap();

    //     self.ints_1.resize(&self.dims, ocl_queue);
    //     // self.ints_1.cmd().fill([int_32_min]).enq().unwrap();
    //     self.ints_1.cmd().fill(&[int_32_min], None).enq().unwrap();
    //     // self.chars_0.resize(&self.dims, 0);
    //     // self.chars_1.resize(&self.dims, 0);
    // }
}

//////////////////////


#[cfg(test)]
pub mod tests {
    use rand;
    use rand::distributions::{IndependentSample, Range as RandRange};

    use super::*;
    use cortex::{AxonSpaceTest};
    use cmn::{CelCoords};
    use map::{AreaMapTest};

    pub trait CorticalAreaTest {
        fn axn_state(&self, idx: usize) -> u8;
        fn write_to_axon(&mut self, val: u8, idx: u32);
        fn read_from_axon(&self, idx: u32) -> u8;
        fn rand_safe_src_axn(&mut self, cel_coords: &CelCoords, src_axn_slc: u8
            ) -> (i8, i8, u32, u32);
        fn psal(&self) -> &Box<SpinyStellateLayer>;
        fn psal_mut(&mut self) -> &mut Box<SpinyStellateLayer>;
        fn ptal(&self) -> &Box<PyramidalLayer>;
        fn ptal_mut(&mut self) -> &mut Box<PyramidalLayer>;
        fn print_aux(&mut self);
        fn print_axns(&mut self);
        fn activate_axon(&mut self, idx: u32);
        fn deactivate_axon(&mut self, idx: u32);
    }

    impl CorticalAreaTest for CorticalArea {
        fn axn_state(&self, idx: usize) -> u8 {
            self.finish_queues();
            self.axns.axn_state(idx)
        }

        fn read_from_axon(&self, idx: u32) -> u8 {
            self.finish_queues();
            self.axns.axn_state(idx as usize)
        }

        fn write_to_axon(&mut self, val: u8, idx: u32) {
            self.finish_queues();
            self.axns.write_to_axon(val, idx);
        }

        fn rand_safe_src_axn(&mut self, cel_coords: &CelCoords, src_axn_slc: u8) -> (i8, i8, u32, u32) {
            let v_ofs_range = RandRange::new(-8i8, 9);
            let u_ofs_range = RandRange::new(-8i8, 9);

            let mut rng = rand::weak_rng();

            for _ in 0..50 {
                let v_ofs = v_ofs_range.ind_sample(&mut rng);
                let u_ofs = u_ofs_range.ind_sample(&mut rng);

                if v_ofs | u_ofs == 0 {
                    continue;
                }

                let idx_rslt = self.area_map.axn_idx(src_axn_slc, cel_coords.v_id,
                    v_ofs, cel_coords.u_id, u_ofs);

                match idx_rslt {
                    Ok(idx) => {
                        let col_id = self.area_map.axn_col_id(src_axn_slc, cel_coords.v_id,
                            v_ofs, cel_coords.u_id, u_ofs).unwrap();
                        return (v_ofs, u_ofs, col_id, idx)
                    },

                    Err(_) => (),
                }
            }

            panic!("SynCoords::rand_safe_src_axn_offs(): Error finding valid offset pair.");
        }

        /* PIL(): Get Primary Spatial Associative Layer (immutable) */
        fn psal(&self) -> &Box<SpinyStellateLayer> {
            let e_string = "cortical_area::CorticalArea::psal(): Primary Spatial Associative Layer: '{}' not found. ";
            self.ssts_map.get(self.psal_name).expect(e_string)
        }

        /* PIL_MUT(): Get Primary Spatial Associative Layer (mutable) */
        fn psal_mut(&mut self) -> &mut Box<SpinyStellateLayer> {
            let e_string = "cortical_area::CorticalArea::psal_mut(): Primary Spatial Associative Layer: '{}' not found. ";
            self.ssts_map.get_mut(self.psal_name).expect(e_string)
        }

        /* PAL(): Get Primary Temporal Associative Layer (immutable) */
        fn ptal(&self) -> &Box<PyramidalLayer> {
            let e_string = "cortical_area::CorticalArea::ptal(): Primary Temporal Associative Layer: '{}' not found. ";
            self.pyrs_map.get(self.ptal_name).expect(e_string)
        }

        /* PAL_MUT(): Get Primary Temporal Associative Layer (mutable) */
        fn ptal_mut(&mut self) -> &mut Box<PyramidalLayer> {
            let e_string = "cortical_area::CorticalArea::ptal_mut(): Primary Temporal Associative Layer: '{}' not found. ";
            self.pyrs_map.get_mut(self.ptal_name).expect(e_string)
        }

        fn print_aux(&mut self) {
            use ocl::util;

            self.finish_queues();

            let mut vec = vec![0; self.aux.ints_0.len()];

            print!("aux.ints_0: ");
            let view_radius = 1 << 24;
            self.aux.ints_0.read(&mut vec).enq().unwrap();
            util::print_slice(&vec, 1 << 0, Some((0 - view_radius, view_radius)), None, false);

            print!("aux.ints_1: ");
            self.aux.ints_1.read(&mut vec).enq().unwrap();
            util::print_slice(&vec, 1 << 0, Some((0 - view_radius, view_radius)), None, false);
        }

        fn print_axns(&mut self) {
            use ocl::util;

            self.finish_queues();

            let mut vec = vec![0; self.axns.states().len()];

            print!("axns: ");
            self.axns.states().read(&mut vec).enq().unwrap();
            util::print_slice(&vec, 1 << 0, None, None, false);
        }

        fn activate_axon(&mut self, idx: u32) {
            self.finish_queues();
            let mut rng = rand::weak_rng();
            let val = RandRange::new(200, 255).ind_sample(&mut rng);
            self.axns.write_to_axon(val, idx);
        }

        fn deactivate_axon(&mut self, idx: u32) {
            self.finish_queues();
            self.axns.write_to_axon(0, idx);
        }
    }
}

