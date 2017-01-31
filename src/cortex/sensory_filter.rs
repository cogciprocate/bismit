use std::ops::Range;
use ocl::{Kernel, ProQue, SpatialDims, Buffer, Event};
use cmn::{CmnError, CmnResult, CorticalDims};
use map::{/*LayerAddress,*/ /*SourceLayerInfo,*/ ExecutionGraph, ExecutionCommand, CorticalBuffer,
    ThalamicTract};
use tract_terminal::{SliceBufferSource, OclBufferTarget};

pub struct SensoryFilter {
    filter_idx: usize,
    filter_name: String,
    cl_file_name: Option<String>,
    // src_layer_addr: LayerAddress,
    input_buffer: Buffer<u8>,
    cycle_kernel: Kernel,
    exe_cmd_idx_cycle: usize,
    exe_cmd_idx_write: Option<usize>,
}

impl SensoryFilter {
    pub fn new(
            area_id: usize,
            filter_idx: usize,
            filter_chain_count: usize,
            filter_name: String,
            cl_file_name: Option<String>,

            // src_lyr_info: &SourceLayerInfo,
            // axn_states: &Buffer<u8>,
            src_tract_info: Option<(usize, Range<u8>)>,
            dims: &CorticalDims,
            output_buffer: &Buffer<u8>,
            output_slc_range: Range<u8>,
            ocl_pq: &ProQue,
            exe_graph: &mut ExecutionGraph,
        ) -> CmnResult<SensoryFilter>
    {
        // let dims = src_lyr_info.dims();
        // let slc_range = src_lyr_info.tar_slc_range();

        let input_buffer = Buffer::<u8>::new(ocl_pq.queue().clone(), None, dims, None).unwrap();

        let cycle_kernel = ocl_pq.create_kernel(&filter_name.clone()).expect("[FIXME]: HANDLE ME")
            .gws(SpatialDims::Three(dims.depth() as usize, dims.v_size() as usize, dims.u_size() as usize))
            .lws(SpatialDims::Three(1, 8, 8 as usize))
            .arg_buf(&input_buffer)
            // .arg_scl(slc_range.start)
            .arg_scl(output_slc_range.start)
            // .arg_buf(axn_states);
            .arg_buf(output_buffer);

        // let output_cmd_srcs = vec![
        //     CorticalBuffer::data_soma_lyr(&pyrs.best_den_states_raw(), pyrs.layer_addr()),
        //     CorticalBuffer::data_soma_lyr(&pyrs.soma(), pyrs.layer_addr()),
        // ];

        // let mut output_cmd_tars = vec![
        //     CorticalBuffer::control_soma_lyr(&flag_sets, layer_addr),
        //     CorticalBuffer::control_soma_lyr(&best_den_states, layer_addr),
        //     CorticalBuffer::axon_slice(&axons.states, layer_addr.area_id(), mcol_axn_slc_id),
        // ];

        let filter_is_first = filter_idx == 0;
        let filter_is_last = filter_idx == (filter_chain_count - 1);

        // Cycle execution command:
        let cycle_cmd_srcs = vec![CorticalBuffer::axon_input_filter(&input_buffer)];

        let cycle_cmd_tars = if filter_is_last {
            output_slc_range
                .map(|slc_id| CorticalBuffer::axon_slice(output_buffer, area_id, slc_id))
                .collect()
        } else {
            vec![CorticalBuffer::axon_input_filter(&output_buffer)]
        };

        let exe_cmd_idx_cycle = exe_graph.add_command(ExecutionCommand::cortical_kernel(
            cycle_cmd_srcs, cycle_cmd_tars))?;

        // Write execution command:
        let exe_cmd_idx_write = if filter_is_first {
            let (src_area_id, src_slc_range) = src_tract_info.expect("SensoryFilter::new: \
                No source tract info found for first filter.");

            let write_cmd_srcs = src_slc_range
                .map(|slc_id| ThalamicTract::axon_slice(src_area_id, slc_id))
                .collect();

            Some(exe_graph.add_command(ExecutionCommand::thalamocortical_write(
                write_cmd_srcs,
                vec![CorticalBuffer::axon_input_filter(&input_buffer)],
            ))?)
        } else {
            None
        };

        Ok(SensoryFilter {
            filter_idx: filter_idx,
            filter_name: filter_name,
            cl_file_name: cl_file_name,
            // src_layer_addr: src_lyr_info.layer_addr().clone(),
            input_buffer: input_buffer,
            cycle_kernel: cycle_kernel,
            exe_cmd_idx_cycle: exe_cmd_idx_cycle,
            exe_cmd_idx_write: exe_cmd_idx_write,
        })
    }

    pub fn set_exe_order_cycle(&self, exe_graph: &mut ExecutionGraph) -> CmnResult<usize> {
        Ok(exe_graph.order_next(self.exe_cmd_idx_cycle)?)
    }

    pub fn set_exe_order_write(&self, exe_graph: &mut ExecutionGraph) -> CmnResult<usize> {
        Ok(exe_graph.order_next(self.exe_cmd_idx_write.ok_or(CmnError::new(
            "SensoryFilter::set_exe_order_write: Write command not created for this filter."))?)?)
    }

    pub fn write(&self, source: SliceBufferSource) -> CmnResult<Event> {
        Ok(OclBufferTarget::new(&self.input_buffer,
                0..self.input_buffer.len() as u32, source.dims().clone(), None, true)?
            .copy_from_slice_buffer(source)?.event().unwrap_or(Event::empty()))
    }

    pub fn cycle(&self, wait_event: &Event) -> Event {
        //println!("Printing {} for {}:\n", &self.filter_name, self.area_name);

        let mut fltr_event = Event::empty();
        self.cycle_kernel.cmd().ewait(wait_event).enew(&mut fltr_event).enq()
            .expect("SensoryFilter::cycle()");
        fltr_event
    }

    pub fn filter_name(&self) -> &str { self.filter_name.as_str() }
    pub fn filter_idx(&self) -> usize { self.filter_idx }
    pub fn cl_file_name(&self) -> Option<&str> { self.cl_file_name.as_ref().map(|clfn| clfn.as_str()) }
    pub fn input_buffer(&self) -> &Buffer<u8> { &self.input_buffer }
    // pub fn axn_tags(&self) -> &AxonTags { &self.axn_tags }
}
