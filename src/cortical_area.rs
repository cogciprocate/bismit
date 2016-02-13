use std::collections::{ HashMap };
use std::ops::{ Range };
use rand;

use cmn::{ self, ParaHexArray, CorticalDims, Renderer, Sdr, DataCellLayer };
use map::{ self, AreaMap, LayerTags, GanglionMap };
use ocl::{ self, ProQue, Context, Buffer, EventList };
use proto::{  Cellular, Pyramidal, SpinyStellate, Inhibitory,  DendriteKind };

use axon_space::{ AxonSpace };
use minicolumns::{ Minicolumns };
use iinn::{ InhibitoryInterneuronNetwork };
use pyramidals::{ PyramidalLayer };
use spiny_stellates::{ SpinyStellateLayer };
use sensory_filter::{ SensoryFilter };
use thalamus::{ Thalamus };

#[cfg(test)]
pub use self::tests::{ CorticalAreaTest };


pub type CorticalAreas = HashMap<&'static str, Box<CorticalArea>>;


pub struct CorticalArea {
    pub name: &'static str,
    pub dims: CorticalDims,
    area_map: AreaMap,
    pub axns: AxonSpace,
    pub mcols: Box<Minicolumns>,
    pub pyrs_map: HashMap<&'static str, Box<PyramidalLayer>>,        // MAKE ME PRIVATE -- FIX tests::hybrid
    pub ssts_map: HashMap<&'static str, Box<SpinyStellateLayer>>,    // MAKE ME PRIVATE -- FIX tests::hybrid
    pub iinns: HashMap<&'static str, Box<InhibitoryInterneuronNetwork>>,    // MAKE ME PRIVATE -- FIX tests::hybrid
    pub filters: Option<Vec<Box<SensoryFilter>>>,
    ptal_name: &'static str,    // PRIMARY TEMPORAL ASSOCIATIVE LAYER NAME
    psal_name: &'static str,    // PRIMARY SPATIAL ASSOCIATIVE LAYER NAME
    pub aux: Aux,
    ocl_pq: ProQue,
    // ocl_context: Context,
    renderer: Renderer,
    counter: usize,
    rng: rand::XorShiftRng,
    // thal_gangs: ThalamicGanglions,
    events_lists: HashMap<LayerTags, EventList>,
    pub bypass_inhib: bool,
    pub bypass_filters: bool,
    pub disable_pyrs: bool,
    pub disable_ssts: bool,
    pub disable_mcols: bool,
    pub disable_regrowth: bool,
    pub disable_learning: bool,
}

