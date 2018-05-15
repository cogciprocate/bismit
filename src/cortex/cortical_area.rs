// #![allow(dead_code, unused_mut, unused_imports)]

use std::collections::{HashSet, BTreeMap};
use std::ops::Range;
use futures::FutureExt;
use ocl::{flags, Device, ProQue, Context, Buffer, Event, Queue, RwVec};
use ocl::core::CommandQueueProperties;
use ocl::builders::{BuildOpt, ProgramBuilder};
use cmn::{self, CmnResult, CorticalDims};
use map::{AreaMap, SliceTractMap, LayerKind, DataCellKind, ControlCellKind,
    ExecutionGraph, CellClass, LayerTags, LayerAddress, CommandUid, CommandRelations, CorticalBuffer};
use ::Thalamus;
use cortex::{AxonSpace, InhibitoryInterneuronNetwork, PyramidalLayer,
    SpinyStellateLayer, DataCellLayer, ControlCellLayer, ActivitySmoother, PyrOutputter,
    CompletionPool, ControlCellLayers, IntraColumnInhib};
use subcortex::{self, TractSender, TractReceiver};

#[cfg(any(test, feature = "eval"))]
pub use self::tests::{CorticalAreaTest};

// Create separate read and write queues in addition to the kernel queue:
const SEPARATE_IO_QUEUES: bool = true;
// Enable out of order asynchronous command queues:
const QUEUE_OUT_OF_ORDER: bool = true;
// Enable queue profiling:
const QUEUE_PROFILING: bool = false;
// GDB debug mode:
const KERNEL_DEBUG_SYMBOLS: bool = false;
// Layer role execution order:
static ROLE_ORDER: [LayerTags; 4] = [LayerTags::FOCUS, LayerTags::SPATIAL, LayerTags::TEMPORAL, LayerTags::MOTOR];


// pub type ControlCellLayers = BTreeMap<(LayerAddress, usize), Box<ControlCellLayer>>;


// macro_rules! rx {
//     ( $buf:ident ) => {
//         {
//             let (len, cmd_srcs) = {
//                 let lyr = lyr(&self.data_layers, lyr_addr);
//                 (lyr.$buf.len(),
//                     vec![CorticalBuffer::data_soma_lyr(lyr.$buf, lyr_addr)])
//             };
//             self.sampler_rx_single_u8(len, cmd_srcs, kind.clone(), None, backpressure)
//         }
//     };
// }


#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SamplerBufferKind {
    None,
    Map,
    Single,
    Double,
    Triple,
}


#[derive(Debug, Clone, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub enum SamplerKind {
    None,
    /// Axons for a specific layer or all layers.
    Axons(Option<LayerAddress>),
    SomaStates(LayerAddress),
    SomaEnergies(LayerAddress),
    SomaActivities(LayerAddress),
    SomaFlagSets(LayerAddress),
    TuftStates(LayerAddress),
    TuftBestDenIds(LayerAddress),
    TuftBestDenStatesRaw(LayerAddress),
    TuftBestDenStates(LayerAddress),
    TuftPrevStates(LayerAddress),
    TuftPrevBestDenIds(LayerAddress),
    TuftPrevBestDenStatesRaw(LayerAddress),
    TuftPrevBestDenStates(LayerAddress),
    DenStates(LayerAddress),
    DenStatesRaw(LayerAddress),
    DenEnergies(LayerAddress),
    DenActivities(LayerAddress),
    DenThresholds(LayerAddress),
    SynStates(LayerAddress),
    SynStrengths(LayerAddress),
    SynSrcColVOffs(LayerAddress),
    SynSrcColUOffs(LayerAddress),
    SynFlagSets(LayerAddress),

}


#[derive(Debug)]
struct Sampler {
    kind: SamplerKind,
    src_idx_range: Range<usize>,
    tx: TractSender,
    cmd_uid: CommandUid,
    cmd_idx: Option<usize>,
}

impl Sampler {
    fn new(kind: SamplerKind, src_idx_range: Option<Range<usize>>,
            tx: TractSender, cmd_uid: CommandUid) -> Sampler {
        let src_idx_range = src_idx_range.unwrap_or(tx.buffer_idx_range());
        Sampler { kind, src_idx_range, tx, cmd_uid, cmd_idx: None }
    }

    fn set_exe_order(&mut self, exe_graph: &mut ExecutionGraph) -> CmnResult<()> {
        self.cmd_idx = Some(exe_graph.order_command(self.cmd_uid)?);
        Ok(())
    }
}


#[derive(Debug)]
enum Layer {
    SpinyStellateLayer(SpinyStellateLayer),
    PyramidalLayer(PyramidalLayer),
}

impl Layer {
    fn pyr(lyr: PyramidalLayer) -> Layer {
        Layer::PyramidalLayer(lyr)
    }

    fn ssc(lyr: SpinyStellateLayer) -> Layer {
        Layer::SpinyStellateLayer(lyr)
    }

    fn tags(&self) -> LayerTags {
        match *self {
            Layer::SpinyStellateLayer(ref lyr) => lyr.layer_tags(),
            Layer::PyramidalLayer(ref lyr) => lyr.layer_tags(),
        }
    }

    fn layer_addr(&self) -> LayerAddress {
        match *self {
            Layer::SpinyStellateLayer(ref lyr) => lyr.layer_addr(),
            Layer::PyramidalLayer(ref lyr) => lyr.layer_addr(),
        }
    }

    fn layer_name(&self) -> &str {
        match *self {
            Layer::SpinyStellateLayer(ref lyr) => lyr.layer_name(),
            Layer::PyramidalLayer(ref lyr) => lyr.layer_name(),
        }
    }

    fn set_exe_order_cycle(&mut self, control_layers: &mut ControlCellLayers,
            exe_graph: &mut ExecutionGraph) -> CmnResult<()>
    {
        match *self {
            Layer::SpinyStellateLayer(ref mut lyr) => lyr.set_exe_order_cycle(control_layers, exe_graph),
            Layer::PyramidalLayer(ref mut lyr) => lyr.set_exe_order_cycle(control_layers, exe_graph),
        }
    }

    fn set_exe_order_learn(&mut self, exe_graph: &mut ExecutionGraph) -> CmnResult<()> {
        match *self {
            Layer::SpinyStellateLayer(ref mut lyr) => lyr.set_exe_order_learn(exe_graph),
            Layer::PyramidalLayer(ref mut lyr) => lyr.set_exe_order_learn(exe_graph),
        }
    }

    fn cycle(&mut self, control_layers: &mut ControlCellLayers,
        exe_graph: &mut ExecutionGraph) -> CmnResult<()>
    {
        match *self {
            Layer::SpinyStellateLayer(ref mut lyr) => lyr.cycle(control_layers, exe_graph),
            Layer::PyramidalLayer(ref mut lyr) => lyr.cycle(control_layers, exe_graph),
        }
    }

    fn learn(&mut self, exe_graph: &mut ExecutionGraph) -> CmnResult<()> {
        match *self {
            Layer::SpinyStellateLayer(ref mut lyr) => lyr.learn(exe_graph),
            Layer::PyramidalLayer(ref mut lyr) => lyr.learn(exe_graph),
        }
    }

    fn regrow(&mut self) {
        match *self {
            Layer::SpinyStellateLayer(ref mut lyr) => lyr.regrow(),
            Layer::PyramidalLayer(ref mut lyr) => lyr.regrow(),
        }
    }

    fn as_data_cell_layer(&self) -> CmnResult<&DataCellLayer> {
        match *self {
            Layer::SpinyStellateLayer(ref lyr) => Ok(lyr),
            Layer::PyramidalLayer(ref lyr) => Ok(lyr),
        }
    }

