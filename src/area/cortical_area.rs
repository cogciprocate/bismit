use std::collections::HashMap;
use std::ops::Range;
use std::borrow::Borrow;

use cmn::{self, CorticalDims, DataCellLayer};
use map::{self, AreaMap, LayerTags, SliceTractMap};
use ocl::{ProQue, Context, Buffer, EventList, Event};
use ocl::core::ClWaitList;
use map::{DendriteKind, LayerKind, CellKind};
use thalamus::Thalamus;
use area::{AxonSpace, Minicolumns, InhibitoryInterneuronNetwork, PyramidalLayer,
    SpinyStellateLayer, SensoryFilter};

#[cfg(test)] pub use self::tests::{CorticalAreaTest};

// GDB debug mode:
const KERNEL_DEBUG_MODE: bool = true;
// const DEBUG_PRINT: bool = false;

pub type CorticalAreas = HashMap<&'static str, Box<CorticalArea>>;


/// Information needed to read from and write to the thalamus for a layer
/// uniquely identified by `tags`.
///
/// [TODO]: Convert area_name to a numeric id:
///
#[derive(Debug)]
pub struct IoLayerInfo {
    tract_key: (String, LayerTags),
    axn_range: Range<u32>,
}

impl IoLayerInfo {
    pub fn new(src_area_name: String, tags: LayerTags, axn_range: Range<u32>) -> IoLayerInfo {
        IoLayerInfo {
            tract_key: (src_area_name, tags),
            axn_range: axn_range
        }
    }

    pub fn key(&self) -> &(String, LayerTags) {
        &self.tract_key
    }

    pub fn axn_range(&self) -> Range<u32> {
        self.axn_range.clone()
    }

    #[allow(dead_code)]
    pub fn area_name<'a>(&'a self) -> &'a str {
        &self.tract_key.0
    }

    #[allow(dead_code)]
    pub fn tags<'a>(&'a self) -> LayerTags {
        self.tract_key.1
    }
}


/// A group of `IoLayerInfo` structs sharing a common set of `LayerTags`.
#[derive(Debug)]
pub struct IoLayerInfoGroup {
    layers: Vec<IoLayerInfo>,
}

impl IoLayerInfoGroup {
    pub fn new(area_map: &AreaMap, group_tags: LayerTags, tract_keys: Vec<(String, LayerTags)>)
            -> IoLayerInfoGroup
    {
        // Create a container for our i/o layer(s):
        let mut layers = Vec::<IoLayerInfo>::with_capacity(tract_keys.len());

        for (layer_area_name, src_layer_tags) in tract_keys.into_iter() {
            let local_layer_tags = if group_tags.contains(map::OUTPUT) {
                src_layer_tags
            } else {
                src_layer_tags.mirror_io()
            };

            let axn_range = match area_map.axn_range_meshing_tags(local_layer_tags) {
                Some(axn_range) => axn_range,
                None => panic!("IoLayerInfoCache::new(): Internal consistency error: \
                    tags: {}.", local_layer_tags),
            };

            let io_layer = IoLayerInfo::new(layer_area_name, src_layer_tags, axn_range);
            layers.push(io_layer);
        }

        IoLayerInfoGroup {
            layers: layers,
        }
    }

    pub fn layers(&self) -> &[IoLayerInfo] {
        self.layers.as_slice()
    }

    pub fn layers_mut(&mut self) -> &mut [IoLayerInfo] {
        self.layers.as_mut_slice()
    }
}


/// A collection of all of the information needed to read from and write to
/// i/o layers via the thalamus.
#[derive(Debug)]
pub struct IoLayerInfoCache {
    groups: HashMap<LayerTags, (IoLayerInfoGroup, EventList)>,
}

