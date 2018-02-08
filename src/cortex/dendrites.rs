use rand::Rng;
use ocl::{ProQue, SpatialDims, Buffer, Kernel, Event};
use ocl::traits::OclPrm;
use cmn::{self, CmnResult, CorticalDims, XorShiftRng};
use map::{AreaMap, CellScheme, ExecutionGraph, CommandRelations,
    CorticalBuffer, LayerAddress, CommandUid};
use cortex::{AxonSpace, Synapses};
// #[cfg(test)]
pub use self::tests::{DenCoords, DendritesTest, den_idx};

const PRNT: bool = false;

#[derive(Debug)]
pub struct Dendrites {
    layer_name: String,
    layer_id: usize,
    dims: CorticalDims,
    kernels: Vec<Kernel>,
    thresholds: Buffer<u8>,
    states_raw: Buffer<u8>,
    states: Buffer<u8>,
    energies: Buffer<u8>,
    activities: Buffer<u8>,
    activity_counter: usize,
    syns: Synapses,
    den_idzs_by_tft: Vec<u32>,
    den_counts_by_tft: Vec<u32>,
    exe_cmd_uids: Vec<CommandUid>,
    exe_cmd_idxs: Vec<usize>,
    rng: XorShiftRng,
    bypass_exe_graph: bool,
}