impl CorticalArea {
    pub fn new(area_map: AreaMap, device_idx: usize, ocl_context: &Context) -> CorticalArea {
        let emsg = "cortical_area::CorticalArea::new()";

        let area_name = area_map.area_name();    

        println!("\n\nCORTICALAREA::NEW(): Creating Cortical Area: \"{}\"...", area_name);        

        
        let mut ocl_pq: ocl::ProQue = ocl::ProQue::new(&ocl_context, Some(device_idx));

        ocl_pq.build_program(&area_map.gen_build_options())
            .expect("CorticalArea::new(): ocl_pq.build(): error");

        let dims = area_map.dims().clone_with_incr(ocl_pq.max_work_group_size());

        println!("{mt}CORTICALAREA::NEW(): Area \"{}\" details: \
            (u_size: {}, v_size: {}, depth: {}), eff_areas: {:?}, aff_areas: {:?}, device: {:?}", 
            area_name, dims.u_size(), dims.v_size(), dims.depth(), area_map.eff_areas(), 
            area_map.aff_areas(), ocl_pq.queue().device_id(), mt = cmn::MT);

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
                &Cellular(ref pcell) => {
                    // println!("{mt}::NEW(): making a(n) {:?} layer: '{}' (depth: {})", 
                    //     pcell.cell_kind, layer.name(), layer.depth(), mt = cmn::MT);

                    match pcell.cell_kind {
                        Pyramidal => {
                            let pyrs_dims = dims.clone_with_depth(layer.depth());

                            let pyr_lyr = PyramidalLayer::new(
                                layer.name(), pyrs_dims, pcell.clone(), &area_map, &axns, /*&aux,*/ &ocl_pq);

                            pyrs_map.insert(layer.name(), Box::new(pyr_lyr));
                        },

                        SpinyStellate => {                            
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
                &Cellular(ref pcell) => {
                    match pcell.cell_kind {
                        Inhibitory => {
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
                &Some(ref protofilters) => {
                    for pf in protofilters.iter() {
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

        let renderer = Renderer::new(&dims);

        let aux = Aux::new(pyrs_map[ptal_name].dens().syns().dims(), &ocl_pq);

        // <<<<< TODO: CLEAN THIS UP >>>>>
        // MAKE ABOVE LIKE BELOW (eliminate set_arg_buf_named() methods and just call directly on buffer)
        mcols.set_arg_buf_named("aux_ints_0", &aux.ints_0);
        pyrs_map.get_mut(ptal_name).unwrap().set_arg_buf_named("aux_ints_0", &aux.ints_0);
        pyrs_map.get_mut(ptal_name).unwrap().dens_mut().syns_mut()
            .set_arg_buf_named("aux_ints_0", &aux.ints_0);

        // mcols.set_arg_buf_named("aux_ints_1", &aux.ints_0);
        pyrs_map.get_mut(ptal_name).unwrap().kern_ltp().set_arg_buf_named("aux_ints_1", Some(&aux.ints_1));
        pyrs_map.get_mut(ptal_name).unwrap().kern_cycle().set_arg_buf_named("aux_ints_1", Some(&aux.ints_1));

        // pyrs_map.get_mut(ptal_name).unwrap().dens_mut().syns_mut()
            // .set_arg_buf_named("aux_ints_1", &aux.ints_0);
        let mut events_lists = HashMap::new();
        events_lists.insert(map::FF_IN, EventList::new());    
        events_lists.insert(map::FB_IN, EventList::new());
        events_lists.insert(map::FF_OUT, EventList::new());
        

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
            aux: aux,
            ocl_pq: ocl_pq,
            // ocl_context: ocl_context,
            renderer: renderer,
            counter: 0,
            rng: rand::weak_rng(),            
            events_lists: events_lists,
            bypass_inhib: false,
            bypass_filters: false,
            disable_pyrs: false,
            disable_ssts: false,
            disable_mcols: false,
            disable_regrowth: false,
            disable_learning: false,
        };

        cortical_area
    }


    // CYCLE(): <<<<< TODO: ISOLATE LEARNING INTO SEPARATE THREAD >>>>>
    pub fn cycle(&mut self, thal: &mut Thalamus) {
        let emsg = format!("cortical_area::CorticalArea::cycle(): Invalid layer.");

        // self.output(map::FF_OUT, thal);

        self.intake(map::FF_IN, thal);

        if !self.disable_ssts {    
            let aff_input_events = { self.events_lists.get(&map::FF_IN) };
            self.psal().cycle(aff_input_events); 
            // self.psal().cycle(None); 
        }

        // self.intake(map::FF_IN, thal);

        self.iinns.get_mut("iv_inhib").expect(&emsg).cycle(self.bypass_inhib);

        if !self.disable_ssts {    if !self.disable_learning { self.psal_mut().learn(); } }

        if !self.disable_mcols { self.mcols.activate(); }

        self.intake(map::FB_IN, thal);

        if !self.disable_pyrs {            
            if !self.disable_learning { self.ptal_mut().learn(); }
            let eff_input_events = { self.events_lists.get(&map::FB_IN) };
            self.ptal().cycle(eff_input_events);
            // self.ptal().cycle(None);
        }        

        // self.intake(map::FB_IN, thal);

        if !self.disable_mcols { 
            let output_events = { self.events_lists.get_mut(&map::FF_OUT) };
            self.mcols.output(output_events); 
        }

        if !self.disable_regrowth { self.regrow(); }

        self.output(map::FF_OUT, thal);
    }

    // Read input from thalamus and write to axon space.
    fn intake(&mut self, layer_tags: LayerTags, thal: &mut Thalamus) {
        // let src_layers = self.area_map.layers().layer_src_info(layer_tags);
        
        // for &(src_area_name, tags) in src_area_names.iter() {
        for src_layer in self.area_map.layers().layer_src_area_names_by_tags(layer_tags) {
            self.write_input(
                thal.ganglion(src_layer, layer_tags.mirror_io())
                    .expect("CorticalArea::output()"),
                layer_tags,
            );
        }
    }

    // Read output from axon space and write to thalamus.
    fn output(&self, layer_tags: LayerTags, thal: &mut Thalamus) {
        self.read_output(
            thal.ganglion_mut(self.name, layer_tags)
                .expect("CorticalArea::output()"),
            layer_tags, 
        );
    }

    pub fn regrow(&mut self) {
        if !self.disable_regrowth { 
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
    pub fn layer_input_ranges(&self, layer_name: &'static str, den_kind: &DendriteKind
            ) -> Vec<Range<u32>> 
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

    pub fn write_input(&mut self, events_sdr: (&EventList, &Sdr), layer_tags: LayerTags) {
        let (wait_events, sdr) = events_sdr;

        if layer_tags.contains(map::FF_IN) && !self.bypass_filters {
            match self.filters {
                Some(ref mut filters_vec) => {
                    filters_vec[0].write(sdr);

                    for fltr in filters_vec.iter() { // ***** UN-MUT ME
                        fltr.cycle();
                    }

                    return
                },
                None => (),
            }
        }

        let axn_range = self.area_map.axn_range_by_tags(layer_tags);
        //println!("\nCORTICALAREA::WRITE_INPUT(): axn_range: {:?}", axn_range);
        debug_assert!(sdr.len() == axn_range.len() as usize, "\n\
            cortical_area::CorticalArea::write_input(): Sdr/ganglion length is not equal to \
            the destination axon range. sdr.len(): {} != axn_range.len(): {}, (area: '{}', \
            layer_tags: '{:?}', range: '{:?}').", sdr.len(), 
            axn_range.len(), self.name, layer_tags, axn_range);
        
        debug_assert!((axn_range.end - axn_range.start) as usize == sdr.len());

        let new_events = self.events_lists.get_mut(&layer_tags)
            .expect("CorticalArea::write_input(): 'events_lists' error.");

        // new_events.wait();
        // new_events.release_all();
        new_events.clear_completed();    

        self.axns.states.write_async(sdr, axn_range.start as usize, 
            Some(wait_events), Some(new_events));
        // self.axns.states.write_async(sdr, axn_range.start as usize, 
        //     None, Some(new_events));
    }    

    pub fn read_output(&self, sdr_events: (&mut Sdr, &mut EventList), layer_tags: LayerTags) {
        let wait_events = &self.events_lists.get(&layer_tags)
            .expect("CorticalArea::read_output(): 'events_lists' error.");
        let (sdr, new_events) = sdr_events;
        let axn_range = self.area_map.axn_range_by_tags(layer_tags);

        debug_assert!(sdr.len() == axn_range.len() as usize, format!("\n\
            cortical_area::CorticalArea::read_output()<area: '{}', tags: '{:?}'>: \
            sdr.len(): {} != axn_range.len(): {}", self.name, layer_tags, sdr.len(), axn_range.len()));

        debug_assert!((axn_range.end - axn_range.start) as usize == sdr.len());

        // new_events.wait();
        // new_events.release_all();
        new_events.clear_completed();
        self.axns.states.read_async(sdr, axn_range.start as usize, 
            Some(wait_events), Some(new_events));
    }        

    #[inline]
    pub fn mcols(&self) -> &Box<Minicolumns> {
        &self.mcols
    }

    #[inline]
    pub fn mcols_mut(&mut self) -> &mut Box<Minicolumns> {
        &mut self.mcols
    }


    /* PIL(): Get Primary Spatial Associative Layer (immutable) */
    #[inline]
    pub fn psal(&self) -> &Box<SpinyStellateLayer> {
        let e_string = "cortical_area::CorticalArea::psal(): Primary Spatial Associative Layer: '{}' not found. ";
        self.ssts_map.get(self.psal_name).expect(e_string)
    }

    /* PIL_MUT(): Get Primary Spatial Associative Layer (mutable) */
    #[inline]
    pub fn psal_mut(&mut self) -> &mut Box<SpinyStellateLayer> {
        let e_string = "cortical_area::CorticalArea::psal_mut(): Primary Spatial Associative Layer: '{}' not found. ";
        self.ssts_map.get_mut(self.psal_name).expect(e_string)
    }


    /* PAL(): Get Primary Temporal Associative Layer (immutable) */
    #[inline]
    pub fn ptal(&self) -> &Box<PyramidalLayer> {
        let e_string = "cortical_area::CorticalArea::ptal(): Primary Temporal Associative Layer: '{}' not found. ";
        self.pyrs_map.get(self.ptal_name).expect(e_string)
    }

    /* PAL_MUT(): Get Primary Temporal Associative Layer (mutable) */
    #[inline]
    pub fn ptal_mut(&mut self) -> &mut Box<PyramidalLayer> {
        let e_string = "cortical_area::CorticalArea::ptal_mut(): Primary Temporal Associative Layer: '{}' not found. ";
        self.pyrs_map.get_mut(self.ptal_name).expect(e_string)
    }

    #[inline]
    pub fn axns(&self) -> &AxonSpace {
        &self.axns
    }    

    #[inline]
    pub fn dims(&self) -> &CorticalDims {
        &self.dims
    }

    #[inline]
    pub fn psal_name(&self) -> &'static str {
        self.psal_name
    }

    #[inline]
    pub fn ptal_name(&self) -> &'static str {
        self.ptal_name
    }

    #[inline]
    pub fn afferent_target_names(&self) -> &Vec<&'static str> {
        &self.area_map.aff_areas()
    }

    #[inline]
    pub fn efferent_target_names(&self) -> &Vec<&'static str> {
        &self.area_map.eff_areas()
    }

    #[inline]
    pub fn ocl_pq(&self) -> &ProQue {
        &self.ocl_pq
    }

    // TODO: MOVE TO TESTS
    pub fn render_aff_out(&mut self, input_status: &str, print_summary: bool) {
        let out_axns = &self.axns.states[self.mcols.aff_out_axn_range()];
        let sst_axns = &self.axns.states[self.psal().axn_range()];
        self.renderer.render(out_axns, Some(sst_axns), None, input_status, print_summary);
    }

    // TODO: MOVE TO TESTS
    pub fn render_axn_space(&mut self) {
        let axn_states = &self.axns.states[..];
        self.renderer.render_axn_space(axn_states, &self.area_map.slices())
    }

    // pub fn sample_aff_out(&self, buf: &mut [u8]) {
    //     // let aff_out_range = self.mcols.aff_out_axn_range();
    //     // debug_assert!(buf.len() == aff_out_range.len());
    //     // self.axns.states.read_async(buf, aff_out_range.start, None, None);
    //     let aff_out_slc = self.mcols.aff_out_axn_slc();
    //     self.sample_axn_slc(aff_out_slc, buf);
    // }

    #[inline]
    pub fn sample_axn_slc(&self, slc_id: u8, buf: &mut [u8]) {
        let slc_axn_range = self.area_map.slices().slc_axn_range(slc_id);
        debug_assert!(buf.len() == slc_axn_range.len(), "Sample buffer length ({}) not \
            equal to slice axon length({}). slc_axn_range: {:?}, slc_id: {}", 
            buf.len(), slc_axn_range.len(), slc_axn_range, slc_id);
        self.axns.states.read_async(buf, slc_axn_range.start, None, None);
    }    

    #[inline]
    pub fn sample_axn_space(&self, buf: &mut [u8]) {
        debug_assert!(buf.len() == self.area_map.slices().axn_count() as usize);
        self.axns.states.read_async(buf, 0, None, None);
    }

    #[inline]
    pub fn axn_gang_map(&self) -> GanglionMap {
        self.area_map.slices().gang_map()
    }

    #[inline]
    pub fn area_map(&self) -> &AreaMap {
        &self.area_map
    }
}

impl Drop for CorticalArea {
    fn drop(&mut self) {
        // Context being released by Cortex.
        print!("Releasing OpenCL components for '{}'... ", self.name);
        self.ocl_pq.release();
        print!("[ Program ][ Command Queue ]");
        print!(" ...complete. \n");
    }
}


pub struct AreaParams {
    den_per_cel_distal_l2: u8,
    syn_per_den_distal_l2: u8,

    den_per_cel_proximal: u8,
    syn_per_den_proximal: u8,
}

// pub struct ThalamicGanglions {
//     map: HashMap<LayerTags, GanglionInfo>,
// }

// pub struct GanglionInfo {
//     tract_range: Range<usize>,
//     axn_range: Range<usize>,
// }


pub struct Aux {
    dims: CorticalDims,
    pub ints_0: Buffer<i32>,
    pub ints_1: Buffer<i32>,
    // pub chars_0: Buffer<ocl::i8>,
    // pub chars_1: Buffer<ocl::i8>,
}

impl Aux {
    pub fn new(dims: &CorticalDims, ocl_pq: &ProQue) -> Aux {
        //let dims_multiplier: u32 = 512;
        //dims.columns() *= 512;
        let int_32_min = -2147483648;

        Aux { 
            ints_0: Buffer::<i32>::with_vec_initialized_to(int_32_min, dims, ocl_pq.queue()),
            ints_1: Buffer::<i32>::with_vec_initialized_to(int_32_min, dims, ocl_pq.queue()),
            // chars_0: Buffer::<ocl::i8>::new(dims, 0, ocl),
            // chars_1: Buffer::<ocl::i8>::new(dims, 0, ocl),
            dims: dims.clone(),
        }
    }

    pub unsafe fn resize(&mut self, new_dims: &CorticalDims) {
        let int_32_min = -2147483648;
        self.dims = new_dims.clone();
        
        self.ints_0.resize(&self.dims);
        self.ints_0.set_all_to(int_32_min);

        self.ints_1.resize(&self.dims);
        self.ints_1.set_all_to(int_32_min);
        // self.chars_0.resize(&self.dims, 0);
        // self.chars_1.resize(&self.dims, 0);
    }
}




#[cfg(test)]
pub mod tests {
    use rand::distributions::{ IndependentSample, Range as RandRange };

    use super::*;
    use axon_space::{ AxonSpaceTest };
    use cmn::{ CelCoords };
    use map::{ AreaMapTest };

    pub trait CorticalAreaTest {
        fn axn_state(&self, idx: usize) -> u8;
        fn write_to_axon(&mut self, val: u8, idx: u32);
        fn read_from_axon(&self, idx: u32) -> u8;
        fn rand_safe_src_axn(&mut self, cel_coords: &CelCoords, src_axn_slc: u8
            ) -> (i8, i8, u32, u32);
        fn print_aux(&mut self);
        fn print_axns(&mut self);
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

            for i in 0..50 {
                let v_ofs = v_ofs_range.ind_sample(&mut self.rng);
                let u_ofs = u_ofs_range.ind_sample(&mut self.rng);

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

        fn print_aux(&mut self) {
            print!("aux.ints_0: ");
            let view_radius = 1 << 24;
            self.aux.ints_0.print((1 << 0) as usize, 
                Some((0 - view_radius, view_radius)), None, true);
            
            print!("aux.ints_1: ");
            self.aux.ints_1.print((1 << 0) as usize, 
                Some((0 - view_radius, view_radius)), None, true);
        }

        fn print_axns(&mut self) {
            print!("axns: ");
            self.axns.states.print(1 << 0, Some((1, 255)), None, false);
        }

        fn activate_axon(&mut self, idx: u32) {
            let val = RandRange::new(1, 255).ind_sample(&mut self.rng);
            self.axns.write_to_axon(val, idx);
        }

        fn deactivate_axon(&mut self, idx: u32) {
            self.axns.write_to_axon(0, idx);
        }
    }
}