impl IoLayerInfoCache {
    pub fn new(area_name: String, area_map: &AreaMap) -> IoLayerInfoCache {
        let group_tags_list: [LayerTags; 6] = [
            map::FF_IN, map::FB_IN, map::NS_IN,
            map::FF_OUT, map::FB_OUT, map::NS_OUT
        ];

        let mut groups = HashMap::with_capacity(group_tags_list.len());

        for &group_tags in group_tags_list.iter() {
            // If the layer is an output layer, consult the layer info
            // directly. If an input layer, consult the layer source info for
            // that layer. Either way, construct a tuple of '(area_name,
            // layer_tags)' which can be used to construct a key to access the
            // correct thalamic tract:
            let tract_keys: Vec<(String, LayerTags)> = if group_tags.contains(map::OUTPUT) {
                area_map.layers().layers_containing_tags(group_tags).iter()
                    .map(|li| (area_name.clone(), li.tags())).collect()
            } else {
                debug_assert!(group_tags.contains(map::INPUT));
                area_map.layers().layers_containing_tags_src_layers(group_tags).iter()
                    .map(|sli| (sli.area_name().to_owned(), sli.tags())) .collect()
            };

            // If there was nothing in the area map for this group's tags,
            // continue to the next set of tags in the `group_tags_list`:
            if tract_keys.len() != 0 {
                let io_lyr_grp = IoLayerInfoGroup::new(area_map, group_tags,
                    tract_keys);
                groups.insert(group_tags, (io_lyr_grp, EventList::new()));
            }
        }

        groups.shrink_to_fit();

        IoLayerInfoCache {
            groups: groups,
        }
    }

    pub fn group(&self, group_tags: LayerTags) -> Option<(&[IoLayerInfo], &EventList)> {
        self.groups.get(&group_tags)
            .map(|&(ref lg, ref events)| (lg.layers(), events))
    }

    pub fn group_mut(&mut self, group_tags: LayerTags) -> Option<(&mut [IoLayerInfo], &mut EventList)> {
        self.groups.get_mut(&group_tags)
            .map(|&mut (ref mut lg, ref mut events)| (lg.layers_mut(), events))
    }

    #[allow(dead_code)]
    pub fn group_info(&self, group_tags: LayerTags) -> Option<&[IoLayerInfo]> {
        self.groups.get(&group_tags).map(|&(ref lg, _)| lg.layers())
    }

    #[allow(dead_code)]
    pub fn group_info_mut(&mut self, group_tags: LayerTags) -> Option<&mut [IoLayerInfo]> {
        self.groups.get_mut(&group_tags).map(|&mut (ref mut lg, _)| lg.layers_mut())
    }

    #[allow(dead_code)]
    pub fn group_events(&self, group_tags: LayerTags) -> Option<&EventList> {
        self.groups.get(&group_tags).map(|&(_, ref events)| events)
    }

    #[allow(dead_code)]
    pub fn group_events_mut(&mut self, group_tags: LayerTags) -> Option<&mut EventList> {
        self.groups.get_mut(&group_tags).map(|&mut (_, ref mut events)| events)
    }
}

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
    name: &'static str,
    dims: CorticalDims,
    area_map: AreaMap,
    axns: AxonSpace,
    mcols: Box<Minicolumns>,
    pyrs_map: HashMap<&'static str, Box<PyramidalLayer>>,        // MAKE ME PRIVATE -- FIX tests::hybrid
    ssts_map: HashMap<&'static str, Box<SpinyStellateLayer>>,    // MAKE ME PRIVATE -- FIX tests::hybrid
    iinns: HashMap<&'static str, Box<InhibitoryInterneuronNetwork>>,    // MAKE ME PRIVATE -- FIX tests::hybrid
    filters: Option<Vec<Box<SensoryFilter>>>,
    ptal_name: &'static str,    // PRIMARY TEMPORAL ASSOCIATIVE LAYER NAME
    psal_name: &'static str,    // PRIMARY SPATIAL ASSOCIATIVE LAYER NAME
    // aux: Aux,
    ocl_pq: ProQue,
    // ocl_context: Context,
    // renderer: Renderer,
    counter: usize,
    // rng: rand::XorShiftRng,
    // thal_gangs: ThalamicGanglions,
    // events_lists: HashMap<LayerTags, EventList>,
    io_info: IoLayerInfoCache,
    settings: CorticalAreaSettings,
}

