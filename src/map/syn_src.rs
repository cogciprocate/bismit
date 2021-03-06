// use std::hash::{Hash, Hasher};
use std::collections::{BTreeMap, BTreeSet};
use std::cmp;
use rand::distributions::{Distribution, Range as RandRange};
use rand::rngs::SmallRng;
use cortex::TuftDims;
use cmn::{self, CmnError, CmnResult, CorticalDims, SliceDims};
use map::{AreaMap, AxonTopology, TuftScheme};
use SrcOfs;

const INTENSITY_REDUCTION_L2: i8 = 3;
const STR_MIN: i8 = -3;
const STR_MAX: i8 = 4;


/// Tests to ensure a list of synapse source offsets has a balanced set.
///
/// Currently being called by `::gen_syn_offs` in debug builds.
pub fn offs_list_is_balanced(syn_offs: &Vec<(SrcOfs, SrcOfs)>) -> CmnResult<()> {
    let mut ttls = (0usize, 0usize);

    for off in syn_offs {
        ttls.0 += off.0 as usize;
        ttls.1 += off.1 as usize;
    }

    if ttls.0 != 0 || ttls.1 != 0 { return Err("Synapse offset list imbalanced.".into()); }

    Ok(())
}


/// List of offsets to form a hexagon-shaped pattern of tiles.
///
/// `..._dims` contain [v, u] values respectively.
///
/// '..._z' suffix := index[0], first element, starting element
///
/// '..._n' suffix := index[len]: element after final element, termination
/// point, number of elements (ex.: for(int i = 0, i < idn, i++))
///
/// * TODO: Create extra version of `::calc_scale` which accepts an additional
/// precision (log2) parameter and returns it's scale adjusted accordingly.
///
// #[warn(dead_code, unused_variables, unused_mut)]
// pub fn encode_hex_mold_scaled(radius: i8, scales: [u32; 2], center: [u32; 2], tract: &mut TractFrameMut) {
pub fn gen_syn_offs(radius: SrcOfs, scales: [u32; 2]) -> CmnResult<Vec<(SrcOfs, SrcOfs)>> {
    // // TEMPORARY (* TODO: Investigate):
    // for val in tract_frame.iter() {
    //     debug_assert!(*val == 0);
    // }

    // Extra precision used in scale calculations:
    const EXTRA_PRECISION_L2: u32 = 3;
    // Redeclarations for brevity:
    const RAD_MAX: i32 = cmn::SYNAPSE_REACH_MAX as i32;
    const RAD_MIN: i32 = cmn::SYNAPSE_REACH_MIN as i32;

    // assert!(radius > 0);

    // Radii of 0 needs to be allowed (mainly for proximal dens/syns).
    assert!(radius >= 0);

    // let dst_dims = [tract_frame.dims().v_size(), tract_frame.dims().u_size()];
    // assert!(dst_dims[0] == tract_frame.dims().v_size() && dst_dims[1] == tract_frame.dims().u_size());

    // Scale factor needed to translate from the destination slice to the
    // source slice. Effectively an inverse scale factor when viewed from the
    // perspective of the destination slice.
    // let scales = [cmn::calc_scale(dst_dims[0], src_dims[0]).unwrap(),
    //     cmn::calc_scale(dst_dims[1], src_dims[1]).unwrap()];

    // println!("###### scales: {:?}", scales);

    // Scales a value:
    #[inline]
    fn scl(val: i32, scl: u32) -> i32 {
        (cmn::scale(val as i32, scl) as i32)
    }

    // Scales a value both inversely by `scl_inv` and directly by `scl`.
    #[inline]
    fn scl_inv_scl(val: i32, scl_inv: u32, scl: u32) -> i32 {
        ((val as i32 * ((scl as i32) << EXTRA_PRECISION_L2)) /
            ((scl_inv as i32) << EXTRA_PRECISION_L2))
    }

    let radius_max_scaled = cmp::max(cmn::scale(radius as i32, scales[0]), cmn::scale(radius as i32, scales[1]));
    assert!(radius_max_scaled <= RAD_MAX);

    // Maximum number of possible results:
    let tile_count = (3 * radius_max_scaled as usize) * (radius_max_scaled as usize + 1) + 1;

    // The eventual result:
    let mut offs_list = Vec::with_capacity(tile_count);

    // The radius scaled in the 'v' dimension:
    let v_rad = scl(radius as i32, scales[0]);
    // let rad_u = cmn::scale(radius as i32, scales[1]);

    // '-v_rad' (additive inverse of 'v' radius), stored for efficiency's sake:
    let v_rad_inv = 0 - v_rad;
    let v_ofs_z = v_rad_inv;
    let v_ofs_n = v_rad + 1;

    for v_ofs in v_ofs_z..v_ofs_n {
        // '-v_ofs' (additive inverse of 'v_ofs'), stored for efficiency's sake:
        let v_ofs_inv = 0 - v_ofs;

        // Find the 'u' minimum (zero) for this 'v':
        // * Determine the greater of either the absolute minimum possible 'v'
        //   value or the additive inverse of the current 'v' ('-v_ofs') minus
        //   the radius of 'v' ('v_rad').
        // * Scale that value first by the inverse of the 'v' scale then by
        //   the 'u' scale:
        let u_ofs_z = scl_inv_scl(
            cmp::max(v_rad_inv, v_ofs_inv + v_rad_inv),
            scales[0],
            scales[1],
        );

        // Find the 'u' maximum for this 'v':
        // * Determine the lesser of either the minimum 'v' radius or the 'v'
        //   radius minus the inverse of the current 'v' (performed in
        //   reversed order using the previously stored 'v_ofs_inv').
        // * Scale that value first by the inverse of the 'v' scale then by
        //   the 'u' scale (same as above):
        let u_ofs_n = scl_inv_scl(
            cmp::min(v_rad, v_ofs_inv + v_rad),
            scales[0],
            scales[1],
        ) + 1;

        // Loop through the calculated range of 'u's and push the tuple to the
        // result Vec:
        for u_ofs in u_ofs_z..u_ofs_n {
            if !(v_ofs <= RAD_MAX && v_ofs >= RAD_MIN &&
                u_ofs <= RAD_MAX && u_ofs >= RAD_MIN) {
                return CmnError::err("cmn::hex_tile_offs_skewed: Calculated \
                    offsets are outside valid radius range: (v_ofs: {}, u_ofs: {}).");
            }
            offs_list.push((v_ofs as SrcOfs, u_ofs as SrcOfs));
        }
    }

    offs_list.shrink_to_fit();
    if cfg!(debug) { try!(offs_list_is_balanced(&offs_list)) }
    Ok(offs_list)
}


