#![allow(dead_code, unused_mut, unused_imports)]

use std::thread::{self, JoinHandle};
use std::collections::{HashMap, HashSet, BTreeMap};
use std::ops::Range;
use std::borrow::Borrow;
use futures::{Sink, Stream, Future};
// use futures::future::BoxFuture;

use futures::sync::mpsc::{self, Sender};
use tokio_core::reactor::{Core, Remote};
use ocl::{async, flags, Device, ProQue, Context, Buffer, Event, Queue, OclPrm, RwVec};
use ocl::core::CommandQueueProperties;
use ocl::builders::{BuildOpt, ProgramBuilder};
use cmn::{self, CmnError, CmnResult, CorticalDims};
use map::{self, AreaMap, SliceTractMap, LayerKind, DataCellKind, ControlCellKind,
    ExecutionGraph, CellClass, LayerTags, LayerAddress, CommandUid};
use ::Thalamus;
use cortex::{AxonSpace, /*Minicolumns,*/ InhibitoryInterneuronNetwork, PyramidalLayer,
    SpinyStellateLayer, DataCellLayer, ControlCellLayer, ActivitySmoother, PyrOutputter};
use subcortex::{self, TractBuffer, TractSender, TractReceiver};

#[cfg(test)] pub use self::tests::{CorticalAreaTest};

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


pub type ControlCellLayers = BTreeMap<(LayerAddress, usize), Box<ControlCellLayer>>;


#[derive(Debug)]
struct Sampler {
    kind: SamplerKind,
    src_idx_range: Range<usize>,
    // buffer: SamplerBuffer,
    tx: TractSender,
    cmd_uid: CommandUid,
    cmd_idx: Option<usize>,
}

impl Sampler {
    fn new(kind: SamplerKind, src_idx_range: Range<usize>, /*buffer: SamplerBuffer,*/
            tx: TractSender, cmd_uid: CommandUid) -> Sampler
    {
        Sampler { kind, src_idx_range, /*buffer,*/ tx, cmd_uid, cmd_idx: None }
    }

    fn set_exe_order(&mut self, exe_graph: &mut ExecutionGraph) -> CmnResult<()> {
        self.cmd_idx = Some(exe_graph.order_command(self.cmd_uid)?);
        Ok(())
    }

    // fn buffer_single_u8(&self) -> RwVec<u8> {
    //     self.tx.buffer_single_u8()
    // }
}


#[derive(Debug, Clone)]
pub enum SamplerKind {
    /// Axons for a specific layer.
    AxonLayer(Option<usize>),
    // /// All axons.
    // AxonSpace,
    Dummy,
}


#[derive(Debug, Clone)]
pub enum SamplerBufferKind {
    None,
    Single,
    Double,
    Triple,
}


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

    // fn set_exe_order(&mut self, control_layers: &mut ControlCellLayers,
    //         exe_graph: &mut ExecutionGraph) -> CmnResult<()>
    // {
    //     match *self {
    //         Layer::SpinyStellateLayer(ref mut _lyr) => Err("".into()),
    //         Layer::PyramidalLayer(ref mut lyr) => lyr.set_exe_order(control_layers, exe_graph),
    //     }
    // }

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

    fn is_pyramidal(&self) -> bool {
        match *self {
            Layer::SpinyStellateLayer(_) => false,
            Layer::PyramidalLayer(_) => true,
        }
    }

    fn as_pyr_lyr(&self) -> CmnResult<&PyramidalLayer> {
        match *self {
            Layer::SpinyStellateLayer(_) => Err("not a pyramidal layer".into()),
            Layer::PyramidalLayer(ref lyr) => Ok(lyr),
        }
    }
}


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

    fn ssc_by_name(&self, name: &str) -> CmnResult<&SpinyStellateLayer> {
        for lyr in self.lyrs.iter() {
            if let Layer::SpinyStellateLayer(ref ssc) = *lyr {
                if ssc.layer_name() == name {
                    return Ok(ssc);
                }
            }
        }
        Err(format!("Layers::ssc_by_name: No layer named '{}' found.", name).into())
    }

    fn ssc_by_name_mut(&mut self, name: &str) -> CmnResult<&mut SpinyStellateLayer> {
        for lyr in self.lyrs.iter_mut() {
            if let Layer::SpinyStellateLayer(ref mut ssc) = *lyr {
                if ssc.layer_name() == name {
                    return Ok(ssc);
                }
            }
        }
        Err(format!("Layers::ssc_by_name: No layer named '{}' found.", name).into())
    }

    fn pyr_by_name(&self, name: &str) -> CmnResult<&PyramidalLayer> {
        for lyr in self.lyrs.iter() {
            if let Layer::PyramidalLayer(ref pyr) = *lyr {
                if pyr.layer_name() == name {
                    return Ok(pyr);
                }
            }
        }
        Err(format!("Layers::pyr_by_name: No layer named '{}' found.", name).into())
    }

    fn pyr_by_name_mut(&mut self, name: &str) -> CmnResult<&mut PyramidalLayer> {
        for lyr in self.lyrs.iter_mut() {
            if let Layer::PyramidalLayer(ref mut pyr) = *lyr {
                if pyr.layer_name() == name {
                    return Ok(pyr);
                }
            }
        }
        Err(format!("Layers::pyr_by_name: No layer named '{}' found.", name).into())
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
    pub build_options: Vec<BuildOpt>
}

