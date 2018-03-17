use std::ops::Range;
// use rand;
use cmn::{self, CorticalDims, CmnResult};
use map::{AreaMap, LayerAddress, ExecutionGraph, ExecutionCommand, CorticalBuffer};
use ocl::{self, ProQue, SpatialDims, Buffer, Kernel, Result as OclResult, Event};
use ocl::traits::OclPrm;
use cortex::{AxonSpace, PyramidalLayer, SpinyStellateLayer, CorticalAreaSettings, DataCellLayer};
#[cfg(test)]
pub use self::tests::{MinicolumnsTest};

const PRNT: bool = false;


pub struct Minicolumns {
    layer_id: usize,
    dims: CorticalDims,
    axn_slc_id: u8,
    lyr_axn_idz: u32,
    ff_layer_axn_idz: usize,
    // kern_activate: ocl::Kernel,
    // activate_exe_cmd_idx: Option<usize>,
    kern_activate: Vec<Kernel>,
    activate_exe_cmd_idx: Vec<usize>,
    // kern_output: ocl::Kernel,
    // output_exe_cmd_idx: Option<usize>,
    // rng: rand::XorShiftRng,
    pub flag_sets: Buffer<u8>,
    pub best_den_states: Buffer<u8>,
}

impl Minicolumns {
    pub fn new(layer_id: usize, dims: CorticalDims, area_map: &AreaMap, axons: &AxonSpace,
                ssts: &SpinyStellateLayer, temporal_pyrs: Vec<&PyramidalLayer>,
                ocl_pq: &ProQue,
                settings: CorticalAreaSettings, exe_graph: &mut ExecutionGraph,
            ) -> CmnResult<Minicolumns>
    {
        assert!(dims.depth() == 1);
        let mcol_axn_slc_range = area_map.layer(layer_id).unwrap().slc_range().unwrap();
        assert!(mcol_axn_slc_range.len() == 1);
        assert!(ssts.dims().depth() == 1, "Minicolumns cannot yet handle spiny stellate layers \
            with depth greater than 1.");
        assert!(ssts.dims().depth() as usize == ssts.axn_slc_ids().len());
        let sst_axn_slc_id = ssts.axn_slc_ids()[0];

        let layer_addr = LayerAddress::new(area_map.area_id(), layer_id);

        // UPDATE ME TO AREA_MAP SETUP
        let ff_layer_axn_idz = ssts.axn_range().0;

        println!("{mt}{mt}MINICOLUMNS::NEW() dims: {:?}", dims, mt = cmn::MT);

        let flag_sets = Buffer::<u8>::builder().queue(ocl_pq.queue().clone()).dims(&dims).fill_val(0).build()?;
        let best_den_states = Buffer::<u8>::builder().queue(ocl_pq.queue().clone()).dims(&dims).fill_val(0).build()?;

        // [FIXME]: TEMPORARY?:
        // [FIXME]: MAKE THIS CONSISTENT WITH 'aff_out_slc_range()':
        assert!(area_map.aff_out_slcs().len() == 1,
            "Afferent output slices currently limited to a maximum of 1.");

        // let aff_out_axn_slc = area_map.aff_out_slcs()[0];
        let mcol_axn_slc_id = mcol_axn_slc_range.start;
        let mcol_lyr_axn_idz = area_map.axn_idz(mcol_axn_slc_id);
        let pyr_lyr_axn_idz = area_map.axn_idz(temporal_pyrs.base_axn_slc());

        // let pyr_depth = area_map.ptal_layer().depth();

        let mut activate_kernels = Vec::with_capacity(temporal_pyrs.len());
        let mut activate_cmd_idxs = Vec::with_capacity(temporal_pyrs.len());

        /*=============================================================================
        ===============================================================================
        =============================================================================*/



        assert!(dims.v_size() == temporal_pyrs.dims().v_size() && dims.u_size() == temporal_pyrs.dims().u_size());

        // Activation kernel:
        let activate_kern_name = "mcol_activate_pyrs";
        let kern_activate = ocl_pq.create_kernel(activate_kern_name)
            .expect("Minicolumns::new()")
            .global_work_size(SpatialDims::Three(temporal_pyrs.dims().depth() as usize, dims.v_size() as usize,
                dims.u_size() as usize))
            .arg(&flag_sets)
            .arg(&best_den_states)
            .arg(temporal_pyrs.best_den_states_raw())
            .arg(temporal_pyrs.states())
            .arg(ff_layer_axn_idz as u32)
            .arg(pyr_lyr_axn_idz)
            // .arg(temporal_pyrs.cell_scheme().dens_per_tft_l2)
            .arg(temporal_pyrs.flag_sets())
            // .arg_named::<i32>("aux_ints_0", None)
            // .arg_named::<i32>("aux_ints_1", None)
            .arg(axons.states());

        // Activation execution command:
        let activate_cmd_srcs = vec![
            CorticalBuffer::control_soma_lyr(&flag_sets, layer_addr),
            CorticalBuffer::control_soma_lyr(&best_den_states, layer_addr),
            CorticalBuffer::data_soma_lyr(&temporal_pyrs.best_den_states_raw(), temporal_pyrs.layer_addr()),
            CorticalBuffer::data_soma_lyr(&temporal_pyrs.states(), temporal_pyrs.layer_addr()),
            CorticalBuffer::axon_slice(&axons.states(), layer_addr.area_id(), sst_axn_slc_id),
        ];

        let mut activate_cmd_tars = temporal_pyrs.axn_slc_ids().iter()
            .map(|&pyr_slc_id| CorticalBuffer::axon_slice(&axons.states(), layer_addr.area_id(),
                pyr_slc_id))
            .collect::<Vec<_>>();

        activate_cmd_tars.push(CorticalBuffer::data_soma_lyr(&temporal_pyrs.flag_sets(), temporal_pyrs.layer_addr()));

        // let activate_exe_cmd_idx = if !settings.disable_learning && !settings.disable_mcols {
        // let activate_exe_cmd_idx = if !settings.disable_learning {
        let activate_exe_cmd_idx = if !settings.disable_mcols {
            Some(exe_graph.add_command(ExecutionCommand::cortical_kernel(
                activate_kern_name, activate_cmd_srcs, activate_cmd_tars))?)
        } else {
            None
        };

        /*=============================================================================
        ===============================================================================
        =============================================================================*/

        // // Output kernel:
        // let output_kern_name = "mcol_output";
        // let kern_output = ocl_pq.create_kernel(output_kern_name)
        //     .expect("Minicolumns::new()")
        //     .global_work_size(SpatialDims::Two(dims.v_size() as usize, dims.u_size() as usize))
        //     .arg(pyrs.best_den_states_raw())
        //     .arg(pyrs.soma())
        //     // .arg(pyrs.tfts_per_cel())
        //     .arg(ff_layer_axn_idz as u32)
        //     .arg(pyr_depth)
        //     .arg(mcol_axn_slc_id)
        //     .arg(&flag_sets)
        //     .arg(&best_den_states)
        //     .arg(axons.states());

        // // Output execution command:
        // let output_cmd_srcs = vec![
        //     CorticalBuffer::data_soma_lyr(&pyrs.best_den_states_raw(), pyrs.layer_addr()),
        //     CorticalBuffer::data_soma_lyr(&pyrs.soma(), pyrs.layer_addr()),
        // ];

        // let output_cmd_tars = vec![
        //     CorticalBuffer::control_soma_lyr(&flag_sets, layer_addr),
        //     CorticalBuffer::control_soma_lyr(&best_den_states, layer_addr),
        //     CorticalBuffer::axon_slice(&axons.states(), layer_addr.area_id(), mcol_axn_slc_id),
        // ];

        // // let output_exe_cmd_idx = if settings.disable_learning {
        // let output_exe_cmd_idx = if !settings.disable_mcols {
        //     Some(exe_graph.add_command(ExecutionCommand::cortical_kernel(
        //         output_kern_name, output_cmd_srcs, output_cmd_tars))?)
        // } else {
        //     None
        // };

        /*=============================================================================
        ===============================================================================
        =============================================================================*/

        Ok(Minicolumns {
            layer_id: layer_id,
            dims: dims,
            axn_slc_id: mcol_axn_slc_id,
            lyr_axn_idz: mcol_lyr_axn_idz,
            ff_layer_axn_idz: ff_layer_axn_idz,
            kern_activate: kern_activate,
            activate_exe_cmd_idx: activate_exe_cmd_idx,
            // kern_output: kern_output,
            // output_exe_cmd_idx: output_exe_cmd_idx,
            // rng: rand::weak_rng(),
            flag_sets: flag_sets,
            best_den_states: best_den_states,
        })
    }

