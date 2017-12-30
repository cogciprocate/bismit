//! Renders a two dimensional vector as a SDR.
//!
//! Likely how spatial points/velocities are represented in vivo. Has all of
//! the hallmarks of what we would expect to see including extreme robustness
//! and noise tolerance with an arbitrary level of precision. The radix can be
//! varied arbitrarily with higher radixes providing a wider scale at the cost
//! of lower redundancy/overlap.
//!
//!
//! ### Further Improvements
//!
//! - 3/omnidimensionality
//! - Raw uvw input
//!
//!
//! ### Conventions
//!
//! x: 0˚, y: 90˚
//! u: 330˚ (-30˚), v: 90˚, w: 210˚,
//!
//
// [NOTE]: `f64` is complete overkill. `f32` is perfectly fine.
//


// use cmn::{TractFrameMut, TractDims};
use cmn::TractDims;
// use encode::ScalarEncodable;

// The difference between each level of scale:
const RADIX: u32 = 3;
// The ratio of render line 'width' (`u_size`) to padding.
const PADDING_QUOTIENT: isize = 128;

/// Converts (x, y) coordinates into (u, v, w).
///
/// x: 0˚, y: 90˚
/// u: 330˚ (-30˚), v: 90˚, w: 210˚
///
#[inline]
fn convert(xy: [f64; 2]) -> [f64; 3] {
    let u = (xy[0] * (3.0f64).sqrt() - xy[1]) / 2.0;
    let v = xy[1];
    let w = -(u + v);
    [u, v, w]
}

/// Returns a renderable value-compliment pair for `val`. The compliment is the
/// nearest relevant (renderable) point in a triangle grid.
#[inline]
fn rc_pairs(mut val: f64) -> [f64; 2] {
    // Scale and shift value such that [-1.0, 1.0] -> [0.0, 1.0]:
    // val = (val / 2.) + 0.5;
    val = val.mul_add(2.0f64.recip(), 0.5);

    // Values repeat every -/+ 1.0.
    val = val.fract();

    // Add 1 to value if negative:
    val += (val < 0.) as i32 as f64;

    // Create a compliment partner for rendering:
    let lt_half = (val <= 0.5) as i32 as f64;
    let gt_half = (val > 0.5) as i32 as f64;
    [val - gt_half, val + lt_half]
}


/// Renders a two dimensional vector as a SDR.
#[derive(Clone, Debug)]
pub struct Vector2dWriter {
    tract_dims: TractDims,
    // Unused: [QUESTION]: Implement offs/scale setter functions or leave to
    // caller to scale appropriately?
    raw_offs: f64,
    // Unused:
    raw_scale: f64,
    // Precalculated scales (-/+ powers of RADIX):
    scale_levels: Vec<f64>,
    // Length of each render line:
    precision_redundancy: u32,
    // Padding for drawing:
    render_pad: isize,
}

impl Vector2dWriter {
    pub fn new<Td: Into<TractDims>>(tract_dims: Td) -> Vector2dWriter {
        let tract_dims = tract_dims.into();
        assert!(tract_dims.v_size() as i32 >= 3, "Vector2dWriter::new: 'v' dimension too small.");
        // println!("####### tract_dims: {:?}", tract_dims);
        let scale_level_count = tract_dims.v_size() as i32 / 3;
        let precision_redundancy = tract_dims.u_size();
        let render_pad = precision_redundancy as isize / PADDING_QUOTIENT;

        let scale_level_mid_idx = scale_level_count / 2;

        let mut scale_levels = Vec::with_capacity(scale_level_count as usize);

        for scale_level_idx in 0..scale_level_count {
            let unit = (RADIX as f64).powi(scale_level_mid_idx - scale_level_idx);
            // println!("###### VECTOR2DWRITER: Adding scale level: {}", unit);
            scale_levels.push(unit);
        }

        Vector2dWriter {
            tract_dims,
            raw_offs: 0.,
            raw_scale: 1.,
            scale_levels,
            precision_redundancy,
            render_pad,
        }
    }

