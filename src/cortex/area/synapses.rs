//! Synapses: The smallest and most numerous unit in the cortex -- the soldier
//! at the bottom.
//!
//! #### [TODO]:
//! - [high priority] Testing:
//!   - [INCOMPLETE] Check for uniqueness and correct distribution frequency
//!     among src_slcs and cols
//! - [low priority] Optimization:
//!   - [Complete] Obviously grow() and it's ilk need a lot of work
//!
//! Synapse index space (for each of the synapse property buffers) is first
//! divided by tuft, then slice, then cell, then synapse. This means that even
//! though a cell may have three (or any number of) tufts, and you would
//! naturally tend to think that synapse space would be first divided by
//! slice, then cell, then tuft, tufts are moved to the front of that list.
//! The reason for this is nuanced but it basically boils down to performance.
//! When a kernel is processing synapses it's best to process tuft-at-a-time
//! as the first order iteration rather than slice or cell-at-a-time because
//! the each tuft inherently shares synapses whose axon sources are going to
//! tend to be similar, making cache performance consistently better. This
//! makes indexing very confusing so there's a definite trade off in
//! complexity (for us poor humans).
//!
//! Calculating a particular synapse index is shown below in syn_idx(). This
//! is the exact same method the kernel uses for addressing: tuft is most
//! significant, followed by slice, then cell, then synapse. Dendrites are not
//! necessary to calculate a synapses index unless you happen only to have a
//! synapses id (address) within a dendrite. Mostly the id within a cell is
//! used and the dendrite is irrelevant, especially on the host side.
//!
//! Synapse space breakdown (m := n - 1, n being the number of elements in any
//! particular segment):
//!     - Tuft[0]
//!         - Slice[0]               }-+_
//!             - Column[0] (v, w)   }---Cell
//!                 - Synapse[0]
//!                 ...
//!                 - Synapse[m]
//!             ...
//!             - Cell[m]
//!                 ...
//!         - Slice[1]
//!              ...
//!         ...
//!         - Slice[m]
//!              ...
//!     ...
//!     - Tuft[m]
//!         ...
//!
//! So even though tufts are, conceptually, children (sub-components) of a cell...
//! +-->
//! |    - Slice
//! |        - Cell
//! +--------<    - Tuft
//!                 - Dendrite
//!                     -Synapse
//!
//!  ... **for indexing purposes** tufts are parent to slices, which are
//!  parent to cells (then dendrites, then synapses).
//!


use rand::{self, XorShiftRng};

use cmn::{self, CmnResult, CorticalDims};
use map::{AreaMap, SynSrcSlices, SynSrcIdxCache, SynSrc};
use ocl::{ProQue, SpatialDims, Buffer, Kernel, Result as OclResult};
use ocl::traits::OclPrm;
use ocl::core::ClWaitList;
use map::{CellKind, CellScheme, DendriteKind};
use cortex::AxonSpace;

#[cfg(test)]
pub use self::tests::{SynCoords, SynapsesTest, syn_idx};

const DEBUG_NEW: bool = true;
const DEBUG_GROW: bool = true;
const DEBUG_REGROW_DETAIL: bool = false;
const DEBUG_KERN: bool = false;

#[derive(Clone, Debug)]
pub struct TuftDims {
    dens_per_tft_l2: u8,
    syns_per_den_l2: u8,
}

impl TuftDims {
    pub fn new(dens_per_tft_l2: u8, syns_per_den_l2: u8) -> TuftDims {
        TuftDims { dens_per_tft_l2: dens_per_tft_l2, syns_per_den_l2: syns_per_den_l2 }
    }

    #[inline] pub fn dens_per_tft_l2(&self) -> u8 { self.dens_per_tft_l2 }
    #[inline] pub fn syns_per_den_l2(&self) -> u8 { self.syns_per_den_l2 }
}


pub struct Synapses {
    layer_name: &'static str,
    layer_id: usize,
    dims: CorticalDims,
    // syns_per_den_l2: u8,
    // cell_scheme: CellScheme,
    // src_slc_ids_by_tft: Vec<Vec<u8>>,
    den_kind: DendriteKind,
    // cell_kind: CellKind,
    // since_decay: usize,
    kernels: Vec<Box<Kernel>>,
    src_idx_caches_by_tft: Vec<SynSrcIdxCache>,
    // src_slices_by_tft: Vec<SynSrcSlices>,
    syn_src_slices: SynSrcSlices,
    rng: XorShiftRng,
    states: Buffer<u8>,

    strengths: Buffer<i8>,
    src_slc_ids: Buffer<u8>,
    src_col_u_offs: Buffer<i8>,
    src_col_v_offs: Buffer<i8>,

    flag_sets: Buffer<u8>,

