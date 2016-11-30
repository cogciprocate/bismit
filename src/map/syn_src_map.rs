use rand::{XorShiftRng};
use rand::distributions::{IndependentSample, Range as RandRange};
use std::collections::{BTreeMap, BTreeSet};
use std::cmp;

use cmn::{self, CmnError, CmnResult, CorticalDims, SliceDims};
use map::{AreaMap, AxonKind};

const INTENSITY_REDUCTION_L2: i8 = 3;
const STR_MIN: i8 = -3;
const STR_MAX: i8 = 4;



/// List of offsets to form a hexagon-shaped pattern of tiles.
///
/// `..._dims` contain [v, u] values respectively.
///
/// '..._z' suffix := index[0], first element, starting element
///
/// '..._n' suffix := index[len]: element after final element, termination
/// point, number of elements (ex.: for(int i = 0, i < idn, i++))
///
/// [TODO]: Create extra version of `::calc_scale` which accepts an additional
/// precision (log2) parameter and returns it's scale adjusted accordingly.
///
#[warn(dead_code, unused_variables, unused_mut)]
// pub fn encode_hex_mold_scaled(radius: i8, scales: [u32; 2], center: [u32; 2], tract: &mut TractFrameMut) {
pub fn gen_syn_offs(radius: i8, scales: [u32; 2]) -> CmnResult<Vec<(i8, i8)>> {
    // // TEMPORARY ([TODO]: Investigate):
    // for val in tract_frame.iter() {
    //     debug_assert!(*val == 0);
    // }

    // Extra precision used in scale calculations:
    const EXTRA_PRECISION_L2: u32 = 3;
    // Redeclarations for brevity:
    const RAD_MAX: i32 = cmn::SYNAPSE_REACH_MAX as i32;
    const RAD_MIN: i32 = cmn::SYNAPSE_REACH_MIN as i32;

    assert!(radius > 0);

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
            offs_list.push((v_ofs as i8, u_ofs as i8));
        }
    }

    offs_list.shrink_to_fit();
    if cfg!(debug) { try!(offs_list_is_balanced(&offs_list)) }
    Ok(offs_list)
}


/// Pool of potential synapse values.
pub enum OfsPool {
    Horizontal((RandRange<i8>, RandRange<i8>)),
    Spatial((Vec<(i8, i8)>, RandRange<usize>)),
}


/// Source location and strength for a synapse.
pub struct SynSrc {
    pub slc_id: u8,
    pub v_ofs: i8,
    pub u_ofs: i8,
    pub strength: i8,
}


/// Parameters describing a slice.
///
#[allow(dead_code)]
pub struct SrcSliceInfo {
    slc_off_pool: OfsPool,
    v_size: u32,
    u_size: u32,
    syn_reach: i8,
    scaled_syn_reaches: (i8, i8),
}

impl SrcSliceInfo {
    pub fn new(axn_kind: &AxonKind, src_slc_dims: &SliceDims, syn_reach: i8, den_syn_count: u32)
                    -> CmnResult<SrcSliceInfo> {
        let slc_off_pool = match axn_kind {
            &AxonKind::Horizontal => {
                // Already checked within `SliceDims` (keep here though).
                debug_assert!(src_slc_dims.v_size() <= cmn::MAX_HRZ_DIM_SIZE);
                debug_assert!(src_slc_dims.u_size() <= cmn::MAX_HRZ_DIM_SIZE);

                if src_slc_dims.v_size() & 0x01 != 0 || src_slc_dims.v_size() & 0x01 != 0 {
                    return Err("Horizontal slices must have u and v sizes evenly divisible by 2.".into());
                }

                if syn_reach != 0 {
                    return Err("The reach of a synapse with non-spatial (horizontal) sources \
                        must be zero (0).".into());
                }

                let poss_syn_offs_val_count = src_slc_dims.v_size() * src_slc_dims.u_size();

                if poss_syn_offs_val_count < den_syn_count {
                    return Err(format!("The cells of this slice do not have enough possible \
                        synapse source offset values ({}/{}) to avoid duplicate source values \
                        due to the relative sizes of the source and destination slices. \
                        Decrease the number of synapses or increase synapse reach.",
                        poss_syn_offs_val_count, den_syn_count).into());
                }

                let v_reach = (src_slc_dims.v_size() / 2) as i8;
                let u_reach = (src_slc_dims.u_size() / 2) as i8;

                OfsPool::Horizontal((
                    RandRange::new(0 - v_reach, v_reach + 1),
                    RandRange::new(0 - u_reach, u_reach + 1), ))
            },

            &AxonKind::Spatial | &AxonKind::None => {
                let hex_tile_offs = gen_syn_offs(syn_reach,
                    [src_slc_dims.v_scale(), src_slc_dims.u_scale()])?;

                if (hex_tile_offs.len() as u32) < den_syn_count {
                    return Err(format!("The cells of this slice do not have enough possible \
                        synapse source offset values ({}/{}) to avoid duplicate source values \
                        due to the relative sizes of the source and destination slices. \
                        Decrease the number of synapses or increase synapse reach.",
                        hex_tile_offs.len(), den_syn_count).into());
                }

                // println!("###### SrcSliceInfo::new: hex_tile_offs.len(): {}", hex_tile_offs.len());

                let len = hex_tile_offs.len();

                OfsPool::Spatial((
                    hex_tile_offs,
                    RandRange::new(0, len), ))
            },
        };

        let scaled_syn_reaches = try!(src_slc_dims.scale_offs((syn_reach, syn_reach)));

        Ok(SrcSliceInfo {
            slc_off_pool: slc_off_pool,
            v_size: src_slc_dims.v_size(),
            u_size: src_slc_dims.u_size(),
            syn_reach: syn_reach,
            scaled_syn_reaches: scaled_syn_reaches,
        })
    }