impl CorticalArea {
    pub fn new(area_map: AreaMap, device_idx: usize, ocl_context: &Context,
                    settings: Option<CorticalAreaSettings>) -> CorticalArea {
        let emsg = "cortical_area::CorticalArea::new()";
        let area_name = area_map.area_name();

        println!("\n\nCORTICALAREA::NEW(): Creating Cortical Area: \"{}\"...", area_name);

        // Optionally pass `-g` and `-s {cl path}` flags to compiler:
        let build_options = if KERNEL_DEBUG_MODE && cfg!(target_os = "linux") {
            // [TODO]: Add something to identify the platform vendor and match:
            // let debug_opts = format!("-g -s {}", cmn::cl_root_path().join("bismit.cl").to_str()
            let debug_opts = "-g";

            area_map.gen_build_options().cmplr_opt(debug_opts)
        } else {
            area_map.gen_build_options()
        };

        let ocl_pq = ProQue::builder()
            .device(device_idx)
            .context(ocl_context.clone())
            .prog_bldr(build_options)
            .build().expect("CorticalArea::new(): ocl_pq.build(): error");

        let dims = area_map.dims().clone_with_incr(ocl_pq.max_wg_size());

        println!("{mt}CORTICALAREA::NEW(): Area \"{}\" details: \
            (u_size: {}, v_size: {}, depth: {}), eff_areas: {:?}, aff_areas: {:?}, \n\
            {mt}{mt}device_idx: [{}], device.name(): {}, device.vendor(): {}",
            area_name, dims.u_size(), dims.v_size(), dims.depth(), area_map.eff_areas(),
            area_map.aff_areas(), device_idx, ocl_pq.device().name().trim(),
            ocl_pq.device().vendor().trim(), mt = cmn::MT);

        let psal_name = area_map.layer_name_by_tags(map::SPATIAL_ASSOCIATIVE);
        let ptal_name = area_map.layer_name_by_tags(map::TEMPORAL_ASSOCIATIVE);

            /* <<<<< BRING BACK UPDATED VERSIONS OF BELOW >>>>> */
        //assert!(SYNAPSES_PER_DENDRITE_PROXIMAL_LOG2 >= 2);
        //assert!(SYNAPSES_PER_DENDRITE_DISTAL_LOG2 >= 2);
        //assert!(DENDRITES_PER_CELL_DISTAL_LOG2 <= 8);
        //assert!(DENDRITES_PER_CELL_DISTAL <= 256);
        //assert!(DENDRITES_PER_CELL_PROXIMAL_LOG2 == 0);
        //assert!(depth_cellular > 0, "cortical_area::CorticalArea::new(): Region has no cellular layers.");

        let axns = AxonSpace::new(&area_map, &ocl_pq);

        let mut pyrs_map = HashMap::new();
        let mut ssts_map = HashMap::new();
        let mut iinns = HashMap::new();


        /*=============================================================================
        ================================== DATA CELLS =================================
        =============================================================================*/
        // BREAK OFF THIS CODE INTO NEW STRUCT DEF

        for layer in area_map.layers().iter() {
            match layer.kind() {
                &LayerKind::Cellular(ref pcell) => {
                    println!("{mt}::NEW(): making a(n) {:?} layer: '{}' (depth: {})",
                        pcell.cell_kind, layer.name(), layer.depth(), mt = cmn::MT);

                    match pcell.cell_kind {
                        CellKind::Pyramidal => {
                            let pyrs_dims = dims.clone_with_depth(layer.depth());

                            let pyr_lyr = PyramidalLayer::new(
                                layer.name(), pyrs_dims, pcell.clone(), &area_map, &axns, /*&aux,*/ &ocl_pq);

                            pyrs_map.insert(layer.name(), Box::new(pyr_lyr));
                        },

                        CellKind::SpinyStellate => {
                            let ssts_map_dims = dims.clone_with_depth(layer.depth());
                            let sst_lyr = SpinyStellateLayer::new(
                                layer.name(), ssts_map_dims, pcell.clone(), &area_map, &axns, /*&aux,*/ &ocl_pq);
                            ssts_map.insert(layer.name(), Box::new(sst_lyr));
                        },
                        _ => (),
                    }
                },
                _     => (),    /*println!("{mt}::NEW(): Axon layer: '{}' (depth: {})",
                            layer.name(), layer.depth(), mt = cmn::MT),*/
            }
        }


        /*=============================================================================
        ================================ CONTROL CELLS ================================
        =============================================================================*/
        // BREAK OFF THIS CODE INTO NEW STRUCT DEF

        for layer in area_map.layers().iter() {
            match layer.kind() {
                &LayerKind::Cellular(ref pcell) => {
                    match pcell.cell_kind {
                        CellKind::Inhibitory => {
                            let src_lyr_names = layer.src_lyr_names(DendriteKind::Distal);
                            assert!(src_lyr_names.len() == 1);

                            let src_lyr_name = src_lyr_names[0];
                            let src_slc_ids = area_map.layer_slc_ids(vec![src_lyr_name]);
                            let src_layer_depth = src_slc_ids.len() as u8;
                            let src_base_axn_slc = src_slc_ids[0];

                            // println!("{mt}::NEW(): Inhibitory cells: src_lyr_names: \
                            //     {:?}, src_base_axn_slc: {:?}", src_lyr_names, src_base_axn_slc,
                            //     mt = cmn::MT);

                            let em1 = format!("{}: '{}' is not a valid layer", emsg, src_lyr_name);
                            let src_soma_env = &ssts_map.get_mut(src_lyr_name).expect(&em1).soma();

                            let iinns_dims = dims.clone_with_depth(src_layer_depth);
                            let iinn_lyr = InhibitoryInterneuronNetwork::new(layer.name(), iinns_dims,
                                pcell.clone(), &area_map, src_soma_env,
                                src_base_axn_slc, &axns, /*&aux,*/ &ocl_pq);

                            iinns.insert(layer.name(), Box::new(iinn_lyr));

                        },
                        _ => (),
                    }
                },
                _ => (),
            }
        }


        let mcols_dims = dims.clone_with_depth(1);

        // <<<<< EVENTUALLY ADD TO CONTROL CELLS (+PROTOCONTROLCELLS) >>>>>
        let mut mcols = Box::new({
            //let em_ssts = emsg.to_string() + ": ssts - em2";
            let em_ssts = format!("{}: '{}' is not a valid layer", emsg, psal_name);
            let ssts = ssts_map.get(psal_name).expect(&em_ssts);

            let em_pyrs = format!("{}: '{}' is not a valid layer", emsg, ptal_name);
            let pyrs = pyrs_map.get(ptal_name).expect(&em_pyrs);

            debug_assert!(area_map.aff_out_slcs().len() > 0, "CorticalArea::new(): \
                No afferent output slices found for area: '{}'", area_name);
            Minicolumns::new(mcols_dims, &area_map, &axns, ssts, pyrs, /*&aux,*/ &ocl_pq)
        });


        /*=============================================================================
        =================================== FILTERS ===================================
        =============================================================================*/
        // BREAK OFF THIS CODE INTO NEW STRUCT DEF

        // <<<<< CHANGE TO LAYER**S**_WITH_FLAG() >>>>>
        let filters = {
            let mut filters_vec = Vec::with_capacity(5);

            match area_map.filters() {
                &Some(ref filter_schemes) => {
                    for pf in filter_schemes.iter() {
                        filters_vec.push(Box::new(SensoryFilter::new(
                            pf.filter_name(),
                            pf.cl_file_name(),
                            &area_map,
                            &axns,
                            &ocl_pq
                        )));
                    }
                    Some(filters_vec)
                },
                &None => None,
            }
        };

        // let renderer = Renderer::new(&dims);

        let aux = Aux::new(pyrs_map[ptal_name].dens().syns().dims(), &ocl_pq);

        // <<<<< TODO: CLEAN THIS UP >>>>>
        // MAKE ABOVE LIKE BELOW (eliminate set_arg_buf_named() methods and just call directly on buffer)
        mcols.set_arg_buf_named("aux_ints_0", &aux.ints_0).unwrap();
        pyrs_map.get_mut(ptal_name).unwrap()
            .set_arg_buf_named("aux_ints_0", &aux.ints_0).unwrap();
        pyrs_map.get_mut(ptal_name).unwrap().dens_mut().syns_mut()
            .set_arg_buf_named("aux_ints_0", &aux.ints_0).unwrap();

        // mcols.set_arg_buf_named("aux_ints_1", &aux.ints_0).unwrap();
        pyrs_map.get_mut(ptal_name).unwrap().kern_ltp()
            .set_arg_buf_named("aux_ints_1", Some(&aux.ints_1)).unwrap();
        pyrs_map.get_mut(ptal_name).unwrap().kern_cycle()
            .set_arg_buf_named("aux_ints_1", Some(&aux.ints_1)).unwrap();

        // pyrs_map.get_mut(ptal_name).unwrap().dens_mut().syns_mut()
            // .set_arg_buf_named("aux_ints_1", &aux.ints_0).unwrap();
        // let mut events_lists = HashMap::new();
        // events_lists.insert(map::FF_IN, EventList::new());
        // events_lists.insert(map::FB_IN, EventList::new());
        // events_lists.insert(map::NS_IN, EventList::new());
        // events_lists.insert(map::FF_OUT, EventList::new());
        // events_lists.insert(map::NS_OUT, EventList::new());

        let io_info = IoLayerInfoCache::new(area_name.to_owned(), &area_map);

        println!("    CORTICAL_AREA::NEW(): IO_INFO: {:?}, Settings: {:?}", io_info, settings);

        let settings = settings.unwrap_or(CorticalAreaSettings::new());

        let cortical_area = CorticalArea {
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
            filters: filters,
            // aux: aux,
            ocl_pq: ocl_pq,
            // ocl_context: ocl_context,
            // renderer: renderer,
            counter: 0,
            // rng: rand::weak_rng(),
            // events_lists: events_lists,
            io_info: io_info,
            settings: settings,
        };

        cortical_area
    }