    vec_strengths: Vec<i8>,
    vec_src_slc_ids: Vec<u8>,
    vec_src_col_u_offs: Vec<i8>,
    vec_src_col_v_offs: Vec<i8>,
    // pub slc_pool: Buffer<u8>,  // BRING THIS BACK (OPTIMIZATION)

    syn_idzs_by_tft: Vec<u32>,
    syn_counts_by_tft: Vec<u32>,
    tft_dims_by_tft: Vec<TuftDims>,
}

impl Synapses {
    pub fn new(layer_name: &'static str, layer_id: usize, dims: CorticalDims, cell_scheme: CellScheme,
            den_kind: DendriteKind, _: CellKind, area_map: &AreaMap, axons: &AxonSpace,
            ocl_pq: &ProQue,
            ) -> CmnResult<Synapses>
    {
        let tft_count = cell_scheme.tft_count();

        let mut kernels = Vec::with_capacity(tft_count);
        // let mut src_slc_ids_by_tft = Vec::with_capacity(tft_count);
        let mut src_idx_caches_by_tft = Vec::with_capacity(tft_count);
        // let mut src_slices_by_tft = Vec::with_capacity(tft_count);
        let mut syn_idzs_by_tft = Vec::with_capacity(tft_count);
        let mut syn_counts_by_tft = Vec::with_capacity(tft_count);
        let mut tft_dims_by_tft = Vec::with_capacity(tft_count);
        let mut syn_count_ttl = 0u32;

        debug_assert!(cell_scheme.tft_schemes().len() == tft_count);

        // for tft_id in 0..tft_count {
        for tft_scheme in cell_scheme.tft_schemes() {
            let tft_dims = TuftDims::new(tft_scheme.dens_per_tft_l2(),
                tft_scheme.syns_per_den_l2());

            let syns_per_tft_l2: u8 = tft_scheme.dens_per_tft_l2() + tft_scheme.syns_per_den_l2();


            if DEBUG_NEW {
                println!("{mt}{mt}{mt}{mt}SYNAPSES::NEW(): Tuft: kind: {:?}, len: {},\n\
                    {mt}{mt}{mt}{mt}{mt}dims: {:?} \n\
                    {mt}{mt}{mt}{mt}{mt}dens_per_tft_l2: {}, syns_per_den_l2: {}",
                    den_kind, dims.to_len(), &dims, tft_scheme.dens_per_tft_l2(),
                    tft_scheme.syns_per_den_l2(), mt = cmn::MT);
            }

            // [TODO]: Use kernel to ascertain the optimal workgroup size increment.
            let min_wg_sqrt = 8 as usize;
            assert_eq!((min_wg_sqrt * min_wg_sqrt), cmn::OPENCL_MINIMUM_WORKGROUP_SIZE as usize);

            // // The number of cell-tufts in a syn-tuft-group-thingy. Obviously this
            // // is a bit confusing. Better naming needed. See module notes above
            // // for details.
            // let cels_per_syntuft = dims.cells();

            let tft_syn_idz = syn_count_ttl;
            let tft_syn_count = dims.cells() << syns_per_tft_l2;
            syn_count_ttl += tft_syn_count;

            syn_idzs_by_tft.push(tft_syn_idz);
            syn_counts_by_tft.push(tft_syn_count);

            tft_dims_by_tft.push(tft_dims.clone());

            src_idx_caches_by_tft.push(SynSrcIdxCache::new(tft_syn_idz as usize,
                tft_dims, dims.clone()));

            kernels.push(Box::new({
                ocl_pq.create_kernel("tft_cycle_syns")
                // ocl_pq.create_kernel("tft_cycle_syns_vec4")
                // ocl_pq.create_kernel("layer_cycle_syns_wow")
                // ocl_pq.create_kernel("layer_cycle_syns_wow_vec4")
                    .expect("Synapses::new()")
                    .gws(SpatialDims::Two(dims.v_size() as usize, (dims.u_size()) as usize))
                    .lws(SpatialDims::Two(min_wg_sqrt, min_wg_sqrt))
                    .arg_buf(&axons.states)
                    .arg_buf_named("src_col_u_offs", None::<&Buffer<u8>>)
                    .arg_buf_named("src_col_v_offs", None::<&Buffer<u8>>)
                    .arg_buf_named("src_slc_ids", None::<&Buffer<u8>>)
                    // .arg_scl(tft_id as u32 * cels_per_syntuft)
                    .arg_scl(tft_syn_idz)
                    .arg_scl(syns_per_tft_l2)
                    .arg_scl(dims.depth() as u8)
                    .arg_buf_named::<i32>("aux_ints_0", None)
                    .arg_buf_named::<i32>("aux_ints_1", None)
                    .arg_buf_named("states", None::<&Buffer<u8>>)
            }));
        }

        let syn_src_slices = SynSrcSlices::new(layer_id, cell_scheme.tft_schemes(), area_map)?;

        // * Loop through kernels first, use named kernel args, then loop
        //   again using the determined synapse totals to create the buffers.

        // let slc_pool = Buffer::with_vec(cmn::SYNAPSE_ROW_POOL_SIZE, 0, ocl_pq); // BRING THIS BACK
        let states = Buffer::<u8>::new(ocl_pq.queue().clone(), None, [syn_count_ttl], None).unwrap();
        let strengths = Buffer::<i8>::new(ocl_pq.queue().clone(), None, [syn_count_ttl], None).unwrap();
        let src_slc_ids = Buffer::<u8>::new(ocl_pq.queue().clone(), None, [syn_count_ttl], None).unwrap();
        let src_col_u_offs = Buffer::<i8>::new(ocl_pq.queue().clone(), None, [syn_count_ttl], None).unwrap();
        let src_col_v_offs = Buffer::<i8>::new(ocl_pq.queue().clone(), None, [syn_count_ttl], None).unwrap();
        let flag_sets = Buffer::<u8>::new(ocl_pq.queue().clone(), None, [syn_count_ttl], None).unwrap();


        for kernel in kernels.iter_mut() {
            kernel.set_arg_buf_named("src_col_u_offs", Some(&src_col_u_offs))?;
            kernel.set_arg_buf_named("src_col_v_offs", Some(&src_col_v_offs))?;
            kernel.set_arg_buf_named("src_slc_ids", Some(&src_slc_ids))?;
            kernel.set_arg_buf_named("states", Some(&states))?;
        }

        debug_assert!(strengths.len() == src_slc_ids.len() &&
            strengths.len() == src_col_v_offs.len() &&
            strengths.len() == src_col_u_offs.len());

        // These are for learning (to avoid allocating it every time).
        let vec_strengths = vec![0; strengths.len()];
        let vec_src_slc_ids = vec![0; src_slc_ids.len()];
        let vec_src_col_u_offs = vec![0; src_col_u_offs.len()];
        let vec_src_col_v_offs = vec![0; src_col_v_offs.len()];

        let mut syns = Synapses {
            layer_name: layer_name,
            layer_id: layer_id,
            dims: dims,
            // syns_per_den_l2: cell_scheme.syns_per_den_l2,
            // cell_scheme: cell_scheme,
            // src_slc_ids_by_tft: src_slc_ids_by_tft,
            den_kind: den_kind,
            // cell_kind: cell_kind,
            // since_decay: 0,
            kernels: kernels,
            src_idx_caches_by_tft: src_idx_caches_by_tft,
            // src_slices_by_tft: src_slices_by_tft,
            syn_src_slices: syn_src_slices,
            rng: rand::weak_rng(),
            states: states,
            strengths: strengths,
            src_slc_ids: src_slc_ids,
            src_col_u_offs: src_col_u_offs,
            src_col_v_offs: src_col_v_offs,
            flag_sets: flag_sets,
            vec_strengths: vec_strengths,
            vec_src_slc_ids: vec_src_slc_ids,
            vec_src_col_u_offs: vec_src_col_u_offs,
            vec_src_col_v_offs: vec_src_col_v_offs,
            // slc_pool: slc_pool,  // BRING THIS BACK

            syn_counts_by_tft: syn_counts_by_tft,
            syn_idzs_by_tft: syn_idzs_by_tft,
            tft_dims_by_tft: tft_dims_by_tft,

        };

        syns.grow(true);
        // syns.refresh_slc_pool(); // BRING THIS BACK

        Ok(syns)
    }

