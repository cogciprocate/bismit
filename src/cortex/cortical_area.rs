#![allow(dead_code, unused_mut, unused_imports)]

use std::thread::{self, JoinHandle};
use std::collections::{HashMap, BTreeMap};
use std::ops::Range;
use std::borrow::Borrow;
use futures::{Sink, Stream, Future};
use futures::future::BoxFuture;
use futures::sync::mpsc::{self, Sender};
use tokio_core::reactor::{Core, Remote};
use ocl::{async, flags, Device, ProQue, Context, Buffer, Event, Queue};
use ocl::core::CommandQueueProperties;
use cmn::{self, CmnError, CmnResult, CorticalDims};
use map::{self, AreaMap, SliceTractMap, LayerKind, DataCellKind, ControlCellKind,
    ExecutionGraph, CellClass /*AxonDomainRoute,*/};
use ::Thalamus;
use cortex::{AxonSpace, Minicolumns, InhibitoryInterneuronNetwork, PyramidalLayer,
    SpinyStellateLayer, DataCellLayer, ControlCellLayer, ActivitySmoother};

#[cfg(test)] pub use self::tests::{CorticalAreaTest};

// Create separate read and write queues in addition to the kernel queue:
const SEPARATE_IO_QUEUES: bool = true;
// Enable out of order asynchronous command queues:
const QUEUE_OUT_OF_ORDER: bool = true;
// Enable queue profiling:
const QUEUE_PROFILING: bool = false;
// GDB debug mode:
const KERNEL_DEBUG_SYMBOLS: bool = true;


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
    // iinns: HashMap<&'static str, Box<InhibitoryInterneuronNetwork>>,
    ptal_name: Option<&'static str>,    // PRIMARY TEMPORAL ASSOCIATIVE LAYER NAME
    psal_name: Option<&'static str>,    // PRIMARY SPATIAL ASSOCIATIVE LAYER NAME
    psal_idx: usize,
    ptal_idx: usize,
    spatial_layers: Vec<SpinyStellateLayer>,
    temporal_layers: Vec<PyramidalLayer>,
    focus_layers: Vec<PyramidalLayer>,
    motor_layers: Vec<PyramidalLayer>,
    other_layers: Vec<Box<DataCellLayer>>,
    // control_layers: Vec<Box<ControlCellLayer>>,
    control_layers: BTreeMap<usize, Box<ControlCellLayer>>,
    aux: Aux,
    ocl_pq: ProQue,
    write_queue: Queue,
    read_queue: Queue,
    counter: usize,
    settings: CorticalAreaSettings,
    exe_graph: ExecutionGraph,
    work_tx: Option<Sender<BoxFuture<(), ()>>>,
    _work_thread: Option<JoinHandle<()>>,
}

