#![allow(dead_code)]

use ocl::{Kernel, ProQue, SpatialDims, Buffer, Event, EventList};
use cmn::{/*Sdr,*/ CmnResult};
use cortex::AxonSpace;
use map::{self, AreaMap};
use tract_terminal::{SliceBufferSource, OclBufferTarget};

pub struct SensoryFilter {
    filter_name: String,
    cl_file_name: Option<String>,
    area_name: &'static str,
    //dims: CorticalDims,
    input: Buffer<u8>,
    kern_cycle: Kernel,
}

impl SensoryFilter {
    pub fn new(
                filter_name: String,
                cl_file_name: Option<String>,
                area_map: &AreaMap,
                //area_name: &'static str,
                //dims: CorticalDims,
                axns: &AxonSpace,
                //base_axn_slc: u8,
                ocl_pq: &ProQue,
            ) -> SensoryFilter
    {
        let layer_tags = map::FF_IN;

        let layers = area_map.layers().layers_containing_tags(layer_tags);
        assert!(layers.len() == 1, "\n\nERROR: SensoryFilter::new(): Found multiple layers \
            containing the same tags: \n{}\n Layers: \n{:#?}\n", layer_tags, layers);
        let layer = layers[0];
        let axn_slc_range = layer.slc_range().expect("SensoryFilter::new(): \
            Invalid slice range.").clone();

        assert!(layers[0].sources().len() == 1, "\n\nERROR: SensoryFilter::new(): Multiple \
            source areas found for the feed-forward input layer with tags: \n\n{}\n\n\
            Source layers: \n{:#?}\n\n", layer_tags, layers);
        let ref src_layer = layers[0].sources()[0];
        let dims = src_layer.dims();

        assert!(dims.depth() == 1, "\nAfferent input layer depths of more than one for cortical \
            areas with sensory filters are not yet supported. Please set the depth of any \
            afferent input layers with filters to 1.");

        let input = Buffer::<u8>::new(ocl_pq.queue().clone(), None, dims, None).unwrap();

        let kern_cycle = ocl_pq.create_kernel(&filter_name.clone()).expect("[FIXME]: HANDLE ME")
            // .expect("SensoryFilter::new()")
            .gws(SpatialDims::Three(dims.depth() as usize, dims.v_size() as usize, dims.u_size() as usize))
            .lws(SpatialDims::Three(1, 8, 8 as usize))
            .arg_buf(&input)
            .arg_scl(axn_slc_range.start)
            .arg_buf(&axns.states);

        SensoryFilter {
            filter_name: filter_name,
            cl_file_name: cl_file_name,
            area_name: area_map.area_name(),
            //dims: dims,
            input: input,
            kern_cycle: kern_cycle,
        }
    }

    // pub fn write(&self, sdr: &Sdr, wait_list: &EventList) -> Event {
    //     assert!(sdr.len() <= self.input.len());
    //     let mut fltr_event = Event::empty();
    //     self.input.write(sdr).ewait(wait_list).enew(&mut fltr_event).enq()
    //         .expect("SensoryFilter::write()");
    //     fltr_event
    // }

    pub fn write(&self, source: SliceBufferSource) -> CmnResult<Event> {
        let mut events = EventList::new();

        OclBufferTarget::new(&self.input, 0..self.input.len() as u32, source.dims().clone(),
                Some(&mut events))?
            .copy_from_slice_buffer(source)?;

        debug_assert_eq!(events.len(), 1);
        Ok(events.pop().unwrap_or(Event::empty()))
    }

    pub fn cycle(&self, wait_event: &Event) -> Event {
        //println!("Printing {} for {}:\n", &self.filter_name, self.area_name);
        let mut fltr_event = Event::empty();
        self.kern_cycle.cmd().ewait(wait_event).enew(&mut fltr_event).enq()
            .expect("SensoryFilter::cycle()");
        fltr_event
    }
}