    #[inline]
    pub fn cycle(&self, wait_events: Option<&ClWaitList>) {
        for kern in self.kernels.iter() {
            if DEBUG_KERN { printlnc!(yellow: "Syns: Enqueuing kernel: '{}'...", kern.name()); }

            kern.cmd().ewait_opt(wait_events).enq().expect("bismit::Synapses::cycle");

            if DEBUG_KERN { kern.default_queue().finish(); }
        }
    }

    #[inline]
    pub fn regrow(&mut self) {
        self.grow(false);
    }

    // pub fn confab(&mut self) {
    //     self.states.fill_vec();
    //     self.strengths.fill_vec();
    //     self.src_slc_ids.fill_vec();
    //     self.src_col_v_offs.fill_vec();
    // }

    // Debugging purposes
    // <<<<< TODO: DEPRICATE >>>>>
    pub fn set_arg_buf_named<T: OclPrm>(&mut self, name: &'static str, buf: &Buffer<T>)
            -> OclResult<()>
    {
        let using_aux = true;

        if using_aux {
            for kernel in self.kernels.iter_mut() {
                try!(kernel.set_arg_buf_named(name, Some(buf)));
            }
        }

        Ok(())
    }

    #[inline] pub fn len(&self) -> usize { self.states.len() }
    #[inline] pub fn layer_id(&self) -> usize { self.layer_id }
    #[inline] pub fn den_kind(&self) -> DendriteKind { self.den_kind.clone() }
    #[inline] pub fn lyr_dims(&self) -> &CorticalDims { &self.dims }
    #[inline] pub fn states(&self) -> &Buffer<u8> { &self.states }
    #[inline] pub fn strengths(&self) -> &Buffer<i8> { &self.strengths }
    #[inline] pub fn src_slc_ids(&self) -> &Buffer<u8> { &self.src_slc_ids }
    #[inline] pub fn src_col_v_offs(&self) -> &Buffer<i8> { &self.src_col_v_offs }
    #[inline] pub fn src_col_u_offs(&self) -> &Buffer<i8> { &self.src_col_u_offs }
    #[inline] pub fn flag_sets(&self) -> &Buffer<u8> { &self.flag_sets }
    #[inline] pub fn count(&self) -> u32 { self.states.len() as u32 }
    #[inline] pub fn tft_count(&self) -> usize { self.src_idx_caches_by_tft.len() }
    #[inline] pub fn syn_idzs_by_tft(&self) -> &[u32] { self.syn_idzs_by_tft.as_slice() }
    #[inline] pub fn tft_dims_by_tft(&self) -> &[TuftDims] { self.tft_dims_by_tft.as_slice() }


    // [FIXME] TODO: VERIFY AXON INDEX SAFETY (notes below and in syn_src_map.rs).
    // - Will need to know u and v coords of host cell or deconstruct from syn_idx.
    fn regrow_syn(&mut self, syn_idx: usize, tft_id: usize, _: bool) {
        debug_assert!(syn_idx < self.src_slc_ids.len());
        debug_assert!(syn_idx < self.src_col_v_offs.len());
        debug_assert!(syn_idx < self.src_col_u_offs.len());
        debug_assert!(tft_id < self.src_idx_caches_by_tft.len());

        loop {
            let old_src = unsafe { SynSrc {
                slc_id: *self.vec_src_slc_ids.get_unchecked(syn_idx),
                v_ofs: *self.vec_src_col_v_offs.get_unchecked(syn_idx),
                u_ofs: *self.vec_src_col_u_offs.get_unchecked(syn_idx),
                strength: 0
            } };

            let new_src = self.syn_src_slices.gen_src(tft_id, &mut self.rng);

            let insert_success = unsafe {
                self.src_idx_caches_by_tft.get_unchecked_mut(tft_id)
                    .insert(syn_idx, &old_src, &new_src)
            };

            if insert_success {
                unsafe {
                    *self.vec_src_slc_ids.get_unchecked_mut(syn_idx) = new_src.slc_id;
                    *self.vec_src_col_v_offs.get_unchecked_mut(syn_idx) = new_src.v_ofs;
                    *self.vec_src_col_u_offs.get_unchecked_mut(syn_idx) = new_src.u_ofs;
                    *self.vec_strengths.get_unchecked_mut(syn_idx) = new_src.strength;
                }

                if DEBUG_GROW && DEBUG_REGROW_DETAIL { print!("$"); }
                break;
            } else {
                if DEBUG_GROW && DEBUG_REGROW_DETAIL { print!("^"); }
            }
        }
    }

    // [FIXME]: THIS IS A PERFORMANCE NIGHTMARE (since we have to stop the
    // world to do it).
    // - SET UP AN EVENTLIST.
    // - BREAK THIS DOWN INTO PIECES.
    // - PROCESS SMALLER CHUNKS MORE FREQUENTLY.
    //
    fn grow(&mut self, init: bool) {
        if DEBUG_GROW && DEBUG_REGROW_DETAIL && !init {
            println!("REGROW:{:?}: [PRE:(SLICE)(OFFSET)(STRENGTH)=>($:UNIQUE, ^:DUPL)=>POST:\
                (SLICE)(OFFSET)(STRENGTH)]\n", self.den_kind);
        }

        // Fill our vectors with fresh data;
        self.strengths.cmd().read(&mut self.vec_strengths).enq().unwrap();
        self.src_slc_ids.cmd().read(&mut self.vec_src_slc_ids).enq().unwrap();
        self.src_col_v_offs.cmd().read(&mut self.vec_src_col_v_offs).enq().unwrap();
        self.src_col_u_offs.cmd().read(&mut self.vec_src_col_u_offs).enq().unwrap();

        // let syns_per_layer_tft = self.dims.per_slc_per_tft() as usize * self.dims.depth() as usize;
        // let src_slc_ids_by_tft = self.src_slc_ids_by_tft.clone();
        // let src_slc_counts_by_tft = self.syn_src_slices.src_slc_counts_by_tft();
        // let mut src_tft_id = 0usize;
        let tft_count = self.syn_idzs_by_tft.len();
        debug_assert!(tft_count == self.syn_counts_by_tft.len());

        for tft_id in 0..tft_count {
            // if src_slc_count == 0 { unreachable!(); }

            // syn_counts_by_tft
            // let syn_idz = syns_per_layer_tft * src_tft_id as usize;
            let syn_idz = unsafe { *self.syn_idzs_by_tft.get_unchecked(tft_id) as usize };
            let syn_idn = unsafe { syn_idz + *self.syn_counts_by_tft.get_unchecked(tft_id) as usize };

            if DEBUG_GROW && init {
                println!("{mt}{mt}{mt}{mt}{mt}\
                    SYNAPSES::GROW()[INIT]: '{}' ({:?}): src_slc_id_rchs: {:?}, \
                    syns_per_layer_tft:{}, idz:{}, idn:{}", self.layer_name, self.den_kind,
                    self.syn_src_slices.src_slc_id_rchs_by_tft(),
                    unsafe { *self.syn_counts_by_tft.get_unchecked(tft_id) },
                    syn_idz, syn_idn, mt = cmn::MT);
            }

            for syn_idx in syn_idz..syn_idn {
                debug_assert!(syn_idx < self.vec_strengths.len());

                if init || (unsafe { *self.vec_strengths
                    .get_unchecked(syn_idx) } <= cmn::SYNAPSE_STRENGTH_FLOOR)
                {
                    self.regrow_syn(syn_idx, tft_id, init);
                }
            }

            // src_tft_id += 1;
        }

        self.strengths.cmd().write(&self.vec_strengths).enq().unwrap();
        self.src_slc_ids.cmd().write(&self.vec_src_slc_ids).enq().unwrap();
        self.src_col_v_offs.cmd().write(&self.vec_src_col_v_offs).enq().unwrap();
        self.src_col_u_offs.cmd().write(&self.vec_src_col_u_offs).enq().unwrap();
    }

    // #[inline]
    // pub fn syns_per_den_l2(&self) -> u8 {
    //     self.syns_per_den_l2
    // }

    // #[inline]
    // pub fn syns_per_tftsec(&self) -> u32 {
    //     let slcs_per_tftsec = self.dims.depth();
    //     let cels_per_slc = self.dims.columns();
    //     let syns_per_cel_tft = self.dims.per_tft();

    //     slcs_per_tftsec as u32 * cels_per_slc * syns_per_cel_tft
    // }

    // // [FIXME] TODO: Deprecate me eventually
    // pub fn set_offs_to_zero_temp(&mut self) {
    //     self.src_col_v_offs.cmd().fill(&[0], None).enq().unwrap();
    //     self.src_col_u_offs.cmd().fill(&[0], None).enq().unwrap();
    // }

}



#[cfg(test)]
pub mod tests {
    #![allow(non_snake_case, dead_code)]
    use std::ops::{Range};
    use std::fmt::{Display, Formatter, Result as FmtResult};
    use rand::{XorShiftRng};
    use rand::distributions::{IndependentSample, Range as RandRange};
    use ocl::util;
    use cmn::{CelCoords, CorticalDims};
    use super::super::dendrites::{self};
    use super::{Synapses, TuftDims};