    // CYCLE(): <<<<< TODO: ISOLATE LEARNING INTO SEPARATE THREAD >>>>>
    pub fn cycle(&mut self, thal: &mut Thalamus) {
        let emsg = format!("cortical_area::CorticalArea::cycle(): Invalid layer.");

        self.intake(map::FF_IN, thal);
        self.intake(map::NS_IN, thal);

        if !self.settings.disable_ssts {
            let aff_input_events = { self.io_info.group_events(map::FF_IN).map(|wl| wl as &ClWaitList) };
            self.psal().cycle(aff_input_events);
        }

        self.iinns.get_mut("iv_inhib").expect(&emsg).cycle(self.settings.bypass_inhib);

        if !self.settings.disable_ssts { if !self.settings.disable_learning { self.psal_mut().learn(); } }

        if !self.settings.disable_mcols { self.mcols.activate(); }

        self.intake(map::FB_IN, thal);

        if !self.settings.disable_pyrs {
            if !self.settings.disable_learning { self.ptal_mut().learn(); }
            let eff_input_events = { self.io_info.group_events(map::FB_IN).map(|wl| wl as &ClWaitList) };
            self.ptal().cycle(eff_input_events);
        }

        if !self.settings.disable_mcols {
            let output_events = { self.io_info.group_events_mut(map::FF_OUT) };
            self.mcols.output(output_events);
        }

        if !self.settings.disable_regrowth { self.regrow(); }

        self.output(map::FF_OUT, thal);
    }

