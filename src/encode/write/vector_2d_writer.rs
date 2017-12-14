#![allow(unused_variables, dead_code)]

//!
//! x: 0˚, y: 90˚
//! u: 330˚ (-30˚), v: 90˚, w: 210˚,

use cmn::{TractFrameMut, TractDims};
// use encode::ScalarEncodable;

/// Converts (x, y) coordinates into (u, v, w).
///
/// x: 0˚, y: 90˚
/// u: 330˚ (-30˚), v: 90˚, w: 210˚
///
fn convert(xy: [f32; 2]) -> [f32; 3] {
    let u = (xy[0] * (3.0f32).sqrt() - xy[1]) / 2.0;
    let v = xy[1];
    let w = -(u + v);
    [u, v, w]
}

#[derive(Clone, Debug)]
pub struct Vector2dWriter {
    raw_offs: f32,
    raw_scale: f32,
    // dim_count: u32,
    scale_level_count: u32,
    precision_redundancy: u32,
    scale_level_mid: u32,
}

impl Vector2dWriter {
    pub fn new(tract_dims: TractDims) -> Vector2dWriter {
        assert!(tract_dims.u_size() >= 7, "Vector2dWriter::new: 'u' dimension too small.");
        let scale_level_count = tract_dims.v_size() / 7;
        let precision_redundancy = tract_dims.u_size();

        let scale_level_mid = (scale_level_count / 2) + (scale_level_count % 2);

        Vector2dWriter {
            raw_offs: 0.,
            raw_scale: 1.,
            // dim_count: 2,
            scale_level_count,
            precision_redundancy,
            scale_level_mid,
        }
    }

    /// Applies the preset offset and scale (default 0.0 and 1.0
    /// respectively).
    #[inline]
    fn xform(&self, raw: f32) -> f32 {
        (raw + self.raw_offs) * self.raw_scale
    }

    /// Transforms (offsets & scales) then converts (x, y) coordinates into
    /// normalized (u, v, w) and a magnitude.
    fn decompose(&self, mut xy: [f32; 2]) -> ([f32; 3], f32) {
        xy = [self.xform(xy[0]),
            self.xform(xy[1])];

        let mag = (xy[0].powi(2) + xy[1].powi(2)).sqrt();
        xy = [xy[0] / mag, xy[1] / mag];

        (convert(xy), mag)
    }

    pub fn encode(&mut self, xy_raw: [f32; 2], tract: &mut TractFrameMut) {
        // let (uvw, mag) = self.decompose(xy_raw);
        let xy = [self.xform(xy_raw[0]), self.xform(xy_raw[1])];
        let uvw = convert(xy);

        // for scale_level_idx in 0..self.scale_level_count {
        //     let unit = if scale_level_idx < self.scale_level_mid {
        //         0.
        //     } else if scale_level_idx > self.scale_level_mid {
        //         0.
        //     } else {
        //         1.0f32
        //     };
        // }


    }
}