/// Allows rapid comparison for duplicate synapse sources.
///
/// Not multi-tuft. In other words, one must be created separately for each
/// tuft.
///
#[allow(dead_code)]
#[derive(Debug)]
pub struct SynSrcIdxCache {
    tft_syn_idz: usize,
    tft_dims: TuftDims,
    dims: CorticalDims,
    dens: Vec<BTreeSet<i32>>,
    source_saturated: bool,
}

impl SynSrcIdxCache {
    pub fn new(tft_syn_idz: usize, tft_dims: TuftDims, dims: CorticalDims, source_saturated: bool)
            -> SynSrcIdxCache {
        let dens_per_tft = tft_dims.dens_per_tft() as usize;
        let tft_den_count = dens_per_tft * dims.cells() as usize;
        let mut dens = Vec::with_capacity(tft_den_count);

        for _ in 0..tft_den_count {
            dens.push(BTreeSet::new());
        }

        SynSrcIdxCache {
            tft_syn_idz,
            tft_dims,
            dims,
            dens,
            source_saturated,
        }
    }

    // TODO: Consider bypassing entire function if `self.source_saturated` is true.
    pub fn insert(&mut self, syn_idx: usize, old_ofs: &SynSrc, new_ofs: &SynSrc) -> bool {
        let syn_id_tft = syn_idx - self.tft_syn_idz;
        let den_id_tft = syn_id_tft / self.tft_dims.syns_per_den() as usize;

        debug_assert!(den_id_tft < self.dens.len(), format!("den_id_tft: '{}' ![<] \
            self.dens.len(): '{}', (syn_id_tft: '{}')", den_id_tft, self.dens.len(), syn_id_tft));

        let new_ofs_key: i32 = self.axon_ofs(new_ofs);
        let is_unique: bool = unsafe { self.dens.get_unchecked_mut(den_id_tft).insert(new_ofs_key) };

        if is_unique {
            let old_ofs_key: i32 = self.axon_ofs(old_ofs);
            unsafe { self.dens.get_unchecked_mut(den_id_tft).remove(&old_ofs_key) };
        }

        is_unique || self.source_saturated
    }