impl Dendrites {
    pub fn new<S: Into<String>>(
            layer_name: S,
            layer_id: usize,
            dims: CorticalDims,
            cell_scheme: CellScheme,
            area_map: &AreaMap,
            axons: &AxonSpace,
            ocl_pq: &ProQue,
            bypass_exe_graph: bool,
            exe_graph: &mut ExecutionGraph)
            -> CmnResult<Dendrites> {
        let layer_name = layer_name.into();
        let tft_count = cell_scheme.tft_count();
        let layer_addr = LayerAddress::new(area_map.area_id(), layer_id);

        let mut kernels = Vec::with_capacity(tft_count);
        let mut den_idzs_by_tft = Vec::with_capacity(tft_count);
        let mut den_counts_by_tft = Vec::with_capacity(tft_count);
        let mut exe_cmd_uids = Vec::with_capacity(tft_count);
        let exe_cmd_idxs = Vec::with_capacity(tft_count);
        let mut den_count_ttl = 0u32;

        for tft_scheme in cell_scheme.tft_schemes() {
            let tft_den_idz = den_count_ttl;
            den_idzs_by_tft.push(tft_den_idz);

            let tft_den_count = dims.cells() * tft_scheme.dens_per_tft();
            assert!(tft_den_count > 0, "Dendrites::new: Tuft dendrite count may not be zero. \
                (dims.cells(): {}, tft_scheme.dens_per_tft(): {})",
                dims.cells(), tft_scheme.dens_per_tft());
            den_counts_by_tft.push(tft_den_count);

            den_count_ttl += tft_den_count;


            // // [DEBUG]:
            // println!("###########  DENDRITE: tft_den_idz: {}", tft_den_idz);
            // println!("###########  DENDRITE: tft_den_count: {}", tft_den_count);
            // println!("###########  DENDRITE: den_count_ttl: {}", den_count_ttl);
            // println!("");
        }

        let states_raw = Buffer::<u8>::builder().queue(ocl_pq.queue().clone()).len([den_count_ttl]).fill_val(0).build()?;
        let states = Buffer::<u8>::builder().queue(ocl_pq.queue().clone()).len([den_count_ttl]).fill_val(0).build()?;
        let thresholds = Buffer::<u8>::builder().queue(ocl_pq.queue().clone()).len([den_count_ttl]).fill_val(0).build()?;
        let energies = Buffer::builder().queue(ocl_pq.queue().clone()).len(den_count_ttl).fill_val(0).build()?;
        let activities = Buffer::builder().queue(ocl_pq.queue().clone()).len(den_count_ttl).fill_val(0).build()?;
        // energies.cmd().fill(255, None).enq().unwrap();
        // energies.cmd().fill(1, None).enq().unwrap();
        // energies.default_queue().unwrap().finish().unwrap();

        println!("{mt}{mt}{mt}DENDRITES::NEW(): '{}': dendrites with: dims:{:?}, len:{}",
            layer_name, dims, states.len(), mt = cmn::MT);

        let syns = Synapses::new(layer_name.clone(), layer_id, dims, cell_scheme.clone(),
            area_map, axons, ocl_pq, bypass_exe_graph, exe_graph)?;

        /*=============================================================================
        ===============================================================================
        =============================================================================*/

        for (tft_id, ((tft_scheme, &tft_den_idz), &tft_den_count)) in cell_scheme.tft_schemes().iter()
                .zip(den_idzs_by_tft.iter())
                .zip(den_counts_by_tft.iter())
                .enumerate()
        {
            let syns_per_den = tft_scheme.syns_per_den();
            let den_threshold = tft_scheme.thresh_init().unwrap_or(cmn::DENDRITE_DEFAULT_INITIAL_THRESHOLD);

            assert!(tft_id == tft_scheme.tft_id());

            let tft_syn_idz = syns.syn_idzs_by_tft()[tft_scheme.tft_id()];

            // [DEBUG]:
            println!("{mt}{mt}{mt}{mt}{mt}DENDRITE TUFT: \
                tft_id: {}, tft_den_idz: {}, tft_syn_idz: {}, tft_scheme: {:?}",
                tft_id, tft_den_idz, tft_syn_idz, tft_scheme, mt = cmn::MT);

            /*=============================================================================
            ===============================================================================
            =============================================================================*/

            let kern_name = "den_cycle_tft";
            kernels.push(ocl_pq.create_kernel(kern_name)?
                .gws(SpatialDims::One(tft_den_count as usize))
                .arg_buf(syns.states())
                .arg_buf(syns.strengths())
                .arg_scl(tft_den_idz)
                .arg_scl(tft_syn_idz)
                .arg_scl(syns_per_den)
                .arg_scl(den_threshold)
                .arg_scl_named::<i32>("rnd", None)
                .arg_buf(&energies)
                .arg_buf(&activities)
                .arg_buf(&states_raw)
                .arg_buf_named("aux_ints_0", None::<Buffer<i32>>)
                .arg_buf_named("aux_ints_1", None::<Buffer<i32>>)
                .arg_buf(&states)
            );

            if !bypass_exe_graph {
                exe_cmd_uids.push(exe_graph.add_command(CommandRelations::cortical_kernel(
                    kern_name,
                    vec![
                        CorticalBuffer::data_syn_tft(syns.states(), layer_addr, tft_id),
                        CorticalBuffer::data_syn_tft(syns.strengths(), layer_addr, tft_id)
                    ],
                    vec![
                        CorticalBuffer::data_den_tft(&energies, layer_addr, tft_id),
                        CorticalBuffer::data_den_tft(&activities, layer_addr, tft_id),
                        CorticalBuffer::data_den_tft(&states_raw, layer_addr, tft_id),
                        CorticalBuffer::data_den_tft(&states, layer_addr, tft_id),
                    ]
                ))?);
            }
        }

        /*=============================================================================
        ===============================================================================
        =============================================================================*/

        Ok(Dendrites {
            layer_name: layer_name,
            layer_id: layer_id,
            dims: dims,
            kernels: kernels,
            thresholds: thresholds,
            states_raw: states_raw,
            states: states,
            energies: energies,
            activities,
            activity_counter: 0,
            syns: syns,
            den_idzs_by_tft: den_idzs_by_tft,
            den_counts_by_tft: den_counts_by_tft,
            exe_cmd_uids,
            exe_cmd_idxs,
            rng: cmn::weak_rng(),
            bypass_exe_graph,
        })
    }