    /// Read input from thalamus and write to axon space.
    ///
    /// [FIXME]: Currently cloning each list of keys (with strings inside).
    /// This is bad on a couple of levels. Generate a list of keys upon
    /// creation for each category of layer tags THEN convert the `(String,
    /// LayerTags)` keys into `(usize, LayerTags)`. [UPDATE]: Need now only
    /// convert the strings to ints.
    ///
    fn intake(&mut self, group_tags: LayerTags, thal: &mut Thalamus) {
        if let Some((src_layers, new_events)) = self.io_info.group_mut(group_tags) {
            new_events.clear_completed().expect("CorticalArea::write_input");

            for src_layer in src_layers.iter_mut() {
                let (wait_events, sdr) = thal.tract(src_layer.key())
                    .expect("CorticalArea::intake()");

                if group_tags.contains(map::FF_IN) && self.filters.is_some()
                        && !self.settings.bypass_filters
                {
                    let filters_vec = self.filters.as_ref().unwrap();
                    let mut fltr_event = filters_vec[0].write(sdr.frame(), wait_events);

                    for fltr in filters_vec.iter() {
                        fltr_event = fltr.cycle(&fltr_event);
                    }
                } else {
                    let axn_range = src_layer.axn_range();
                    assert!(sdr.len() == axn_range.len() as usize,
                        "CorticalArea::intake(): Sdr/ganglion length must be \
                        equal to the destination axon range. sdr.len(): {} != axn_range.len(): \
                        {}, (area: '{}', layer_tags: '{}', range: '{:?}').", sdr.len(),
                        axn_range.len(), self.name, src_layer.tags(), axn_range);

                    self.axns.states.cmd().write(sdr.frame()).offset(axn_range.start as usize)
                        .block(false).ewait(wait_events).enew(new_events).enq().unwrap();
                }
            }
        }
    }