    fn axon_ofs(&self, axon_ofs: &SynSrc) -> i32 {
        (axon_ofs.slc_id as i32 * self.dims.columns() as i32) +
            (axon_ofs.v_ofs as i32 * self.dims.u_size() as i32) +
            axon_ofs.u_ofs as i32
    }
}


/// Pool of potential synapse values.
#[derive(Clone, Debug)]
pub enum OfsPool {
    Nonspatial(RandRange<SrcOfs>, RandRange<SrcOfs>),
    Spatial { offs: Vec<(SrcOfs, SrcOfs)>, ofs_idx_range: RandRange<usize> },
}


/// Parameters describing a slice.
///
#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct SynSrcSliceInfo {
    slc_off_pool: OfsPool,
    v_size: u32,
    u_size: u32,
    syn_reach: SrcOfs,
    scaled_syn_reaches: (SrcOfs, SrcOfs),
    poss_syn_offs_val_count: u32,
}

impl SynSrcSliceInfo {
    pub fn new(axon_kind: &AxonTopology, src_slc_dims: &SliceDims, syn_reach: SrcOfs,
            tft_slc_id_pool_len: u32) -> CmnResult<SynSrcSliceInfo>
    {
        let poss_syn_offs_val_count;
        let slc_off_pool;

        match axon_kind {
            &AxonTopology::Nonspatial => {
                // Already checked within `SliceDims` (keep here anyway):
                debug_assert!(src_slc_dims.v_size() <= cmn::MAX_HRZ_DIM_SIZE);
                debug_assert!(src_slc_dims.u_size() <= cmn::MAX_HRZ_DIM_SIZE);

                if src_slc_dims.v_size() & 0x01 != 0 || src_slc_dims.v_size() & 0x01 != 0 {
                    return Err("Nonspatial slices must have u and v sizes evenly divisible by 2.".into());
                }

                if syn_reach != 0 {
                    // return Err("The reach of a synapse with non-spatial (horizontal) sources \
                    //     must be zero (0).".into());
                    printlnc!(red_bold: "\nCONFIG: LAYER MAP SCHEME: The reach of synapses \
                        in layer ([FIXME: ADD LAYER INFO]) should be zero when the source is a \
                        non-spatial (horizontal) layer. The currently set reach of '{}' \
                        will be ignored. \n",
                        syn_reach);
                }

                poss_syn_offs_val_count = src_slc_dims.v_size() * src_slc_dims.u_size() *
                    tft_slc_id_pool_len;

                let v_reach = (src_slc_dims.v_size() / 2) as SrcOfs;
                let u_reach = (src_slc_dims.u_size() / 2) as SrcOfs;

                slc_off_pool = OfsPool::Nonspatial(
                    RandRange::new(0 - v_reach, v_reach + 1),
                    RandRange::new(0 - u_reach, u_reach + 1),
                );
            },

            &AxonTopology::Spatial | &AxonTopology::None => {
                let hex_tile_offs = gen_syn_offs(syn_reach,
                    [src_slc_dims.v_scale(), src_slc_dims.u_scale()])?;

                poss_syn_offs_val_count = hex_tile_offs.len() as u32 * tft_slc_id_pool_len;

                let len = hex_tile_offs.len();

                slc_off_pool = OfsPool::Spatial {
                    offs: hex_tile_offs,
                    ofs_idx_range: RandRange::new(0, len)
                };
            },
        };

        let scaled_syn_reaches = try!(src_slc_dims.scale_offs((syn_reach, syn_reach)));

        Ok(SynSrcSliceInfo {
            slc_off_pool,
            v_size: src_slc_dims.v_size(),
            u_size: src_slc_dims.u_size(),
            syn_reach,
            scaled_syn_reaches,
            poss_syn_offs_val_count,
        })
    }

