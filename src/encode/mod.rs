#![allow(dead_code, unused_variables, unused_mut)]



mod idx_data;
mod glyph_buckets;
mod glyph_sequences;
mod sensory_tract;
mod scalar;
mod scalar_sequence;
mod reverso_scalar_sequence;
mod vector_encoder;
mod scalar_glyph_writer;
mod hex_mold_test;
pub mod idx_streamer;

use std::cmp;
use std::ops::AddAssign;
use std::fmt::{Debug, Display};
use num::{Num, NumCast};
use rand;
use rand::distributions::{Range, IndependentSample};
use cmn::{self, TractFrameMut, ParaHexArray};
pub use self::idx_streamer::IdxStreamer;
pub use self::idx_data::IdxData;
pub use self::glyph_buckets::GlyphBuckets;
pub use self::glyph_sequences::GlyphSequences;
pub use self::sensory_tract::SensoryTract;
pub use self::scalar_sequence::ScalarSequence;
pub use self::reverso_scalar_sequence::ReversoScalarSequence;
pub use self::vector_encoder::VectorEncoder;
pub use self::scalar_glyph_writer::ScalarGlyphWriter;
pub use self::hex_mold_test::HexMoldTest;

const SQRT_3: f32 = 1.73205080756f32;


pub trait ScalarEncodable: Num + NumCast + PartialOrd + Debug + Display + Clone +
    AddAssign + Copy + Default {}
impl<T> ScalarEncodable for T where T: Num + NumCast + PartialOrd + Debug + Display + Clone +
    AddAssign + Copy + Default {}



fn calc_offs(v_size: usize, u_size: usize, y_size: usize, x_size: usize, hex_side: f32) -> (f32, f32) {
    let v_mid = v_size >> 1;
    let u_mid = u_size >> 1;

    let (x_ofs_inv, y_ofs_inv, _) = coord_hex_to_pixel(v_mid as f32, u_mid as f32,
        x_size as f32, y_size as f32, hex_side, 0.0, 0.0);

    let x_mid = x_size >> 1;
    let y_mid = y_size >> 1;

    ((x_ofs_inv - x_mid as f32), (y_mid as f32 - y_ofs_inv))
}


// COORD_HEX_TO_PIXEL(): Eventually either move this to GPU or at least use SIMD.
pub fn coord_hex_to_pixel(v_id: f32, u_id: f32, x_size: f32, y_size: f32, hex_side: f32,
            x_ofs: f32, y_ofs: f32,
        ) -> (f32, f32, bool)
{
    let u = u_id;
    let u_inv = 0.0 - u;
    let v = v_id;
    let w_inv = v + u;

    let mut x = w_inv * 1.5 * hex_side;
    let mut y = (u_inv + (w_inv / 2.0)) * SQRT_3 * hex_side;

    x -= x_ofs;
    y += y_ofs;

    let valid = (y >= 0.0 && y < y_size) && (x >= 0.0 && x < x_size);

    (x, y, valid)
}


// ENCODE_2D_IMAGE(): Horribly unoptimized.
pub fn encode_2d_image<P: ParaHexArray>(src_dims: (usize, usize), tar_dims: &P,
    scale_factor: f32, source: &[u8], mut target: &mut TractFrameMut)
{
    let (x_size, y_size) = (src_dims.0, src_dims.1);
    let (v_size, u_size) = (tar_dims.v_size() as usize, tar_dims.u_size() as usize);
    let hex_side = (x_size + y_size) as f32 /
        (scale_factor * (v_size + u_size) as f32);

    let (x_ofs, y_ofs) = calc_offs(v_size, u_size, x_size, y_size, hex_side);

    for v_id in 0..v_size  {
        for u_id in 0..u_size {
            let (x, y, valid) = coord_hex_to_pixel(v_id as f32, u_id as f32, x_size as f32,
                y_size as f32, hex_side, x_ofs, y_ofs);

            if valid {
                let tar_idx = (v_id * u_size) + u_id;
                let src_idx = (y as usize * x_size) + x as usize;

                unsafe { *target.get_unchecked_mut(tar_idx) = *source.get_unchecked(src_idx); }

                // SHOW INPUT SQUARE
                // target[tar_idx] = 1 as u8;
            }
        }
    }
}


