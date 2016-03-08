
use rand::{self, XorShiftRng, Rng};

use cmn::{self, CorticalDims, DataCellLayer};
use map::{AreaMap};
use ocl::{ProQue, SpatialDims, Buffer, Kernel, EventList, Result as OclResult};
use ocl::traits::OclPrm;
use proto::{CellKind, Protocell, DendriteKind};
use dendrites::{Dendrites};
use axon_space::{AxonSpace};


/* PyramidalLayer
    flag_sets: 0b10000000 (0x80) -> previously active

*/
pub struct PyramidalLayer {
    layer_name: &'static str,
    dims: CorticalDims,
    protocell: Protocell,
    kern_ltp: Kernel,
    kern_cycle: Kernel,
    base_axn_slc: u8,
    pyr_lyr_axn_idz: u32,
    rng: XorShiftRng,
    tfts_per_cel: u32,
    dens_per_tft_l2: u8,
    syns_per_den_l2: u8,
    states: Buffer<u8>,
    flag_sets: Buffer<u8>,
    best_den_states: Buffer<u8>,
    tft_best_den_ids: Buffer<u8>,
    tft_best_den_states: Buffer<u8>,
    // energies: Buffer<u8>, // <<<<< SLATED FOR REMOVAL
    pub dens: Dendrites,
}

impl PyramidalLayer {
    pub fn new(layer_name: &'static str, dims: CorticalDims, protocell: Protocell, 
        area_map: &AreaMap, axons: &AxonSpace, ocl_pq: &ProQue
    ) -> PyramidalLayer {
        let base_axn_slcs = area_map.layer_slc_ids(vec![layer_name]);
        let base_axn_slc = base_axn_slcs[0];
        let pyr_lyr_axn_idz = area_map.axn_idz(base_axn_slc);        

        let tfts_per_cel = area_map.layer_dst_srcs(layer_name).len() as u32;

        // let best_dens_per_cel = tfts_per_cel;
        let dims_best_dens = dims.clone().with_tfts(tfts_per_cel);

        let states = Buffer::<u8>::newer_new(ocl_pq.queue(), None, &dims, None).unwrap();
        let flag_sets = Buffer::<u8>::newer_new(ocl_pq.queue(), None, &dims, None).unwrap();
        let best_den_states = Buffer::<u8>::newer_new(ocl_pq.queue(), None, &dims, None).unwrap();
        // let tft_best_den_ids = Buffer::<u8>::with_vec(&dims_best_dens, ocl_pq.queue());
        let tft_best_den_ids = Buffer::<u8>::newer_new(ocl_pq.queue(), None, &dims_best_dens, None).unwrap();
        // let tft_best_den_states = Buffer::<u8>::with_vec(&dims_best_dens, ocl_pq.queue());        
        let tft_best_den_states = Buffer::<u8>::newer_new(ocl_pq.queue(), None, &dims_best_dens, None).unwrap();
        // let energies = Buffer::<u8>::with_vec(&dims, 255, ocl); // <<<<< SLATED FOR REMOVAL

        let dens_per_tft_l2 = protocell.dens_per_tuft_l2;
        let syns_per_den_l2 = protocell.syns_per_den_l2;
        // let syns_per_tft_l2 = dens_per_tft_l2 + syns_per_den_l2;

        let dims_dens = dims.clone_with_ptl2(dens_per_tft_l2 as i8).with_tfts(tfts_per_cel);

        println!("{mt}{mt}PYRAMIDALS::NEW(): layer: '{}' base_axn_slc: {}, \
            pyr_lyr_axn_idz: {}, tfts_per_cel: {}, syns_per_den_l2: {}, dens_per_tft_l2: {}, \
            best_den_len: {}.", 
            layer_name, base_axn_slc, pyr_lyr_axn_idz, tfts_per_cel, syns_per_den_l2, dens_per_tft_l2, 
            tft_best_den_ids.len(), mt = cmn::MT);

        let dens = Dendrites::new(layer_name, dims_dens, protocell.clone(), DendriteKind::Distal, CellKind::Pyramidal, area_map, axons, ocl_pq);        
        
        let kern_cycle = ocl_pq.create_kernel("pyr_cycle")
            // .expect("PyramidalLayer::new()")
            .gws(SpatialDims::One(dims.cells() as usize))
            .arg_buf(dens.states_raw())
            .arg_buf(dens.states())
            .arg_scl(tfts_per_cel)
            .arg_scl(dens_per_tft_l2)
            //.arg_buf(&energies) // <<<<< SLATED FOR REMOVAL
            .arg_buf(&tft_best_den_ids)
            .arg_buf(&tft_best_den_states)
            .arg_buf(&best_den_states)
            .arg_buf_named::<i32>("aux_ints_0", None)
            .arg_buf_named::<i32>("aux_ints_1", None)
            .arg_buf(&states);

        // let syns_per_tftsec = dens.syns().syns_per_tftsec();
        let cel_grp_count = cmn::OPENCL_MINIMUM_WORKGROUP_SIZE;
        let cels_per_cel_grp = dims.per_subgrp(cel_grp_count, ocl_pq).expect("PyramidalLayer::new()");
        let learning_rate_l2i = 0i32;

        let kern_ltp = ocl_pq.create_kernel("pyrs_ltp")
            // .expect("PyramidalLayer::new()")
            .gws(SpatialDims::One(cel_grp_count as usize))
            .arg_buf(&axons.states)
            .arg_buf(&states)
            .arg_buf(&tft_best_den_ids)
            .arg_buf(&tft_best_den_states)
            .arg_buf(dens.states())
            .arg_buf(dens.syns().states())
            .arg_scl(tfts_per_cel as u32)
            .arg_scl(dens_per_tft_l2 as u32)
            .arg_scl(syns_per_den_l2 as u32)            
            .arg_scl(cels_per_cel_grp)
            .arg_scl(pyr_lyr_axn_idz)
            .arg_scl_named::<i32>("lr_l2i", Some(learning_rate_l2i))
            .arg_scl_named::<i32>("rnd", None)        
            .arg_buf(dens.syns().flag_sets())
            .arg_buf(&flag_sets)
            .arg_buf_named::<i32>("aux_ints_0", None)
            .arg_buf_named::<i32>("aux_ints_1", None)
            .arg_buf(dens.syns().strengths());

        PyramidalLayer {
            layer_name: layer_name,
            dims: dims,
            protocell: protocell,
            kern_ltp: kern_ltp,
            kern_cycle: kern_cycle,
            base_axn_slc: base_axn_slc,
            pyr_lyr_axn_idz: pyr_lyr_axn_idz,
            rng: rand::weak_rng(),
            tfts_per_cel: tfts_per_cel,
            dens_per_tft_l2: dens_per_tft_l2,
            syns_per_den_l2: syns_per_den_l2,
            states: states,
            flag_sets: flag_sets,
            best_den_states: best_den_states,
            tft_best_den_ids: tft_best_den_ids,
            tft_best_den_states: tft_best_den_states,            
            // energies: energies, // <<<<< SLATED FOR REMOVAL
            dens: dens,
        }
    }