    #[allow(dead_code)]
    pub fn slc_off_pool(&self) -> &OfsPool {
        &self.slc_off_pool
    }

    pub fn scaled_syn_reaches(&self) -> (SrcOfs, SrcOfs) {
        self.scaled_syn_reaches
    }

    #[allow(dead_code)]
    pub fn v_size(&self) -> u32 {
        self.v_size
    }

    #[allow(dead_code)]
    pub fn u_size(&self) -> u32 {
        self.u_size
    }

    #[allow(dead_code)]
    pub fn poss_syn_offs_val_count(&self) -> u32 {
        self.poss_syn_offs_val_count
    }
}


/// Source location and strength for a synapse.
pub struct SynSrc {
    pub slc_id: u8,
    pub v_ofs: SrcOfs,
    pub u_ofs: SrcOfs,
    pub strength: i8,
}

#[derive(Debug)]
pub struct SynSrcSlice {
    info_by_slc: BTreeMap<u8, SynSrcSliceInfo>,
    id_pools: Vec<u8>,
    id_pool_ranges: RandRange<usize>,
    str_ranges: RandRange<i8>,
    is_saturated: bool,
}

impl SynSrcSlice {
    pub fn id_pools(&self) -> &[u8] {
        &self.id_pools
    }

    pub fn is_saturated(&self) -> bool {
        self.is_saturated
    }
}


/// Information about the boundaries and synapse ranges for each source slice, on
/// each tuft.
///
/// Used to calculate a valid source axon index during synapse growth or regrowth.
#[derive(Debug)]
pub struct SynSrcSlices {
    slices_by_tft: Vec<SynSrcSlice>,
}