    pub fn set_exe_order(&mut self, exe_graph: &mut ExecutionGraph) -> CmnResult<()> {
        if !self.bypass_exe_graph {
            self.syns.set_exe_order(exe_graph)?;
            self.exe_cmd_idxs.clear();

            for &cmd_uid in self.exe_cmd_uids.iter() {
                if PRNT { println!("##### Ordering dendrite cmd_uid: {}", cmd_uid); }
                self.exe_cmd_idxs.push(exe_graph.order_command(cmd_uid)?);
            }
        }
        Ok(())
    }

    pub fn cycle(&mut self, exe_graph: &mut ExecutionGraph) -> CmnResult<()> {
        if PRNT { println!("    Dens: Cycling syns..."); }
        self.syns.cycle(exe_graph)?;

        for (kern, &cmd_idx) in self.kernels.iter_mut().zip(self.exe_cmd_idxs.iter()) {
            if PRNT { println!("    Dens: Cycling kern_cycle (exe_cmd_idx: [{}])...", cmd_idx); }

            kern.set_arg_scl_named("rnd", self.rng.gen::<i32>())?;

            let mut event = Event::empty();
            unsafe { kern.cmd().ewait(exe_graph.get_req_events(cmd_idx)?).enew(&mut event).enq()?; }
            exe_graph.set_cmd_event(cmd_idx, Some(event))?;

            if PRNT { kern.default_queue().unwrap().finish().unwrap(); }
        }

        self.activity_counter += 1;
        Ok(())
    }

    pub fn regrow(&mut self) {
        self.syns.regrow();
    }

    // Debugging purposes
    pub fn set_arg_buf_named<T: OclPrm>(&mut self, name: &'static str, buf: &Buffer<T>)
            -> CmnResult<()> {
        let using_aux = true;

        if using_aux {
            for kernel in self.kernels.iter_mut() {
                kernel.set_arg_buf_named(name, Some(buf))?;
            }
        }

        Ok(())
    }

    pub fn zero_activities(&mut self) {
        self.activity_counter = 0;
    }

    #[inline] pub fn layer_id(&self) -> usize { self.layer_id }
    #[inline] pub fn thresholds(&self) -> &Buffer<u8> { &self.thresholds }
    #[inline] pub fn states_raw(&self) -> &Buffer<u8> { &self.states_raw }
    #[inline] pub fn states(&self) -> &Buffer<u8> { &self.states }
    #[inline] pub fn states_mut(&mut self) -> &mut Buffer<u8> { &mut self.states }
    #[inline] pub fn energies(&self) -> &Buffer<u8> { &self.energies }
    #[inline] pub fn activities(&self) -> &Buffer<u8> { &self.activities }
    #[inline] pub fn activity_counter(&self) -> usize { self.activity_counter }
    #[inline] pub fn dims(&self) -> &CorticalDims { &self.dims }
    #[inline] pub fn syns(&self) -> &Synapses { &self.syns }
    #[inline] pub fn syns_mut(&mut self) -> &mut Synapses { &mut self.syns }
    #[inline] pub fn layer_name<'s>(&'s self) -> &'s str { &self.layer_name }
    #[inline] pub fn count(&self) -> u32 { self.states.len() as u32 }
    #[inline] pub fn tft_count(&self) -> usize { self.kernels.len() }
    #[inline] pub fn den_idzs_by_tft(&self) -> &[u32] { self.den_idzs_by_tft.as_slice() }
    #[inline] pub fn den_counts_by_tft(&self) -> &[u32] { self.den_counts_by_tft.as_slice() }
}



// #[cfg(test)]
pub mod tests {
    #![allow(non_snake_case, dead_code)]
    use std::ops::{Range};
    use std::fmt::{Display, Formatter, Result};
    use rand::distributions::{IndependentSample, Range as RandRange};
    use ocl::{util, OclPrm, Buffer};
    // use tests;
    use cmn::{CorticalDims};
    use cortex::{SynapsesTest, TuftDims, CelCoords, syn_idx};
    use super::{Dendrites};

