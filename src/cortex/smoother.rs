use cmn::{CorticalDims, CmnResult};
use map::{AreaMap, LayerAddress, ExecutionGraph, ExecutionCommand, CorticalBuffer};
use ocl::{Kernel, ProQue, SpatialDims, Buffer, Event};
use map::CellScheme;
use cortex::{AxonSpace, ControlCellLayer, DataCellLayer, CorticalAreaSettings};


// const OVERLAP_COUNT: usize = 6;
const GRP_SIDE_LEN: i32 = 4;


/// Generates a set of 'center' coordinates for cells grouped by overlap-layer
/// (and haphazardly sorted within).
///
/// `side_len` is the circumradius of the hexagon-shaped area which each cell
/// will influence.
fn gen_grp_centers(side_len: i32, dims: [i32; 2]) -> (Vec<i32>, Vec<i32>) {
    use cmn::HexGroupCenters;

    // Boundaries
    let l_bound = [0 - side_len, 0 - side_len];
    let u_bound = [dims[0] + side_len, dims[1] + side_len];

    let mut centers_u = Vec::with_capacity(4096);
    let mut centers_v = Vec::with_capacity(4096);

    assert!(side_len % 2 == 0);
    let ofs_dist = side_len / 2;

    let starts = [[0, ofs_dist], [-ofs_dist, ofs_dist], [-ofs_dist, 0],
        [0, -ofs_dist], [ofs_dist, -ofs_dist], [ofs_dist, 0]];

    for lyr in 0..starts.len() {
        let mut centers = HexGroupCenters::new(side_len, l_bound, u_bound);
        centers.populate(Some(starts[lyr]));

        for center in centers.set() {
            centers_v.push(center[0]);
            centers_u.push(center[1]);
        }
    }

    centers_u.shrink_to_fit();
    centers_v.shrink_to_fit();

    (centers_u, centers_v)
}


#[derive(Debug)]
pub struct ActivitySmoother {
    layer_name: &'static str,
    layer_addr: LayerAddress,
    host_lyr_addr: LayerAddress,
    centers_v: Buffer<i32>,
    centers_u: Buffer<i32>,
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

        let (centers_u_vec, centers_v_vec) = gen_grp_centers(GRP_SIDE_LEN,
            [dims.v_size() as i32, dims.u_size() as i32]);

        assert!(centers_v_vec.len() == centers_u_vec.len());
        let cell_count = centers_v_vec.len();

        let centers_u = Buffer::builder().queue(ocl_pq.queue().clone()).dims(cell_count).build()?;
        let centers_v = Buffer::builder().queue(ocl_pq.queue().clone()).dims(cell_count).build()?;

        // Kernel:
        let kern_name = "smooth";
        let kern = ocl_pq.create_kernel(kern_name)?
            // .gws(SpatialDims::Three(dims.depth() as usize, dims.v_size() as usize,
            //     dims.u_size() as usize))
            .gws(SpatialDims::One(cell_count))
            // .lws(SpatialDims::Three(1, 8, 8 as usize))
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
            centers_v,
            centers_u,
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