    const PRINT_DEBUG_INFO: bool = false;

    pub trait SynapsesTest {
        fn set_offs_to_zero(&mut self);
        fn set_all_to_zero(&mut self);
        fn set_src_offs(&mut self, v_ofs: i8, u_ofs: i8, idx: usize);
        fn set_src_slc(&mut self, src_slc_id: u8, idx: usize);
        fn syn_state(&self, idx: u32) -> u8;
        fn rand_syn_coords(&mut self, cel_coords: CelCoords) -> SynCoords;
        fn print_src_slc_ids(&self, idx_range: Option<Range<usize>>);
        fn print_range(&self, range: Option<Range<usize>>);
        fn print_all(&self);
        fn rng(&mut self) -> &mut XorShiftRng;
    }

    impl SynapsesTest for Synapses {
        fn set_offs_to_zero(&mut self) {
            // self.src_col_v_offs.cmd().fill(&[0], None).enq().unwrap();
            self.src_col_v_offs.cmd().fill(0, None).enq().unwrap();
            // self.src_col_u_offs.cmd().fill(&[0], None).enq().unwrap();
            self.src_col_u_offs.cmd().fill(0, None).enq().unwrap();
        }

        fn set_all_to_zero(&mut self) {
            // self.states.cmd().fill(&[0], None).enq().unwrap();
            // self.strengths.cmd().fill(&[0], None).enq().unwrap();
            // self.src_slc_ids.cmd().fill(&[0], None).enq().unwrap();
            // self.src_col_u_offs.cmd().fill(&[0], None).enq().unwrap();
            // self.src_col_v_offs.cmd().fill(&[0], None).enq().unwrap();
            // self.flag_sets.cmd().fill(&[0], None).enq().unwrap();
            self.states.cmd().fill(0, None).enq().unwrap();
            self.strengths.cmd().fill(0, None).enq().unwrap();
            self.src_slc_ids.cmd().fill(0, None).enq().unwrap();
            self.src_col_u_offs.cmd().fill(0, None).enq().unwrap();
            self.src_col_v_offs.cmd().fill(0, None).enq().unwrap();
            self.flag_sets.cmd().fill(0, None).enq().unwrap();
        }