    #[allow(dead_code)]
    pub fn slc_off_pool(&self) -> &OfsPool {
        &self.slc_off_pool
    }

    pub fn scaled_syn_reaches(&self) -> (i8, i8) {
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
}


/// Information about the boundaries and synapse ranges for each source slice, on
/// each tuft.
///
/// Used to calculate a valid source axon index during synapse growth or regrowth.
pub struct SrcSlices {
    tft_slcs: Vec<BTreeMap<u8, SrcSliceInfo>>,
    slc_ids: Vec<Vec<u8>>,
    slc_id_ranges: Vec<RandRange<usize>>,
    str_ranges: Vec<RandRange<i8>>,
}

impl SrcSlices {
    pub fn new(src_slc_ids_by_tft: &Vec<Vec<u8>>, syn_reaches_by_tft: Vec<i8>, den_syn_count: u32,
                area_map: &AreaMap) -> CmnResult<SrcSlices> {
        let mut tft_slcs = Vec::with_capacity(src_slc_ids_by_tft.len());
        let mut slc_id_ranges = Vec::with_capacity(tft_slcs.len());
        let mut str_ranges = Vec::with_capacity(tft_slcs.len());

        let mut tft_id = 0;

        for src_slc_ids in src_slc_ids_by_tft.iter() {
            let mut slcs = BTreeMap::new();
            let syn_reaches = *syn_reaches_by_tft.get(tft_id).expect("SrcSlices::new(): {{1}}");

            assert!(src_slc_ids.len() > 0, "SrcSlices::new(): No source slices found for \
                a layer in area: \"{}\".", area_map.area_name());

            slc_id_ranges.push(RandRange::new(0, src_slc_ids.len()));
            str_ranges.push(RandRange::new(STR_MIN, STR_MAX));

            for &slc_id in src_slc_ids {
                let axn_kind = area_map.slices().axn_kinds().get(slc_id as usize)
                    .expect("SrcSlices::new(): {{2}}");

                let src_slc_dims = area_map.slices().dims().get(slc_id as usize)
                    .expect("SrcSlices::new(): {{3}}");

                let src_slc_info = SrcSliceInfo::new(axn_kind, src_slc_dims, syn_reaches, den_syn_count)
                    .map_err(|err| err.prepend(&format!("SrcSlices::new(): Source slice error \
                        (area: {}, slice: {}): ", area_map.area_name(), slc_id)))?;

                slcs.insert(slc_id, src_slc_info);
            }

            tft_slcs.push(slcs);
            tft_id += 1;
        }

        Ok(SrcSlices { tft_slcs: tft_slcs, slc_id_ranges: slc_id_ranges,
            slc_ids: src_slc_ids_by_tft.clone(), str_ranges: str_ranges, })
    }