    pub fn read_idx_direct<T: OclPrm>(idx: usize, buf: &Buffer<T>) -> T {
        let mut val: [T; 1] = [Default::default()];
        buf.cmd().read(&mut val[..]).offset(idx).enq().unwrap();
        val[0]
    }

    pub trait DendritesTest {
        fn set_all_to_zero(&mut self, set_syns_zero: bool);
        fn den_state_direct(&self, idx: u32) -> u8;
        fn rand_den_coords(&mut self, cel_coords: CelCoords) -> DenCoords;
        fn den_idx(&self, cel_coords: &CelCoords, tft_den_idz: u32,
            tft_dims: &TuftDims, den_id_celtft: u32) -> u32;
        fn cycle_solo(&self);
        fn tft_id_range(&self) -> Range<usize>;
        fn den_id_range_celtft(&self, tft_id: usize) -> Range<u32>;
        fn den_idx_range_celtft(&self, cel_coords: &CelCoords, tft_id: usize) -> Range<usize>;
        fn print_range(&self, idx_range: Option<Range<usize>>);
        fn print_all(&self);
    }

    impl DendritesTest for Dendrites {
        fn set_all_to_zero(&mut self, set_syns_zero: bool) {
            self.thresholds.default_queue().unwrap().finish().unwrap();
            self.states_raw.default_queue().unwrap().finish().unwrap();
            self.states.default_queue().unwrap().finish().unwrap();
            self.energies.default_queue().unwrap().finish().unwrap();

            self.thresholds.cmd().fill(0, None).enq().unwrap();
            self.states_raw.cmd().fill(0, None).enq().unwrap();
            self.states.cmd().fill(0, None).enq().unwrap();
            self.energies.cmd().fill(0, None).enq().unwrap();

            self.thresholds.default_queue().unwrap().finish().unwrap();
            self.states_raw.default_queue().unwrap().finish().unwrap();
            self.states.default_queue().unwrap().finish().unwrap();
            self.energies.default_queue().unwrap().finish().unwrap();

            if set_syns_zero { self.syns.set_all_to_zero() };
        }

        fn den_state_direct(&self, idx: u32) -> u8 {
            // let mut sdr = vec![0u8];
            // self.states.read(idx as usize, &mut sdr[..]).unwrap();
            // sdr[0]
            read_idx_direct(idx as usize, &self.states)
        }

        fn rand_den_coords(&mut self, cel_coords: CelCoords) -> DenCoords {
            let tft_id_range = RandRange::new(0, self.tft_count());
            let tft_id = tft_id_range.ind_sample(self.syns.rng());

            let tft_den_idz = self.den_idzs_by_tft[tft_id];
            let tft_dims = self.syns.tft_dims_by_tft()[tft_id].clone();

            // let dens_per_tft = self.den_id_range_celtft(tft_id).end;
            let dens_per_tft = tft_dims.dens_per_tft();
            let den_id_range_celtft = RandRange::new(0, dens_per_tft);
            let den_id_celtft = den_id_range_celtft.ind_sample(self.syns.rng());

            DenCoords::new(cel_coords, tft_id, tft_den_idz, tft_dims, den_id_celtft)
        }

        fn cycle_solo(&self) {
            for kern in self.kernels.iter() {
                kern.default_queue().unwrap().finish().unwrap();
                unsafe { kern.cmd().enq().expect("DendritesTest::cycle_solo"); }
                kern.default_queue().unwrap().finish().unwrap();
            }
        }

        fn den_idx(&self, cel_coords: &CelCoords, tft_den_idz: u32,
                tft_dims: &TuftDims, den_id_celtft: u32) -> u32 {
            den_idx(&self.dims, cel_coords.slc_id_lyr, cel_coords.v_id, cel_coords.u_id,
                tft_den_idz, tft_dims, den_id_celtft)
        }

        fn tft_id_range(&self) -> Range<usize> {
            0..(self.tft_count())
        }

