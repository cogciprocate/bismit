
use std::cmp;
use rand;
use rand::distributions::{Range, IndependentSample};
use cmn::{TractFrameMut, TractDims};
use encode::ScalarEncodable;


#[derive(Debug, Clone)]
pub struct ScalarGlyphWriter<T> {
    val_range_original: (T, T),
    val_range: (f32, f32),
    val_span: f32,
    glyph_radius: u32,
    margin: u32,
    track_tiles_v: u32,
    track_tiles_u: u32,
    track_tiles_total: u32,
    track_tiles_total_f32: f32,
    tract_dims: TractDims,
    sides: (u32, u32, u32, u32),
}

impl<T: ScalarEncodable> ScalarGlyphWriter<T> {
    pub fn new(val_range: (T, T), tract_dims: &TractDims) -> ScalarGlyphWriter<T> {
        let v_size = tract_dims.v_size() as u32;
        let u_size = tract_dims.u_size() as u32;
        assert!(v_size >= 8 && u_size >= 8, "ScalarGlyphWriter::new(): Tract frame too small. Side \
            lengths must each be greater than 8.");
        assert!(val_range.0 <= val_range.1);

        // [NOTE]: Side length = radius + 1;
        let radius = (v_size + u_size) / 32;
        let extra_margin = radius + 1;
        let margin = radius + extra_margin + 1;

        // Length of the 'track' running along each margin:
        let track_tiles_v = v_size - (margin * 2) - 1;
        let track_tiles_u = u_size - (margin * 2) - 1;
        let track_tiles_total = (track_tiles_v * 2) + (track_tiles_u * 2);
        let track_tiles_total_f32 = track_tiles_total as f32;

        let val_range_original = val_range.clone();
        let val_range = (val_range.0.to_f32().unwrap(), val_range.1.to_f32().unwrap());
        let val_span = val_range.1 - val_range.0;

        let sides = (
            track_tiles_u,
            track_tiles_u + track_tiles_v,
            (2 * track_tiles_u) + track_tiles_v,
            track_tiles_total,
        );

        ScalarGlyphWriter {
            val_range_original: val_range_original,
            val_range: val_range,
            val_span: val_span,
            glyph_radius: radius,
            margin: margin,
            track_tiles_v: track_tiles_v,
            track_tiles_u: track_tiles_u,
            track_tiles_total: track_tiles_total,
            track_tiles_total_f32: track_tiles_total_f32,
            tract_dims: tract_dims.clone(),
            sides: sides,
        }
    }

    pub fn encode(&self, val: T, tract: &mut TractFrameMut) {
        let val = val.to_f32().unwrap();

        assert!(val >= self.val_range.0 && val <= self.val_range.1, "Unable to encode scalar \
            value: '{}'. The value is outside of the allowed range: '{:?}'.", val, self.val_range);
        assert!(tract.dims() == &self.tract_dims, "Tract frame dimension mismatch.");

        let val_pct = (val - self.val_range.0) / self.val_span;
        debug_assert!(val_pct <= 1.0);
        let val_tile = (val_pct * self.track_tiles_total_f32).floor() as u32;

        let ttv = self.track_tiles_v;
        let ttu = self.track_tiles_u;
        let margin = self.margin;
        let sides = self.sides;

        debug_assert!(self.tract_dims.v_size() - margin - 1 == ttv + margin);
        debug_assert!(self.tract_dims.u_size() - margin - 1 == ttu + margin);

        #[derive(Debug)]
        struct Center {
            v: u32,
            u: u32,
        }

        let center = if val_tile < sides.0 {
            Center {
                v: margin,
                u: margin + val_tile,
            }
        } else if val_tile < sides.1 {
            Center {
                v: margin + (val_tile - sides.0),
                u: margin + ttu,
            }
        } else if val_tile < sides.2 {
            Center {
                v: margin + ttv,
                u: (margin + ttu) - (val_tile - sides.1),
            }
        } else if val_tile < sides.3 {
            Center {
                v: (margin + ttv) - (val_tile - sides.2),
                u: margin,
            }
        } else {
            assert!(val_tile == self.track_tiles_total);

            Center {
                v: margin,
                u: margin,
            }
        };

        // Set up an RNG for no particular reason:
        let mut rng = rand::weak_rng();
        let r_base = (val_pct * 128.0f32) as u8;
        let r_range = Range::<u8>::new(r_base, r_base + 64);

        // Clear tract frame:
        for e in tract.frame_mut().iter_mut() {
            *e = 0;
        }

        // Notation reminder:
        // * '_z': zero (idx[0])
        // * '_m': max (idx[len - 1])
        // * '_n': number of elements, length (idx[len])
        // Save some inverses just to avoid repeated calculation:
        let r = self.glyph_radius as i32;
        let r_neg = 0 - r;
        let v_z = r_neg;
        let v_m = r;
        let v_n = v_m + 1;
        let u_size = self.tract_dims.u_size() as i32;

        for v in v_z..v_n {
            let v_neg = 0 - v;
            let u_z = cmp::max(r_neg, v_neg + r_neg);
            let u_m = cmp::min(r, v_neg + r);
            let u_n = u_m + 1;

            for u in u_z..u_n {
                let idx = (((v + center.v as i32) * u_size) + u + center.u as i32) as usize;

                unsafe {
                    *tract.get_unchecked_mut(idx) = r_range.ind_sample(&mut rng);
                    // *tract.get_unchecked_mut(idx) = 255;
                }
            }
        }
    }
}