    pub fn set_exe_order_activate(&self, exe_graph: &mut ExecutionGraph) -> CmnResult<()> {
        if let Some(cmd_idx) = self.activate_exe_cmd_idx {
            exe_graph.order_command(cmd_idx)?;
        }
        Ok(())
    }

    pub fn set_exe_order_output(&self, exe_graph: &mut ExecutionGraph) -> CmnResult<()> {
        if let Some(cmd_idx) = self.output_exe_cmd_idx {
            exe_graph.order_command(cmd_idx)?;
        }
        Ok(())
    }

    // <<<<< TODO: DEPRICATE >>>>>
    pub fn set_arg<T: OclPrm>(&mut self, name: &'static str, env: &Buffer<T>)
            -> OclResult<()>
    {
        let activate_using_aux = false;
        // let output_using_aux = false;

        if activate_using_aux {
            try!(self.kern_activate.set_arg(name, Some(env)));
        }

        // if output_using_aux {
        //     try!(self.kern_output.set_arg(name, Some(env)));
        // }

        Ok(())
    }

    #[inline]
    pub fn activate(&self, exe_graph: &mut ExecutionGraph) -> CmnResult<()> {
        if let Some(cmd_idx) = self.activate_exe_cmd_idx {
            if PRNT { printlnc!(lime: "Mcols: Activating (cmd_idx: [{}])...", cmd_idx); }
            let mut event = Event::empty();
            self.kern_activate.cmd().ewait(exe_graph.get_req_events(cmd_idx).unwrap()).enew(&mut event).enq()?;
            exe_graph.set_cmd_event(cmd_idx, Some(event)).unwrap();
            if PRNT { printlnc!(lime: "Mcols: Activation complete."); }
        }
        Ok(())
    }

    pub fn output(&self, exe_graph: &mut ExecutionGraph) -> CmnResult<()> {
        if let Some(cmd_idx) = self.output_exe_cmd_idx {
            if PRNT { printlnc!(lime: "Mcols: Outputting (cmd_idx: [{}])...", cmd_idx); }
            let mut event = Event::empty();
            self.kern_output.cmd().ewait(exe_graph.get_req_events(cmd_idx).unwrap()).enew(&mut event).enq()?;
            exe_graph.set_cmd_event(cmd_idx, Some(event)).unwrap();
            if PRNT { printlnc!(lime: "Mcols: Output complete."); }
        }
        Ok(())
    }


    // pub fn confab(&mut self) {
    //     self.flag_sets.fill_vec();
    //     self.best_den_states.fill_vec();
    // }

    #[inline]
    pub fn ff_layer_axn_idz(&self) -> usize {
        self.ff_layer_axn_idz
    }

    // [FIXME]: CONVERT TO A RANGE (area_map.aff_out_slc_range)
    #[inline]
    pub fn axn_slc_id(&self) -> u8 {
        self.axn_slc_id
    }

    // AXN_OUTPUT_RANGE(): USED FOR TESTING / DEBUGGING PURPOSES
    pub fn aff_out_axn_range(&self) -> Range<usize> {
        self.lyr_axn_idz as usize..self.lyr_axn_idz as usize + self.dims.columns() as usize
    }

    #[inline]
    pub fn len(&self) -> usize {
        assert!(self.dims.to_len() == self.flag_sets.len());
        assert!(self.flag_sets.len() == self.best_den_states.len());
        self.flag_sets.len()
    }

    #[inline] pub fn kern_activate(&self) -> &ocl::Kernel { &self.kern_activate }
    // #[inline] pub fn kern_output(&self) -> &ocl::Kernel { &self.kern_output }
    #[inline] pub fn layer_id(&self) -> usize { self.layer_id }
}


#[cfg(test)]
pub mod tests {
    use std::ops::Range;
    use ocl::util;
    use super::Minicolumns;