impl CorticalArea {
    /// Creates a new cortical area.
    ///
    //
    // * TODO: Break this function up a bit. Probably break the major sections
    // out into new types.
    //
    // The only use for `thal` is currently within `axon_space` for the
    // purpose of getting precise slice ids for layers in other areas for use
    // by the execution graph system.
    //
    pub fn new(area_map: AreaMap, device_idx: usize, ocl_context: &Context,
            settings: Option<CorticalAreaSettings>, thal: &mut Thalamus) -> CmnResult<CorticalArea>
    {
        let emsg = "cortical_area::CorticalArea::new()";
        let area_id = area_map.area_id();
        let area_name = area_map.area_name();

        println!("\n\nCORTICALAREA::NEW(): Creating Cortical Area: \"{}\"...", area_name);

        // Optionally pass `-g` and `-s {cl path}` flags to compiler:
        let build_options = if KERNEL_DEBUG_SYMBOLS && cfg!(target_os = "linux") {
            // * TODO: Add something to identify the platform vendor and match:
            // let kernel_path = concat!(env!("CARGO_MANIFEST_DIR"), "/cl/bismit.cl");
            // let debug_opts = format!("-g -s \"{}\"", kernel_path);

            if ocl_context.platform()?.unwrap().vendor().contains("Intel") {
                panic!("[cortical_area::KERNEL_DEBUG_SYMBOLS == true]: \
                    Cannot debug kernels on an Intel based driver platform (not sure why).
                    Use the AMD platform drivers with Intel devices instead.");
            }

            let debug_opts = "-g";
            area_map.gen_build_options().cmplr_opt(debug_opts)
        } else {
            area_map.gen_build_options()
        };

        let mut queue_flags = if QUEUE_OUT_OF_ORDER {
            flags::QUEUE_OUT_OF_ORDER_EXEC_MODE_ENABLE
        } else {
            CommandQueueProperties::empty()
        };

        queue_flags = queue_flags | if QUEUE_PROFILING {
            flags::QUEUE_PROFILING_ENABLE
        } else {
            CommandQueueProperties::empty()
        };

        let ocl_pq = ProQue::builder()
            .device(device_idx)
            // .device(1)
            .context(ocl_context.clone())
            .prog_bldr(build_options)
            .queue_properties(queue_flags)
            .build().expect("CorticalArea::new(): ocl_pq.build(): error");

        let (write_queue, read_queue) = if SEPARATE_IO_QUEUES {
            (Queue::new(ocl_context, ocl_pq.device().clone(), Some(queue_flags))?,
                Queue::new(ocl_context, ocl_pq.device().clone(), Some(queue_flags))?)
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

        let psal_name = area_map.layer_map().layers_containing_tags(map::PSAL)
            .first().map(|lyr| lyr.name());
        let ptal_name = area_map.layer_map().layers_containing_tags(map::PTAL)
            .first().map(|lyr| lyr.name());

        let mut psal_idx = usize::max_value();
        let mut ptal_idx = usize::max_value();

        let settings = settings.unwrap_or(CorticalAreaSettings::new());

        /*=============================================================================
        =============================== EXECUTION GRAPH ===============================
        =============================================================================*/

        let mut exe_graph = ExecutionGraph::new();

        /*=============================================================================
        ================================ CELLS & AXONS ================================
        =============================================================================*/

        // let mut iinns = HashMap::new();
        let mut mcols = None;
        let mut spatial_layers = Vec::with_capacity(4);
        let mut temporal_layers = Vec::with_capacity(4);
        let mut focus_layers: Vec<PyramidalLayer> = Vec::with_capacity(4);
        let mut motor_layers: Vec<PyramidalLayer> = Vec::with_capacity(4);
        let mut other_layers: Vec<Box<DataCellLayer>> = Vec::with_capacity(4);
        // let mut control_layers: Vec<Box<ControlCellLayer>> = Vec::with_capacity(4);
        let mut control_layers: BTreeMap<usize, Box<ControlCellLayer>> = BTreeMap::new();
        let axns = AxonSpace::new(&area_map, &ocl_pq, read_queue.clone(),
            write_queue.clone(), &mut exe_graph, thal)?;

        /*=============================================================================
        ================================== DATA CELLS =================================
        =============================================================================*/
        // * TODO: BREAK OFF THIS CODE INTO NEW STRUCT DEF

        for layer in area_map.layer_map().iter() {
            // match layer.kind() {
            if let LayerKind::Cellular(ref cell_scheme) = *layer.kind() {
                // &LayerKind::Cellular(ref cell_scheme) => {
                    println!("{mt}::NEW(): making a(n) {:?} layer: '{}' (depth: {})",
                        cell_scheme.data_cell_kind(), layer.name(), layer.depth(), mt = cmn::MT);

                    match cell_scheme.data_cell_kind() {
                        Some(&DataCellKind::Pyramidal) => {
                            let pyrs_dims = dims.clone_with_depth(layer.depth());

                            let pyr_lyr = try!(PyramidalLayer::new(layer.name(), layer.layer_id(),
                                pyrs_dims, cell_scheme.clone(), &area_map, &axns, &ocl_pq,
                                settings.clone(), &mut exe_graph));

                            if pyr_lyr.layer_tags().contains(map::SPATIAL) {
                                // spatial_layers.push(pyr_lyr);
                                panic!("Spatial pyramidal cells not yet supported.");
                            } else if pyr_lyr.layer_tags().contains(map::TEMPORAL) {
                                temporal_layers.push(pyr_lyr);
                            } else if pyr_lyr.layer_tags().contains(map::FOCUS) {
                                focus_layers.push(pyr_lyr)
                            } else if pyr_lyr.layer_tags().contains(map::MOTOR) {
                                motor_layers.push(pyr_lyr)
                            } else {
                                other_layers.push(Box::new(pyr_lyr))
                            }
                        },
                        Some(&DataCellKind::SpinyStellate) => {
                            let ssts_map_dims = dims.clone_with_depth(layer.depth());

                            let sst_lyr = try!(SpinyStellateLayer::new(layer.name(), layer.layer_id(),
                                ssts_map_dims, cell_scheme.clone(), &area_map, &axns, &ocl_pq,
                                settings.clone(), &mut exe_graph));

                            if sst_lyr.layer_tags().contains(map::SPATIAL) {
                                spatial_layers.push(sst_lyr);
                            } else {
                                other_layers.push(Box::new(sst_lyr))
                            }
                        },
                        _ => (),
                    }
                // },
                // _ => (),
            }
        }


        /*=============================================================================
        ================================ CONTROL CELLS ================================
        =============================================================================*/
        // * TODO: BREAK OFF THIS CODE INTO NEW STRUCT DEF

        for layer in area_map.layer_map().iter() {
            if let LayerKind::Cellular(ref cell_scheme) = *layer.kind() {
                // if let Some(&ControlCellKind::InhibitoryBasketSurround { ref host_lyr_name,
                //         field_radius: _ }) = cell_scheme.control_cell_kind()
                // {
                match *cell_scheme.cell_class() {
                        CellClass::Control {
                                kind: ControlCellKind::InhibitoryBasketSurround {
                                    ref host_lyr_name, field_radius },
                                exe_order, } =>
                        {
                            if field_radius != 99 { panic!("field_radius not yet implemented (use 99).")};
                    // Some(&ControlCellKind::InhibitoryBasketSurround { ref host_lyr_name, field_radius: _ }) => {
                        // let em1 = format!("{}: '{}' is not a valid layer", emsg, host_lyr_name);
                        let host_lyr = spatial_layers.iter().find(|lyr|
                                lyr.layer_name() == psal_name.unwrap())
                            .expect(&format!("{}: '{}' is not a valid layer", emsg, host_lyr_name));
                        // let src_soma_buf = src_soma.soma();

                        let host_lyr_slc_ids = area_map.layer_slc_ids(&[host_lyr_name.clone()]);
                        // let host_lyr_depth = host_lyr_slc_ids.len() as u8;
                        let host_lyr_base_axn_slc = host_lyr_slc_ids[0];

                        // let ccs_dims = dims.clone_with_depth(host_lyr_depth);
                        let cc_lyr = InhibitoryInterneuronNetwork::new(layer.name(),
                            layer.layer_id(), /*ccs_dims,*/ cell_scheme.clone(),
                            host_lyr, host_lyr_base_axn_slc, &axns, &area_map, &ocl_pq,
                            settings.clone(), &mut exe_graph)?;

                        // ccs.insert(layer.name(), Box::new(cc_lyr));
                        // control_layers.push(Box::new(cc_lyr));
                        if control_layers.insert(exe_order, Box::new(cc_lyr)).is_some() {
                            panic!("Duplicate control cell order index found for layer: {} ({})",
                                layer.name(), exe_order);
                        };
                    },
                    CellClass::Control {
                            kind: ControlCellKind::ActivitySmoother {
                                ref host_lyr_name, field_radius },
                                exe_order, } =>
                    {
                        if field_radius != 99 { panic!("field_radius not yet implemented (use 99).")};
                    // Some(&ControlCellKind::ActivitySmoother { ref host_lyr_name, field_radius: _ }) => {
                        // let em1 = format!("{}: '{}' is not a valid layer", emsg, host_lyr_name);
                        let host_lyr = spatial_layers.iter().find(|lyr|
                                lyr.layer_name() == psal_name.unwrap())
                            .expect(&format!("{}: '{}' is not a valid layer", emsg, host_lyr_name));
                        // let src_soma_buf = src_soma.soma();

                        let host_lyr_slc_ids = area_map.layer_slc_ids(&[host_lyr_name.clone()]);
                        // let host_lyr_depth = host_lyr_slc_ids.len() as u8;
                        let host_lyr_base_axn_slc = host_lyr_slc_ids[0];

                        // let ccs_dims = dims.clone_with_depth(host_lyr_depth);
                        let cc_lyr = ActivitySmoother::new(layer.name(),
                            layer.layer_id(), /*ccs_dims,*/ cell_scheme.clone(),
                            host_lyr, host_lyr_base_axn_slc, &axns, &area_map, &ocl_pq,
                            settings.clone(), &mut exe_graph)?;

                        // ccs.insert(layer.name(), Box::new(cc_lyr));
                        // control_layers.push(Box::new(cc_lyr));
                        if control_layers.insert(exe_order, Box::new(cc_lyr)).is_some() {
                            panic!("Duplicate control cell order index found for layer: {} ({})",
                                layer.name(), exe_order);
                        }
                    },
                    _ => (),
                }
            }
        }

        for layer in area_map.layer_map().iter() {
            match layer.kind() {
                &LayerKind::Cellular(ref cell_scheme) => {
                    println!("{mt}::NEW(): making a(n) {:?} layer: '{}' (depth: {})",
                        cell_scheme.control_cell_kind(), layer.name(), layer.depth(), mt = cmn::MT);

                    match cell_scheme.control_cell_kind() {
                        Some(&ControlCellKind::Complex) => {
                            let mcols_dims = dims.clone_with_depth(1);

                            mcols = Some(Box::new({
                                psal_idx = spatial_layers.iter().position(|lyr| lyr.layer_name() == psal_name.unwrap())
                                    .expect(&format!("{}: '{}' is not a valid layer", emsg, psal_name.unwrap()));
                                let ssts = &spatial_layers[psal_idx];

                                ptal_idx = temporal_layers.iter().position(|lyr| lyr.layer_name() == ptal_name.unwrap())
                                    .expect(&format!("{}: '{}' is not a valid layer", emsg, ptal_name.unwrap()));
                                let pyrs = &temporal_layers[ptal_idx];

                                let layer_id = layer.layer_id();

                                debug_assert!(area_map.aff_out_slcs().len() > 0, "CorticalArea::new(): \
                                    No afferent output slices found for area: '{}'", area_name);
                                Minicolumns::new(layer_id, mcols_dims, &area_map, &axns, ssts, pyrs,
                                    &ocl_pq, settings.clone(), &mut exe_graph)?
                            }));
                        },
                        _ => (),
                    }
                },
                _ => (),
            }
        }

        let mut mcols = mcols.expect("CorticalArea::new(): No Minicolumn layer found!");

        /*=============================================================================
        ===================================== AUX =====================================
        =============================================================================*/

        let aux = Aux::new(temporal_layers[ptal_idx].dens().syns().len(), &ocl_pq);

        // <<<<< TODO: CLEAN THIS UP >>>>>
        // MAKE ABOVE LIKE BELOW (eliminate set_arg_buf_named() methods and just call directly on buffer)
        // mcols.set_arg_buf_named("aux_ints_0", &aux.ints_0).unwrap();
        temporal_layers[ptal_idx]
            .set_arg_buf_named("aux_ints_0", &aux.ints_0).unwrap();
        temporal_layers[ptal_idx]
            .set_arg_buf_named("aux_ints_1", &aux.ints_1).unwrap();

        temporal_layers[ptal_idx].dens_mut()
            .set_arg_buf_named("aux_ints_0", &aux.ints_0).unwrap();
        temporal_layers[ptal_idx].dens_mut()
            .set_arg_buf_named("aux_ints_1", &aux.ints_1).unwrap();

        temporal_layers[ptal_idx].dens_mut().syns_mut()
            .set_arg_buf_named("aux_ints_0", &aux.ints_0).unwrap();
        temporal_layers[ptal_idx].dens_mut().syns_mut()
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
        for sst in &mut spatial_layers {
            // sst.set_ctrl_lyr_idxs(&control_layers);
            sst.set_exe_order_cycle(&control_layers, &mut exe_graph)?;
        }

        // (3.) IINNs Cycle:
        // for iinn in iinns.values() {
        //     iinn.set_exe_order(&mut exe_graph)?;
        // }

        // // (3.) IINNs Cycle:
        // for layer in &control_layers {
        //     layer.set_exe_order(&mut exe_graph)?;
        // }

        // (4.) SSTs Learn:
        if !settings.disable_learning {
            for sst in &spatial_layers {
                sst.set_exe_order_learn(&mut exe_graph)?;
            }
        }

        // (5.) MCOLSs Activate:
        if !settings.disable_mcols {
            mcols.as_mut().set_exe_order_activate(&mut exe_graph)?;
        }

        // (6.) Pyramidal Layers Learn & Cycle:
        if !settings.disable_pyrs {
            // (6a.) Temporal Layers Learn & Cycle:
            for layer in &temporal_layers {
                layer.set_exe_order_cycle(&control_layers, &mut exe_graph)?;
            }

            // (6b.) Focus Layers Learn & Cycle:
            for layer in &focus_layers {
                layer.set_exe_order_cycle(&control_layers, &mut exe_graph)?;
            }

            // (6c.) Motor Layers Learn & Cycle:
            for layer in &motor_layers {
                layer.set_exe_order_cycle(&control_layers, &mut exe_graph)?;
            }

            // (6d.) Other Layers Learn & Cycle:
            for _layer in &other_layers {
                // layer.set_exe_order_cycle(&control_layers, &mut exe_graph)?;
            }
        }

        // (7.) MCOLs Output:
        if !settings.disable_mcols {
            mcols.as_mut().set_exe_order_output(&mut exe_graph)?;
        }

        // (9.) Axon Output:
        axns.set_exe_order_output(&mut exe_graph)?;

        exe_graph.populate_requisites();

        /*=============================================================================
        =========================== WORK COMPLETION THREAD ============================
        =============================================================================*/

        let (tx, rx) = mpsc::channel(0);
        let thread_name = format!("CorticalArea_{}", area_name.clone());

        let thread: JoinHandle<_> = thread::Builder::new().name(thread_name).spawn(move || {
            let rx = rx;
            let mut core = Core::new().unwrap();
            let work = rx.buffer_unordered(3).for_each(|_| Ok(()));
            core.run(work).unwrap();
        }).unwrap();

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
            ptal_idx: ptal_idx,
            psal_idx: psal_idx,
            axns: axns,
            mcols: mcols,
            // iinns: iinns,
            spatial_layers,
            temporal_layers,
            motor_layers,
            focus_layers,
            other_layers,
            control_layers,
            aux: aux,
            ocl_pq: ocl_pq,
            write_queue: write_queue,
            read_queue: read_queue,
            counter: 0,
            settings: settings,
            exe_graph: exe_graph,
            work_tx: Some(tx),
            _work_thread: Some(thread),
        };

        Ok(cortical_area)
    }

    /// Cycles the area: running kernels, intaking, and outputting.
    ///
    //
    // * TODO: ISOLATE LEARNING INTO SEPARATE THREAD
    pub fn cycle(&mut self, thal: &mut Thalamus) -> CmnResult<()> {

        // (1.) Axon Intake:
        self.axns.intake(thal, &mut self.exe_graph, self.settings.bypass_filters,
            self.work_tx.as_ref().unwrap())?;

        // (2.) SSTs Cycle:
        if !self.settings.disable_ssts {
            for lyr in &mut self.spatial_layers { lyr.cycle(&mut self.control_layers,
                &mut self.exe_graph)? }
        }

        // // (3.) IINNs Cycle:
        // self.iinns.get_mut("iv_inhib")
        //     .ok_or(CmnError::new("cortical_area::CorticalArea::cycle(): Invalid layer."))?
        //     .cycle(&mut self.exe_graph, self.settings.bypass_inhib)?;

        // // (3.) IINNs Cycle:
        // self.control_layers.get_mut(0)
        //     .ok_or(CmnError::new("cortical_area::CorticalArea::cycle(): Invalid control layer."))?
        //     .cycle(&mut self.exe_graph, self.settings.bypass_inhib)?;

        // (4.) SSTs Learn:
        if !self.settings.disable_ssts && !self.settings.disable_learning {
            for lyr in &mut self.spatial_layers { lyr.learn(&mut self.exe_graph)? }
        }

        // (5.) MCOLSs Activate:
        if !self.settings.disable_mcols { self.mcols.activate(&mut self.exe_graph)?; }

        // (6.) Pyramidal Layers Learn & Cycle:
        if !self.settings.disable_pyrs {
            // (6a.) Temporal Layers Learn & Cycle:
            for lyr in &mut self.temporal_layers {
                if !self.settings.disable_learning { lyr.learn(&mut self.exe_graph)?; }
                lyr.cycle(&mut self.control_layers, &mut self.exe_graph)?;
            }

            // (6b.) Focus Layers Learn & Cycle:
            for lyr in &mut self.focus_layers {
                if !self.settings.disable_learning { lyr.learn(&mut self.exe_graph)?; }
                lyr.cycle(&mut self.control_layers, &mut self.exe_graph)?;
            }

            // (6c.) Motor Layers Learn & Cycle:
            for lyr in &mut self.motor_layers {
                if !self.settings.disable_learning { lyr.learn(&mut self.exe_graph)?; }
                lyr.cycle(&mut self.control_layers, &mut self.exe_graph)?;
            }

            // (6d.) Other Layers Learn & Cycle:
            for lyr in &mut self.other_layers {
                if !self.settings.disable_learning { lyr.learn(&mut self.exe_graph)?; }
                lyr.cycle(&mut self.control_layers, &mut self.exe_graph)?;
            }
        }

        // (7.) MCOLs Output:
        if !self.settings.disable_mcols {
            self.mcols.output(&mut self.exe_graph)?;
        }

        // (8.) Regrow:
        if !self.settings.disable_regrowth { self.regrow(); }

        // (9.) Axon Output:
        self.axns.output(thal, &mut self.exe_graph, self.work_tx.as_ref().unwrap())?;

        // // Finish queues [DEBUGGING]:
        // self.finish_queues();

        Ok(())
    }

    /// Attaches synapses which are below strength threshold to new axons.
    pub fn regrow(&mut self) {
        self.finish_queues();

        if !self.settings.disable_regrowth {
            if self.counter >= cmn::SYNAPSE_REGROWTH_INTERVAL {
                //print!("$");
                self.spatial_layers[self.psal_idx].regrow();
                self.temporal_layers[self.ptal_idx].regrow();
                self.counter = 0;
            } else {
                self.counter += 1;
            }
        }
    }

    pub fn finish_queues(&self) {
        self.write_queue.finish().unwrap();
        self.ocl_pq.queue().finish().unwrap();
        self.read_queue.finish().unwrap();
    }

    /// [FIXME]: Currnently assuming aff out slice is == 1. Ascertain the
    /// slice range correctly by consulting area_map.layer_map().
    pub fn sample_aff_out(&self, buf: &mut [u8]) -> Event {
        let aff_out_slc = self.mcols.axn_slc_id();
        self.sample_axn_slc_range(aff_out_slc..(aff_out_slc + 1), buf)
    }

    pub fn sample_axn_slc_range<R: Borrow<Range<u8>>>(&self, slc_range: R, buf: &mut [u8])
                -> Event {
        let slc_range = slc_range.borrow();
        assert!(slc_range.len() > 0, "CorticalArea::sample_axn_slc_range(): \
            Invalid slice range: '{:?}'. Slice range length must be at least one.", slc_range);
        let axn_range_start = self.area_map.slice_map().axn_range(slc_range.start).start;
        let axn_range_end = self.area_map.slice_map().axn_range(slc_range.end - 1).end;
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
        debug_assert!(buf.len() == self.area_map.slice_map().axn_count() as usize);
        let mut event = Event::empty();

        self.finish_queues();

        self.axns.states().read(buf).enew(&mut event).enq().expect("[FIXME]: HANDLE ME!");
        event
    }

    /// Get Primary Spatial Associative Layer (immutable)
    #[allow(non_snake_case)]
    pub fn psal_TEMP(&self) -> &SpinyStellateLayer {
        &self.spatial_layers[self.psal_idx]
    }

    #[inline] pub fn mcols(&self) -> &Box<Minicolumns> { &self.mcols }
    #[inline] pub fn mcols_mut(&mut self) -> &mut Box<Minicolumns> { &mut self.mcols }
    #[inline] pub fn axns(&self) -> &AxonSpace { &self.axns }
    #[inline] pub fn dims(&self) -> &CorticalDims { &self.dims }
    #[inline] pub fn psal_name(&self) -> Option<&'static str> { self.psal_name }
    #[inline] pub fn ptal_name(&self) -> Option<&'static str> { self.ptal_name }
    #[inline] pub fn afferent_target_names(&self) -> &Vec<&'static str> { &self.area_map.aff_areas() }
    #[inline] pub fn efferent_target_names(&self) -> &Vec<&'static str> { &self.area_map.eff_areas() }
    #[inline] pub fn ocl_pq(&self) -> &ProQue { &self.ocl_pq }
    #[inline] pub fn device(&self) -> Device { self.ocl_pq.queue().device() }
    #[inline] pub fn axn_tract_map(&self) -> SliceTractMap { self.area_map.slice_map().tract_map() }
    #[inline] pub fn area_map(&self) -> &AreaMap { &self.area_map }
    #[inline] pub fn area_id(&self) -> usize { self.area_id }
    #[inline] pub fn aux(&self) -> &Aux { &self.aux }
    #[inline] pub fn exe_graph_mut(&mut self) -> &mut ExecutionGraph { &mut self.exe_graph }
}


impl Drop for CorticalArea {
    fn drop(&mut self) {
        println!("Releasing work thread for '{}'... ", &self.name);
        self.work_tx.take();
        self._work_thread.take().unwrap().join().unwrap();
        print!("Releasing OpenCL components for '{}'... ", &self.name);
        print!("[ Buffers ][ Event Lists ][ Program ][ Command Queues ]");
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
        // let int_32_min = INT_32_MIN;
        let int_32_min = i32::min_value();

        let ints_0 = Buffer::<i32>::new(ocl_pq.queue().clone(), None, [ptal_syn_len * 4], None, Some((0, None::<()>))).unwrap();
        ints_0.cmd().fill(int_32_min, None).enq().unwrap();
        let ints_1 = Buffer::<i32>::new(ocl_pq.queue().clone(), None, [ptal_syn_len * 4], None, Some((0, None::<()>))).unwrap();
        ints_1.cmd().fill(int_32_min, None).enq().unwrap();

        ocl_pq.queue().finish().unwrap();

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
    use cortex::{AxonSpaceTest, CelCoords};
    use map::{AreaMapTest};

    pub trait CorticalAreaTest {
        fn axn_state(&self, idx: usize) -> u8;
        fn write_to_axon(&mut self, val: u8, idx: u32);
        fn read_from_axon(&self, idx: u32) -> u8;
        fn rand_safe_src_axn(&mut self, cel_coords: &CelCoords, src_axn_slc: u8
            ) -> (i8, i8, u32, u32);
        fn psal(&self) -> &SpinyStellateLayer;
        fn psal_mut(&mut self) -> &mut SpinyStellateLayer;
        fn ptal(&self) -> &PyramidalLayer;
        fn ptal_mut(&mut self) -> &mut PyramidalLayer;
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
        fn psal(&self) -> &SpinyStellateLayer {
            &self.spatial_layers[self.psal_idx]
        }

        /* PIL_MUT(): Get Primary Spatial Associative Layer (mutable) */
        fn psal_mut(&mut self) -> &mut SpinyStellateLayer {
            &mut self.spatial_layers[self.psal_idx]
        }

        /* PAL(): Get Primary Temporal Associative Layer (immutable) */
        fn ptal(&self) -> &PyramidalLayer {
            &self.temporal_layers[self.ptal_idx]
        }

        /* PAL_MUT(): Get Primary Temporal Associative Layer (mutable) */
        fn ptal_mut(&mut self) -> &mut PyramidalLayer {
            &mut self.temporal_layers[self.ptal_idx]
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