    fn as_data_cell_layer_mut(&mut self) -> CmnResult<&mut DataCellLayer> {
        match *self {
            Layer::SpinyStellateLayer(ref mut lyr) => Ok(lyr),
            Layer::PyramidalLayer(ref mut lyr) => Ok(lyr),
        }
    }
}


#[derive(Debug)]
struct Layers {
    lyrs: Vec<Layer>,
}

impl Layers {
    fn new() -> Layers {
        Layers { lyrs: Vec::with_capacity(16) }
    }

    fn push(&mut self, lyr: Layer) {
        self.lyrs.push(lyr);
    }

    fn by_name(&self, name: &str) -> CmnResult<&DataCellLayer> {
        for lyr in self.lyrs.iter() {
            if lyr.layer_name() == name {
                return lyr.as_data_cell_layer();
            }
        }
        Err(format!("Layers::by_addr: No layer named '{}' found.", name).into())
    }

    fn by_name_mut(&mut self, name: &str) -> CmnResult<&mut DataCellLayer> {
        for lyr in self.lyrs.iter_mut() {
            if lyr.layer_name() == name {
                return lyr.as_data_cell_layer_mut();
            }
        }
        Err(format!("Layers::by_addr: No layer named '{}' found.", name).into())
    }

    fn by_addr(&self, addr: LayerAddress) -> CmnResult<&DataCellLayer> {
        for lyr in self.lyrs.iter() {
            if lyr.layer_addr() == addr {
                return lyr.as_data_cell_layer();
            }
        }
        Err(format!("Layers::by_addr: No layer with '{}' found.", addr).into())
    }

    fn len(&self) -> usize {
        self.lyrs.len()
    }
}


/// Cortical area settings.
#[derive(Debug, Clone)]
pub struct CorticalAreaSettings {
    pub bypass_inhib: bool,
    pub bypass_filters: bool,
    pub disable_pyrs: bool,
    pub disable_sscs: bool,
    pub disable_mcols: bool,
    pub disable_regrowth: bool,
    pub disable_learning: bool,
    pub build_options: Vec<BuildOpt>,
}

impl CorticalAreaSettings {
    /// Returns a new settings struct.
    pub fn new() -> CorticalAreaSettings {
        CorticalAreaSettings {
            bypass_inhib: false,
            bypass_filters: false,
            disable_pyrs: false,
            disable_sscs: false,
            disable_mcols: false,
            disable_regrowth: false,
            disable_learning: false,
            build_options: Vec::new(),
        }
    }

    /// Disables inhibition.
    pub fn bypass_inhib(mut self) -> CorticalAreaSettings {
        self.bypass_inhib = true;
        self
    }

    /// Disables filters.
    pub fn bypass_filters(mut self) -> CorticalAreaSettings {
        self.bypass_filters = true;
        self
    }

    /// Disable all pyramidal (temporal) cell layers.
    pub fn disable_pyrs(mut self) -> CorticalAreaSettings {
        self.disable_pyrs = true;
        self
    }

    /// Disable all spiny stellate (spatial) cell layers.
    pub fn disable_sscs(mut self) -> CorticalAreaSettings {
        self.disable_sscs = true;
        self
    }

    /// Disable minicolumn output and temporal layer activation.
    pub fn disable_mcols(mut self) -> CorticalAreaSettings {
        self.disable_mcols = true;
        self
    }

    /// Disable learning based regrowth.
    pub fn disable_regrowth(mut self) -> CorticalAreaSettings {
        self.disable_regrowth = true;
        self
    }

    /// Disable learning for all layers.
    pub fn disable_learning(mut self) -> CorticalAreaSettings {
        self.disable_learning = true;
        self
    }

    /// Adds a build option.
    //
    // BuildOpt::include_def("DEFINITION", 1)
    pub fn build_opt(mut self, bo: BuildOpt) -> CorticalAreaSettings {
        self.build_options.push(bo);
        self
    }

    /// Adds all build options to a program builder.
    pub fn add_build_options<'b>(&self, pbldr: &mut ProgramBuilder<'b>) {
        for bo in self.build_options.iter() {
            // pbldr = pbldr.bo(bo.clone())
            pbldr.bo(bo.clone());
        }
    }
}