    pub trait MinicolumnsTest {
        fn activate_solo(&self);
        fn output_solo(&self);
        fn print_range(&self, range: Option<Range<usize>>);
        fn print_all(&self);
    }

    impl MinicolumnsTest for Minicolumns {
        fn activate_solo(&self) {
            self.kern_activate.default_queue().unwrap().finish().unwrap();
            self.kern_activate.cmd().enq().expect("MinicolumnsTest::activate_solo");
            self.kern_activate.default_queue().unwrap().finish().unwrap();
        }

        // fn output_solo(&self) {
        //     self.kern_output.default_queue().unwrap().finish().unwrap();
        //     self.kern_output.cmd().enq().expect("MinicolumnsTest::output_solo");
        //     self.kern_output.default_queue().unwrap().finish().unwrap();
        // }

        fn print_range(&self, idx_range: Option<Range<usize>>) {
            let mut vec = vec![0; self.len()];

            print!("mcols.flag_sets: ");
            self.flag_sets.read(&mut vec).enq().unwrap();
            util::print_slice(&vec, 1 << 0, None, idx_range.clone(), false);

            print!("mcols.best_den_states: ");
            self.best_den_states.read(&mut vec).enq().unwrap();
            util::print_slice(&vec, 1 << 0, None, idx_range.clone(), false);
        }

        fn print_all(&self) {
            self.print_range(None);
        }
    }

}
