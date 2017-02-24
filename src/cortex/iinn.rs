use cmn::{CorticalDims, CmnResult};
use map::{AreaMap, LayerAddress, ExecutionGraph, ExecutionCommand, CorticalBuffer};
use ocl::{Kernel, ProQue, SpatialDims, Buffer, Event};
use map::CellScheme;
use cortex::AxonSpace;


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
    pub fn new(layer_name: &'static str, layer_id: usize, dims: CorticalDims, _: CellScheme,
            area_map: &AreaMap, src_soma: &Buffer<u8>, src_layer_id: usize, src_base_axn_slc: u8,
            src_layer_tft_count: usize, axns: &AxonSpace, ocl_pq: &ProQue, exe_graph: &mut ExecutionGraph)
        -> CmnResult<InhibitoryInterneuronNetwork>
    {
        // let layer_addr = LayerAddress::new(area_map.area_id(), layer_id);

        let spi_ids = Buffer::<u8>::new(ocl_pq.queue().clone(), None, &dims, None, None::<(_, Option<()>)>).unwrap();
        let wins = Buffer::<u8>::new(ocl_pq.queue().clone(), None, &dims, None, None::<(_, Option<()>)>).unwrap();
        let states = Buffer::<u8>::new(ocl_pq.queue().clone(), None, &dims, None, None::<(_, Option<()>)>).unwrap();

        // Simple (active) kernel:
        let kern_inhib_simple = ocl_pq.create_kernel("inhib_simple")
            .expect("InhibitoryInterneuronNetwork::new()")
            .gws(SpatialDims::Three(dims.depth() as usize, dims.v_size() as usize,
                dims.u_size() as usize))
            .lws(SpatialDims::Three(1, 8, 8 as usize))
            .arg_buf(src_soma)
            .arg_scl(src_base_axn_slc)
            // .arg_buf_named("aux_ints_0", None)
            // .arg_buf_named("aux_ints_1", None)
            .arg_buf(axns.states());

        // Passthrough kernel:
        let kern_inhib_passthrough = ocl_pq.create_kernel("inhib_passthrough")
            .expect("InhibitoryInterneuronNetwork::new()")
            .gws(SpatialDims::Three(dims.depth() as usize, dims.v_size() as usize,
                dims.u_size() as usize))
            .arg_buf(src_soma)
            .arg_scl(src_base_axn_slc)
            .arg_buf(axns.states());


        let exe_cmd_srcs = (0..src_layer_tft_count)
            .map(|src_tft_id| CorticalBuffer::data_den_tft(&src_soma,
                LayerAddress::new(area_map.area_id(), src_layer_id), src_tft_id))
            .collect();

        // Set up execution command:
        let exe_cmd_tars = (src_base_axn_slc..src_base_axn_slc + dims.depth())
            .map(|slc_id| CorticalBuffer::axon_slice(&axns.states(), area_map.area_id(), slc_id))
            .collect();

        let exe_cmd_idx = exe_graph.add_command(ExecutionCommand::cortical_kernel(
            exe_cmd_srcs,
            exe_cmd_tars,
        ))?;


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

        exe_graph.set_cmd_event(self.exe_cmd_idx, event)?;

        Ok(())
    }

    #[inline] pub fn layer_name(&self) -> &'static str { self.layer_name }
    #[inline] pub fn layer_id(&self) -> usize { self.layer_id }

}
