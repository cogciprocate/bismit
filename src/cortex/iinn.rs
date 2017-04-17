use cmn::{CorticalDims, CmnResult};
use map::{AreaMap, LayerAddress, ExecutionGraph, ExecutionCommand, CorticalBuffer};
use ocl::{Kernel, ProQue, SpatialDims, Buffer, Event};
use map::CellScheme;
use cortex::{AxonSpace, ControlCellLayer, DataCellLayer};


pub struct InhibitoryInterneuronNetwork {
    layer_name: &'static str,
    layer_id: usize,
    // dims: CorticalDims,

    kern_inhib_simple: Kernel,
    kern_inhib_passthrough: Kernel,
    exe_cmd_idx: usize,

    pub spi_ids: Buffer<u8>,
    pub wins: Buffer<u8>,
    pub states: Buffer<u8>,

}

impl InhibitoryInterneuronNetwork {
    // FIXME: This function should take a 'bypass' argument instead of `::cycle`.
    pub fn new<D>(layer_name: &'static str, layer_id: usize, dims: CorticalDims, _: CellScheme,
            host_lyr: &D, host_lyr_base_axn_slc: u8, axns: &AxonSpace, area_map: &AreaMap,
            ocl_pq: &ProQue, exe_graph: &mut ExecutionGraph)
            -> CmnResult<InhibitoryInterneuronNetwork>
            where D: DataCellLayer
    {
        // let layer_addr = LayerAddress::new(area_map.area_id(), layer_id);

        let spi_ids = Buffer::<u8>::new(ocl_pq.queue().clone(), None, &dims, None, Some((0, None::<()>))).unwrap();
        let wins = Buffer::<u8>::new(ocl_pq.queue().clone(), None, &dims, None, Some((0, None::<()>))).unwrap();
        let states = Buffer::<u8>::new(ocl_pq.queue().clone(), None, &dims, None, Some((0, None::<()>))).unwrap();

        // Simple (active) kernel:
        let kern_inhib_simple_name = "inhib_simple";
        let kern_inhib_simple = ocl_pq.create_kernel(kern_inhib_simple_name)
            .expect("InhibitoryInterneuronNetwork::new()")
            .gws(SpatialDims::Three(dims.depth() as usize, dims.v_size() as usize,
                dims.u_size() as usize))
            .lws(SpatialDims::Three(1, 8, 8 as usize))
            .arg_buf(host_lyr.soma())
            .arg_scl(host_lyr_base_axn_slc)
            // .arg_buf_named("aux_ints_0", None)
            // .arg_buf_named("aux_ints_1", None)
            .arg_buf(axns.states());

        // Passthrough kernel:
        let kern_inhib_passthrough_name = "inhib_passthrough";
        let kern_inhib_passthrough = ocl_pq.create_kernel(kern_inhib_passthrough_name)
            .expect("InhibitoryInterneuronNetwork::new()")
            .gws(SpatialDims::Three(dims.depth() as usize, dims.v_size() as usize,
                dims.u_size() as usize))
            .arg_buf(host_lyr.soma())
            .arg_scl(host_lyr_base_axn_slc)
            .arg_buf(axns.states());


        let exe_cmd_srcs = (0..host_lyr.tft_count())
            .map(|host_lyr_tft_id| CorticalBuffer::data_den_tft(&host_lyr.soma(),
                LayerAddress::new(area_map.area_id(), host_lyr.layer_addr().layer_id()), host_lyr_tft_id))
            .collect();

        // Set up execution command:
        let exe_cmd_tars = (host_lyr_base_axn_slc..host_lyr_base_axn_slc + dims.depth())
            .map(|slc_id| CorticalBuffer::axon_slice(&axns.states(), area_map.area_id(), slc_id))
            .collect();

        let exe_cmd_idx = exe_graph.add_command(ExecutionCommand::cortical_kernel(
             "inhib_...", exe_cmd_srcs, exe_cmd_tars))?;


        Ok(InhibitoryInterneuronNetwork {
            layer_name: layer_name,
            layer_id: layer_id,
            // dims: dims,

            kern_inhib_simple: kern_inhib_simple,
            kern_inhib_passthrough: kern_inhib_passthrough,
            exe_cmd_idx: exe_cmd_idx,

            spi_ids: spi_ids,
            wins: wins,
            states: states,
        })
    }

    pub fn set_exe_order(&self, exe_graph: &mut ExecutionGraph) -> CmnResult<()> {
        exe_graph.order_next(self.exe_cmd_idx)?;
        Ok(())
    }

    // FIXME: `::new` should take the `bypass` argument instead.
    #[inline]
    pub fn cycle(&mut self, exe_graph: &mut ExecutionGraph, bypass: bool) -> CmnResult<()> {
        let mut event = Event::empty();

        if bypass {
            self.kern_inhib_passthrough.cmd()
                .ewait(exe_graph.get_req_events(self.exe_cmd_idx)?)
                .enew(&mut event)
                .enq()?;
        } else {
            self.kern_inhib_simple.cmd()
                .ewait(exe_graph.get_req_events(self.exe_cmd_idx)?)
                .enew(&mut event)
                .enq()?;
        }

        exe_graph.set_cmd_event(self.exe_cmd_idx, Some(event))?;

        Ok(())
    }

    #[inline] pub fn layer_name(&self) -> &'static str { self.layer_name }
    #[inline] pub fn layer_id(&self) -> usize { self.layer_id }

}

impl ControlCellLayer for InhibitoryInterneuronNetwork {
    fn set_exe_order(&self, exe_graph: &mut ExecutionGraph) -> CmnResult<()> {
        self.set_exe_order(exe_graph)
    }

    fn cycle(&mut self, exe_graph: &mut ExecutionGraph, bypass: bool) -> CmnResult<()> {
        self.cycle(exe_graph, bypass)
    }

    fn layer_name(&self) -> &'static str { self.layer_name() }
    fn layer_id(&self) -> usize { self.layer_id() }
}