        fn set_src_offs(&mut self, v_ofs: i8, u_ofs: i8, idx: usize) {
            let sdr_v = vec![v_ofs];
            let sdr_u = vec![u_ofs];
            self.src_col_v_offs.cmd().write(&sdr_v[..]).offset(idx as usize).enq().unwrap();
            self.src_col_u_offs.cmd().write(&sdr_u[..]).offset(idx as usize).enq().unwrap();
        }

        fn set_src_slc(&mut self, src_slc_id: u8, idx: usize) {
            let sdr = vec![src_slc_id];
            // self.src_slc_ids.write(idx, &sdr[..]).unwrap();
            self.src_slc_ids.cmd().write(&sdr[..]).offset(idx as usize).enq().unwrap();
        }

        fn syn_state(&self, idx: u32) -> u8 {
            let mut sdr = vec![0u8];
            // self.states.read(idx as usize, &mut sdr[..]).unwrap();
            self.states.cmd().read(&mut sdr[..]).offset(idx as usize).enq().unwrap();
            sdr[0]
        }

        fn rand_syn_coords(&mut self, cel_coords: CelCoords) -> SynCoords {
            let tft_id_range = RandRange::new(0, self.tft_count());
            let tft_id = tft_id_range.ind_sample(self.rng());

            let tft_syn_idz = self.syn_idzs_by_tft[tft_id];
            let tft_dims = self.tft_dims_by_tft[tft_id].clone();

            let dens_per_tft = 1 << (self.tft_dims_by_tft()[tft_id].dens_per_tft_l2() as u32);
            let den_id_celtft_range = RandRange::new(0, dens_per_tft);
            let syns_per_den = 1 << (self.tft_dims_by_tft()[tft_id].syns_per_den_l2() as u32);
            let syn_id_den_range = RandRange::new(0, syns_per_den);

            let den_id_celtft = den_id_celtft_range.ind_sample(&mut self.rng);
            let syn_id_den = syn_id_den_range.ind_sample(&mut self.rng);

            SynCoords::new(cel_coords, tft_id, tft_syn_idz, tft_dims,
                den_id_celtft, syn_id_den)
        }

