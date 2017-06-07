use cmn::{CorticalDims, CmnResult};
use map::{AreaMap, LayerAddress, ExecutionGraph, ExecutionCommand, CorticalBuffer};
use ocl::{Kernel, ProQue, SpatialDims, /*Buffer,*/ Event};
use map::CellScheme;
use cortex::{AxonSpace, ControlCellLayer, DataCellLayer, CorticalAreaSettings};


#[derive(Debug)]
pub struct ActivitySmoother {
    layer_name: &'static str,
    layer_addr: LayerAddress,
    host_lyr_addr: LayerAddress,
    kern: Kernel,
    // kern_inhib_passthrough: Kernel,
    exe_cmd_idx: usize,
    settings: CorticalAreaSettings,
}

impl ActivitySmoother {
    pub fn new<D>(layer_name: &'static str, layer_id: usize, dims: CorticalDims, _: CellScheme,
            host_lyr: &D, host_lyr_base_axn_slc: u8, axns: &AxonSpace, area_map: &AreaMap,
            ocl_pq: &ProQue, settings: CorticalAreaSettings, exe_graph: &mut ExecutionGraph)
            -> CmnResult<ActivitySmoother>
            where D: DataCellLayer
    {
        let layer_addr = LayerAddress::new(area_map.area_id(), layer_id);


        // TODO (adapt into documentation):
        //
        // - Generate lists of focal "smoother" cells and fill buffer(s?).
        //   - Because we will need to be able to calculate target layer `v`
        //     and `u`, separate buffers for each may be necessary (or store
        //     as tuples?).
        //     - Mash each layer together within kernel (loop through them --
        //       imagine that the each smoother cell controls all layers).
        //
        // - global work dims:
        //   - linear?
        //
        // - activities
        //

        // Kernel:
        let kern_name = "smooth";
        let kern = ocl_pq.create_kernel(kern_name)?
            .gws(SpatialDims::Three(dims.depth() as usize, dims.v_size() as usize,
                dims.u_size() as usize))
            .lws(SpatialDims::Three(1, 8, 8 as usize))
            .arg_buf(host_lyr.soma())
            .arg_buf(host_lyr.activities())
            .arg_scl(host_lyr_base_axn_slc)
            .arg_scl_named::<i32>("rnd", None)
            .arg_buf(host_lyr.energies())
            // .arg_buf_named("aux_ints_0", None)
            // .arg_buf_named("aux_ints_1", None)
            .arg_buf(axns.states());

        let exe_cmd_srcs = (0..host_lyr.tft_count())
            .map(|host_lyr_tft_id| CorticalBuffer::data_den_tft(&host_lyr.soma(),
                host_lyr.layer_addr(), host_lyr_tft_id))
            .collect();

        // Set up execution command:
        let exe_cmd_tars = (host_lyr_base_axn_slc..host_lyr_base_axn_slc + dims.depth())
            .map(|slc_id| CorticalBuffer::axon_slice(&axns.states(), area_map.area_id(), slc_id))
            .collect();

        let exe_cmd_idx = exe_graph.add_command(ExecutionCommand::cortical_kernel(
             kern_name, exe_cmd_srcs, exe_cmd_tars))?;


        Ok(ActivitySmoother {
            layer_name: layer_name,
            layer_addr: layer_addr,
            host_lyr_addr: host_lyr.layer_addr(),
            kern,
            exe_cmd_idx: exe_cmd_idx,
            settings: settings,
        })
    }

    pub fn set_exe_order(&self, exe_graph: &mut ExecutionGraph) -> CmnResult<()> {
        exe_graph.order_next(self.exe_cmd_idx)?;
        Ok(())
    }

    pub fn cycle(&self, exe_graph: &mut ExecutionGraph, _host_lyr_addr: LayerAddress) -> CmnResult<()> {
        let mut event = Event::empty();

        self.kern.cmd()
            .ewait(exe_graph.get_req_events(self.exe_cmd_idx)?)
            .enew(&mut event)
            .enq()?;

        exe_graph.set_cmd_event(self.exe_cmd_idx, Some(event))?;

        Ok(())
    }

    #[inline] pub fn layer_name(&self) -> &'static str { self.layer_name }
    #[inline] pub fn layer_addr(&self) -> LayerAddress { self.layer_addr }

}

impl ControlCellLayer for ActivitySmoother {
    fn set_exe_order_pre(&self, _exe_graph: &mut ExecutionGraph, _host_lyr_addr: LayerAddress) -> CmnResult<()> {
        Ok(())
    }

    fn set_exe_order_post(&self, exe_graph: &mut ExecutionGraph, _host_lyr_addr: LayerAddress) -> CmnResult<()> {
        self.set_exe_order(exe_graph)
    }

    fn cycle_pre(&mut self, _exe_graph: &mut ExecutionGraph, _host_lyr_addr: LayerAddress) -> CmnResult<()> {
        Ok(())
    }

    fn cycle_post(&mut self, exe_graph: &mut ExecutionGraph, host_lyr_addr: LayerAddress) -> CmnResult<()> {
        self.cycle(exe_graph, host_lyr_addr)
    }

    fn layer_name(&self) -> &'static str { self.layer_name() }
    fn layer_addr(&self) -> LayerAddress { self.layer_addr }
    fn host_layer_addr(&self) -> LayerAddress { self.host_lyr_addr }
}