    // Read output from axon space and write to thalamus.
    fn output(&self, group_tags: LayerTags, thal: &mut Thalamus) {
        if let Some((src_layers, wait_events)) = self.io_info.group(group_tags) {
            for src_layer in src_layers.iter() {
                let (mut sdr, new_events) = thal.tract_mut(src_layer.key())
                    .expect("CorticalArea::output()");

                new_events.clear_completed().expect("CorticalArea::write_input");
                let axn_range = src_layer.axn_range();

                assert!(sdr.dims().to_len() == axn_range.len() as usize,
                    "CorticalArea::output(): Sdr/ganglion length must be \
                    equal to the source axon range. sdr.len(): {} != axn_range.len(): \
                    {}, (area: '{}', layer_tags: '{}', range: '{:?}').", sdr.len(),
                    axn_range.len(), self.name, src_layer.tags(), axn_range);

                // let wait_events = &src_grp.events;

                unsafe { self.axns.states.cmd().read_async(sdr.frame_mut()).offset(axn_range.start as usize)
                    .block(false).ewait(wait_events).enew(new_events).enq().unwrap(); }
            }
        }
    }

    // // Read output from axon space and write to thalamus.
    // fn output_SDR_RANGE(&self, group_tags: LayerTags, thal: &mut Thalamus) {
    //     if let Some((src_layers, wait_events)) = self.io_info.group(group_tags) {
    //         for src_layer in src_layers.iter() {
    //             let (sdr, new_events) = thal.tract_frame_mut(src_layer.key())
    //                 .expect("CorticalArea::output()");

    //             new_events.clear_completed().expect("CorticalArea::write_input");
    //             let axn_range = src_layer.axn_range();

    //             assert!(sdr.len() == axn_range.len() as usize,
    //                 "CorticalArea::output(): Sdr/ganglion length must be \
    //                 equal to the source axon range. sdr.len(): {} != axn_range.len(): \
    //                 {}, (area: '{}', layer_tags: '{}', range: '{:?}').", sdr.len(),
    //                 axn_range.len(), self.name, src_layer.tags(), axn_range);

    //             // let wait_events = &src_grp.events;

    //             unsafe { self.axns.states.cmd().read_async(sdr).offset(axn_range.start as usize)
    //                 .block(false).ewait(wait_events).enew(new_events).enq().unwrap(); }
    //         }
    //     }
    // }

    pub fn regrow(&mut self) {
        if !self.settings.disable_regrowth {
            if self.counter >= cmn::SYNAPSE_REGROWTH_INTERVAL {
                //print!("$");
                self.ssts_map.get_mut(self.psal_name).expect("cortical_area.rs").regrow();
                self.ptal_mut().regrow();
                self.counter = 0;
            } else {
                self.counter += 1;
            }
        }
    }

    /* LAYER_INPUT_RANGES(): NEEDS UPDATE / REMOVAL */
    pub fn layer_input_ranges(&self, layer_name: &'static str, den_kind: &DendriteKind)
            -> Vec<Range<u32>>
    {
        let mut axn_irs: Vec<Range<u32>> = Vec::with_capacity(10);
        let src_slc_ids = self.area_map.layer_src_slc_ids(layer_name, *den_kind);

        for ssid in src_slc_ids {
            let idz = self.area_map.axn_idz(ssid);
             let idn = idz + self.dims.columns();
            axn_irs.push(idz..idn);
        }

        axn_irs
    }

    pub fn mcols(&self) -> &Box<Minicolumns> {
        &self.mcols
    }

    pub fn mcols_mut(&mut self) -> &mut Box<Minicolumns> {
        &mut self.mcols
    }

    /* PIL(): Get Primary Spatial Associative Layer (immutable) */
    pub fn psal(&self) -> &Box<SpinyStellateLayer> {
        let e_string = "cortical_area::CorticalArea::psal(): Primary Spatial Associative Layer: '{}' not found. ";
        self.ssts_map.get(self.psal_name).expect(e_string)
    }

    /* PIL_MUT(): Get Primary Spatial Associative Layer (mutable) */
    pub fn psal_mut(&mut self) -> &mut Box<SpinyStellateLayer> {
        let e_string = "cortical_area::CorticalArea::psal_mut(): Primary Spatial Associative Layer: '{}' not found. ";
        self.ssts_map.get_mut(self.psal_name).expect(e_string)
    }

    /* PAL(): Get Primary Temporal Associative Layer (immutable) */
    pub fn ptal(&self) -> &Box<PyramidalLayer> {
        let e_string = "cortical_area::CorticalArea::ptal(): Primary Temporal Associative Layer: '{}' not found. ";
        self.pyrs_map.get(self.ptal_name).expect(e_string)
    }

    /* PAL_MUT(): Get Primary Temporal Associative Layer (mutable) */
    pub fn ptal_mut(&mut self) -> &mut Box<PyramidalLayer> {
        let e_string = "cortical_area::CorticalArea::ptal_mut(): Primary Temporal Associative Layer: '{}' not found. ";
        self.pyrs_map.get_mut(self.ptal_name).expect(e_string)
    }

    pub fn axns(&self) -> &AxonSpace {
        &self.axns
    }

    pub fn dims(&self) -> &CorticalDims {
        &self.dims
    }

    pub fn psal_name(&self) -> &'static str {
        self.psal_name
    }