        fn print_src_slc_ids(&self, idx_range: Option<Range<usize>>) {
            let mut vec = vec![0; self.states.len()];

            let interval = if idx_range.is_some() { 1 << 0 } else { 1 << 8 };

            print!("syns.src_slc_ids: ");
            self.src_slc_ids.read(&mut vec).enq().unwrap();
            util::print_slice(&vec, interval, None, idx_range, false);
        }

        /// Prints a range of each of the synapse property buffers.
        ///
        //
        ////// Ocl print function signature:
        //
        // ocl::util::print_slice<T: OclScl>(vec: &[T], every: usize, val_range: Option<(T, T)>,
        // idx_range: Option<Range<usize>>, show_zeros: bool)
        //
        fn print_range(&self, idx_range: Option<Range<usize>>) {
            let mut vec = vec![0; self.states.len()];

            print!("syns.states: ");
            self.states.read(&mut vec).enq().unwrap();
            util::print_slice(&vec, 1 << 0, None, idx_range.clone(), false);

            print!("syns.flag_sets: ");
            self.flag_sets.read(&mut vec).enq().unwrap();
            util::print_slice(&vec, 1 << 0, None, idx_range.clone(), false);

            // print!("syns.src_slc_ids: ");
            // self.src_slc_ids.read(&mut vec).enq().unwrap();
            // util::print_slice(&vec, 1 << 0, None, idx_range.clone(), false);

            let mut vec = vec![0i8; self.states.len()];

            print!("syns.strengths: ");
            self.strengths.read(&mut vec).enq().unwrap();
            util::print_slice(&vec, 1 << 0, None, idx_range.clone(), false);

            print!("syns.src_col_v_offs: ");
            self.src_col_v_offs.read(&mut vec).enq().unwrap();
            util::print_slice(&vec, 1 << 0, None, idx_range.clone(), false);

            print!("syns.src_col_u_offs: ");
            self.src_col_u_offs.read(&mut vec).enq().unwrap();
            util::print_slice(&vec, 1 << 0, None, idx_range.clone(), false);
        }