/// An area of the cortex.
#[derive(Debug)]
pub struct CorticalArea {
    area_id: usize,
    name: &'static str,
    dims: CorticalDims,
    area_map: AreaMap,
    axns: AxonSpace,
    /// Primary neuron layers.
    data_layers: Layers,
    /// Interneuron layers.
    control_layers: ControlCellLayers,
    aux: Aux,
    ocl_pq: ProQue,
    write_queue: Queue,
    read_queue: Queue,
    unmap_queue: Queue,
    counter: usize,
    settings: CorticalAreaSettings,
    cycle_order: Vec<usize>,
    exe_graph: ExecutionGraph,
    samplers: Vec<Sampler>,
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
            settings: Option<CorticalAreaSettings>, thal: &mut Thalamus) -> CmnResult<CorticalArea> {
        // let emsg = "cortical_area::CorticalArea::new()";
        let area_id = area_map.area_id();
        let area_name = area_map.area_name();
        let settings = settings.unwrap_or(CorticalAreaSettings::new());

        println!("\n\nCORTICALAREA::NEW(): Creating Cortical Area: \"{}\"...", area_name);

        let mut p_bldr = area_map.gen_build_options();

        if KERNEL_DEBUG_SYMBOLS && cfg!(target_os = "linux") {
            if ocl_context.platform()?.unwrap().vendor()?.contains("Intel") {
                panic!("[cortical_area::KERNEL_DEBUG_SYMBOLS == true]: \
                    Cannot debug kernels on an Intel based driver platform (not sure why).
                    Use the AMD platform drivers with Intel devices instead.");
            }
            // * TODO: Save kernel file for debugging on Intel.
            // // Optionally pass `-g` and `-s {cl path}` flags to compiler:
            // let debug_opts = format!("-g -s \"{}\"", kernel_path);

            let debug_opts = "-g";
            p_bldr.cmplr_opt(debug_opts);
        };

        settings.add_build_options(&mut p_bldr);

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
            .context(ocl_context.clone())
            .prog_bldr(p_bldr)
            .queue_properties(queue_flags)
            .build().expect("CorticalArea::new(): ocl_pq.build(): error");

        let (write_queue, read_queue, unmap_queue) = if SEPARATE_IO_QUEUES {
            (Queue::new(ocl_context, ocl_pq.device().clone(), Some(queue_flags))?,
                Queue::new(ocl_context, ocl_pq.device().clone(), Some(queue_flags))?,
                Queue::new(ocl_context, ocl_pq.device().clone(), Some(queue_flags))?)
        } else {
            (ocl_pq.queue().clone(), ocl_pq.queue().clone(), ocl_pq.queue().clone())
        };

        ///// TODO: Revisit this... is increment useful anymore?
        // let dims = area_map.dims().clone_with_incr(ocl_pq.max_wg_size().unwrap());
        let dims = area_map.dims().clone();

        println!("{mt}CORTICALAREA::NEW(): Area \"{}\" details: \
            (u_size: {}, v_size: {}, depth: {}), eff_areas: {:?}, aff_areas: {:?}, \n\
            {mt}{mt}device_idx: [{}], device.name(): {}, device.vendor(): {}",
            area_name, dims.u_size(), dims.v_size(), dims.depth(), area_map.eff_areas(),
            area_map.aff_areas(), device_idx, ocl_pq.device().name()?.trim(),
            ocl_pq.device().vendor()?.trim(), mt = cmn::MT);

        /*=============================================================================
        =============================== EXECUTION GRAPH ===============================
        =============================================================================*/

        let mut exe_graph = ExecutionGraph::new();

        /*=============================================================================
        ================================ CELLS & AXONS ================================
        =============================================================================*/

        // let mut mcols = None;
        let mut data_layers = Layers::new();
        let mut control_layers: ControlCellLayers = BTreeMap::new();
        let axns = AxonSpace::new(&area_map, &ocl_pq, read_queue.clone(),
            write_queue.clone(), unmap_queue.clone(), &mut exe_graph, thal)?;

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
                            data_layers.push(Layer::pyr(pyr_lyr));
                        },
                        Some(&DataCellKind::SpinyStellate) => {
                            let sscs_map_dims = dims.clone_with_depth(layer.depth());

                            let ssc_lyr = try!(SpinyStellateLayer::new(layer.name(), layer.layer_id(),
                                sscs_map_dims, cell_scheme.clone(), &area_map, &axns, &ocl_pq,
                                settings.clone(), &mut exe_graph));
                            data_layers.push(Layer::ssc(ssc_lyr));
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

        fn insert_control_layer<C>(control_layers: &mut ControlCellLayers, layer_name: &str,
                cc_lyr: C, host_lyr: &DataCellLayer, exe_order: usize)
                where C: ControlCellLayer {
            if control_layers.insert((host_lyr.layer_addr(), exe_order),
                    Box::new(cc_lyr)).is_some() {
                panic!("Duplicate control cell layer address / order index \
                    found for layer: {} ({})", layer_name, exe_order);
            }
        }

        for layer in area_map.layer_map().iter() {
            if let LayerKind::Cellular(ref cell_scheme) = *layer.kind() {
                match *cell_scheme.class() {
                    CellClass::Control { kind: ControlCellKind::InhibitoryBasketSurround {
                            ref host_lyr_name, field_radius: _ }, exe_order } => {
                        let host_lyr = data_layers.by_name(host_lyr_name)?;

                        let cc_lyr = InhibitoryInterneuronNetwork::new(layer.name(),
                            layer.layer_id(), cell_scheme.clone(), host_lyr, &axns, &area_map,
                            &ocl_pq, settings.clone(), &mut exe_graph)?;

                        insert_control_layer(&mut control_layers, layer.name(), cc_lyr,
                            host_lyr, exe_order);
                    },
                    CellClass::Control { kind: ControlCellKind::ActivitySmoother {
                            ref host_lyr_name, field_radius: _ }, exe_order } => {
                        let host_lyr = data_layers.by_name(host_lyr_name)?;

                        let cc_lyr = ActivitySmoother::new(layer.name(), layer.layer_id(),
                            cell_scheme.clone(), host_lyr, &axns, &area_map, &ocl_pq,
                            settings.clone(), &mut exe_graph)?;

                        insert_control_layer(&mut control_layers, layer.name(), cc_lyr,
                            host_lyr, exe_order);
                    },
                    CellClass::Control { kind: ControlCellKind::PyrOutputter {
                            ref host_lyr_name }, exe_order } => {
                        let host_lyr = data_layers.by_name(host_lyr_name)?;

                        let cc_lyr = PyrOutputter::new(layer.name(), layer.layer_id(),
                            cell_scheme.clone(), host_lyr, &axns, &area_map, &ocl_pq,
                            settings.clone(), &mut exe_graph)?;

                        insert_control_layer(&mut control_layers, layer.name(), cc_lyr,
                            host_lyr, exe_order);
                    },
                    CellClass::Control { kind: ControlCellKind::IntraColumnInhib {
                            ref host_lyr_name }, exe_order } => {
                        let host_lyr = data_layers.by_name(host_lyr_name)?;

                        let cc_lyr = IntraColumnInhib::new(layer.name(), layer.layer_id(),
                            cell_scheme.clone(), host_lyr, &axns, &area_map, &ocl_pq,
                            settings.clone(), &mut exe_graph)?;

                        insert_control_layer(&mut control_layers, layer.name(), cc_lyr,
                            host_lyr, exe_order);
                    },
                    _ => (),
                }
            }
        }

        /*=============================================================================
        ===================================== AUX =====================================
        =============================================================================*/

        let aux = Aux::new(1 << 15, &ocl_pq);

        // <<<<< TODO: CLEAN THIS UP >>>>>
        // MAKE ABOVE LIKE BELOW (eliminateset_arg() methods and just call directly on buffer)
        // mcols.set_arg("aux_ints_0", &aux.ints_0).unwrap();

        // temporal_layers[ptal_idx]
        //    set_arg("aux_ints_0", &aux.ints_0).unwrap();
        // temporal_layers[ptal_idx]
        //    set_arg("aux_ints_1", &aux.ints_1).unwrap();

        // temporal_layers[ptal_idx].dens_mut()
        //    set_arg("aux_ints_0", &aux.ints_0).unwrap();
        // temporal_layers[ptal_idx].dens_mut()
        //    set_arg("aux_ints_1", &aux.ints_1).unwrap();

        // temporal_layers[ptal_idx].dens_mut().syns_mut()
        //    set_arg("aux_ints_0", &aux.ints_0).unwrap();
        // temporal_layers[ptal_idx].dens_mut().syns_mut()
        //    set_arg("aux_ints_1", &aux.ints_1).unwrap();


        // mcols.set_arg("aux_ints_1", &aux.ints_0).unwrap();
        // pyrs_map.get_mut(ptal_name).unwrap().kern_ltp()
        //    set_arg("aux_ints_1", Some(&aux.ints_1)).unwrap();
        // pyrs_map.get_mut(ptal_name).unwrap().kern_cycle()
        //    set_arg("aux_ints_1", Some(&aux.ints_1)).unwrap();

        /*=============================================================================
        ======================= LAYER ROLE EXECUTION ORDERING =========================
        =============================================================================*/

        // Set up layer cycling order based on layer 'role' (motor, temporal,
        // etc.). Layers with multiple 'role' flags will be ordered based on
        // the role with the earliest precedence. Within each role, execution
        // will be ordered by layer id (determined by order specified in layer
        // map scheme).
        let mut cycle_order = Vec::with_capacity(data_layers.len());
        let mut used_idxs = HashSet::with_capacity(data_layers.len());
        for role_idx in 0..ROLE_ORDER.len() {
            let role_tag = ROLE_ORDER[role_idx];
            for (lyr_idx, lyr) in data_layers.lyrs.iter().enumerate() {
                if lyr.tags().contains(role_tag) {
                    if !used_idxs.contains(&lyr_idx) {
                        cycle_order.push(lyr_idx);
                        used_idxs.insert(lyr_idx);
                    }
                }
            }
        }


        /*=============================================================================
        ===============================================================================
        =============================================================================*/

        let mut cortical_area = CorticalArea {
            area_id: area_id,
            name: area_name,
            dims: dims,
            area_map: area_map,
            axns: axns,
            data_layers,
            control_layers,
            aux: aux,
            ocl_pq: ocl_pq,
            write_queue: write_queue,
            read_queue: read_queue,
            unmap_queue: unmap_queue,
            counter: 0,
            settings: settings,
            cycle_order,
            exe_graph: exe_graph,
            samplers: Vec::with_capacity(8),
        };

        cortical_area.order()?;
        Ok(cortical_area)
    }

    /// Establish loose order for commands in execution graph.
    fn order(&mut self) -> CmnResult<()> {
        if self.exe_graph.is_locked() { self.exe_graph.unlock(); }

        // (1.) Axon Intake:
        self.axns.set_exe_order_intake(&mut self.exe_graph)?;

        // (2.) SSTs Cycle:
        for lyr_idx in self.cycle_order.clone() {
            let lyr = self.data_layers.lyrs.get_mut(lyr_idx).unwrap();
            if lyr.tags().contains(LayerTags::SPATIAL) {
                lyr.set_exe_order_cycle(&mut self.control_layers,
                    &mut self.exe_graph)?;
            }
        }

        // (4.) SSTs Learn:
        for lyr_idx in self.cycle_order.clone() {
            let lyr = self.data_layers.lyrs.get_mut(lyr_idx).unwrap();
            if lyr.tags().contains(LayerTags::SPATIAL) {
                lyr.set_exe_order_learn(&mut self.exe_graph)?;
            }
        }

        // // (5.) MCOLSs Activate:
        // if !settings.disable_mcols {
        //     mcols.as_mut().set_exe_order_activate(&mut exe_graph)?;
        // }

        // // (6.) Pyramidal Layers Learn (Now called within `::cycle`):
        // if !self.settings.disable_pyrs {
        //     for &lyr_idx in self.cycle_order.iter() {
        //         let lyr = self.data_layers.lyrs.get_mut(lyr_idx).unwrap();
        //         if !lyr.tags().contains(LayerTags::SPATIAL) {
        //             lyr.set_exe_order_learn(&mut self.exe_graph)?;
        //         }
        //     }
        // }

        // (7.) Pyramidal Layers Cycle:
        if !self.settings.disable_pyrs {
            for &lyr_idx in self.cycle_order.iter() {
                let lyr = self.data_layers.lyrs.get_mut(lyr_idx).unwrap();
                if !lyr.tags().contains(LayerTags::SPATIAL) {
                    lyr.set_exe_order_cycle(&mut self.control_layers,
                        &mut self.exe_graph)?;
                }
            }
        }

        // // (8.) MCOLs Output:
        // if !settings.disable_mcols {
        //     mcols.as_mut().set_exe_order_output(&mut exe_graph)?;
        // }

        // (9.) Axon Output:
        self.axns.set_exe_order_output(&mut self.exe_graph)?;

        // (10.) Samplers:
        for sampler in self.samplers.iter_mut() {
            // println!("######### Ordering sampler: {:?}", sampler);
            sampler.set_exe_order(&mut self.exe_graph)?;
        }

        // Lock and populate execution graph:
        Ok(self.exe_graph.lock())
    }

    /// Cycles the area: running kernels, intaking, and outputting.
    ///
    //
    // * TODO: ISOLATE LEARNING INTO SEPARATE THREAD
    pub fn cycle(&mut self, thal: &mut Thalamus, completion_pool: &mut CompletionPool) -> CmnResult<()> {
        // (1.) Axon Intake:
        self.axns.intake(thal, &mut self.exe_graph, self.settings.bypass_filters,
            completion_pool)?;

        // (2.) SSTs Cycle:
        if !self.settings.disable_sscs {
            for &lyr_idx in self.cycle_order.iter() {
                let lyr = self.data_layers.lyrs.get_mut(lyr_idx).unwrap();
                if lyr.tags().contains(LayerTags::SPATIAL) {
                    lyr.cycle(&mut self.control_layers, &mut self.exe_graph)?
                }
            }
        }

        // (4.) SSTs Learn:
        if !self.settings.disable_sscs {
            for &lyr_idx in self.cycle_order.iter() {
                let lyr = self.data_layers.lyrs.get_mut(lyr_idx).unwrap();
                if lyr.tags().contains(LayerTags::SPATIAL) {
                    lyr.learn(&mut self.exe_graph)?
                }
            }
        }

        // // (5.) MCOLSs Activate:
        // if !self.settings.disable_mcols {
        //     self.mcols.activate(&mut self.exe_graph)?;
        // }

        // // (6.) Pyramidal Layers Learn (Now called within `::cycle`):
        // if !self.settings.disable_pyrs {
        //     for &lyr_idx in self.cycle_order.iter() {
        //         let lyr = self.data_layers.lyrs.get_mut(lyr_idx).unwrap();
        //         if !lyr.tags().contains(LayerTags::SPATIAL) {
        //             if !self.settings.disable_learning {
        //                 lyr.learn(&mut self.exe_graph)?;
        //             }
        //         }
        //     }
        // }

        // (6.) Pyramidal Layers Cycle:
        if !self.settings.disable_pyrs {
            for &lyr_idx in self.cycle_order.iter() {
                let lyr = self.data_layers.lyrs.get_mut(lyr_idx).unwrap();
                if !lyr.tags().contains(LayerTags::SPATIAL) {
                    lyr.cycle(&mut self.control_layers, &mut self.exe_graph)?;
                }
            }
        }

        // // (7.) MCOLs Output:
        // if !self.settings.disable_mcols {
        //     self.mcols.output(&mut self.exe_graph)?;
        // }

        // (8.) Regrow:
        if !self.settings.disable_regrowth {
            self.regrow();
        }

        // self.flush_queues();

        // (9.) Axon Output:
        self.axns.output(thal, &mut self.exe_graph, completion_pool)?;

        // (10.) Samplers:
        self.cycle_samplers(completion_pool)?;

        // println!("######### Cycle complete.");

        Ok(())
    }

    /// Attaches synapses which are below strength threshold to new axons.
    pub fn regrow(&mut self) {
        if !self.settings.disable_regrowth {
            if self.counter >= cmn::SYNAPSE_REGROWTH_INTERVAL {
                self.finish_queues();
                //print!("$");
                for &lyr_idx in self.cycle_order.iter() {
                    let lyr = self.data_layers.lyrs.get_mut(lyr_idx).unwrap();
                    lyr.regrow();
                }
                self.counter = 0;
            } else {
                self.counter += 1;
            }
        }
    }

    /// Cycles through sampling requests.
    fn cycle_samplers(&mut self, completion_pool: &mut CompletionPool) -> CmnResult<()> {
        use ocl::OclPrm;
        use ocl::async::FutureWriteGuard;

        fn lyr<'a>(data_layers: &'a Layers, layer_addr: LayerAddress) -> &'a DataCellLayer {
            data_layers.by_addr(layer_addr)
                .expect(&format!("CorticalArea::cycle_samplers: Invalid layer: {}", layer_addr))
        }

        fn cycle<T: OclPrm>(buf: &Buffer<T>, fwg: FutureWriteGuard<Vec<T>>,
                sampler: &Sampler, cmd_idx: usize, exe_graph: &mut ExecutionGraph,
                new_event: &mut Event, completion_pool: &mut CompletionPool) -> CmnResult<()> {
            let future_read = buf.cmd().read(fwg)
                .offset(sampler.src_idx_range.start)
                .len(sampler.src_idx_range.len())
                .dst_offset(sampler.tx.buffer_idx_range().start)
                .ewait(exe_graph.get_req_events(cmd_idx)?)
                .enew(new_event)
                .enq_async()?
                .map(|_guard| ())
                .map_err(|err| panic!("{}", err));
            completion_pool.complete(Box::new(future_read))?;
            Ok(())
        }

        // // NOTE: Enable sleep only for testing:
        // ::std::thread::sleep(::std::time::Duration::from_millis(1000));

        for sampler in &self.samplers {
            let cmd_idx = sampler.cmd_idx.expect("sampler order not set");

            // Check to see if we need to send this frame. `::wait` will only
            // block if sampler backpressure is on (and tract buffer is
            // already fresh).
            if let Some(write_buf) = sampler.tx.send().wait()? {
                debug_assert!(sampler.tx.buffer_idx_range().len() ==
                    sampler.src_idx_range.len());
                let mut new_event = Event::empty();
                match sampler.kind {
                    SamplerKind::Axons(_lyr_addr) =>  {
                        let buf = self.axns.states();
                        cycle(buf, write_buf.write_u8(), sampler, cmd_idx, &mut self.exe_graph,
                            &mut new_event, completion_pool)?;
                    },
                    SamplerKind::SomaStates(lyr_addr) => {
                        let buf = lyr(&self.data_layers, lyr_addr).soma();
                        cycle(buf, write_buf.write_u8(), sampler, cmd_idx, &mut self.exe_graph,
                            &mut new_event, completion_pool)?;
                    },
                    SamplerKind::SomaEnergies(lyr_addr) => {
                        let buf = lyr(&self.data_layers, lyr_addr).energies();
                        cycle(buf, write_buf.write_u8(), sampler, cmd_idx, &mut self.exe_graph,
                            &mut new_event, completion_pool)?;
                    },
                    SamplerKind::SomaActivities(lyr_addr) => {
                        let buf = lyr(&self.data_layers, lyr_addr).activities();
                        cycle(buf, write_buf.write_u8(), sampler, cmd_idx, &mut self.exe_graph,
                            &mut new_event, completion_pool)?;
                    },
                    SamplerKind::SomaFlagSets(lyr_addr) => {
                        let buf = lyr(&self.data_layers, lyr_addr).flag_sets();
                        cycle(buf, write_buf.write_u8(), sampler, cmd_idx, &mut self.exe_graph,
                            &mut new_event, completion_pool)?;
                    },
                    SamplerKind::TuftStates(lyr_addr) => {
                        let buf = lyr(&self.data_layers, lyr_addr).tufts().states();
                        cycle(buf, write_buf.write_u8(), sampler, cmd_idx, &mut self.exe_graph,
                            &mut new_event, completion_pool)?;
                    },
                    SamplerKind::TuftBestDenIds(lyr_addr) => {
                        let buf = lyr(&self.data_layers, lyr_addr).tufts().best_den_ids();
                        cycle(buf, write_buf.write_u8(), sampler, cmd_idx, &mut self.exe_graph,
                            &mut new_event, completion_pool)?;
                    },
                    SamplerKind::TuftBestDenStatesRaw(lyr_addr) => {
                        let buf = lyr(&self.data_layers, lyr_addr).tufts().best_den_states_raw();
                        cycle(buf, write_buf.write_u8(), sampler, cmd_idx, &mut self.exe_graph,
                            &mut new_event, completion_pool)?;
                    },
                    SamplerKind::TuftBestDenStates(lyr_addr) => {
                        let buf = lyr(&self.data_layers, lyr_addr).tufts().best_den_states();
                        cycle(buf, write_buf.write_u8(), sampler, cmd_idx, &mut self.exe_graph,
                            &mut new_event, completion_pool)?;
                    },
                    SamplerKind::TuftPrevStates(lyr_addr) => {
                        let buf = lyr(&self.data_layers, lyr_addr).tufts().prev_states();
                        cycle(buf, write_buf.write_u8(), sampler, cmd_idx, &mut self.exe_graph,
                            &mut new_event, completion_pool)?;
                    },
                    SamplerKind::TuftPrevBestDenIds(lyr_addr) => {
                        let buf = lyr(&self.data_layers, lyr_addr).tufts().prev_best_den_ids();
                        cycle(buf, write_buf.write_u8(), sampler, cmd_idx, &mut self.exe_graph,
                            &mut new_event, completion_pool)?;
                    },
                    SamplerKind::TuftPrevBestDenStatesRaw(lyr_addr) => {
                        let buf = lyr(&self.data_layers, lyr_addr).tufts().prev_best_den_states_raw();
                        cycle(buf, write_buf.write_u8(), sampler, cmd_idx, &mut self.exe_graph,
                            &mut new_event, completion_pool)?;
                    },
                    SamplerKind::TuftPrevBestDenStates(lyr_addr) => {
                        let buf = lyr(&self.data_layers, lyr_addr).tufts().prev_best_den_states();
                        cycle(buf, write_buf.write_u8(), sampler, cmd_idx, &mut self.exe_graph,
                            &mut new_event, completion_pool)?;
                    },
                    SamplerKind::DenStates(lyr_addr) => {
                        let buf = lyr(&self.data_layers, lyr_addr).dens().states();
                        cycle(buf, write_buf.write_u8(), sampler, cmd_idx, &mut self.exe_graph,
                            &mut new_event, completion_pool)?;
                    },
                    SamplerKind::DenStatesRaw(lyr_addr) => {
                        let buf = lyr(&self.data_layers, lyr_addr).dens().states_raw();
                        cycle(buf, write_buf.write_u8(), sampler, cmd_idx, &mut self.exe_graph,
                            &mut new_event, completion_pool)?;
                    },
                    SamplerKind::DenEnergies(lyr_addr) => {
                        let buf = lyr(&self.data_layers, lyr_addr).dens().energies();
                        cycle(buf, write_buf.write_u8(), sampler, cmd_idx, &mut self.exe_graph,
                            &mut new_event, completion_pool)?;
                    },
                    SamplerKind::DenActivities(lyr_addr) => {
                        let buf = lyr(&self.data_layers, lyr_addr).dens().activities();
                        cycle(buf, write_buf.write_u8(), sampler, cmd_idx, &mut self.exe_graph,
                            &mut new_event, completion_pool)?;
                    },
                    SamplerKind::DenThresholds(lyr_addr) => {
                        let buf = lyr(&self.data_layers, lyr_addr).dens().thresholds();
                        cycle(buf, write_buf.write_u8(), sampler, cmd_idx, &mut self.exe_graph,
                            &mut new_event, completion_pool)?;
                    },
                    SamplerKind::SynStates(lyr_addr) => {
                        let buf = lyr(&self.data_layers, lyr_addr).dens().syns().states();
                        cycle(buf, write_buf.write_u8(), sampler, cmd_idx, &mut self.exe_graph,
                            &mut new_event, completion_pool)?;
                    },
                    SamplerKind::SynStrengths(lyr_addr) => {
                        let buf = lyr(&self.data_layers, lyr_addr).dens().syns().strengths();
                        cycle(buf, write_buf.write_i8(), sampler, cmd_idx, &mut self.exe_graph,
                            &mut new_event, completion_pool)?;
                    },
                    SamplerKind::SynSrcColVOffs(lyr_addr) => {
                        let buf = lyr(&self.data_layers, lyr_addr).dens().syns().src_col_v_offs();
                        cycle(buf, write_buf.write_i8(), sampler, cmd_idx, &mut self.exe_graph,
                            &mut new_event, completion_pool)?;
                    },
                    SamplerKind::SynSrcColUOffs(lyr_addr) => {
                        let buf = lyr(&self.data_layers, lyr_addr).dens().syns().src_col_u_offs();
                        cycle(buf, write_buf.write_i8(), sampler, cmd_idx, &mut self.exe_graph,
                            &mut new_event, completion_pool)?;
                    },
                    SamplerKind::SynFlagSets(lyr_addr) => {
                        let buf = lyr(&self.data_layers, lyr_addr).dens().syns().flag_sets();
                        cycle(buf, write_buf.write_u8(), sampler, cmd_idx, &mut self.exe_graph,
                            &mut new_event, completion_pool)?;
                    },
                    _ => unimplemented!(),
                }
                self.exe_graph.set_cmd_event(cmd_idx, Some(new_event))?;
            } else {
                self.exe_graph.set_cmd_event(cmd_idx, None)?;
            }
        }
        Ok(())
    }

    /// Creates and adds a sampler from the provided transmitter and
    /// configures the execution graph appropriately.
    fn add_sampler(&mut self, cmd_srcs: Vec<CorticalBuffer>, kind: SamplerKind,
            src_idx_range: Option<Range<usize>>, tx: TractSender) {
        // Add command to graph and get uid:
        self.exe_graph.unlock();
        let cmd_uid = self.exe_graph.add_command(
                CommandRelations::cortical_sample(cmd_srcs))
            .expect("CorticalArea::sampler: Error adding exe. graph command");
        // Create and push sampler:
        self.samplers.push(Sampler::new(kind, src_idx_range, tx, cmd_uid));
        // Repopulate execution graph:
        self.order().expect("CorticalArea::sampler: Error reordering");
    }

    /// Creates a tract channel and configures execution graph appropriately.
    fn sampler_rx_single_u8(&mut self, len: usize, cmd_srcs: Vec<CorticalBuffer>,
            kind: SamplerKind, src_idx_range: Option<Range<usize>>, backpressure: bool) -> TractReceiver {
        // Create a new tract channel:
        let (tx, rx) = subcortex::tract_channel_single_u8(RwVec::from(vec![0u8; len]), None,
            backpressure);
        // Add sampler and config exe graph:
        self.add_sampler(cmd_srcs, kind, src_idx_range, tx);
        rx
    }

    /// Creates a tract channel and configures execution graph appropriately.
    fn sampler_rx_single_i8(&mut self, len: usize, cmd_srcs: Vec<CorticalBuffer>,
            kind: SamplerKind, src_idx_range: Option<Range<usize>>, backpressure: bool) -> TractReceiver {
        // Create a new tract channel:
        let (tx, rx) = subcortex::tract_channel_single_i8(RwVec::from(vec![0i8; len]), None,
            backpressure);
        // Add sampler and config exe graph:
        self.add_sampler(cmd_srcs, kind, src_idx_range, tx);
        rx
    }

    /// Requests a cortical 'sampler' which provides external read access to
    /// cortical cells and axons.
    pub fn sampler(&mut self, kind: SamplerKind, buffer_kind: SamplerBufferKind,
            backpressure: bool) -> TractReceiver {
        fn slc_range(area_map: &AreaMap, layer_id: usize) -> Range<usize> {
            area_map.layer_map().layer_info(layer_id)
                .expect(&format!("CorticalArea::sample: Invalid layer: [id:{}]", layer_id))
                .slc_range()
                .expect(&format!("CorticalArea::sample: Layer [id:{}] has no slices", layer_id))
                .clone()
        }

        fn lyr<'a>(data_layers: &'a Layers, lyr_addr: LayerAddress) -> &'a DataCellLayer {
            data_layers.by_addr(lyr_addr)
                .expect(&format!("CorticalArea::sample: Invalid layer: {}", lyr_addr))
        }

        match kind {
            // Axons:
            SamplerKind::Axons(lyr_addr) => {
                let slc_range = match lyr_addr {
                    Some(addr) => slc_range(&self.area_map, addr.layer_id()),
                    None => 0..self.area_map.slice_map().depth() as usize,
                };
                let axon_range = self.area_map.slice_map().axon_range(slc_range.clone());
                match buffer_kind {
                    SamplerBufferKind::Single => {
                        let cmd_srcs = slc_range.map(|slc_id| {
                            CorticalBuffer::axon_slice(self.axns.states(), self.area_id, slc_id as u8)
                        }).collect();
                        self.sampler_rx_single_u8(axon_range.len(), cmd_srcs, kind.clone(),
                            Some(axon_range), backpressure)
                    },
                    _ => unimplemented!(),
                }
            },

            // Soma:
            SamplerKind::SomaStates(lyr_addr) => {
                let (len, cmd_srcs) = {
                    let lyr = lyr(&self.data_layers, lyr_addr);
                    (lyr.soma().len(),
                        vec![CorticalBuffer::data_soma_lyr(lyr.soma(), lyr_addr)])
                };
                self.sampler_rx_single_u8(len, cmd_srcs, kind.clone(), None, backpressure)
            },
            SamplerKind::SomaEnergies(lyr_addr) => {
                let (len, cmd_srcs) = {
                    let lyr = lyr(&self.data_layers, lyr_addr);
                    (lyr.energies().len(),
                        vec![CorticalBuffer::data_soma_lyr(lyr.energies(), lyr_addr)])
                };
                self.sampler_rx_single_u8(len, cmd_srcs, kind.clone(), None, backpressure)
            },
            SamplerKind::SomaActivities(lyr_addr) => {
                let (len, cmd_srcs) = {
                    let lyr = lyr(&self.data_layers, lyr_addr);
                    (lyr.activities().len(),
                        vec![CorticalBuffer::data_soma_lyr(lyr.activities(), lyr_addr)])
                };
                self.sampler_rx_single_u8(len, cmd_srcs, kind.clone(), None, backpressure)
            },
            SamplerKind::SomaFlagSets(lyr_addr) => {
                let (len, cmd_srcs) = {
                    let lyr = lyr(&self.data_layers, lyr_addr);
                    (lyr.flag_sets().len(),
                        vec![CorticalBuffer::data_soma_lyr(lyr.flag_sets(), lyr_addr,)])
                };
                self.sampler_rx_single_u8(len, cmd_srcs, kind.clone(), None, backpressure)
            },

            // Tufts:
            SamplerKind::TuftStates(lyr_addr) => {
                let (len, cmd_srcs) = {
                    let lyr = lyr(&self.data_layers, lyr_addr);
                    let srcs = (0..lyr.tft_count()).map(|tft_id| {
                        CorticalBuffer::data_tft(lyr.tufts().states(), lyr_addr, tft_id)
                    }).collect();
                    (lyr.tufts().states().len(), srcs)
                };
                self.sampler_rx_single_u8(len, cmd_srcs, kind.clone(), None, backpressure)
            },
            SamplerKind::TuftBestDenIds(lyr_addr) => {
                let (len, cmd_srcs) = {
                    let lyr = lyr(&self.data_layers, lyr_addr);
                    let srcs = (0..lyr.tft_count()).map(|tft_id| {
                        CorticalBuffer::data_tft(lyr.tufts().best_den_ids(), lyr_addr, tft_id)
                    }).collect();
                    (lyr.tufts().best_den_ids().len(), srcs)
                };
                self.sampler_rx_single_u8(len, cmd_srcs, kind.clone(), None, backpressure)
            },
            SamplerKind::TuftBestDenStatesRaw(lyr_addr) => {
                let (len, cmd_srcs) = {
                    let lyr = lyr(&self.data_layers, lyr_addr);
                    let srcs = (0..lyr.tft_count()).map(|tft_id| {
                        CorticalBuffer::data_tft(lyr.tufts().best_den_states_raw(), lyr_addr, tft_id)
                    }).collect();
                    (lyr.tufts().best_den_states_raw().len(), srcs)
                };
                self.sampler_rx_single_u8(len, cmd_srcs, kind.clone(), None, backpressure)
            },
            SamplerKind::TuftBestDenStates(lyr_addr) => {
                let (len, cmd_srcs) = {
                    let lyr = lyr(&self.data_layers, lyr_addr);
                    let srcs = (0..lyr.tft_count()).map(|tft_id| {
                        CorticalBuffer::data_tft(lyr.tufts().best_den_states(), lyr_addr, tft_id)
                    }).collect();
                    (lyr.tufts().best_den_states().len(), srcs)
                };
                self.sampler_rx_single_u8(len, cmd_srcs, kind.clone(), None, backpressure)
            },
            SamplerKind::TuftPrevStates(lyr_addr) => {
                let (len, cmd_srcs) = {
                    let lyr = lyr(&self.data_layers, lyr_addr);
                    let srcs = (0..lyr.tft_count()).map(|tft_id| {
                        CorticalBuffer::data_tft(lyr.tufts().prev_states(), lyr_addr, tft_id)
                    }).collect();
                    (lyr.tufts().prev_states().len(), srcs)
                };
                self.sampler_rx_single_u8(len, cmd_srcs, kind.clone(), None, backpressure)
            },
            SamplerKind::TuftPrevBestDenIds(lyr_addr) => {
                let (len, cmd_srcs) = {
                    let lyr = lyr(&self.data_layers, lyr_addr);
                    let srcs = (0..lyr.tft_count()).map(|tft_id| {
                        CorticalBuffer::data_tft(lyr.tufts().prev_best_den_ids(), lyr_addr, tft_id)
                    }).collect();
                    (lyr.tufts().prev_best_den_ids().len(), srcs)
                };
                self.sampler_rx_single_u8(len, cmd_srcs, kind.clone(), None, backpressure)
            },
            SamplerKind::TuftPrevBestDenStatesRaw(lyr_addr) => {
                let (len, cmd_srcs) = {
                    let lyr = lyr(&self.data_layers, lyr_addr);
                    let srcs = (0..lyr.tft_count()).map(|tft_id| {
                        CorticalBuffer::data_tft(lyr.tufts().prev_best_den_states_raw(), lyr_addr, tft_id)
                    }).collect();
                    (lyr.tufts().prev_best_den_states_raw().len(), srcs)
                };
                self.sampler_rx_single_u8(len, cmd_srcs, kind.clone(), None, backpressure)
            },
            SamplerKind::TuftPrevBestDenStates(lyr_addr) => {
                let (len, cmd_srcs) = {
                    let lyr = lyr(&self.data_layers, lyr_addr);
                    let srcs = (0..lyr.tft_count()).map(|tft_id| {
                        CorticalBuffer::data_tft(lyr.tufts().prev_best_den_states(), lyr_addr, tft_id)
                    }).collect();
                    (lyr.tufts().prev_best_den_states().len(), srcs)
                };
                self.sampler_rx_single_u8(len, cmd_srcs, kind.clone(), None, backpressure)
            },

            // Dens:
            SamplerKind::DenStates(lyr_addr) => {
                let (len, cmd_srcs) = {
                    let lyr = lyr(&self.data_layers, lyr_addr);
                    let srcs = (0..lyr.tft_count()).map(|tft_id| {
                        CorticalBuffer::data_den_tft(lyr.dens().states(), lyr_addr, tft_id)
                    }).collect();
                    (lyr.dens().states().len(), srcs)
                };
                self.sampler_rx_single_u8(len, cmd_srcs, kind.clone(), None, backpressure)
            },
            SamplerKind::DenStatesRaw(lyr_addr) => {
                let (len, cmd_srcs) = {
                    let lyr = lyr(&self.data_layers, lyr_addr);
                    let srcs = (0..lyr.tft_count()).map(|tft_id| {
                        CorticalBuffer::data_den_tft(lyr.dens().states_raw(), lyr_addr, tft_id)
                    }).collect();
                    (lyr.dens().states_raw().len(), srcs)
                };
                self.sampler_rx_single_u8(len, cmd_srcs, kind.clone(), None, backpressure)
            },
            SamplerKind::DenEnergies(lyr_addr) => {
                let (len, cmd_srcs) = {
                    let lyr = lyr(&self.data_layers, lyr_addr);
                    let srcs = (0..lyr.tft_count()).map(|tft_id| {
                        CorticalBuffer::data_den_tft(lyr.dens().energies(), lyr_addr, tft_id)
                    }).collect();
                    (lyr.dens().energies().len(), srcs)
                };
                self.sampler_rx_single_u8(len, cmd_srcs, kind.clone(), None, backpressure)
            },
            SamplerKind::DenActivities(lyr_addr) => {
                let (len, cmd_srcs) = {
                    let lyr = lyr(&self.data_layers, lyr_addr);
                    let srcs = (0..lyr.tft_count()).map(|tft_id| {
                        CorticalBuffer::data_den_tft(lyr.dens().activities(), lyr_addr, tft_id)
                    }).collect();
                    (lyr.dens().activities().len(), srcs)
                };
                self.sampler_rx_single_u8(len, cmd_srcs, kind.clone(), None, backpressure)
            },
            SamplerKind::DenThresholds(lyr_addr) => {
                let (len, cmd_srcs) = {
                    let lyr = lyr(&self.data_layers, lyr_addr);
                    let srcs = (0..lyr.tft_count()).map(|tft_id| {
                        CorticalBuffer::data_den_tft(lyr.dens().thresholds(), lyr_addr, tft_id)
                    }).collect();
                    (lyr.dens().thresholds().len(), srcs)
                };
                self.sampler_rx_single_u8(len, cmd_srcs, kind.clone(), None, backpressure)
            },

            // Syns:
            SamplerKind::SynStates(lyr_addr) => {
                let (len, cmd_srcs) = {
                    let lyr = lyr(&self.data_layers, lyr_addr);
                    let srcs = (0..lyr.tft_count()).map(|tft_id| {
                        CorticalBuffer::data_syn_tft(lyr.dens().syns().states(), lyr_addr, tft_id)
                    }).collect();
                    (lyr.dens().syns().states().len(), srcs)
                };
                self.sampler_rx_single_u8(len, cmd_srcs, kind.clone(), None, backpressure)
            },
            SamplerKind::SynStrengths(lyr_addr) => {
                let (len, cmd_srcs) = {
                    let lyr = lyr(&self.data_layers, lyr_addr);
                    let srcs = (0..lyr.tft_count()).map(|tft_id| {
                        CorticalBuffer::data_syn_tft(lyr.dens().syns().strengths(), lyr_addr, tft_id)
                    }).collect();
                    (lyr.dens().syns().strengths().len(), srcs)
                };
                self.sampler_rx_single_i8(len, cmd_srcs, kind.clone(), None, backpressure)
            },
            SamplerKind::SynSrcColVOffs(lyr_addr) => {
                let (len, cmd_srcs) = {
                    let lyr = lyr(&self.data_layers, lyr_addr);
                    let srcs = (0..lyr.tft_count()).map(|tft_id| {
                        CorticalBuffer::data_syn_tft(lyr.dens().syns().src_col_v_offs(), lyr_addr, tft_id)
                    }).collect();
                    (lyr.dens().syns().src_col_v_offs().len(), srcs)
                };
                self.sampler_rx_single_i8(len, cmd_srcs, kind.clone(), None, backpressure)
            },
            SamplerKind::SynSrcColUOffs(lyr_addr) => {
                let (len, cmd_srcs) = {
                    let lyr = lyr(&self.data_layers, lyr_addr);
                    let srcs = (0..lyr.tft_count()).map(|tft_id| {
                        CorticalBuffer::data_syn_tft(lyr.dens().syns().src_col_u_offs(), lyr_addr, tft_id)
                    }).collect();
                    (lyr.dens().syns().src_col_u_offs().len(), srcs)
                };
                self.sampler_rx_single_i8(len, cmd_srcs, kind.clone(), None, backpressure)
            },
            SamplerKind::SynFlagSets(lyr_addr) => {
                let (len, cmd_srcs) = {
                    let lyr = lyr(&self.data_layers, lyr_addr);
                    let srcs = (0..lyr.tft_count()).map(|tft_id| {
                        CorticalBuffer::data_syn_tft(lyr.dens().syns().flag_sets(), lyr_addr, tft_id)
                    }).collect();
                    (lyr.dens().syns().flag_sets().len(), srcs)
                };
                self.sampler_rx_single_u8(len, cmd_srcs, kind.clone(), None, backpressure)
            },

            _ => unimplemented!(),
        }
    }

    /// Blocks until all previously queued OpenCL commands in all
    /// command-queues are issued to the associated device and have completed.
    pub fn finish_queues(&self) {
        self.write_queue.finish().unwrap();
        self.ocl_pq.queue().finish().unwrap();
        self.read_queue.finish().unwrap();
        self.unmap_queue.finish().unwrap();
        self.exe_graph.finish().unwrap();
    }

    /// Returns an immutable reference to the requested data cell layer.
    ///
    /// This performs a linear search through all layers.
    pub fn layer(&self, layer_name: &'static str) -> CmnResult<&DataCellLayer> {
        self.data_layers.by_name(layer_name)
    }

    /// Returns a mutable reference to the requested data cell layer.
    ///
    /// This performs a linear search through all layers.
    pub fn layer_mut(&mut self, layer_name: &'static str) -> CmnResult<&mut DataCellLayer> {
        self.data_layers.by_name_mut(layer_name)
    }

    #[inline] pub fn axns(&self) -> &AxonSpace { &self.axns }
    #[inline] pub fn dims(&self) -> &CorticalDims { &self.dims }
    #[inline] pub fn afferent_target_names(&self) -> &[&'static str] { &self.area_map.aff_areas() }
    #[inline] pub fn efferent_target_names(&self) -> &[&'static str] { &self.area_map.eff_areas() }
    #[inline] pub fn ocl_pq(&self) -> &ProQue { &self.ocl_pq }
    #[inline] pub fn device(&self) -> Device { self.ocl_pq.queue().device() }
    #[inline] pub fn axon_tract_map(&self) -> SliceTractMap { self.area_map.slice_map().tract_map() }
    #[inline] pub fn area_map(&self) -> &AreaMap { &self.area_map }
    #[inline] pub fn area_id(&self) -> usize { self.area_id }
    #[inline] pub fn aux(&self) -> &Aux { &self.aux }
    #[inline] pub fn exe_graph_mut(&mut self) -> &mut ExecutionGraph { &mut self.exe_graph }
}