    /// Applies the preset offset and scale (default 0.0 and 1.0
    /// respectively).
    #[inline]
    fn xform(&self, raw: f64) -> f64 {
        (raw + self.raw_offs) * self.raw_scale
    }

    /// Transforms (offsets & scales) then converts (x, y) coordinates into
    /// normalized (u, v, w) and a magnitude.
    #[allow(dead_code)]
    fn decompose(&self, mut xy: [f64; 2]) -> ([f64; 3], f64) {
        xy = [self.xform(xy[0]), self.xform(xy[1])];

        let mag = (xy[0].powi(2) + xy[1].powi(2)).sqrt();
        xy = [xy[0] / mag, xy[1] / mag];

        (convert(xy), mag)
    }

    pub fn encode(&mut self, xy_raw: [f64; 2], tract: &mut [u8]) {
        assert_eq!(tract.len(), self.tract_dims.to_len());
        // let (uvw, mag) = self.decompose(xy_raw);
        let xy = [self.xform(xy_raw[0]), self.xform(xy_raw[1])];
        let uvw = convert(xy);

        // println!("\n########## UVW: {:?}", uvw);

        // NOTE: Consider removing this check and clamp the values at maximums
        // or make the whole thing non-linear and scale to infinity. There may
        // be a good reason to allow an arbitrary range of random-ish values
        // to represent 0/null.
        assert!(uvw[0].abs().max(uvw[1].abs()).max(uvw[2].abs()) <= self.scale_levels[0],
            "Vector2dWriter::encode: A vector value exceeds the maximum (values: {:?}). \
                Increase tract 'v' size to accommodate a larger range of values \
                or scale/shift passed values to get them closer to zero. ", xy_raw);

        let tract_chunk_size = 3 * self.tract_dims.u_size() as usize;

        let render_state = 1u8;

        for (scale_level, tract_chunk) in self.scale_levels.iter()
                .zip(tract.chunks_mut(tract_chunk_size)) {
            // Determine quotients:
            let quots = [uvw[0] / scale_level,
                uvw[1] / scale_level,
                uvw[2] / scale_level];

            // println!("\n## quots: {:?}", quots);

            // Center point pairs to be rendered.
            let centers = [rc_pairs(quots[0]),
                rc_pairs(quots[1]),
                rc_pairs(quots[2])];

            // println!("###### centers: {:?}", centers);

            let prec = self.precision_redundancy as f64;

            // TODO: Refactor/clean up.
            let center_idxs = [
                [(centers[0][0] * prec) as isize,
                    (centers[0][1] * prec) as isize],
                [(centers[1][0] * prec) as isize,
                    (centers[1][1] * prec) as isize],
                [(centers[2][0] * prec) as isize,
                    (centers[2][1] * prec) as isize],
            ];

            // TODO: Refactor/clean up.
            let edge_idxs = [
                [[center_idxs[0][0] - self.render_pad, center_idxs[0][0] + self.render_pad],
                    [center_idxs[0][1] - self.render_pad, center_idxs[0][1] + self.render_pad]],
                [[center_idxs[1][0] - self.render_pad, center_idxs[1][0] + self.render_pad],
                    [center_idxs[1][1] - self.render_pad, center_idxs[1][1] + self.render_pad]],
                [[center_idxs[2][0] - self.render_pad, center_idxs[2][0] + self.render_pad],
                    [center_idxs[2][1] - self.render_pad, center_idxs[2][1] + self.render_pad]],
            ];

            // println!("######## edge_idxs: {:?}", edge_idxs);

            for (v_id_chunk, chunk_row) in tract_chunk.chunks_mut(self.tract_dims.u_size() as usize)
                    .enumerate() {
                for (u_id, axon) in chunk_row.iter_mut().enumerate() {
                    let edges = &edge_idxs[v_id_chunk];
                    let is_active = (u_id as isize >= edges[0][0] && u_id as isize <= edges[0][1]) ||
                        (u_id as isize >= edges[1][0] && u_id as isize <= edges[1][1]);
                    *axon = (is_active as u8) * render_state;
                }
            }
        }
    }
}