    /// Generates a tuft specific `SynSrc` for a synapse.
    ///
    //
    // [FIXME]: TODO: Bounds check ofs against v and u id -- will need to
    // figure out how to deconstruct this from the syn_idx or something.
    //
    pub fn gen_src(&self, tft_id: usize, rng: &mut XorShiftRng) -> SynSrc {
        debug_assert!(tft_id < self.slc_ids.len() && tft_id < self.tft_slcs.len());

        let slc_id = unsafe { *self.slc_ids.get_unchecked(tft_id).get_unchecked(
            self.slc_id_ranges.get_unchecked(tft_id).ind_sample(rng)) };

        let slc_info = unsafe { &self.tft_slcs.get_unchecked(tft_id)
            .get(&slc_id).expect("SrcSlices::gen_offs(): Internal error: invalid slc_id.") };

        match slc_info.slc_off_pool {
            OfsPool::Horizontal((ref v_rr, ref u_rr)) => {
                SynSrc {
                    slc_id: slc_id,
                    v_ofs: v_rr.ind_sample(rng),
                    u_ofs: u_rr.ind_sample(rng),
                    strength: 0,
                }
            },

            OfsPool::Spatial((ref offs, ref range)) => {
                let (v_ofs, u_ofs) = offs[range.ind_sample(rng)];

                let syn_reaches = slc_info.scaled_syn_reaches();

                let syn_str_intensity = (((syn_reaches.0 as i32 - v_ofs.abs() as i32) +
                        (syn_reaches.1 as i32 - u_ofs.abs() as i32)) >> INTENSITY_REDUCTION_L2) as i8;

                let strength = syn_str_intensity *
                    unsafe {self.str_ranges.get_unchecked(tft_id).ind_sample(rng) };

                SynSrc {
                    slc_id: slc_id,
                    v_ofs: v_ofs,
                    u_ofs: u_ofs,
                    strength: strength,
                }
            },
        }
    }
}


#[allow(dead_code)]
pub struct SrcIdxCache {
    syns_per_den_l2: u8,
    dens_per_tft_l2: u8,
    dims: CorticalDims,
    dens: Vec<Box<BTreeSet<i32>>>,
}

impl SrcIdxCache {
    pub fn new(syns_per_den_l2: u8, dens_per_tft_l2: u8, dims: CorticalDims) -> SrcIdxCache {
        let dens_per_tft = 1 << dens_per_tft_l2 as u32;
        let area_dens = (dens_per_tft * dims.cel_tfts()) as usize;
        let mut dens = Vec::with_capacity(dens_per_tft as usize);

        for _ in 0..area_dens { dens.push(Box::new(BTreeSet::new())); }

        //println!("##### CREATING SRCIDXCACHE WITH: dens: {}", dens.len());

        SrcIdxCache {
            syns_per_den_l2: syns_per_den_l2,
            dens_per_tft_l2: dens_per_tft_l2,
            dims: dims,
            dens: dens,
        }
    }

    pub fn insert(&mut self, syn_idx: usize, old_ofs: &SynSrc, new_ofs: &SynSrc) -> bool {
        let den_idx = syn_idx >> self.syns_per_den_l2;
        debug_assert!(den_idx < self.dens.len());

        let new_ofs_key: i32 = self.axn_ofs(new_ofs);
        let is_unique: bool = unsafe { self.dens.get_unchecked_mut(den_idx).insert(new_ofs_key) };

        if is_unique {
            let old_ofs_key: i32 = self.axn_ofs(old_ofs);
            unsafe { self.dens.get_unchecked_mut(den_idx).remove(&old_ofs_key) };
        }

        is_unique
    }

    fn axn_ofs(&self, axn_ofs: &SynSrc) -> i32 {
        (axn_ofs.slc_id as i32 * self.dims.columns() as i32)
        + (axn_ofs.v_ofs as i32 * self.dims.u_size() as i32)
        + axn_ofs.u_ofs as i32
    }
}


/// Tests to ensure a list of synapse source offsets has a balanced set.
pub fn offs_list_is_balanced(syn_offs: &Vec<(i8, i8)>) -> CmnResult<()> {
    // use std::collections::HashMap;

    // let mut
    let mut ttls = (0usize, 0usize);

    for off in syn_offs {
        ttls.0 += off.0 as usize;
        ttls.1 += off.1 as usize;
    }

    if ttls.0 != 0 || ttls.1 != 0 { return Err("Synapse offset list imbalanced.".into()); }

    Ok(())
}

// #[cfg(test)]
// mod tests {
//     use cmn::{CmnResult};
//     pub fn offs_list_is_balanced(syn_offs: &Vec<(i8, i8)>) -> CmnResult<()> {
//         Ok(())
//     }
// }
