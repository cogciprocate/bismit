use std::fmt;
use rand::{FromEntropy, rngs::SmallRng,
    distributions::{Range as RandRange, Distribution}};
use cmn::{TractFrameMut, TractDims};
use encode::ScalarEncodable;

type TractAxonIdx = u32;

// Inverse factor of SDR columns to activate (SDR_TTL / SPARSITY = SDR_ACTIVE):
const SPARSITY: usize = 48;
const AXON_VALUE: u8 = 127;


#[inline]
pub fn gen_axn_idxs(rng: &mut SmallRng, active_count: usize, sdr_len: usize) -> Vec<TractAxonIdx> {
    let range = RandRange::new(0, sdr_len as TractAxonIdx);
    (0..active_count).map(|_| range.sample(rng)).collect()
}

/// Writes to deterministic indices.
#[inline]
#[allow(dead_code)]
pub fn write_rand_subset_linear(contrib_count: usize, rev: bool, _: usize,
        indices: &[TractAxonIdx], _: &mut SmallRng, tar: &mut [u8])
{
    fn write(idx_idx: usize, indices: &[TractAxonIdx], tar: &mut [u8]) {
        unsafe {
            let tract_idx = *indices.get_unchecked(idx_idx);
            debug_assert!((tract_idx as usize) < tar.len());
            // *tar.get_unchecked_mut(tract_idx) = 64 + (idx_idx & 63) as u8;
            *tar.get_unchecked_mut(tract_idx as usize) = 128;
        }
    }

    if !rev {
        for idx_idx in 0..contrib_count {
            write(idx_idx, indices, tar);
        }
    } else {
        for idx_idx in (0..contrib_count).rev() {
            write(idx_idx, indices, tar);
        }
    }
}

/// Writes to randomized indices from the `indices` set.
#[inline]
#[allow(dead_code)]
pub fn write_rand_subset_stochastic(subset_count: usize, set_count_ttl: usize,
        indices: &[TractAxonIdx], rng: &mut SmallRng, tar: &mut [u8]) {
    let idx_range = RandRange::new(0, set_count_ttl);

    for _ in 0..subset_count {
        let idx_idx = idx_range.sample(rng);

        unsafe {
            let tract_idx = *indices.get_unchecked(idx_idx);
            debug_assert!((tract_idx as usize) < tar.len());
            *tar.get_unchecked_mut(tract_idx as usize) = 96 + (idx_idx & 63) as u8;
        }
    }
}


#[derive(Clone)]
pub struct ScalarSdrWriter<T> {
    val_range_orig: (T, T),
    val_range: (f32, f32),
    val_span: f32,
    way_span: f32,
    tract_dims: TractDims,
    sdr_len: usize,
    sdr_active_count: usize,
    waypoint_indices: Vec<Vec<TractAxonIdx>>,
    sdrs: Vec<Vec<u8>>,
    rng: SmallRng,
}

impl<T: ScalarEncodable> ScalarSdrWriter<T> {
    pub fn new(val_range: (T, T), way_span: T, tract_dims: &TractDims) -> ScalarSdrWriter<T> {
        let v_size = tract_dims.v_size() as u32;
        let u_size = tract_dims.u_size() as u32;
        assert!(v_size >= 8 && u_size >= 8, "ScalarSdrWriter::new(): Tract frame too small. Side \
            lengths must each be greater than 8.");
        debug_assert!(val_range.0 <= val_range.1);

        let way_span = way_span.to_f32().unwrap();
        assert!(way_span > 0., "ScalarSdrWriter::new(): Waypoint span ('way_span') must be greater \
            than zero.");

        let val_range_orig = val_range.clone();
        let val_range = (val_range.0.to_f32().unwrap(), val_range.1.to_f32().unwrap());

        let val_span = val_range.1 - val_range.0;
        let val_span_norm = val_span / way_span;
        let way_count = (val_span_norm).ceil() as usize + 1;
        let sdr_len = tract_dims.to_len();
        let sdr_active_count = sdr_len / SPARSITY;

        let mut rng = SmallRng::from_entropy();

        // let mut waypoint_indices = Vec::with_capacity(way_count);
        // for _ in 0..way_count {
        //     let sdr = gen_axn_idxs(&mut rng, sdr_active_count, sdr_len);
        //     waypoint_indices.push(sdr);
        // }
        let waypoint_indices: Vec<_> = (0..way_count).map(|_| {
                gen_axn_idxs(&mut rng, sdr_active_count, sdr_len)
            }).collect();

        // let mut sdrs = Vec::with_capacity(way_count);
        // for axn_idxs in waypoint_indices.iter() {
        //     let mut sdr = vec![0u8; sdr_len];
        //     for &axn_idx in axn_idxs.iter() {
        //         sdr[axn_idx] = AXON_VALUE;
        //     }
        //     sdrs.push(sdr);
        // }
        let sdrs: Vec<_> = waypoint_indices.iter().map(|axn_idxs| {
                let mut sdr = vec![0u8; sdr_len];
                for &axn_idx in axn_idxs.iter() {
                    sdr[axn_idx as usize] = AXON_VALUE;
                }
                sdr
            }).collect();

        // /////// [DEBUG]:
        // println!("########## ScalarSdrWriter::new: Value Range: {:?}; Waypoint Span: {}; \
        //     Waypoint Count: {}; Active Count: {};", val_range, way_span, way_count,
        //     sdr_active_count);
        // ///////

        ScalarSdrWriter {
            val_range_orig,
            val_range,
            val_span,
            way_span,
            tract_dims: tract_dims.clone(),
            sdr_len,
            sdr_active_count,
            waypoint_indices,
            sdrs,
            rng,
        }
    }

