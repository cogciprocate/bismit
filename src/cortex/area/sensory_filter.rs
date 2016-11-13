#![allow(dead_code)]

// use std::hash::{Hash, BuildHasherDefault};
use std::collections::HashMap;
// use twox_hash::XxHash;
use ocl::{Kernel, ProQue, SpatialDims, Buffer, Event, /*EventList*/};
use cmn::{/*Sdr,*/ CmnResult};
use cortex::AxonSpace;
use map::{/*self,*/ AreaMap, LayerTags};
use tract_terminal::{SliceBufferSource, OclBufferTarget};

pub struct SensoryFilter {
    filter_name: String,
    cl_file_name: Option<String>,
    layer_tags: LayerTags,
    area_name: &'static str,
    //dims: CorticalDims,
    // input: Buffer<u8>,
    // kern_cycle: Kernel,

    src_area_map: HashMap<&'static str, usize>,
    input_buffers: Vec<Buffer<u8>>,
    cycle_kernels: Vec<Kernel>,
}

impl SensoryFilter {
    pub fn new(
                filter_name: String,
                cl_file_name: Option<String>,
                layer_tags: LayerTags,
                area_map: &AreaMap,
                //area_name: &'static str,
                //dims: CorticalDims,
                axns: &AxonSpace,
                //base_axn_slc: u8,
                ocl_pq: &ProQue,
            ) -> SensoryFilter
    {
        let layers = area_map.layers().layers_meshing_tags(layer_tags);
        assert!(layers.len() == 1, "\n\nERROR: SensoryFilter::new(): Multiple (or no) layers \
            with the same layer tags found. Please refine filter tags to select only a single \
            layer. \nArea: {}\n{}\nLayers: \n{:#?}\n\n",
            area_map.area_name(), layer_tags, layers);
        let layer = layers[0];

        // let axn_slc_range = layer.slc_range().expect(&format!("\n\nERROR: SensoryFilter::new(): \
        //     No slice range found for layer with tags: {}. The source layer is not properly \
        //     connected to this area. Check the area efferent inputs list.", layer_tags)).clone();

        // assert!(layer.sources().len() == 1, "\n\nERROR: SensoryFilter::new(): Multiple \
        //     source areas found for the feed-forward input layer with tags: \n\n{}\n\n\
        //     Source layers: \n{:#?}\n\n", layer_tags, layers);
        // let ref src_layer = layers[0].sources()[0];
        // let dims = src_layer.dims();

        // assert!(dims.depth() == 1, "\nAfferent input layer depths of more than one for cortical \
        //     areas with sensory filters are not yet supported. Please set the depth of any \
        //     afferent input layers with filters to 1.");

        let src_lyr_count = layer.sources().len();

        let mut src_area_map = HashMap::with_capacity(src_lyr_count);
        let mut input_buffers = Vec::with_capacity(src_lyr_count);
        let mut cycle_kernels = Vec::with_capacity(src_lyr_count);

        for src_lyr in layer.sources().iter() {
            let dims = src_lyr.dims();
            let slc_range = src_lyr.tar_slc_range();

            let input = Buffer::<u8>::new(ocl_pq.queue().clone(), None, dims, None).unwrap();

            let kern_cycle = ocl_pq.create_kernel(&filter_name.clone()).expect("[FIXME]: HANDLE ME")
                // .expect("SensoryFilter::new()")
                .gws(SpatialDims::Three(dims.depth() as usize, dims.v_size() as usize, dims.u_size() as usize))
                .lws(SpatialDims::Three(1, 8, 8 as usize))
                .arg_buf(&input)
                .arg_scl(slc_range.start)
                .arg_buf(&axns.states);

            debug_assert_eq!(input_buffers.len(), cycle_kernels.len());
            src_area_map.insert(src_lyr.area_name(), input_buffers.len());
            input_buffers.push(input);
            cycle_kernels.push(kern_cycle);
        }

        SensoryFilter {
            filter_name: filter_name,
            cl_file_name: cl_file_name,
            layer_tags: layer_tags,
            area_name: area_map.area_name(),
            //dims: dims,
            // input: input,
            // kern_cycle: kern_cycle,

            src_area_map: src_area_map,
            input_buffers: input_buffers,
            cycle_kernels: cycle_kernels,
        }
    }

    // pub fn write(&self, sdr: &Sdr, wait_list: &EventList) -> Event {
    //     assert!(sdr.len() <= self.input.len());
    //     let mut fltr_event = Event::empty();
    //     self.input.write(sdr).ewait(wait_list).enew(&mut fltr_event).enq()
    //         .expect("SensoryFilter::write()");
    //     fltr_event
    // }

    pub fn write(&self, source: SliceBufferSource, src_area_name: &str) -> CmnResult<Event> {
        // let mut events = EventList::new();
        let filter_id = self.src_area_map[src_area_name];

        Ok(OclBufferTarget::new(&self.input_buffers[filter_id],
                0..self.input_buffers[filter_id].len() as u32, source.dims().clone(), None, true)?
            .copy_from_slice_buffer(source)?.event().unwrap_or(Event::empty()))

        // debug_assert_eq!(events.len(), 1);
        // Ok(events.pop().unwrap_or(Event::empty()))
    }

    pub fn cycle(&self, wait_event: &Event, src_area_name: &str) -> Event {
        //println!("Printing {} for {}:\n", &self.filter_name, self.area_name);
        let filter_id = self.src_area_map[src_area_name];

        let mut fltr_event = Event::empty();
        self.cycle_kernels[filter_id].cmd().ewait(wait_event).enew(&mut fltr_event).enq()
            .expect("SensoryFilter::cycle()");
        fltr_event
    }

    pub fn layer_tags(&self) -> LayerTags { self.layer_tags }
}
