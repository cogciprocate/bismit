use rand::{XorShiftRng};
use rand::distributions::{IndependentSample, Range as RandRange};
use std::collections::{BTreeMap, BTreeSet};

use cmn::{self, CmnResult, CorticalDims, SliceDims};
use map::{AreaMap, AxonKind};

const INTENSITY_REDUCTION_L2: i8 = 3;
const STR_MIN: i8 = -3;
const STR_MAX: i8 = 4;


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
pub struct SliceInfo {
    slc_off_pool: OfsPool,
    v_size: u32,
    u_size: u32,
    syn_reach: i8,
    scaled_syn_reaches: (i8, i8),
}

impl SliceInfo {
    pub fn new(axn_kind: &AxonKind, slc_dims: &SliceDims, syn_reach: i8) -> CmnResult<SliceInfo> {
        let slc_off_pool = match axn_kind {
            &AxonKind::Horizontal => {
                // Already checked within SliceDims.
                debug_assert!(slc_dims.v_size() <= cmn::MAX_HRZ_DIM_SIZE);
                debug_assert!(slc_dims.u_size() <= cmn::MAX_HRZ_DIM_SIZE);

                // [FIXME] Tweak how the middle and ranges are calc'd!
                // Adjust SliceDims if necessary.

                let v_reach = (slc_dims.v_size() / 2) as i8;
                let u_reach = (slc_dims.u_size() / 2) as i8;

                OfsPool::Horizontal((
                    RandRange::new(0 - v_reach, v_reach + 1),
                    RandRange::new(0 - u_reach, u_reach + 1), ))
            },

            &AxonKind::Spatial | &AxonKind::None => {
                let mut hex_tile_offs = cmn::hex_tile_offs(syn_reach);

                // println!("###### SliceInfo::new: hex_tile_offs.len(): {}", hex_tile_offs.len());

                // Scale each potential offset value according to the source slice:
                // for offs in hex_tile_offs.iter_mut() {
                //     *offs = try!(slc_dims.scale_offs(*offs));
                // }

                let len = hex_tile_offs.len();

                OfsPool::Spatial((
                    hex_tile_offs,
                    RandRange::new(0, len), ))
            },
        };

        let scaled_syn_reaches = try!(slc_dims.scale_offs((syn_reach, syn_reach)));

        Ok(SliceInfo {
            slc_off_pool: slc_off_pool,
            v_size: slc_dims.v_size(),
            u_size: slc_dims.u_size(),
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
    tft_slcs: Vec<BTreeMap<u8, SliceInfo>>,
    slc_ids: Vec<Vec<u8>>,
    slc_id_ranges: Vec<RandRange<usize>>,
    str_ranges: Vec<RandRange<i8>>,
}

impl SrcSlices {
    pub fn new(src_slc_ids_by_tft: &Vec<Vec<u8>>, syn_reaches_by_tft: Vec<i8>, area_map: &AreaMap
            ) -> CmnResult<SrcSlices>
    {
        let mut tft_slcs = Vec::with_capacity(src_slc_ids_by_tft.len());
        let mut slc_id_ranges = Vec::with_capacity(tft_slcs.len());
        let mut str_ranges = Vec::with_capacity(tft_slcs.len());

        let mut tft_id = 0;

        for src_slc_ids in src_slc_ids_by_tft.iter() {
            let mut slcs = BTreeMap::new();
            let syn_reaches = *syn_reaches_by_tft.get(tft_id).expect("SrcSlices::new(): {{1}}");

            slc_id_ranges.push(RandRange::new(0, src_slc_ids.len()));
            str_ranges.push(RandRange::new(STR_MIN, STR_MAX));

            for &slc_id in src_slc_ids {
                let axn_kind = area_map.slices().axn_kinds().get(slc_id as usize)
                    .expect("SrcSlices::new(): {{2}}");
                let dims = area_map.slices().dims().get(slc_id as usize)
                    .expect("SrcSlices::new(): {{3}}");

                slcs.insert(slc_id, try!(SliceInfo::new(axn_kind, dims, syn_reaches)))
                    .map(|_| panic!("SrcSlices::new(): {{4}}"));
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