    pub fn ptal_name(&self) -> &'static str {
        self.ptal_name
    }

    pub fn afferent_target_names(&self) -> &Vec<&'static str> {
        &self.area_map.aff_areas()
    }
    pub fn efferent_target_names(&self) -> &Vec<&'static str> {
        &self.area_map.eff_areas()
    }

    pub fn ocl_pq(&self) -> &ProQue {
        &self.ocl_pq
    }

    // // TODO: MOVE TO TESTS
    // pub fn render_aff_out(&mut self, input_status: &str, print_summary: bool) {
    //     let out_axns = &self.axns.states[self.mcols.aff_out_axn_range()];
    //     let sst_axns = &self.axns.states[self.psal().axn_range()];
    //     self.renderer.render(out_axns, Some(sst_axns), None, input_status, print_summary);
    // }

    // // TODO: MOVE TO TESTS
    // pub fn render_axn_space(&mut self) {
    //     let axn_states = &self.axns.states[..];
    //     self.renderer.render_axn_space(axn_states, &self.area_map.slices())
    // }

    /// [FIXME]: Currnently assuming aff out slice is == 1. Ascertain the
    /// slice range correctly by consulting area_map.layers().
    pub fn sample_aff_out(&self, buf: &mut [u8]) {
        // let aff_out_range = self.mcols.aff_out_axn_range();
        // debug_assert!(buf.len() == aff_out_range.len());
        // self.axns.states.enqueue_read(buf, aff_out_range.start, None, None);
        let aff_out_slc = self.mcols.aff_out_axn_slc();
        self.sample_axn_slc_range(aff_out_slc..(aff_out_slc + 1), buf);
    }

    // pub fn sample_axn_slc(&self, slc_id: u8, buf: &mut [u8]) -> Event {
    //     let axn_range = self.area_map.slices().axn_range(slc_id);
    //     debug_assert!(buf.len() == axn_range.len(), "Sample buffer length ({}) not \
    //         equal to slice axon length({}). axn_range: {:?}, slc_id: {}",
    //         buf.len(), axn_range.len(), axn_range, slc_id);
    //     // self.axns.states.read(axn_range.start, buf).unwrap();
    //     let mut event = Event::empty();
    //     self.axns.states.cmd().read(buf).offset(axn_range.start).enew(&mut event).enq().unwrap();
    //     event
    // }

    pub fn sample_axn_slc_range<R: Borrow<Range<u8>>>(&self, slc_range: R, buf: &mut [u8])
            -> Event
    {
        let slc_range = slc_range.borrow();
        assert!(slc_range.len() > 0, "CorticalArea::sample_axn_slc_range(): \
            Invalid slice range: '{:?}'. Slice range length must be at least one.", slc_range);
        let axn_range_start = self.area_map.slices().axn_range(slc_range.start).start;
        let axn_range_end = self.area_map.slices().axn_range(slc_range.end - 1).end;
        let axn_range = axn_range_start..axn_range_end;

        debug_assert!(buf.len() == axn_range.len(), "Sample buffer length ({}) not \
            equal to slice axon length({}). axn_range: {:?}, slc_range: {:?}",
            buf.len(), axn_range.len(), axn_range, slc_range);
        // self.axns.states.read(axn_range.start, buf).unwrap();
        let mut event = Event::empty();
        self.axns.states.cmd().read(buf).offset(axn_range.start).enew(&mut event).enq().unwrap();
        event
    }

    pub fn sample_axn_space(&self, buf: &mut [u8]) -> Event {
        debug_assert!(buf.len() == self.area_map.slices().axn_count() as usize);
        let mut event = Event::empty();
        self.axns.states.read(buf).enew(&mut event).enq().expect("[FIXME]: HANDLE ME!");
        event
    }

    pub fn axn_tract_map(&self) -> SliceTractMap {
        self.area_map.slices().tract_map()
    }

    pub fn area_map(&self) -> &AreaMap {
        &self.area_map
    }
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