        fn print_all(&self) {
            // let range = 0..self.states.len();
            self.print_range(None);
        }

        fn rng(&mut self) -> &mut XorShiftRng {
            &mut self.rng
        }
    }

    // <<<<< TODO: NEEDS UPDATING TO MATCH / INTEGRATE WITH DEN_COORDS >>>>>
    #[derive(Debug, Clone)]
    pub struct SynCoords {
        pub idx: u32,
        pub cel_coords: CelCoords,
        pub tft_id: usize,
        pub tft_syn_idz: u32,
        pub tft_dims: TuftDims,
        pub den_id_celtft: u32,
        pub syn_id_den: u32,
    }

    impl SynCoords {
        pub fn new(cel_coords: CelCoords, tft_id: usize, tft_syn_idz: u32, tft_dims: TuftDims,
                den_id_celtft: u32, syn_id_den: u32) -> SynCoords
        {
            // let syns_per_tft = 1 << (cel_coords.dens_per_tft_l2 as u32
            //     + cel_coords.syns_per_den_l2 as u32);

            // 'tft_count' is synonymous with 'tfts_per_cel':
            // let tft_count = cel_coords.tfts_per_cel;
            // let syns_per_den = 1 << (cel_coords.syns_per_den_l2 as u32);
            // let dens_per_tft = 1 << (cel_coords.dens_per_tft_l2 as u32);
            // let dens_per_tft = 1 << (self.tft_dims_by_tft()[tft_id].dens_per_tft_l2() as u32);
            // let syns_per_den = 1 << (self.tft_dims_by_tft()[tft_id].syns_per_den_l2() as u32);

            let syn_idx = syn_idx(&cel_coords.lyr_dims, cel_coords.slc_id_lyr, cel_coords.v_id,
                cel_coords.u_id, tft_syn_idz, &tft_dims, den_id_celtft, syn_id_den);

            SynCoords {
                idx: syn_idx,
                cel_coords: cel_coords,
                tft_id: tft_id,
                tft_syn_idz: tft_syn_idz,
                tft_dims: tft_dims,
                den_id_celtft: den_id_celtft,
                syn_id_den: syn_id_den,
            }
        }

        /// Returns the synapse index range for the entire cell-tuft to which
        /// this synapse belongs.
        pub fn syn_idx_range_celtft(&self) -> Range<usize> {
            let dens_per_celtft = 1 << (self.tft_dims.dens_per_tft_l2 as u32);
            let syns_per_den = 1 << (self.tft_dims.syns_per_den_l2 as u32);
            let syns_per_celtft = syns_per_den * dens_per_celtft;

            // Get the idz for the synapse on this cell-tuft with:
            // den_id_celtft = 0, syn_id_den = 0:
            // let syn_idz = self.tft_syn_idz as usize;
            let syn_idz_celtft = syn_idx(&self.cel_coords.lyr_dims, self.cel_coords.slc_id_lyr,
                self.cel_coords.v_id, self.cel_coords.u_id, self.tft_syn_idz, &self.tft_dims,
                0, 0) as usize;

            // syn_idz_cel_tft..(syn_idz_cel_tft + syns_per_tft as usize)
            syn_idz_celtft..(syn_idz_celtft + syns_per_celtft as usize)
        }

        /// Returns the synapse index range for the dendrite to which this
        /// synapse belongs.
        pub fn syn_idx_range_den(&self) -> Range<usize> {
            // let tft_count = self.cel_coords.tfts_per_cel;
            // let syns_per_den = 1 << (self.cel_coords.syns_per_den_l2 as u32);
            // let dens_per_tft = 1 << (self.cel_coords.dens_per_tft_l2 as u32);

            // let dens_per_tft = 1 << (self.tft_dims.dens_per_tft_l2 as u32);
            let syns_per_den = 1 << (self.tft_dims.syns_per_den_l2 as u32);

            // Get the idz for the synapse on this dendrite with: syn_id_den = 0:
            // let syn_idz_den = syn_idx(&self.cel_coords.layer_dims, tft_count, dens_per_tft,
            //     syns_per_den, self.tft_id, self.cel_coords.idx, self.den_id_tft, 0) as usize;
            let syn_idz_den = syn_idx(&self.cel_coords.lyr_dims, self.cel_coords.slc_id_lyr,
                self.cel_coords.v_id, self.cel_coords.u_id, self.tft_syn_idz, &self.tft_dims,
                self.den_id_celtft, 0) as usize;

            syn_idz_den..(syn_idz_den + syns_per_den as usize)
        }

        // [FIXME] TODO: MOVE THIS TO DEN_COORDS & INTEGRATE
        pub fn pyr_celtft_idx(&self) -> u32 {
            (self.tft_id as u32 * self.cel_coords.lyr_dims.cells()) + self.cel_coords.idx
        }