pub fn print_image(image: &[u8], dims: (usize, usize)) {
    for y in 0..dims.1 {
        print!("\n    ");
        for x in 0..dims.0 {
            let idx = (y * dims.0) + x;
            print!("{:2X} ", image[idx]);
        }
    }
    println!("");
}



/// Encode a scalar as a hexagon somewhere along the border of the tract frame
/// (cyclical).
///
/// [TODO]: Migrate this into a type impl which stores calculated
/// intermediates.
///
pub fn encode_scalar<T>(val: T, val_range: (T, T), tract: &mut TractFrameMut)
            where T: ScalarEncodable {
    assert!(val >= val_range.0 && val <= val_range.1, "Unable to encode scalar value: '{}'. The \
        value is outside of the allowed range: {:?}.", val, val_range);
    let v_size = tract.dims().v_size() as i32;
    let u_size = tract.dims().u_size() as i32;
    assert!(v_size >= 8 && u_size >= 8, "encode::encode_scalar(): Tract frame too small. Side \
        lengths must each be greater than 8.");

    // [NOTE]: To fill to roughly 1.5% density, activate roughly 1/8 of either
    // v_size or usize or 1/16 of v_size + usize.
    //
    // [UPDATE: This is not accurate at small scales]
    //
    //
    // [NOTE]: Side length = radius + 1;
    let radius = (v_size + u_size) / 32;
    let extra_margin = radius + 1;
    let margin = radius + extra_margin + 1;

    // Length of the 'track' running along each margin:
    let track_len_v = v_size - (margin * 2) - 1;
    let track_len_u = u_size - (margin * 2) - 1;

    let val = val.to_f32().unwrap();
    // Quadrant, clockwise starting with 0 -> upper left:
    let quad_size_val = val_range.1.to_f32().unwrap() / 4.0;
    let quadrant = (val / quad_size_val).floor();
    debug_assert!(quadrant < 5.0);
    // val % quad_size_val:
    let quad_pos_val = val - (quad_size_val * quadrant);
    debug_assert!((quad_pos_val - (val % quad_size_val).abs()) < 0.00001,
        format!("quad_pos_val: {}, (val % quad_size_val).abs(): {}", quad_pos_val,
        (val % quad_size_val).abs()));

    #[derive(Debug)]
    struct Center {
        v: i32,
        u: i32,
    }

    #[inline]
    fn val_to_tract(quad_pos_val: f32, quad_size_val: f32, quad_size_tract: i32) -> i32 {
        let quad_pos_tract = (quad_pos_val / quad_size_val) * (quad_size_tract as f32);
        debug_assert!(quad_pos_tract >= 0.0 && (quad_pos_tract as i32) < quad_size_tract);
        quad_pos_tract as i32
    }

    // Center 'tile' of the final rendered hexagon:
    let center = if quadrant < 1.0 || quadrant >= 4.0 {
        Center {
            v: margin,
            u: val_to_tract(quad_pos_val, quad_size_val, track_len_u) + margin,
        }
    } else if quadrant < 2.0 {
        Center {
            v: val_to_tract(quad_pos_val, quad_size_val, track_len_v) + margin,
            u: u_size - margin - 1,
        }
    } else if quadrant < 3.0 {
        Center {
            v: v_size - margin - 1,
            u: u_size - (val_to_tract(quad_pos_val, quad_size_val, track_len_u) + margin) - 1,
        }
    } else {
        Center {
            v: v_size - (val_to_tract(quad_pos_val, quad_size_val, track_len_v) + margin) - 1,
            u: margin,
        }
    };

    // [DEBUG (Track granularity)]:
    // let track_len_ttl = (track_len_v + track_len_u) * 2;
    // let val_range_ttl = quad_size_val * 4.0;
    // let val_per_track = val_range_ttl / track_len_ttl as f32;
    // print!("\n");
    // println!("Total track length: {} (v: {}, u: {})",
    //     track_len_ttl, track_len_v, track_len_u);
    // println!("Val/Track: {}", val_per_track);
    // print!("\n");
    // println!("val: {}, quad_size_val: {}, quadrant: {}, v_size: {}, u_size: {}",
    //     val, quad_size_val, quadrant, v_size, u_size);
    // println!("quad_pos_val: {}, val % quad_size_val: {}", quad_pos_val, val % quad_size_val);
    // println!("{:?}", center);

    // Save some inverses just to avoid repeated calculation:
    let radius_neg = 0 - radius;

    let mut rng = rand::weak_rng();
    let r_range = Range::<u8>::new(64, 128);

    // Clear tract frame:
    for e in tract.frame_mut().iter_mut() {
        *e = 0;
    }

    // Notation reminder:
    // * '_z': zero (idx[0])
    // * '_m': max (idx[len - 1])
    // * '_n': number of elements, length (idx[len])
    let v_z = radius_neg;
    let v_m = radius;
    let v_n = v_m + 1;

    for v in v_z..v_n {
        let v_neg = 0 - v;
        let u_z = cmp::max(radius_neg, v_neg + radius_neg);
        let u_m = cmp::min(radius, v_neg + radius);
        let u_n = u_m + 1;

        for u in u_z..u_n {
            let idx = (((v + center.v) * u_size) + u + center.u) as usize;
            unsafe {
                *tract.get_unchecked_mut(idx) = r_range.ind_sample(&mut rng);
                // *tract.get_unchecked_mut(idx) = 255;
            }
        }
    }
}