// #[allow(dead_code)]
// pub struct AreaParams {
//     den_per_cel_distal_l2: u8,
//     syn_per_den_distal_l2: u8,

//     den_per_cel_proximal: u8,
//     syn_per_den_proximal: u8,
// }

// pub struct ThalamicGanglions {
//     map: HashMap<LayerTags, GanglionInfo>,
// }

// pub struct GanglionInfo {
//     tract_range: Range<usize>,
//     axn_range: Range<usize>,
// }

const INT_32_MIN: i32 = -2147483648;

pub struct Aux {
    // dims: CorticalDims,
    pub ints_0: Buffer<i32>,
    pub ints_1: Buffer<i32>,
    // pub chars_0: Buffer<ocl::i8>,
    // pub chars_1: Buffer<ocl::i8>,
}

impl Aux {
    pub fn new(dims: &CorticalDims, ocl_pq: &ProQue) -> Aux {
        //let dims_multiplier: u32 = 512;
        //dims.columns() *= 512;
        let int_32_min = INT_32_MIN;

        let ints_0 = Buffer::<i32>::new(ocl_pq.queue(), None, dims, None).unwrap();
        ints_0.cmd().fill(int_32_min, None).enq().unwrap();
        let ints_1 = Buffer::<i32>::new(ocl_pq.queue(), None, dims, None).unwrap();
        ints_1.cmd().fill(int_32_min, None).enq().unwrap();

        Aux {
            ints_0: ints_0,
            ints_1: ints_1,
            // chars_0: Buffer::<ocl::i8>::new(dims, 0, ocl),
            // chars_1: Buffer::<ocl::i8>::new(dims, 0, ocl),
            // dims: dims.clone(),
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




#[cfg(test)]
pub mod tests {
    use rand;
    use rand::distributions::{IndependentSample, Range as RandRange};

    use super::*;
    use area::{AxonSpaceTest};
    use cmn::{CelCoords};
    use map::{AreaMapTest};

    pub trait CorticalAreaTest {
        fn axn_state(&self, idx: usize) -> u8;
        fn write_to_axon(&mut self, val: u8, idx: u32);
        fn read_from_axon(&self, idx: u32) -> u8;
        fn rand_safe_src_axn(&mut self, cel_coords: &CelCoords, src_axn_slc: u8
            ) -> (i8, i8, u32, u32);
        // fn print_aux(&mut self);
        // fn print_axns(&mut self);
        fn activate_axon(&mut self, idx: u32);
        fn deactivate_axon(&mut self, idx: u32);
    }

    impl CorticalAreaTest for CorticalArea {
        fn axn_state(&self, idx: usize) -> u8 {
            self.axns.axn_state(idx)
        }

        fn read_from_axon(&self, idx: u32) -> u8 {
            self.axns.axn_state(idx as usize)
        }

        fn write_to_axon(&mut self, val: u8, idx: u32) {
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

        // fn print_aux(&mut self) {
        //     print!("aux.ints_0: ");
        //     let view_radius = 1 << 24;
        //     self.aux.ints_0.print((1 << 0) as usize,
        //         Some((0 - view_radius, view_radius)), None, true);

        //     print!("aux.ints_1: ");
        //     self.aux.ints_1.print((1 << 0) as usize,
        //         Some((0 - view_radius, view_radius)), None, true);
        // }

        // fn print_axns(&mut self) {
        //     print!("axns: ");
        //     self.axns.states.print(1 << 0, Some((1, 255)), None, false);
        // }

        fn activate_axon(&mut self, idx: u32) {
            let mut rng = rand::weak_rng();
            let val = RandRange::new(1, 255).ind_sample(&mut rng);
            self.axns.write_to_axon(val, idx);
        }

        fn deactivate_axon(&mut self, idx: u32) {
            self.axns.write_to_axon(0, idx);
        }
    }
}