        fn den_id_range_celtft(&self, tft_id: usize) -> Range<u32> {
            let dens_per_tft = self.syns().tft_dims_by_tft()[tft_id].dens_per_tft();
            0..dens_per_tft
        }

        // The dendrite index range for a single cell-tuft:
        fn den_idx_range_celtft(&self, cel_coords: &CelCoords, tft_id: usize) -> Range<usize> {
            let tft_den_idz = self.den_idzs_by_tft[tft_id];
            let tft_dims = &self.syns.tft_dims_by_tft()[tft_id];
            let den_idz_celtft = den_idx(&cel_coords.lyr_dims, cel_coords.slc_id_lyr,
                cel_coords.v_id, cel_coords.u_id, tft_den_idz, tft_dims, 0) as usize;
            let dens_per_celtft = tft_dims.dens_per_tft() as usize;

            den_idz_celtft..(den_idz_celtft + dens_per_celtft)
        }

        fn print_range(&self, idx_range: Option<Range<usize>>) {
            let mut vec = vec![0; self.states.len()];

            print!("dens.states_raw: ");
            self.states_raw.read(&mut vec).enq().unwrap();
            util::print_slice(&vec, 1 << 0, None, idx_range.clone(), false);

            print!("dens.states: ");
            self.states.read(&mut vec).enq().unwrap();
            util::print_slice(&vec, 1 << 0, None, idx_range.clone(), false);
        }

        fn print_all(&self) {
            // let range = 0..self.states.len();
            self.print_range(None);
        }

    }

    // <<<<< TODO: NEEDS UPDATING TO MATCH / INTEGRATE WITH SYN_COORDS >>>>>
    #[derive(Debug, Clone)]
    pub struct DenCoords {
        pub idx: u32,
        // pub tft_id: u32,
        // pub den_id_tft: u32,
        // pub cel_coords: CelCoords,
        // pub layer_dims: CorticalDims, // Potentially move / remove dims
        pub cel_coords: CelCoords,
        pub tft_id: usize,
        pub tft_den_idz: u32,
        pub tft_dims: TuftDims,
        pub den_id_celtft: u32,
    }

    impl DenCoords {
        pub fn new(cel_coords: CelCoords, tft_id: usize, tft_den_idz: u32, tft_dims: TuftDims,
                den_id_celtft: u32) -> DenCoords {
            // let den_idx = den_idx(&layer_dims, tft_id, cel_coords.idx, den_id_tft);
            let den_idx = den_idx(&cel_coords.lyr_dims, cel_coords.slc_id_lyr, cel_coords.v_id,
                cel_coords.u_id, tft_den_idz, &tft_dims, den_id_celtft);

            DenCoords {
                // idx: den_idx,
                // tft_id: tft_id,
                // den_id_tft: den_id_tft,
                // cel_coords: cel_coords,
                // layer_dims: layer_dims,
                idx: den_idx,
                cel_coords: cel_coords,
                tft_id: tft_id,
                tft_den_idz: tft_den_idz,
                tft_dims: tft_dims,
                den_id_celtft: den_id_celtft,
            }
        }

        // The dendrite index range for this single cell-tuft:
        pub fn den_idx_range_celtft(&self) -> Range<usize> {
            let den_idz_celtft = den_idx(&self.cel_coords.lyr_dims, self.cel_coords.slc_id_lyr,
                self.cel_coords.v_id, self.cel_coords.u_id, self.tft_den_idz,
                &self.tft_dims, 0) as usize;
            let dens_per_celtft = self.tft_dims.dens_per_tft() as usize;

            den_idz_celtft..(den_idz_celtft + dens_per_celtft)
        }