impl Drop for CorticalArea {
    fn drop(&mut self) {
        println!("Releasing work thread for '{}'... ", &self.name);
        print!("Releasing OpenCL components for '{}'... ", &self.name);
        print!("[ Buffers ][ Event Lists ][ Program ][ Command Queues ]");
        print!(" ...complete. \n");
    }
}


#[derive(Debug)]
pub struct Aux {
    pub ints_0: Buffer<i32>,
    pub ints_1: Buffer<i32>,
}

impl Aux {
    pub fn new(len: usize, ocl_pq: &ProQue) -> Aux {
        let ints_0 = Buffer::<i32>::builder()
            .queue(ocl_pq.queue().clone())
            .len(len)
            .fill_val(i32::min_value())
            .build().unwrap();

        let ints_1 = Buffer::<i32>::builder()
            .queue(ocl_pq.queue().clone())
            .len(len)
            .fill_val(i32::min_value())
            .build().unwrap();

        ocl_pq.queue().finish().unwrap();

        Aux {
            ints_0: ints_0,
            ints_1: ints_1,
        }
    }
}

//////////////////////


#[cfg(any(test, feature = "eval"))]
pub mod tests {
    use rand;
    use rand::distributions::{IndependentSample, Range as RandRange};

    use super::*;
    use cortex::{AxonSpaceTest, CelCoords, DataCellLayerTest};
    use map::{AreaMapTest};
    use SrcOfs;

