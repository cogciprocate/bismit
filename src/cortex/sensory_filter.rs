use std::ops::Range;
use ocl::{flags, Kernel, ProQue, SpatialDims, Buffer, Event, Queue};
use cmn::{CmnError, CmnResult, CorticalDims};
use map::{ExecutionGraph, ExecutionCommand, CorticalBuffer,
    ThalamicTract};
use tract_terminal::{SliceBufferSource, OclBufferTarget};

pub struct SensoryFilter {
    filter_idx: usize,
    filter_name: String,
    cl_file_name: Option<String>,
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
            src_tract_info: Option<(usize, Range<u8>)>,
            dims: &CorticalDims,
            output_buffer: &Buffer<u8>,
            output_slc_range: Range<u8>,
            ocl_pq: &ProQue,
            write_queue: &Queue,
            exe_graph: &mut ExecutionGraph,
        ) -> CmnResult<SensoryFilter>
    {
        let input_buffer = Buffer::<u8>::new(write_queue.clone(),
            Some(flags::MEM_HOST_WRITE_ONLY | flags::MEM_READ_ONLY), dims, None, Some((0, None::<()>))).unwrap();

        let cycle_kernel = ocl_pq.create_kernel(&filter_name.clone()).expect("[FIXME]: HANDLE ME")
            .gws(SpatialDims::Three(dims.depth() as usize, dims.v_size() as usize, dims.u_size() as usize))
            .lws(SpatialDims::Three(1, 8, 8 as usize))
            .arg_buf(&input_buffer)
            .arg_scl(output_slc_range.start)
            .arg_buf(output_buffer);

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

    pub fn write(&self, source: SliceBufferSource, exe_graph: &mut ExecutionGraph) -> CmnResult<()> {
        let cmd_idx = self.exe_cmd_idx_write.ok_or(CmnError::new(
            "SensoryFilter::write: Write command not created for this filter."))?;
        let range = 0..self.input_buffer.len() as u32;

        // let wait_list = exe_graph.get_req_events(cmd_idx)?;

        // Ok(OclBufferTarget::new(&self.input_buffer, range, source.dims().clone(), None, true)?
        //     .copy_from_slice_buffer(source)?
        //     .event().unwrap_or(Event::empty())
        // )
        let event = OclBufferTarget::new(&self.input_buffer, range, source.dims().clone(), None, false)?
            .copy_from_slice_buffer_v2(source, Some(exe_graph.get_req_events(cmd_idx)?))?;

        exe_graph.set_cmd_event(cmd_idx, Some(event))?;
        Ok(())
    }

    // pub fn cycle(&self, wait_event: &Event) -> Event {
    //     let mut fltr_event = Event::empty();
    //     self.cycle_kernel.cmd().ewait(wait_event).enew(&mut fltr_event).enq()
    //         .expect("SensoryFilter::cycle()");
    //     fltr_event
    // }
    pub fn cycle(&self, exe_graph: &mut ExecutionGraph) -> CmnResult<()> {
        // let wait_list = exe_graph.get_req_events(self.exe_cmd_idx_cycle)?;
        let mut event = Event::empty();

        self.cycle_kernel.cmd().ewait(exe_graph.get_req_events(self.exe_cmd_idx_cycle)?)
            .enew(&mut event).enq()?;

        exe_graph.set_cmd_event(self.exe_cmd_idx_cycle, Some(event))?;
        Ok(())
    }

    pub fn filter_name(&self) -> &str { self.filter_name.as_str() }
    pub fn filter_idx(&self) -> usize { self.filter_idx }
    pub fn cl_file_name(&self) -> Option<&str> { self.cl_file_name.as_ref().map(|clfn| clfn.as_str()) }
    pub fn input_buffer(&self) -> &Buffer<u8> { &self.input_buffer }
}