        // The synapse index range for this single cell-tuft:
        pub fn syn_idx_range_celtft(&self, tft_id: usize, tft_syn_idz: u32) -> Range<usize> {
            assert!(tft_id == self.tft_id);
            // let syn_idz_cel_tft = (den_idx(&self.layer_dims, self.tft_id,
            //     self.cel_coords.idx, 0) as usize) << syns_per_den_l2 as usize;
            let syn_idz_celtft = syn_idx(&self.cel_coords.lyr_dims, self.cel_coords.slc_id_lyr,
                self.cel_coords.v_id, self.cel_coords.u_id, tft_syn_idz,
                &self.tft_dims, 0, 0) as usize;
            // let syns_per_cel_tft = (self.layer_dims.per_tft() as usize) << syns_per_den_l2 as usize;
            let syns_per_celtft = (self.tft_dims.dens_per_tft() * self.tft_dims.syns_per_den()) as usize;

            syn_idz_celtft..(syn_idz_celtft + syns_per_celtft)
        }

        // The synapse index range for this dendrite:
        pub fn syn_idx_range_den(&self, tft_id: usize, tft_syn_idz: u32) -> Range<usize> {
            // let syn_idz_den = (self.idx << syns_per_den_l2) as usize;
            // let syns_per_den = (1 << syns_per_den_l2) as usize;
            // syn_idz_den..(syn_idz_den + syns_per_den)

            assert!(tft_id == self.tft_id);

            // let syn_idz_celtft = syn_idx(&self.cel_coords.lyr_dims, self.cel_coords.slc_id_lyr,
            //     self.cel_coords.v_id, self.cel_coords.u_id, tft_syn_idz,
            //     &self.tft_dims, self.den_id_celtft, 0) as usize;
            let syn_idz_den = syn_idx(&self.cel_coords.lyr_dims, self.cel_coords.slc_id_lyr,
                self.cel_coords.v_id, self.cel_coords.u_id, tft_syn_idz,
                &self.tft_dims, self.den_id_celtft, 0) as usize;

            // let syns_per_celtft = 1 << (self.tft_dims.dens_per_tft_l2() as u32 +
            //     self.tft_dims.syns_per_den_l2() as u32);
            let syns_per_den = self.tft_dims.syns_per_den() as usize;

            syn_idz_den..(syn_idz_den + syns_per_den)
        }

        #[allow(dead_code)]
        pub fn lyr_dims(&self) -> &CorticalDims {
            &self.cel_coords.lyr_dims
        }
    }

    impl Display for DenCoords {
        fn fmt(&self, fmtr: &mut Formatter) -> Result {
            write!(fmtr, "DenCoords {{ idx: {}, tft_id: {}, den_id_celtft: {} }}",
                self.idx, self.tft_id, self.den_id_celtft)
        }
    }

    /// Returns the absolute index of a dendrite within a layer.
    ///
    /// * Synapse/Dendrite index space hierarchy:
    ///   { [Layer >] Tuft > Slice > Cell > Dendrite > Synapse }
    ///
    // NOTE: 'lyr_dims' expresses dimensions from the perspective of the
    // { [Layer >] Slice > Cell > Tuft > Dendrite > Synapse } hierarchy
    // which is why the naming here seem confusing (see explanation at top
    // of synapses.rs).
    pub fn den_idx(
            lyr_dims: &CorticalDims,
            slc_id_lyr: u8,
            v_id: u32,
            u_id: u32,
            tft_den_idz: u32,
            tft_dims: &TuftDims,
            den_id_celtft: u32)
            -> u32 {
        // Dendrites per cell-tuft:
        let dens_per_celtft = tft_dims.dens_per_tft();
        // Dendrites per tuft-slice:
        let dens_per_tftslc = lyr_dims.columns() * dens_per_celtft;

        // 0th dendrite in this tuft-slice:
        let tftslc_den_idz = (slc_id_lyr as u32 * dens_per_tftslc) + tft_den_idz;

        // Cell id within this tuft-slice:
        let cel_id_tftslc = (lyr_dims.u_size() * v_id) + u_id;
        // Dendrite id within this tuft-slice:
        let den_id_tftslc = (dens_per_celtft * cel_id_tftslc) + den_id_celtft;

        den_id_tftslc + tftslc_den_idz
    }
}