impl SynSrcSlices {
    pub fn new(lyr_id: usize, tft_schemes: &[TuftScheme], area_map: &AreaMap)
            -> CmnResult<SynSrcSlices>
    {
        let mut slices_by_tft = Vec::with_capacity(tft_schemes.len());

        for tft_scheme in tft_schemes.iter() {
            let tft_id = tft_scheme.tft_id();

            debug_assert!(tft_id == slices_by_tft.len());


            let lyr_id_rchs = area_map.cel_src_slc_id_rchs(lyr_id, tft_id, false);

            assert!(lyr_id_rchs.len() > 0,
                "Synapses::new(): Synapse source resolution error. Layer: '{}', tuft: '{}' \
                has no source layers defined. If a source layer is an input layer, please \
                ensure that the source area for that the layer exists. [FIXME: Resolve layer \
                and tuft ids into names]",
                lyr_id, tft_id);

            let id_pools: Vec<u8> = area_map.cel_src_slc_id_rchs(lyr_id, tft_id, true)
                .into_iter().map(|(id, _)| id).collect();

            let mut info_by_slc = BTreeMap::new();
            let mut poss_syn_offs_val_count = 0;

            for &(slc_id, syn_rch) in lyr_id_rchs.iter() {
                let axon_kind = area_map.slice_map().axon_topologies().get(slc_id as usize)
                    .expect("SynSrcSlices::new(): {{2}}");

                let src_slc_dims = area_map.slice_map().dims().get(slc_id as usize)
                    .expect("SynSrcSlices::new(): {{3}}");

                let src_slc_info = SynSrcSliceInfo::new(axon_kind, src_slc_dims, syn_rch,
                        /*tft_scheme.syns_per_den_l2(),*/ id_pools.len() as u32)
                    .map_err(|err| err.prepend(&format!("SynSrcSlices::new(): Source slice error \
                        (area: {}, slice: {}): ", area_map.area_name(), slc_id)))?;

                assert!(poss_syn_offs_val_count == src_slc_info.poss_syn_offs_val_count() ||
                    poss_syn_offs_val_count == 0);
                poss_syn_offs_val_count = src_slc_info.poss_syn_offs_val_count();

                info_by_slc.insert(slc_id, src_slc_info);
            }

            // Ensure we have enough unique synapse source address values:
            let syns_per_den = tft_scheme.syns_per_den();
            if poss_syn_offs_val_count < syns_per_den {
                panic!("The cells of this slice do not have enough possible \
                    synapse source offset values (possible: {}, needed: {}) to avoid \
                    duplicate source values due to the relative sizes of the source and \
                    destination slices. Decrease the number of synapses or increase synapse \
                    reach. [FIXME: Add layer info]", poss_syn_offs_val_count, syns_per_den);
            }

            let is_saturated = poss_syn_offs_val_count == syns_per_den;

            // println!("\n##### IS_SATURATED: {} (poss_syn_offs_val_count: {}, syns_per_den: {})",
            //     is_saturated, poss_syn_offs_val_count, syns_per_den);

            let id_pool_ranges = RandRange::new(0, id_pools.len());
            let str_ranges = RandRange::new(STR_MIN, STR_MAX);

            slices_by_tft.push(SynSrcSlice {
                info_by_slc,
                id_pools,
                id_pool_ranges,
                str_ranges,
                is_saturated,
            });
        }

        Ok(SynSrcSlices {
            slices_by_tft,
        })
    }

    /// Generates a tuft-specific `SynSrc` for a synapse.
    ///
    //
    // [FIXME]: TODO: Bounds check ofs against v and u id -- will need to
    // figure out how to deconstruct this from the syn_idx or something.
    //
    pub fn gen_src(&self, tft_id: usize, rng: &mut SmallRng) -> SynSrc {
        debug_assert!(tft_id < self.slices_by_tft.len());

        let src_slices = unsafe { self.slices_by_tft.get_unchecked(tft_id) };

        let &slc_id = unsafe {
            let rand_slc_id_lyr = src_slices.id_pool_ranges.sample(rng);
            src_slices.id_pools.get_unchecked(rand_slc_id_lyr)
        };

        let slc_info = &src_slices.info_by_slc.get(&slc_id)
            .expect("SynSrcSlices::gen_offs(): Internal error: invalid slc_id.");


        match slc_info.slc_off_pool {
            OfsPool::Nonspatial(ref v_rr, ref u_rr) => {
                SynSrc {
                    slc_id: slc_id,
                    v_ofs: v_rr.sample(rng),
                    u_ofs: u_rr.sample(rng),
                    strength: 0,
                }
            },

            OfsPool::Spatial { ref offs, ref ofs_idx_range } => {
                let (v_ofs, u_ofs) = offs[ofs_idx_range.sample(rng)];

                let syn_reaches = slc_info.scaled_syn_reaches();

                let syn_str_intensity = (((syn_reaches.0 as i32 - v_ofs.abs() as i32) +
                        (syn_reaches.1 as i32 - u_ofs.abs() as i32)) >> INTENSITY_REDUCTION_L2) as i8;

                let strength = syn_str_intensity * src_slices.str_ranges.sample(rng);

                SynSrc {
                    slc_id: slc_id,
                    v_ofs: v_ofs,
                    u_ofs: u_ofs,
                    strength: strength,
                }
            },
        }
    }

    #[inline]
    pub fn by_tft(&self) -> &[SynSrcSlice] {
        &self.slices_by_tft
    }
}


