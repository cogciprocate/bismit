use rand::Rng;
use cmn::{self, CmnResult};
use map::{AreaMap, LayerAddress, ExecutionGraph, CommandRelations, CorticalBuffer, CellScheme, CommandUid};
use ocl::{Kernel, ProQue, SpatialDims, Event};
use cortex::{AxonSpace, ControlCellLayer, DataCellLayer, CorticalAreaSettings};

/// Basket cells.
#[derive(Debug)]
pub struct InhibitoryInterneuronNetwork {
    layer_name: String,
    layer_addr: LayerAddress,
    host_lyr_addr: LayerAddress,
    kern_inhib_simple: Kernel,
    kern_inhib_passthrough: Kernel,
    exe_cmd_uid: CommandUid,
    exe_cmd_idx: usize,
    rng: cmn::XorShiftRng,
    settings: CorticalAreaSettings,
}

impl InhibitoryInterneuronNetwork {
    // FIXME: This function should take a 'bypass' argument instead of `::cycle`.
    pub fn new<S, D>(layer_name: S, layer_id: usize, scheme: CellScheme,
            host_lyr: &D, axns: &AxonSpace, area_map: &AreaMap,
            ocl_pq: &ProQue, settings: CorticalAreaSettings, exe_graph: &mut ExecutionGraph)
            -> CmnResult<InhibitoryInterneuronNetwork>
            where S: Into<String>, D: DataCellLayer
    {
        let layer_name = layer_name.into();
        let layer_addr = LayerAddress::new(area_map.area_id(), layer_id);
        let host_lyr_slc_ids = area_map.layer_slc_ids(&[host_lyr.layer_name()]);
        let host_lyr_base_axn_slc = host_lyr_slc_ids[0];

        // Ensure that the host layer is constructed correctly.
        debug_assert_eq!(host_lyr.soma().len(), host_lyr.energies().len());

        let inhib_radius = scheme.class().control_kind().field_radius() as i32;

        // Simple (active) kernel:
        let kern_inhib_simple_name = "inhib_simple";
        let kern_inhib_simple = ocl_pq.create_kernel(kern_inhib_simple_name)?
            .gws(SpatialDims::Three(host_lyr.dims().depth() as usize, host_lyr.dims().v_size() as usize,
                host_lyr.dims().u_size() as usize))
            .lws(SpatialDims::Three(1, 8, 8 as usize))
            .arg_buf(host_lyr.soma())
            // .arg_buf(host_lyr.energies())
            .arg_scl(host_lyr_base_axn_slc)
            .arg_scl(inhib_radius)
            .arg_scl_named::<i32>("rnd", None)
            .arg_buf(host_lyr.activities())
            // .arg_buf_named("aux_ints_0", None)
            // .arg_buf_named("aux_ints_1", None)
            .arg_buf(axns.states());

        // Passthrough kernel:
        let kern_inhib_passthrough_name = "inhib_passthrough";
        let kern_inhib_passthrough = ocl_pq.create_kernel(kern_inhib_passthrough_name)?
            .gws(SpatialDims::Three(host_lyr.dims().depth() as usize, host_lyr.dims().v_size() as usize,
                host_lyr.dims().u_size() as usize))
            .arg_buf(host_lyr.soma())
            .arg_scl(host_lyr_base_axn_slc)
            .arg_scl_named::<i32>("rnd", None)
            .arg_buf(host_lyr.activities())
            .arg_buf(axns.states());

        // let exe_cmd_srcs = (0..host_lyr.tft_count())
        //     .map(|host_lyr_tft_id| CorticalBuffer::data_den_tft(&host_lyr.soma(),
        //         host_lyr.layer_addr(), host_lyr_tft_id))
        //     .collect();
        let exe_cmd_srcs = vec![CorticalBuffer::data_soma_lyr(host_lyr.soma(), host_lyr.layer_addr())];

        // Set up execution command:
        let exe_cmd_tars = (host_lyr_base_axn_slc..host_lyr_base_axn_slc + host_lyr.dims().depth())
            .map(|slc_id| CorticalBuffer::axon_slice(&axns.states(), area_map.area_id(), slc_id))
            .collect();

        let exe_cmd_uid = exe_graph.add_command(CommandRelations::cortical_kernel(
             "inhib_...", exe_cmd_srcs, exe_cmd_tars))?;


        Ok(InhibitoryInterneuronNetwork {
            layer_name: layer_name,
            layer_addr: layer_addr,
            host_lyr_addr: host_lyr.layer_addr(),
            kern_inhib_simple: kern_inhib_simple,
            kern_inhib_passthrough: kern_inhib_passthrough,
            exe_cmd_uid,
            exe_cmd_idx: 0,
            rng: cmn::weak_rng(),
            settings: settings,
        })
    }

    pub fn set_exe_order(&mut self, exe_graph: &mut ExecutionGraph) -> CmnResult<()> {
        self.exe_cmd_idx = exe_graph.order_command(self.exe_cmd_uid)?;
        Ok(())
    }

    // FIXME: `::new` should take the `bypass` argument instead.
    pub fn cycle(&mut self, exe_graph: &mut ExecutionGraph, _host_lyr_addr: LayerAddress) -> CmnResult<()> {
        let mut event = Event::empty();

        if self.settings.bypass_inhib {
            unsafe {
                self.kern_inhib_passthrough.cmd()
                .ewait(exe_graph.get_req_events(self.exe_cmd_idx)?)
                .enew(&mut event)
                .enq()?;
            }
        } else {
            self.kern_inhib_simple.set_arg_scl_named("rnd", self.rng.gen::<i32>()).unwrap();
            unsafe {
                self.kern_inhib_simple.cmd()
                    .ewait(exe_graph.get_req_events(self.exe_cmd_idx)?)
                    .enew(&mut event)
                    .enq()?;
            }
        }

        exe_graph.set_cmd_event(self.exe_cmd_idx, Some(event))?;

        Ok(())
    }

    #[inline] pub fn layer_name<'s>(&'s self) -> &'s str { &self.layer_name }
    #[inline] pub fn layer_addr(&self) -> LayerAddress { self.layer_addr }

}

impl ControlCellLayer for InhibitoryInterneuronNetwork {
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

    fn layer_name<'s>(&'s self) -> &'s str { self.layer_name() }
    fn layer_addr(&self) -> LayerAddress { self.layer_addr }
    fn host_layer_addr(&self) -> LayerAddress { self.host_lyr_addr }
}