    // * TODO: Vectorize and port to kernel.
    pub fn encode(&mut self, val_orig: T, tract: &mut TractFrameMut) {
        assert!(tract.dims().to_len() == self.sdr_len);

        // Clear tract frame:
        // for e in tract.frame_mut().iter_mut() { *e = 0; }
        tract.zero();

        // Get a normalized value:
        let val_norm = self.way_norm(val_orig.to_f32().unwrap());

        // Determine the waypoint beneath the current value:
        let way_0 = val_norm.floor();

        // Determine the contribution ratio then count (0-255) for each of the two waypoints:
        let way_1_contrib_ratio = val_norm - way_0;
        let way_1_contrib_count = ((self.sdr_active_count as f32) * way_1_contrib_ratio) as usize;
        debug_assert!(way_1_contrib_count <= self.sdr_active_count);
        let way_0_contrib_count = self.sdr_active_count - way_1_contrib_count;
        // let way_0_contrib = (256. * way_0_contrib_ratio) as isize;
        // debug_assert!(way_0_contrib <= 255);
        // let way_1_contrib = 255 - way_0_contrib;


        // Determine waypoint indices:
        let way_0_idx = way_0 as usize;
        let way_1_idx = way_0_idx + 1;
        debug_assert!(way_0_idx < self.waypoint_indices.len());
        debug_assert!(way_1_idx < self.waypoint_indices.len());

        // /////// [DEBUG]:
        //     let way_0_contrib_ratio = 1.0 - way_1_contrib_ratio;
        //     println!("###### val_orig: {}, val_norm: {}, way_0_idx: {}, way_1_idx, {}, \
        //         way_0_contrib_count: {}({}), way_1_contrib_count: {}({})", val_orig, val_norm, way_0_idx,
        //         way_1_idx, way_0_contrib_count, way_0_contrib_ratio, way_1_contrib_ratio, way_1_contrib_count);
        // ///////

        // let w0_idz = RandRange::new(0, 1 + self.sdr_active_count - way_0_contrib_count)
        //     .sample(&mut self.rng);
        // let w1_idz = RandRange::new(0, 1 + self.sdr_active_count - way_1_contrib_count)
        //     .sample(&mut self.rng);



        // let idx_range = RandRange::new(0, self.sdr_active_count);


        // // Write:
        // // for idx in w0_idz..(w0_idz + way_0_contrib_count) {
        // // for idx in 0..way_0_contrib_count {
        // for _ in 0..way_0_contrib_count {
        //     let idx_idx = idx_range.sample(&mut self.rng);

        //     unsafe {
        //         let tract_idx = *self.waypoint_indices.get_unchecked(way_0_idx)
        //             .get_unchecked(idx_idx);
        //         debug_assert!(tract_idx < tract.len());
        //         *tract.get_unchecked_mut(tract_idx) = 127;
        //     }
        // }

        write_rand_subset_linear(way_0_contrib_count, false, self.sdr_active_count,
            self.waypoint_indices[way_0_idx].as_slice(), &mut self.rng, tract);

        // // for idx in w1_idz..(w1_idz + way_1_contrib_count) {
        // for _ in 0..way_1_contrib_count {
        //     let idx_idx = idx_range.sample(&mut self.rng);

        //     unsafe {
        //         let tract_idx = *self.waypoint_indices.get_unchecked(way_1_idx)
        //             .get_unchecked(idx_idx);
        //         debug_assert!(tract_idx < tract.len());
        //         *tract.get_unchecked_mut(tract_idx) = 127;
        //     }
        // }

        write_rand_subset_linear(way_1_contrib_count, true, self.sdr_active_count,
            self.waypoint_indices[way_1_idx].as_slice(), &mut self.rng, tract);

        ////// SLOW (maybe not -- need to retest):
        // unsafe {
        //     for (idx, (&w0, &w1)) in self.sdrs.get_unchecked(way_0_idx).iter()
        //             .zip(self.sdrs.get_unchecked(way_1_idx).iter())
        //             .enumerate()
        //     {
        //         // Get a random number between 0 and 254:
        //         let rn = range.sample(&mut self.rng);
        //         // Determine if way_0 contrib (0-255) is greater:
        //         let way_0_win = way_0_contrib > rn;
        //         // If so, use w0, else use w1:
        //         let val = (w0 * way_0_win as u8) + (w1 * (!way_0_win) as u8);
        //         // Add a little extra randomness:
        //         *tract.get_unchecked_mut(idx) = val;
        //     }
        // }
    }

    /// Returns a normalized value where the waypoint span is
    /// scaled to 1.0.
    ///
    #[inline]
    fn way_norm(&self, val: f32) -> f32 {
        assert!(val >= self.val_range.0 && val <= self.val_range.1,
            "ScalarSdrWriter::to_norm: Value ({}) out of range ({:?}).", val, self.val_range);
        (val - self.val_range.0) / self.way_span
    }
}

impl<T> fmt::Debug for ScalarSdrWriter<T> where T: fmt::Debug {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("ScalarSdrWriter")
            .field("val_range_orig", &self.val_range_orig)
            .field("val_range", &self.val_range)
            .field("val_span", &self.val_span)
            .field("way_span", &self.way_span)
            .field("tract_dims", &self.tract_dims)
            .field("sdr_len", &self.sdr_len)
            .field("sdr_active_count", &self.sdr_active_count)
            .field("waypoint_indices", &self.waypoint_indices)
            .field("sdrs", &self.sdrs)
            // .field("rng", "SmallRng { .. }")
            .finish()
    }
}