// List of offsets to form a hexagon-shaped pattern of tiles.
//
// `scales` and `center` contain [v, u] values respectively.
//
// '..._z' suffix := index[0], first element, starting element
//
// '..._n' suffix := index[len]: element after final element, termination
// point, number of elements (ex.: for(int i = 0, i < idn, i++))
//
// [TODO]: Create extra version of `::calc_scale` which accepts an additional
// precision (log2) parameter and returns it's scale adjusted accordingly.
//
#[warn(dead_code, unused_variables, unused_mut)]
// pub fn encode_hex_mold_scaled(radius: i8, scales: [u32; 2], center: [u32; 2], tract: &mut TractFrameMut) {
pub fn encode_hex_mold_scaled(radius: i8, src_dims: [u32; 2], tract_frame: &mut TractFrameMut) {
    use map;

    let dst_dims = [tract_frame.dims().v_size(), tract_frame.dims().u_size()];
    let scales = [cmn::calc_scale(dst_dims[0], src_dims[0]).unwrap(),
        cmn::calc_scale(dst_dims[1], src_dims[1]).unwrap()];

    let mold = map::gen_syn_offs(radius, scales).unwrap();

    let dst_mid = [tract_frame.dims().v_size() / 2, tract_frame.dims().u_size() / 2];

    for ofs in mold.into_iter() {
        let idx = (((ofs.0 as i32 + dst_mid[0] as i32) * tract_frame.dims().u_size() as i32) +
            ofs.1 as i32 + dst_mid[1] as i32) as usize;

        // Make sure this isn't a duplicate tile (ensures the mold doesn't have redundancies):
        // debug_assert!(*tract.get(idx).unwrap() == 0, "Destination index out of bounds: {}/{}",
        //     idx, tract.len());

        // Make sure this isn't a duplicate tile (ensures the mold doesn't have redundancies):
        // match tract_frame.get(idx) {
        //     Some(&val) => assert!(val == 0, "Destination index duplicate found."),
        //     None => panic!("Destination index out of bounds: {}/{}", idx, tract_frame.len()),
        // }

        unsafe { *tract_frame.get_unchecked_mut(idx) = 1; }
        // *tract_frame.get_mut(idx).unwrap() = 1;
    }
}

