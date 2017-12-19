#![allow(unused_variables, dead_code)]

//!
//! x: 0˚, y: 90˚
//! u: 330˚ (-30˚), v: 90˚, w: 210˚,

use cmn::{TractFrameMut, TractDims};
// use encode::ScalarEncodable;

const RADIX: i32 = 3;

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

// // If a `val` is less than 0.5, adds 1.0.
// #[inline]
// fn shift_to_pos(val: f32) -> f32 {
//     if val <= -0.5 { val + 1. } else { val }
// }

// If `val` is negative ...
fn np_pairs(val: f32) -> [f32; 2] {
    if val < 0. { [val, val + 2.] } else { [val - 2., val] }
}


#[derive(Clone, Debug)]
pub struct Vector2dWriter {
    tract_dims: TractDims,
    raw_offs: f32,
    raw_scale: f32,
    // dim_count: u32,
    // scale_level_count: u32,
    scale_levels: Vec<f32>,
    precision_redundancy: u32,
    // scale_level_mid_idx: usize,
}

impl Vector2dWriter {
    pub fn new(tract_dims: TractDims) -> Vector2dWriter {
        assert!(tract_dims.v_size() as i32 >= 3, "Vector2dWriter::new: 'v' dimension too small.");
        let scale_level_count = tract_dims.v_size() as i32 / 3;
        let precision_redundancy = tract_dims.u_size();

        let mut scale_level_mid_idx = scale_level_count / 2;
        scale_level_mid_idx +=  scale_level_count - (scale_level_mid_idx * 2);

        let mut scale_levels = Vec::with_capacity(scale_level_count as usize);

        for scale_level_idx in 0..scale_level_count {
            // let unit = if scale_level_idx < scale_level_mid_idx {
            //     let sld = (scale_level_mid_idx - scale_level_idx) as i32;
            //     (3.0).powi(sld)
            // } else {
            //     let sld = (scale_level_idx - scale_level_mid_idx) as i32;
            //     (3.0).powi(sld).recip()
            // };
            let unit = (RADIX as f32).powi(scale_level_mid_idx - scale_level_idx);
            println!("###### VECTOR2DWRITER: Adding scale level: {}", unit);
            scale_levels.push(unit);
        }

        Vector2dWriter {
            tract_dims,
            raw_offs: 0.,
            raw_scale: 1.,
            // dim_count: 2,
            // scale_level_count,
            scale_levels,
            precision_redundancy,
            // scale_level_mid_idx,
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
    #[allow(dead_code)]
    fn decompose(&self, mut xy: [f32; 2]) -> ([f32; 3], f32) {
        xy = [self.xform(xy[0]),
            self.xform(xy[1])];

        let mag = (xy[0].powi(2) + xy[1].powi(2)).sqrt();
        xy = [xy[0] / mag, xy[1] / mag];

        (convert(xy), mag)
    }

    pub fn encode(&mut self, xy_raw: [f32; 2], tract: &mut TractFrameMut) {
        assert!(tract.dims() == &self.tract_dims);
        // let (uvw, mag) = self.decompose(xy_raw);
        let xy = [self.xform(xy_raw[0]), self.xform(xy_raw[1])];
        let mut uvw = convert(xy);

        // NOTE: Consider removing this check and clamp the values at maximums
        // or make the whole thing non-linear and scale to infinity.
        assert!(uvw[0].abs().max(uvw[1].abs()).max(uvw[2].abs()) <= self.scale_levels[0],
            "Vector2dWriter::encode: A vector value exceeds the maximum (values: {:?}). \
                Increase tract 'v' size to accommodate a larger range of values \
                or scale/shift passed values to get them closer to zero. ", xy_raw);

        let tract_chunk_size = 3 * tract.dims().u_size() as usize;

        let render_pad = 2isize;
        let render_state = 1u8;

        for (scale_level, tract_chunk) in self.scale_levels.iter()
                .zip(tract.chunks_mut(tract_chunk_size)) {
            // debug_assert!(uvw[0].abs().max(uvw[1].abs()).max(uvw[2].abs()) < scale_level);

            // The dividend which will give us the portion of the spectrum we
            // are interested in (1.0..-1.0):
            let dividend = scale_level;

            // Determine quotients:
            let quots = [uvw[0] / dividend,
                uvw[1] / dividend,
                uvw[2] / dividend];

            // Determine quotient whole number component:
            let wholes = [quots[0].trunc(),
                quots[1].trunc(),
                quots[2].trunc()];

            // Determine quotient fraction component:
            let fracts = [quots[0] - wholes[0],
                quots[1] - wholes[1],
                quots[2] - wholes[2]];

            // // Shift and scale values to get them within the relevant
            // // drawing range (-0.5..1.5):
            // let centers = [shift_to_pos(fracts[0]) * 2.,
            //     shift_to_pos(fracts[1]) * 2.,
            //     shift_to_pos(fracts[2]) * 2.];

            // Center point pairs to be rendered (spaced by 2.0). This means
            // that values at/near -1.0 will be mirrored at/near 1.0; in
            // effect wrapping the value around.
            let centers = [np_pairs(fracts[0]),
                np_pairs(fracts[1]),
                np_pairs(fracts[2])];

            let prec = self.precision_redundancy as f32;

            let center_idxs = [
                [(centers[0][0] * prec) as isize, (centers[0][1] * prec) as isize],
                [(centers[1][0] * prec) as isize, (centers[1][1] * prec) as isize],
                [(centers[2][0] * prec) as isize, (centers[2][1] * prec) as isize],
            ];

            let edge_idxs = [
                [[center_idxs[0][0] - render_pad, center_idxs[0][0] + render_pad],
                    [center_idxs[0][1] - render_pad, center_idxs[0][1] + render_pad]],
                [[center_idxs[1][0] - render_pad, center_idxs[1][0] + render_pad],
                    [center_idxs[1][1] - render_pad, center_idxs[1][1] + render_pad]],
                [[center_idxs[2][0] - render_pad, center_idxs[2][0] + render_pad],
                    [center_idxs[2][1] - render_pad, center_idxs[2][1] + render_pad]],
            ];

            for (v_id_chunk, chunk_row) in tract_chunk.chunks_mut(self.tract_dims.u_size() as usize)
                    .enumerate() {
                for (u_id, axon) in chunk_row.iter_mut().enumerate() {
                    let edges = &edge_idxs[v_id_chunk];
                    let is_active = (u_id as isize >= edges[0][0] && u_id as isize <= edges[0][1]) ||
                        (u_id as isize >= edges[1][0] && u_id as isize <= edges[1][1]);
                    *axon = (is_active as u8) * render_state;
                }
            }

            // Trim uvw (to avoid precision-noise at small scales):
            uvw = [uvw[0] - (wholes[0] * dividend),
                uvw[1] - (wholes[1] * dividend),
                uvw[2] - (wholes[2] * dividend)];
        }
    }
}