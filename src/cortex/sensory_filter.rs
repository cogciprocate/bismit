use ocl::{Kernel, ProQue, SpatialDims, Buffer, Event};
use cmn::{CmnResult, CorticalDims};
use map::{LayerAddress, SourceLayerInfo};
use tract_terminal::{SliceBufferSource, OclBufferTarget};

pub struct SensoryFilter {
    filter_name: String,
    cl_file_name: Option<String>,
    // src_layer_addr: LayerAddress,
    input_buffer: Buffer<u8>,
    cycle_kernel: Kernel,
}

impl SensoryFilter {
    pub fn new(
            filter_idx: usize,
            filter_name: String,
            cl_file_name: Option<String>,

            // src_lyr_info: &SourceLayerInfo,
            // axn_states: &Buffer<u8>,
            dims: &CorticalDims,
            output_buffer: &Buffer<u8>,
            output_slc_idz: u8,


            ocl_pq: &ProQue,
        ) -> SensoryFilter
    {
        // let dims = src_lyr_info.dims();
        // let slc_range = src_lyr_info.tar_slc_range();

        let input_buffer = Buffer::<u8>::new(ocl_pq.queue().clone(), None, dims, None).unwrap();

        let cycle_kernel = ocl_pq.create_kernel(&filter_name.clone()).expect("[FIXME]: HANDLE ME")
            .gws(SpatialDims::Three(dims.depth() as usize, dims.v_size() as usize, dims.u_size() as usize))
            .lws(SpatialDims::Three(1, 8, 8 as usize))
            .arg_buf(&input_buffer)
            // .arg_scl(slc_range.start)
            .arg_scl(output_slc_idz)
            // .arg_buf(axn_states);
            .arg_buf(output_buffer);

        SensoryFilter {
            filter_name: filter_name,
            cl_file_name: cl_file_name,
            // src_layer_addr: src_lyr_info.layer_addr().clone(),
            input_buffer: input_buffer,
            cycle_kernel: cycle_kernel,
        }
    }

    // pub fn write(&self, source: SliceBufferSource, lyr_id: usize) -> CmnResult<Event> {
    //     Ok(OclBufferTarget::new(&self.input_buffers[lyr_id],
    //             0..self.input_buffers[lyr_id].len() as u32, source.dims().clone(), None, true)?
    //         .copy_from_slice_buffer(source)?.event().unwrap_or(Event::empty()))
    // }

    pub fn write(&self, source: SliceBufferSource) -> CmnResult<Event> {
        Ok(OclBufferTarget::new(&self.input_buffer,
                0..self.input_buffer.len() as u32, source.dims().clone(), None, true)?
            .copy_from_slice_buffer(source)?.event().unwrap_or(Event::empty()))
    }

    // pub fn cycle(&self, wait_event: &Event, lyr_id: usize) -> Event {
    //     //println!("Printing {} for {}:\n", &self.filter_name, self.area_name);

    //     let mut fltr_event = Event::empty();
    //     self.cycle_kernels[lyr_id].cmd().ewait(wait_event).enew(&mut fltr_event).enq()
    //         .expect("SensoryFilter::cycle()");
    //     fltr_event
    // }

    pub fn cycle(&self, wait_event: &Event) -> Event {
        //println!("Printing {} for {}:\n", &self.filter_name, self.area_name);

        let mut fltr_event = Event::empty();
        self.cycle_kernel.cmd().ewait(wait_event).enew(&mut fltr_event).enq()
            .expect("SensoryFilter::cycle()");
        fltr_event
    }

    // pub fn input_lyr_id(&self, src_area_id: usize) -> Option<usize> {
    //     self.src_area_map.get(&src_area_id).cloned()
    // }

    // pub fn src_lyr_addr(&self) -> &LayerAddress {
    //     &self.src_layer_addr
    // }

    pub fn filter_name(&self) -> &str { self.filter_name.as_str() }
    pub fn cl_file_name(&self) -> Option<&str> { self.cl_file_name.as_ref().map(|clfn| clfn.as_str()) }
    pub fn input_buffer(&self) -> &Buffer<u8> { &self.input_buffer }
    // pub fn axn_tags(&self) -> &AxonTags { &self.axn_tags }
}