    pub trait CorticalAreaTest {
        fn axon_state(&self, idx: usize) -> u8;
        fn write_to_axon(&mut self, val: u8, idx: u32);
        fn read_from_axon(&self, idx: u32) -> u8;
        fn rand_safe_src_axn(&mut self, cel_coords: &CelCoords, src_axon_slc: u8)
            -> (SrcOfs, SrcOfs, u32, u32);
        fn print_aux(&mut self);
        fn print_axns(&mut self);
        fn activate_axon(&mut self, idx: u32);
        fn deactivate_axon(&mut self, idx: u32);

        /// Returns an immutable reference to the requested data cell layer.
        ///
        /// This performs a linear search through all layers.
        fn layer_test(&self, layer_name: &'static str) -> CmnResult<&DataCellLayerTest>;

        /// Returns a mutable reference to the requested data cell layer.
        ///
        /// This performs a linear search through all layers.
        fn layer_test_mut(&mut self, layer_name: &'static str) -> CmnResult<&mut DataCellLayerTest>;
    }

    impl CorticalAreaTest for CorticalArea {
        fn axon_state(&self, idx: usize) -> u8 {
            self.finish_queues();
            self.axns.axon_state(idx)
        }

        fn read_from_axon(&self, idx: u32) -> u8 {
            self.finish_queues();
            self.axns.axon_state(idx as usize)
        }

        fn write_to_axon(&mut self, val: u8, idx: u32) {
            self.finish_queues();
            self.axns.write_to_axon(val, idx);
        }

        fn rand_safe_src_axn(&mut self, cel_coords: &CelCoords, src_axon_slc: u8) -> (SrcOfs, SrcOfs, u32, u32) {
            let v_ofs_range = RandRange::new(-8 as SrcOfs, 9);
            let u_ofs_range = RandRange::new(-8 as SrcOfs, 9);

            let mut rng = rand::weak_rng();

            for _ in 0..50 {
                let v_ofs = v_ofs_range.ind_sample(&mut rng);
                let u_ofs = u_ofs_range.ind_sample(&mut rng);

                if v_ofs | u_ofs == 0 {
                    continue;
                }

                let idx_rslt = self.area_map.axon_idx(src_axon_slc, cel_coords.v_id,
                    v_ofs, cel_coords.u_id, u_ofs);

                match idx_rslt {
                    Ok(idx) => {
                        let col_id = self.area_map.axon_col_id(src_axon_slc, cel_coords.v_id,
                            v_ofs, cel_coords.u_id, u_ofs).unwrap();
                        return (v_ofs, u_ofs, col_id, idx)
                    },

                    Err(_) => (),
                }
            }

            panic!("SynCoords::rand_safe_src_axon_offs(): Error finding valid offset pair.");
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

        /// Returns an immutable reference to the requested data cell layer.
        ///
        /// This performs a linear search through all layers.
        fn layer_test(&self, layer_name: &'static str) -> CmnResult<&DataCellLayerTest> {
            for lyr in self.data_layers.lyrs.iter() {
                if lyr.layer_name() == layer_name {
                    match *lyr {
                        Layer::SpinyStellateLayer(ref lyr) => return Ok(lyr),
                        Layer::PyramidalLayer(ref lyr) => return Ok(lyr),
                    }
                }
            }
            Err(format!("CorticalAreaTest::layer_test: No layer named '{}' found.",
                layer_name).into())
        }

        /// Returns a mutable reference to the requested data cell layer.
        ///
        /// This performs a linear search through all layers.
        fn layer_test_mut(&mut self, layer_name: &'static str) -> CmnResult<&mut DataCellLayerTest> {
            for lyr in self.data_layers.lyrs.iter_mut() {
                if lyr.layer_name() == layer_name {
                    match *lyr {
                        Layer::SpinyStellateLayer(ref mut lyr) => return Ok(lyr),
                        Layer::PyramidalLayer(ref mut lyr) => return Ok(lyr),
                    }
                }
            }
            Err(format!("CorticalAreaTest::layer_test_mut: No layer named '{}' found.",
                layer_name).into())
        }
    }
}

