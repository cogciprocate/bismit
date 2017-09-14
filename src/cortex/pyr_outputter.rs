use cmn::{CmnResult};
use map::{AreaMap, LayerAddress, ExecutionGraph, CommandRelations, CorticalBuffer, CellScheme, CommandUid};
use ocl::{Kernel, ProQue, SpatialDims, /*Buffer,*/ Event, /*MemFlags*/};
use cortex::{AxonSpace, ControlCellLayer, DataCellLayer, CorticalAreaSettings};


#[derive(Debug)]
pub struct PyrOutputter {
    layer_name: &'static str,
    layer_addr: LayerAddress,
    host_lyr_addr: LayerAddress,
    kern: Kernel,
    exe_cmd_uid: CommandUid,
    exe_cmd_idx: usize,
    settings: CorticalAreaSettings,
}

impl PyrOutputter {
    pub fn new<D>(layer_name: &'static str, layer_id: usize, _scheme: CellScheme,
            host_lyr: &D, host_lyr_base_axn_slc: u8, axns: &AxonSpace, area_map: &AreaMap,
            ocl_pq: &ProQue, settings: CorticalAreaSettings, exe_graph: &mut ExecutionGraph)
            -> CmnResult<PyrOutputter>
            where D: DataCellLayer
    {
        let layer_addr = LayerAddress::new(area_map.area_id(), layer_id);

        // Kernel:
        let kern_name = "pyr_output";
        let kern = ocl_pq.create_kernel(kern_name)?
            .gws(SpatialDims::Three(
                host_lyr.dims().depth() as usize,
                host_lyr.dims().v_size() as usize,
                host_lyr.dims().u_size() as usize,
            ))
            // .arg_scl(host_lyr.dims().v_size())
            // .arg_scl(host_lyr.dims().u_size())
            .arg_buf(host_lyr.soma())
            .arg_scl(host_lyr_base_axn_slc)
            // .arg_buf_named("aux_ints_0", None)
            // .arg_buf_named("aux_ints_1", None)
            .arg_buf(axns.states());

        let exe_cmd_srcs = vec![CorticalBuffer::data_soma_lyr(host_lyr.soma(), host_lyr.layer_addr())];

        // Set up execution command:
        let exe_cmd_tars = (host_lyr_base_axn_slc..host_lyr_base_axn_slc + host_lyr.dims().depth())
            .map(|slc_id| CorticalBuffer::axon_slice(axns.states(), area_map.area_id(), slc_id))
            .collect();

        let exe_cmd_uid = exe_graph.add_command(CommandRelations::cortical_kernel(
             kern_name, exe_cmd_srcs, exe_cmd_tars))?;
        // let exe_cmd_idx = 0;

        Ok(PyrOutputter {
            layer_name: layer_name,
            layer_addr: layer_addr,
            host_lyr_addr: host_lyr.layer_addr(),
            kern,
            exe_cmd_uid,
            exe_cmd_idx: 0,
            settings: settings,
        })
    }

    pub fn set_exe_order(&mut self, exe_graph: &mut ExecutionGraph) -> CmnResult<()> {
        self.exe_cmd_idx = exe_graph.order_command(self.exe_cmd_uid)?;
        Ok(())
    }

    pub fn cycle(&mut self, exe_graph: &mut ExecutionGraph, _host_lyr_addr: LayerAddress) -> CmnResult<()> {
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

impl ControlCellLayer for PyrOutputter {
    fn set_exe_order_pre(&mut self, _exe_graph: &mut ExecutionGraph, _host_lyr_addr: LayerAddress) -> CmnResult<()> {
        Ok(())
    }

    fn set_exe_order_post(&mut self, exe_graph: &mut ExecutionGraph, _host_lyr_addr: LayerAddress) -> CmnResult<()> {
        self.set_exe_order(exe_graph)
    }

    fn cycle_pre(&mut self, _exe_graph: &mut ExecutionGraph, _host_lyr_addr: LayerAddress) -> CmnResult<()> {
        Ok(())
    }

    fn cycle_post(&mut self, exe_graph: &mut ExecutionGraph, host_lyr_addr: LayerAddress) -> CmnResult<()> {
        self.cycle(exe_graph, host_lyr_addr)
    }

    fn layer_name(&self) -> &'static str { self.layer_name() }
    fn layer_addr(&self) -> LayerAddress { self.layer_addr() }
    fn host_layer_addr(&self) -> LayerAddress { self.host_lyr_addr }
}