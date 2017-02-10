
use rand::{self, XorShiftRng};
use rand::distributions::{Range, IndependentSample};
use cmn::{TractFrameMut, TractDims};
use encode::ScalarEncodable;

type TractAxonIdx = usize;

// Inverse factor of SDR columns to activate (SDR_TTL / SPARSITY = SDR_ACTIVE):
const SPARSITY: usize = 48;


#[derive(Debug, Clone)]
pub struct ScalarSdrWriter<T> {
    val_range_orig: (T, T),
    val_range: (f32, f32),
    val_span: f32,
    way_span: f32,
    tract_dims: TractDims,
    sdr_len: usize,
    sdr_active_count: usize,
    waypoints: Vec<Vec<TractAxonIdx>>,
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

        let mut waypoints = Vec::with_capacity(way_count);

        let mut rng = rand::weak_rng();

        for _ in 0..way_count {
            let sdr = gen_sdr(&mut rng, sdr_active_count, sdr_len);
            waypoints.push(sdr);
        }

        // /////// [DEBUG]:
        // println!("########## ScalarSdrWriter::new: Value Range: {:?}; Waypoint Span: {}; \
        //     Waypoint Count: {}; Active Count: {};", val_range, way_span, way_count,
        //     sdr_active_count);
        // ///////

        ScalarSdrWriter {
            val_range_orig: val_range_orig,
            val_range: val_range,
            val_span: val_span,
            way_span: way_span,
            tract_dims: tract_dims.clone(),
            sdr_len: sdr_len,
            sdr_active_count: sdr_active_count,
            waypoints: waypoints,
        }
    }

    // [TODO]: Vectorize and port to kernel.
    pub fn encode(&self, val_orig: T, tract: &mut TractFrameMut) {
        assert!(tract.dims().to_len() == self.sdr_len);

        // Clear tract frame:
        // for e in tract.frame_mut().iter_mut() { *e = 0; }
        tract.zero();

        // Get a normalized value:
        let val_norm = self.way_norm(val_orig.to_f32().unwrap());

        // Determine the waypoint beneath the current value:
        let way_0 = val_norm.floor();

        // Determine the contribution count for each of the two waypoints:
        let way_0_contrib_ratio = val_norm - way_0;
        let way_1_contrib_count = ((self.sdr_active_count as f32) * way_0_contrib_ratio) as usize;
        let way_0_contrib_count = self.sdr_active_count - way_1_contrib_count;


        // Determine waypoint indices:
        let way_0_idx = way_0 as usize;
        let way_1_idx = way_0_idx + 1;
        debug_assert!(way_0_idx < self.waypoints.len());
        debug_assert!(way_1_idx < self.waypoints.len());

        // /////// [DEBUG]:
        // println!("###### val_orig: {}, val_norm: {}, way_0_idx: {}, way_1_idx, {}, \
        //     way_0_contrib_count: {}({}), way_1_contrib_count: {}", val_orig, val_norm, way_0_idx,
        //     way_1_idx, way_0_contrib_count, way_0_contrib_ratio, way_1_contrib_count);
        // ///////

        // Write:
        for idx in 0..way_0_contrib_count {
            debug_assert!(idx < tract.frame().len());
            unsafe {
                let tract_idx = *self.waypoints.get_unchecked(way_0_idx).get_unchecked(idx);
                *tract.get_unchecked_mut(tract_idx) = 127;
            }
        }

        for idx in 0..way_1_contrib_count {
            debug_assert!(idx < tract.frame().len());
            unsafe {
                let tract_idx = *self.waypoints.get_unchecked(way_1_idx).get_unchecked(idx);
                *tract.get_unchecked_mut(tract_idx) = 127;
            }
        }
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

fn gen_sdr(rng: &mut XorShiftRng, active_count: usize, sdr_len: usize) -> Vec<TractAxonIdx> {
    let mut sdr = Vec::with_capacity(active_count);
    let range = Range::new(0, sdr_len);

    for _ in 0..active_count {
        let idx = range.ind_sample(rng);
        sdr.push(idx);
    }

    sdr
}