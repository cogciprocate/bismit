
// use std::cmp;
use rand::{self, XorShiftRng};
use rand::distributions::{Range, IndependentSample};
use cmn::{TractFrameMut, TractDims};
use encode::ScalarEncodable;

type TractIdx = usize;


#[derive(Debug, Clone)]
pub struct ScalarSdrWriter<T> {
    val_range_original: (T, T),
    val_range: (f32, f32),
    val_span: f32,
    way_span: f32,
    tract_dims: TractDims,
    sdr_len: usize,
    sdr_active_count: usize,
    waypoints: Vec<Vec<TractIdx>>,
}

impl<T: ScalarEncodable> ScalarSdrWriter<T> {
    pub fn new(val_range: (T, T), way_span: T, tract_dims: &TractDims) -> ScalarSdrWriter<T> {
        let v_size = tract_dims.v_size() as u32;
        let u_size = tract_dims.u_size() as u32;
        assert!(v_size >= 8 && u_size >= 8, "ScalarSdrWriter::new(): Tract frame too small. Side \
            lengths must each be greater than 8.");
        assert!(val_range.0 <= val_range.1);

        let val_range_original = val_range.clone();
        let val_range = (val_range.0.to_f32().unwrap(), val_range.1.to_f32().unwrap());
        let val_span = val_range.1 - val_range.0;
        let way_count = (val_span / way_span.to_f32().unwrap()).ceil() as usize + 1;
        println!("########## way_count: {}", way_count);
        let sdr_len = tract_dims.to_len();
        let sdr_active_count = sdr_len / 32;

        let mut waypoints = Vec::with_capacity(way_count);

        let mut rng = rand::weak_rng();

        for way in 0..way_count {
            let sdr = gen_sdr(&mut rng, sdr_active_count, sdr_len);
            waypoints.push(sdr);
        }

        ScalarSdrWriter {
            val_range_original: val_range_original,
            val_range: val_range,
            val_span: val_span,
            way_span: way_span.to_f32().unwrap(),
            tract_dims: tract_dims.clone(),
            sdr_len: sdr_len,
            sdr_active_count: sdr_active_count,
            waypoints: waypoints,
        }
    }

    pub fn encode(&self, val_orig: T, tract: &mut TractFrameMut) {
        assert!(tract.dims().to_len() == self.sdr_len);

        // Clear tract frame:
        for e in tract.frame_mut().iter_mut() {
            *e = 0;
        }

        let val = val_orig.to_f32().unwrap();
        println!("\nval: {}", val);

        // Determine nearest two waypoints:
        let val_ofs = val - self.val_range.0;
        let way_0_f32 = (val_ofs / self.way_span).floor();
        println!("way_0_f32: {}", way_0_f32);
        let way_0 = way_0_f32 as usize;
        println!("way_0: {}", way_0);
        let way_1 = way_0 + 1;
        println!("way_1: {}", way_1);
        debug_assert!(way_0 < self.waypoints.len());
        debug_assert!(way_1 < self.waypoints.len());

        // Determine distance ratio between the two:
        debug_assert!((val_ofs - way_0_f32) <= self.way_span);
        let pos_ratio = (val_ofs - way_0_f32) / self.way_span;
        let pos_idx = ((self.sdr_active_count as f32) * pos_ratio) as usize;

        println!("sdr_active_count: {}, pos_ratio: {}, pos_idx: {}",
            self.sdr_active_count, pos_ratio, pos_idx);

        debug_assert!(self.sdr_active_count - pos_idx > 0);
        let way_0_contrib = pos_idx;
        let way_1_contrib = self.sdr_active_count - pos_idx;

        // Write:
        for idx in 0..way_0_contrib {
            debug_assert!(idx < tract.frame().len());
            unsafe {
                let tract_idx = *self.waypoints.get_unchecked(way_0).get_unchecked(idx);
                *tract.get_unchecked_mut(tract_idx) = 127;
            }
        }

        for idx in 0..way_1_contrib {
            debug_assert!(idx < tract.frame().len());
            unsafe {
                let tract_idx = *self.waypoints.get_unchecked(way_1).get_unchecked(idx);
                *tract.get_unchecked_mut(tract_idx) = 127;
            }
        }
    }
}

fn gen_sdr(rng: &mut XorShiftRng, active_count: usize, sdr_len: usize) -> Vec<TractIdx> {
    let mut sdr = Vec::with_capacity(active_count);
    let range = Range::new(0, sdr_len);

    for i in 0..active_count {
        let idx = range.ind_sample(rng);
        sdr.push(idx);
    }

    sdr
}