impl CorticalAreaSettings {
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
    pub fn add_build_options(&self, mut pbldr: ProgramBuilder) -> ProgramBuilder {
        for bo in self.build_options.iter() {
            pbldr = pbldr.bo(bo.clone())
        }
        pbldr
    }
}


/// An area of the cortex.
pub struct CorticalArea {
    area_id: usize,
    name: &'static str,
    dims: CorticalDims,
    area_map: AreaMap,
    axns: AxonSpace,
    // mcols: Box<Minicolumns>,
    // iinns: HashMap<&'static str, Box<InhibitoryInterneuronNetwork>>,
    ptal_name: Option<&'static str>,
    psal_name: Option<&'static str>,
    psal_idx: usize,
    ptal_idx: usize,
    /// Primary neuron layers.
    data_layers: Layers,
    /// Interneuron layers.
    control_layers: BTreeMap<(LayerAddress, usize), Box<ControlCellLayer>>,
    aux: Aux,
    ocl_pq: ProQue,
    write_queue: Queue,
    read_queue: Queue,
    counter: usize,
    settings: CorticalAreaSettings,
    cycle_order: Vec<usize>,
    exe_graph: ExecutionGraph,
    work_tx: Option<Sender<Box<Future<Item=(), Error=()> + Send>>>,
    // TODO: Move this to a centralized thread pool on thalamus or cortex (maybe?).
    _work_thread: Option<JoinHandle<()>>,

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
            settings: Option<CorticalAreaSettings>, thal: &mut Thalamus) -> CmnResult<CorticalArea>
    {
        // let emsg = "cortical_area::CorticalArea::new()";
        let area_id = area_map.area_id();
        let area_name = area_map.area_name();
        let settings = settings.unwrap_or(CorticalAreaSettings::new());

        println!("\n\nCORTICALAREA::NEW(): Creating Cortical Area: \"{}\"...", area_name);

        let build_options = if KERNEL_DEBUG_SYMBOLS && cfg!(target_os = "linux") {
            if ocl_context.platform()?.unwrap().vendor().contains("Intel") {
                panic!("[cortical_area::KERNEL_DEBUG_SYMBOLS == true]: \
                    Cannot debug kernels on an Intel based driver platform (not sure why).
                    Use the AMD platform drivers with Intel devices instead.");
            }
            // * TODO: Save kernel file for debugging on Intel.
            // // Optionally pass `-g` and `-s {cl path}` flags to compiler:
            // let debug_opts = format!("-g -s \"{}\"", kernel_path);
            let debug_opts = "-g";
            settings.add_build_options(area_map.gen_build_options().cmplr_opt(debug_opts))
        } else {
            settings.add_build_options(area_map.gen_build_options())
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

        let psal_name = area_map.layer_map().layers_containing_tags(LayerTags::PSAL)
            .first().map(|lyr| lyr.name());
        let ptal_name = area_map.layer_map().layers_containing_tags(LayerTags::PTAL)
            .first().map(|lyr| lyr.name());

        // Ensures if they are not set later the indexes will be invalid:
        let mut psal_idx = usize::max_value();
        let mut ptal_idx = usize::max_value();

        /*=============================================================================
        =============================== EXECUTION GRAPH ===============================
        =============================================================================*/

        let mut exe_graph = ExecutionGraph::new();

        /*=============================================================================
        ================================ CELLS & AXONS ================================
        =============================================================================*/

        // let mut mcols = None;
        let mut data_layers = Layers::new();
        let mut control_layers: BTreeMap<(LayerAddress, usize), Box<ControlCellLayer>> = BTreeMap::new();
        let mut axns = AxonSpace::new(&area_map, &ocl_pq, read_queue.clone(),
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

        for layer in area_map.layer_map().iter() {
            if let LayerKind::Cellular(ref cell_scheme) = *layer.kind() {
                match *cell_scheme.class() {
                    CellClass::Control {
                            kind: ControlCellKind::InhibitoryBasketSurround {
                            ref host_lyr_name, field_radius: _ }, exe_order, } =>
                    {
                        let host_lyr = data_layers.ssc_by_name(host_lyr_name)?;
                        let host_lyr_slc_ids = area_map.layer_slc_ids(&[host_lyr_name.clone()]);
                        let host_lyr_base_axn_slc = host_lyr_slc_ids[0];

                        let cc_lyr = InhibitoryInterneuronNetwork::new(layer.name(),
                            layer.layer_id(), cell_scheme.clone(),
                            host_lyr, host_lyr_base_axn_slc, &axns, &area_map, &ocl_pq,
                            settings.clone(), &mut exe_graph)?;

                        if control_layers.insert((host_lyr.layer_addr(), exe_order),
                                Box::new(cc_lyr)).is_some()
                        {
                            panic!("Duplicate control cell layer address / order index \
                                found for layer: {} ({})", layer.name(), exe_order);
                        };
                    },
                    CellClass::Control {
                            kind: ControlCellKind::ActivitySmoother {
                            ref host_lyr_name, field_radius: _ }, exe_order, } =>
                    {
                        let host_lyr = data_layers.ssc_by_name(host_lyr_name)?;
                        let host_lyr_slc_ids = area_map.layer_slc_ids(&[host_lyr_name.clone()]);
                        let host_lyr_base_axn_slc = host_lyr_slc_ids[0];

                        let cc_lyr = ActivitySmoother::new(layer.name(),
                            layer.layer_id(), cell_scheme.clone(),
                            host_lyr, host_lyr_base_axn_slc, &axns, &area_map, &ocl_pq,
                            settings.clone(), &mut exe_graph)?;

                        if control_layers.insert((host_lyr.layer_addr(), exe_order),
                                Box::new(cc_lyr)).is_some()
                        {
                            panic!("Duplicate control cell layer address / order index \
                                found for layer: {} ({})", layer.name(), exe_order);
                        }
                    },
                    CellClass::Control {
                            kind: ControlCellKind::PyrOutputter {
                            ref host_lyr_name }, exe_order, } =>
                    {
                        let host_lyr = data_layers.pyr_by_name(host_lyr_name)?;
                        let host_lyr_slc_ids = area_map.layer_slc_ids(&[host_lyr_name.clone()]);
                        let host_lyr_base_axn_slc = host_lyr_slc_ids[0];

                        let cc_lyr = PyrOutputter::new(layer.name(),
                            layer.layer_id(), cell_scheme.clone(),
                            host_lyr, host_lyr_base_axn_slc, &axns, &area_map, &ocl_pq,
                            settings.clone(), &mut exe_graph)?;

                        if control_layers.insert((host_lyr.layer_addr(), exe_order),
                                Box::new(cc_lyr)).is_some()
                        {
                            panic!("Duplicate control cell layer address / order index \
                                found for layer: {} ({})", layer.name(), exe_order);
                        }
                    },
                    _ => (),
                }
            }
        }

        // for layer in area_map.layer_map().iter() {
        //     match layer.kind() {
        //         &LayerKind::Cellular(ref cell_scheme) => {
        //             println!("{mt}::NEW(): making a(n) {:?} layer: '{}' (depth: {})",
        //                 cell_scheme.control_cell_kind(), layer.name(), layer.depth(), mt = cmn::MT);

        //             match cell_scheme.control_cell_kind() {
        //                 Some(&ControlCellKind::Complex) => {
        //                     let mcols_dims = dims.clone_with_depth(1);

        //                     mcols = Some(Box::new({
        //                         let sscs = data_layers.ssc_by_name(psal_name.unwrap())?;
        //                         // let pyrs = data_layers.pyr_by_name(ptal_name.unwrap())?;
        //                         // let mut sscs = Vec::with_capacity(8);
        //                         let mut temporal_pyrs: Vec<_> = data_layers.lyrs.iter()
        //                             .filter(|lyr| lyr.tags().contains(map::TEMPORAL) && lyr.is_pyramidal())
        //                             .map(|lyr| lyr.as_pyr_lyr().unwrap())
        //                             .collect();

        //                         let layer_id = layer.layer_id();
        //                         debug_assert!(area_map.aff_out_slcs().len() > 0, "CorticalArea::new(): \
        //                             No afferent output slices found for area: '{}'", area_name);
        //                         Minicolumns::new(layer_id, mcols_dims, &area_map, &axns, sscs,
        //                             temporal_pyrs,
        //                             &ocl_pq, settings.clone(), &mut exe_graph)?
        //                     }));
        //                 },
        //                 _ => (),
        //             }
        //         },
        //         _ => (),
        //     }
        // }

        // let mut mcols = mcols.expect("CorticalArea::new(): No Minicolumn layer found!");

        /*=============================================================================
        ===================================== AUX =====================================
        =============================================================================*/

        let aux = Aux::new(1 << 15, &ocl_pq);

        // <<<<< TODO: CLEAN THIS UP >>>>>
        // MAKE ABOVE LIKE BELOW (eliminate set_arg_buf_named() methods and just call directly on buffer)
        // mcols.set_arg_buf_named("aux_ints_0", &aux.ints_0).unwrap();

        // temporal_layers[ptal_idx]
        //     .set_arg_buf_named("aux_ints_0", &aux.ints_0).unwrap();
        // temporal_layers[ptal_idx]
        //     .set_arg_buf_named("aux_ints_1", &aux.ints_1).unwrap();

        // temporal_layers[ptal_idx].dens_mut()
        //     .set_arg_buf_named("aux_ints_0", &aux.ints_0).unwrap();
        // temporal_layers[ptal_idx].dens_mut()
        //     .set_arg_buf_named("aux_ints_1", &aux.ints_1).unwrap();

        // temporal_layers[ptal_idx].dens_mut().syns_mut()
        //     .set_arg_buf_named("aux_ints_0", &aux.ints_0).unwrap();
        // temporal_layers[ptal_idx].dens_mut().syns_mut()
        //     .set_arg_buf_named("aux_ints_1", &aux.ints_1).unwrap();


        // mcols.set_arg_buf_named("aux_ints_1", &aux.ints_0).unwrap();
        // pyrs_map.get_mut(ptal_name).unwrap().kern_ltp()
        //     .set_arg_buf_named("aux_ints_1", Some(&aux.ints_1)).unwrap();
        // pyrs_map.get_mut(ptal_name).unwrap().kern_cycle()
        //     .set_arg_buf_named("aux_ints_1", Some(&aux.ints_1)).unwrap();

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
        =========================== WORK COMPLETION THREAD ============================
        =============================================================================*/

        let (tx, rx) = mpsc::channel(0);
        let thread_name = format!("CorticalArea_{}", area_name.clone());

        // TODO: Move this to a centralized thread pool on thalamus or cortex (maybe?).
        let thread: JoinHandle<_> = thread::Builder::new().name(thread_name).spawn(move || {
            let rx = rx;
            let mut core = Core::new().unwrap();
            let work = rx.buffer_unordered(8).for_each(|_| Ok(()));
            core.run(work).unwrap();
        }).unwrap();

        /*=============================================================================
        ===============================================================================
        =============================================================================*/

        let mut cortical_area = CorticalArea {
            area_id: area_id,
            name: area_name,
            dims: dims,
            area_map: area_map,
            ptal_name: ptal_name,
            psal_name: psal_name,
            ptal_idx: ptal_idx,
            psal_idx: psal_idx,
            axns: axns,
            // mcols: mcols,
            data_layers,
            control_layers,
            aux: aux,
            ocl_pq: ocl_pq,
            write_queue: write_queue,
            read_queue: read_queue,
            counter: 0,
            settings: settings,
            cycle_order,
            exe_graph: exe_graph,
            work_tx: Some(tx),
            _work_thread: Some(thread),
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

        // (6.) Pyramidal Layers Learn:
        if !self.settings.disable_pyrs {
            for &lyr_idx in self.cycle_order.iter() {
                let lyr = self.data_layers.lyrs.get_mut(lyr_idx).unwrap();
                if !lyr.tags().contains(LayerTags::SPATIAL) {
                    lyr.set_exe_order_learn(&mut self.exe_graph)?;
                }
            }
        }

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
    pub fn cycle(&mut self, thal: &mut Thalamus) -> CmnResult<()> {
        // (1.) Axon Intake:
        self.axns.intake(thal, &mut self.exe_graph, self.settings.bypass_filters,
            &mut self.work_tx)?;

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

        // (6.) Pyramidal Layers Learn:
        if !self.settings.disable_pyrs {
            for &lyr_idx in self.cycle_order.iter() {
                let lyr = self.data_layers.lyrs.get_mut(lyr_idx).unwrap();
                if !lyr.tags().contains(LayerTags::SPATIAL) {
                    if !self.settings.disable_learning {
                        lyr.learn(&mut self.exe_graph)?;
                    }
                }
            }
        }

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
        self.axns.output(thal, &mut self.exe_graph, &mut self.work_tx)?;

        // (10.) Samplers:
        self.cycle_samplers()?;

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

    /// Cycles through sampling requests
    fn cycle_samplers(&mut self) -> CmnResult<()> {
        // let mut work_tx = self.work_tx.take();
        for sampler in &self.samplers {
            let cmd_idx = sampler.cmd_idx.expect("sampler order not set");

            // Check to see if we need to send this frame. `::wait` will only
            // block if sampler backpressure is on (and buffer is stale).
            if let Some(write_buf) = sampler.tx.send().wait()? {
                let mut new_event = Event::empty();
                match sampler.kind {
                    SamplerKind::AxonLayer(_) => {
                        debug_assert!(sampler.tx.buffer_idx_range().len() ==
                            sampler.src_idx_range.len());
                        let future_read = self.axns.states().cmd().read(write_buf.write_u8())
                            .offset(sampler.src_idx_range.start)
                            .len(sampler.src_idx_range.len())
                            .dst_offset(sampler.tx.buffer_idx_range().start)
                            .ewait(self.exe_graph.get_req_events(cmd_idx)?)
                            .enew(&mut new_event)
                            .enq_async()?
                            .map(|_guard| ())
                            .map_err(|err| panic!("{}", err));

                        let wtx = self.work_tx.take().unwrap();
                        self.work_tx.get_or_insert(wtx.send(Box::new(future_read)).wait()?);
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

    /// Requests a cortical 'sampler' which provides external read/write
    /// access to cortical cells and axons.
    pub fn sampler(&mut self, kind: SamplerKind, buffer_kind: SamplerBufferKind) -> TractReceiver {
        use ocl::RwVec;
        use map::{CommandRelations, CorticalBuffer};

        match kind {
            SamplerKind::AxonLayer(layer_id) => {
                let slc_range = match layer_id {
                    Some(lyr_id) => {
                        self.area_map.layer_map().layer_info(lyr_id)
                            .expect(&format!("CorticalArea::sample: Invalid layer: [id:{}]", lyr_id))
                            .slc_range()
                            .expect(&format!("CorticalArea::sample: Layer [id:{}] has no slices", lyr_id))
                            .clone()
                    },
                    None => 0..self.area_map.slice_map().depth() as usize,
                };
                let axn_range = self.area_map.slice_map().axn_range(slc_range.clone());
                match buffer_kind {
                    SamplerBufferKind::Single => {
                        let tract_buffer = RwVec::from(vec![0u8; axn_range.len()]);
                        let (tx, rx) = subcortex::tract_channel_single_u8(tract_buffer,
                            0..axn_range.len(), false);

                        // Determine source axon slices for execution graph:
                        let cmd_srcs = slc_range.map(|slc_id| {
                            CorticalBuffer::axon_slice(self.axns.states(), self.area_id, slc_id as u8)
                        }).collect();
                        // Add command to graph and get uid:
                        self.exe_graph.unlock();
                        let cmd_uid = self.exe_graph.add_command(
                            CommandRelations::corticothalamic_read(cmd_srcs, vec![]))
                            .expect("CorticalArea::sampler: Error adding exe. graph command");
                        // Create and push sampler:
                        let sampler = Sampler::new(kind.clone(), axn_range, tx, cmd_uid);
                        self.samplers.push(sampler);
                        // Repopulate execution graph:
                        self.order().expect("CorticalArea::sampler: Error reordering");
                        rx
                    },
                    _ => unimplemented!(),
                }
            },
            SamplerKind::Dummy => {
                let axn_range = 0..1;
                let (_tx, rx) = match buffer_kind {
                    SamplerBufferKind::Single => {
                        let tract_buffer = RwVec::from(vec![0i8; axn_range.len()]);
                        subcortex::tract_channel_single_i8(tract_buffer, 0..axn_range.len(), true)
                    },
                    _ => unimplemented!(),
                };
                rx
            }
        }
    }

    /// Blocks until all previously queued OpenCL commands in all
    /// command-queues are issued to the associated device.
    pub fn flush_queues(&self) {
        self.write_queue.flush().unwrap();
        self.ocl_pq.queue().flush().unwrap();
        self.read_queue.flush().unwrap();
    }

    /// Blocks until all previously queued OpenCL commands in all
    /// command-queues are issued to the associated device and have completed.
    pub fn finish_queues(&self) {
        self.write_queue.finish().unwrap();
        self.ocl_pq.queue().finish().unwrap();
        self.read_queue.finish().unwrap();
        self.exe_graph.finish().unwrap();
    }

    #[deprecated]
    pub fn sample_axn_slc_range(&self, slc_range: Range<usize>, buf: &mut [u8]) -> Event {
        // let slc_range = slc_range.borrow();
        assert!(slc_range.len() > 0, "CorticalArea::sample_axn_slc_range(): \
            Invalid slice range: '{:?}'. Slice range length must be at least one.", slc_range);
        // let axn_range_start = self.area_map.slice_map().axn_range(slc_range.start).start;
        // let axn_range_end = self.area_map.slice_map().axn_range(slc_range.end - 1).end;
        // let axn_range = axn_range_start..axn_range_end;

        let axn_range = self.area_map.slice_map().axn_range(slc_range.clone());

        debug_assert!(buf.len() == axn_range.len(), "Sample buffer length ({}) not \
            equal to slice axon length({}). axn_range: {:?}, slc_range: {:?}",
            buf.len(), axn_range.len(), axn_range, slc_range);
        let mut event = Event::empty();

        self.finish_queues();

        self.axns.states().cmd().read(buf).offset(axn_range.start).enew(&mut event).enq().unwrap();
        event
    }

    #[deprecated]
    pub fn sample_axn_space(&self, buf: &mut [u8]) -> Event {
        debug_assert!(buf.len() == self.area_map.slice_map().axn_count() as usize);
        let mut event = Event::empty();

        self.finish_queues();

        self.axns.states().read(buf).enew(&mut event).enq().expect("[FIXME]: HANDLE ME!");
        event
    }

    /// Returns an immutable reference to the requested spiny stellate cell
    /// layer.
    ///
    /// This does a linear search through all layers.
    pub fn ssc_layer(&self, layer_name: &'static str) -> CmnResult<&SpinyStellateLayer> {
        self.data_layers.ssc_by_name(layer_name)
    }

    /// Returns a mutable reference to the requested spiny stellate cell
    /// layer.
    ///
    /// This does a linear search through all layers.
    pub fn ssc_layer_mut(&mut self, layer_name: &'static str) -> CmnResult<&mut SpinyStellateLayer> {
        self.data_layers.ssc_by_name_mut(layer_name)
    }

    /// Returns an immutable reference to the requested pyramidal cell layer.
    ///
    /// This does a linear search through all layers.
    pub fn pyr_layer(&self, layer_name: &'static str) -> CmnResult<&PyramidalLayer> {
        self.data_layers.pyr_by_name(layer_name)
    }

    /// Returns a mutable reference to the requested pyramidal cell layer.
    ///
    /// This does a linear search through all layers.
    pub fn pyr_layer_mut(&mut self, layer_name: &'static str) -> CmnResult<&mut PyramidalLayer> {
        self.data_layers.pyr_by_name_mut(layer_name)
    }

    // #[inline] pub fn mcols(&self) -> &Box<Minicolumns> { &self.mcols }
    // #[inline] pub fn mcols_mut(&mut self) -> &mut Box<Minicolumns> { &mut self.mcols }
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
        self.work_tx.take().unwrap().close().unwrap();
        self._work_thread.take().unwrap().join().unwrap();
        print!("Releasing OpenCL components for '{}'... ", &self.name);
        print!("[ Buffers ][ Event Lists ][ Program ][ Command Queues ]");
        print!(" ...complete. \n");
    }
}


pub struct Aux {
    pub ints_0: Buffer<i32>,
    pub ints_1: Buffer<i32>,
    // pub chars_0: Buffer<ocl::i8>,
    // pub chars_1: Buffer<ocl::i8>,
}

impl Aux {
    pub fn new(len: usize, ocl_pq: &ProQue) -> Aux {
        let ints_0 = Buffer::<i32>::builder()
            .queue(ocl_pq.queue().clone())
            .dims(len)
            .fill_val(i32::min_value())
            .build().unwrap();

        let ints_1 = Buffer::<i32>::builder()
            .queue(ocl_pq.queue().clone())
            .dims(len)
            .fill_val(i32::min_value())
            .build().unwrap();

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
        // fn psal(&self) -> Option<&SpinyStellateLayer>;
        // fn psal_mut(&mut self) -> Option<&mut SpinyStellateLayer>;
        // fn ptal(&self) -> Option<&PyramidalLayer>;
        // fn ptal_mut(&mut self) -> Option<&mut PyramidalLayer>;
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

        // // PIL(): Get Primary Spatial Associative Layer (immutable)
        // fn psal(&self) -> Option<&SpinyStellateLayer> {
        //     // Some(&self.spatial_layers[self.psal_idx])
        //     None
        // }

        // // PIL_MUT(): Get Primary Spatial Associative Layer (mutable)
        // fn psal_mut(&mut self) -> Option<&mut SpinyStellateLayer> {
        //     // Some(&mut self.spatial_layers[self.psal_idx])
        //     None
        // }

        // // PAL(): Get Primary Temporal Associative Layer (immutable)
        // fn ptal(&self) -> Option<&PyramidalLayer> {
        //     // Some(&self.temporal_layers[self.ptal_idx])
        //     None
        // }

        // // PAL_MUT(): Get Primary Temporal Associative Layer (mutable)
        // fn ptal_mut(&mut self) -> Option<&mut PyramidalLayer> {
        //     // Some(&mut self.temporal_layers[self.ptal_idx])
        //     None
        // }



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