    // USED BY AUX
    #[inline]
    pub fn kern_ltp(&mut self) -> &mut Kernel {
        &mut self.kern_ltp
    }

    // USED BY AUX
    #[inline]
    pub fn kern_cycle(&mut self) -> &mut Kernel {
        &mut self.kern_cycle
    }

    pub fn dens_per_tft_l2(&self) -> u8 {
        self.dens_per_tft_l2
    }

    pub fn syns_per_den_l2(&self) -> u8 {
        self.syns_per_den_l2
    }

    pub fn states(&self) -> &Buffer<u8> {
        &self.states
    }

    pub fn flag_sets(&self) -> &Buffer<u8> {
        &self.flag_sets
    }

    pub fn best_den_states(&self) -> &Buffer<u8> {
        &self.best_den_states
    }

    pub fn tft_best_den_ids(&self) -> &Buffer<u8> {
        &self.tft_best_den_ids
    }

    pub fn tft_best_den_states(&self) -> &Buffer<u8> {
        &self.tft_best_den_states
    }

    // <<<<< TODO: DEPRICATE >>>>>
    pub fn set_arg_buf_named<T: OclPrm>(&mut self, name: &'static str, env: &Buffer<T>)
            -> OclResult<()> 
    {
        let using_aux_cycle = true;
        let using_aux_learning = true;

        if using_aux_cycle {
            try!(self.kern_cycle.set_arg_buf_named(name, Some(env)));
        }

        if using_aux_learning {
            try!(self.kern_ltp.set_arg_buf_named(name, Some(env)));
        }

        Ok(())
    }
}

impl DataCellLayer for PyramidalLayer {
    #[inline]
    fn learn(&mut self) {
        self.kern_ltp.set_arg_scl_named("rnd", self.rng.gen::<i32>()).unwrap();
        self.kern_ltp.enqueue();
    }