        // pub fn den_idx(&self) -> u32 {
        //     let den_dims = self.cel_coords.layer_dims;
        //         // .clone_with_ptl2(self.cel_coords.dens_per_tft_l2 as i8)
        //         // .with_tfts(self.cel_coords.tfts_per_cel);
        //     let dens_per_tft_l2 = self.cel_coords.dens_per_tft_l2;
        //     let tfts_per_cel = self.cel_coords.tfts_per_cel;

        //     dendrites::den_idx(&den_dims, dens_per_tft_l2, tfts_per_cel,
        //         self.tft_id, self.cel_coords.idx, self.den_id_tft)
        // }

        pub fn den_idx(&self, tft_den_idz: u32) -> u32 {
        // pub fn den_idx(&self, dens: &Dendrite) -> u32 {
            // let tft_den_idz = dens.tft_den_idzs_by_tft()[self.tft_id];

            dendrites::den_idx(
                &self.cel_coords.lyr_dims,
                self.cel_coords.slc_id_lyr,
                self.cel_coords.v_id,
                self.cel_coords.u_id,
                tft_den_idz,
                &self.tft_dims,
                self.den_id_celtft,
            )
        }

        pub fn idx(&self) -> u32 {
            self.idx
        }
    }

    impl Display for SynCoords {
        fn fmt(&self, fmtr: &mut Formatter) -> FmtResult {
            write!(fmtr, "SynCoords {{ idx: {}, tft_id: {}, den_id_celtft: {} syn_id_den: {}, parent_cel: {} }}",
                self.idx, self.tft_id, self.den_id_celtft, self.syn_id_den, self.cel_coords)
        }
    }

    #[test]
    fn source_uniqueness_UNIMPLEMENTED() {
        // UNIMPLEMENTED
    }



    /// Returns the absolute index of a synapse within a layer.
    ///
    /// * Synapse/Dendrite index space hierarchy:
    ///   { [Layer >] Tuft > Slice > Cell > Dendrite > Synapse }
    ///
    // NOTE: 'lyr_dims' expresses dimensions from the perspective of the
    // { [Layer >] Slice > Cell > Tuft > Dendrite > Synapse } hierarchy
    // which is why the naming here seem confusing (see explanation at top
    // of synapses.rs).
    pub fn syn_idx(
            lyr_dims: &CorticalDims,
            slc_id_lyr: u8,
            v_id: u32,
            u_id: u32,
            tft_syn_idz: u32,
            tft_dims: &TuftDims,
            den_id_celtft: u32,
            syn_id_den: u32,
        ) -> u32
    {

        // Dendrites per cell-tuft:
        let dens_per_celtft = 1 << (tft_dims.dens_per_tft_l2 as u32);
        // Synapses per dendrite:
        let syns_per_den = 1 << (tft_dims.syns_per_den_l2 as u32);
        // Dendrites per tuft-slice:
        let syns_per_tftslc = lyr_dims.columns() * dens_per_celtft * syns_per_den;

        // 0th synapse in this tuft-slice:
        let tftslc_syn_idz = (slc_id_lyr as u32 * syns_per_tftslc) + tft_syn_idz;

        // Cell id within this tuft-slice:
        let cel_id_tftslc = (lyr_dims.u_size() * v_id) + u_id;
        // Dendrite id within this tuft-slice:
        let den_id_tftslc = (cel_id_tftslc * dens_per_celtft) + den_id_celtft;
        // Synapse id within this tuft-slice:
        let syn_id_tftslc = (den_id_tftslc * syns_per_den) + syn_id_den;

        syn_id_tftslc + tftslc_syn_idz
    }

    #[test]
    fn syn_idx_range_den() {
        use tests::testbed;
        use cortex::DenCoords;
        use cmn::{DataCellLayer, DataCellLayerTest};

        let mut cortex = testbed::fresh_cortex();
        let mut area = cortex.area_mut(testbed::PRIMARY_AREA_NAME);

        // Choose a random synapse:
        let cel_coords = area.ptal_mut().rand_cel_coords();
        let syn_coords = area.ptal_mut().dens_mut().syns_mut().rand_syn_coords(cel_coords.clone());

        let tft_den_idz = area.ptal().dens().den_idzs_by_tft()[syn_coords.tft_id];

        let den_coords = DenCoords::new(syn_coords.cel_coords.clone(), syn_coords.tft_id,
            tft_den_idz, syn_coords.tft_dims.clone(), syn_coords.den_id_celtft);

        // Ensure the two synapse ranges match:
        assert!(syn_coords.syn_idx_range_den() == den_coords.syn_idx_range_den(syn_coords.tft_id,
            syn_coords.tft_syn_idz));
    }
}