    #[inline]
    fn regrow(&mut self) {
        self.dens_mut().regrow();
        // panic!("Pyramidals::regrow(): reimplement me!");
    }

    #[inline]
    fn cycle(&self, wait_events: Option<&EventList>) {
        self.dens().cycle(wait_events);
        // self.kern_cycle.enqueue_events(wait_events, None)
        //     .expect("bismit::PyramidalLayer::cycle");
        self.kern_cycle.cmd().ewait_opt(wait_events).enq().expect("bismit::PyramidalLayer::cycle");
    }

    #[inline]
    fn soma(&self) -> &Buffer<u8> {
        &self.states
    }

    #[inline]
    fn soma_mut(&mut self) -> &mut Buffer<u8> {
        &mut self.states
    }    

    #[inline]
    fn dims(&self) -> &CorticalDims {
        &self.dims
    }

    #[inline]
    fn axn_range(&self) -> (usize, usize) {
        let ssts_axn_idn = self.pyr_lyr_axn_idz + (self.dims.per_slc());

        (self.pyr_lyr_axn_idz as usize, ssts_axn_idn as usize)
    }

    #[inline]
    fn base_axn_slc(&self) -> u8 {
        self.base_axn_slc
    }

    #[inline]
    fn tfts_per_cel(&self) -> u32 {
        self.tfts_per_cel
    }

    #[inline]
    fn layer_name(&self) -> &'static str {
        self.layer_name
    }

    #[inline]
    fn protocell(&self) -> &Protocell {
        &self.protocell
    }

    #[inline]
    fn dens(&self) -> &Dendrites {
        &self.dens
    }

    #[inline]
    fn dens_mut(&mut self) -> &mut Dendrites {
        &mut self.dens
    }
}


#[cfg(test)]
pub mod tests {
    // use std::ops::{Range};
    use rand::{XorShiftRng};
    use rand::distributions::{IndependentSample, Range as RandRange};

    use cmn::{self, DataCellLayer, DataCellLayerTest, CelCoords};
    use super::{PyramidalLayer};
    use synapses::{SynapsesTest};

    impl DataCellLayerTest for PyramidalLayer {
        // CYCLE_SELF_ONLY(): USED BY TESTS
        fn cycle_self_only(&self) {
            self.kern_cycle.enqueue();
        }

        // fn print_cel(&mut self, cel_idx: usize) {
        //     let emsg = "PyramidalLayer::print_cel()";

        //     self.confab();

        //     let cel_den_idz = (cel_idx << self.dens_mut().dims().per_tft_l2_left()) as usize;
        //     let cel_syn_idz = (cel_idx << self.dens_mut().syns_mut().dims().per_tft_l2_left()) as usize;

        //     let dens_per_tft = self.dens_mut().dims().per_cel() as usize;
        //     let syns_per_tft = self.dens_mut().syns_mut().dims().per_cel() as usize;

        //     let cel_den_range = cel_den_idz..(cel_den_idz + dens_per_tft);
        //     let cel_syn_range = cel_syn_idz..(cel_syn_idz + syns_per_tft);

        //     println!("Printing Pyramidal Cell:");
        //     println!("   states[{}]: {}", cel_idx, self.states[cel_idx]);
        //     println!("   flag_sets[{}]: {}", cel_idx, self.flag_sets[cel_idx]);
        //     println!("   best_den_states[{}]: {}", cel_idx, self.best_den_states[cel_idx]);
        //     println!("   tft_best_den_ids[{}]: {}", cel_idx, self.tft_best_den_ids[cel_idx]);
        //     println!("   tft_best_den_states[{}]: {}", cel_idx, self.tft_best_den_states[cel_idx]);
            
        //     // println!("   energies[{}]: {}", cel_idx, self.energies[cel_idx]); // <<<<< SLATED FOR REMOVAL

        //     println!("");

        //     println!("dens.states[{:?}]: ", cel_den_range.clone()); 
        //     self.dens.states.print(1, None, Some(cel_den_range.clone()), false);

        //     println!("dens.syns().states[{:?}]: ", cel_syn_range.clone()); 
        //     self.dens.syns_mut().states.print(1, None, Some(cel_den_range.clone()), false);

        //     println!("dens.syns().strengths[{:?}]: ", cel_syn_range.clone()); 
        //     self.dens.syns_mut().strengths.print(1, None, Some(cel_den_range.clone()), false);

        //     println!("dens.src_col_v_offs[{:?}]: ", cel_syn_range.clone()); 
        //     self.dens.syns_mut().src_col_v_offs.print(1, None, Some(cel_den_range.clone()), false);

        //     println!("dens.src_col_u_offs[{:?}]: ", cel_syn_range.clone()); 
        //     self.dens.syns_mut().src_col_u_offs.print(1, None, Some(cel_den_range.clone()), false);
        // }    

        // // PRINT_ALL(): TODO: [complete] change argument to print dens at some point
        // fn print_range(&mut self, range: Range<usize>, print_children: bool) {
        //     print!("pyrs.states: ");
        //     self.states.print(1, Some((0, 255)), None, false);
        //     print!("pyrs.flag_sets: ");
        //     self.flag_sets.print(1, Some((0, 255)), None, false);
        //     print!("pyrs.best_den_states: ");
        //     self.best_den_states.print(1, Some((0, 255)), None, false);
        //     print!("pyrs.tft_best_den_ids: ");
        //     self.tft_best_den_ids.print(1, Some((0, 255)), None, false);
        //     print!("pyrs.tft_best_den_states: ");
        //     self.tft_best_den_states.print(1, Some((0, 255)), None, false);
                        
        //     // print!("pyrs.energies: ");                            // <<<<< SLATED FOR REMOVAL
        //     // self.energies.print(1, Some((0, 255)), None, false); // <<<<< SLATED FOR REMOVAL


        //     if print_children {
        //         print!("dens.states: ");
        //         // FOR EACH TUFT:
        //             // Calculate range for tuft dens
        //             self.dens.states.print(1, Some((1, 255)), None, false);
        //             // Calculate range for tuft syns
        //             self.dens.syns_mut().print_all(); 
        //     }
        // }

        // fn print_all(&mut self, print_children: bool) {
        //     let range = 0..self.states.len();
        //     self.print_range(range, print_children);
        // }

        fn rng(&mut self) -> &mut XorShiftRng {
            &mut self.rng
        }

        fn rand_cel_coords(&mut self) -> CelCoords {
            let slc_range = RandRange::new(0, self.dims().depth());
            let v_range = RandRange::new(0, self.dims().v_size());
            let u_range = RandRange::new(0, self.dims().u_size());

            let slc_id_lyr = slc_range.ind_sample(self.rng());
            let u_id = u_range.ind_sample(self.rng());
            let v_id = v_range.ind_sample(self.rng());

            let axn_slc_id = self.base_axn_slc() + slc_id_lyr;

            CelCoords::new(axn_slc_id, slc_id_lyr, v_id, u_id, self.dims(),
                self.tfts_per_cel, self.dens_per_tft_l2(), self.syns_per_den_l2())
        }
        

        fn cel_idx(&self, slc_id: u8, v_id: u32, u_id: u32)-> u32 {
            cmn::cel_idx_3d(self.dims().depth(), slc_id, self.dims().v_size(), v_id, self.dims().u_size(), u_id)
        }

        fn set_all_to_zero(&mut self) { // MOVE TO TEST TRAIT IMPL
            self.states.cmd().fill(&[0], None).enq().unwrap();
            self.flag_sets.cmd().fill(&[0], None).enq().unwrap();
            self.best_den_states.cmd().fill(&[0], None).enq().unwrap();
            self.tft_best_den_ids.cmd().fill(&[0], None).enq().unwrap();
            self.tft_best_den_states.cmd().fill(&[0], None).enq().unwrap();
            //self.best2_den_ids.cmd().fill(&[0], None).enq().unwrap();            // <<<<< SLATED FOR REMOVAL
            //self.best2_den_states.cmd().fill(&[0], None).enq().unwrap();        // <<<<< SLATED FOR REMOVAL
            
            // self.energies.cmd().fill(&[0], None).enq().unwrap();                // <<<<< SLATED FOR REMOVAL
        }

        // fn confab(&mut self) {
        //     self.states.fill_vec();
        //     self.best_den_states.fill_vec();
        //     self.tft_best_den_ids.fill_vec();
        //     self.tft_best_den_states.fill_vec();
        //     self.flag_sets.fill_vec();
        //     // self.energies.fill_vec(); // <<<<< SLATED FOR REMOVAL

        //     self.dens_mut().confab();
        // }
    }
}

