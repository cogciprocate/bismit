pub use self::idx_streamer::IdxStreamer;
pub use self::idx_data::IdxData;
pub use self::glyph_buckets::GlyphBuckets;
pub use self::glyph_sequences::GlyphSequences;

pub mod idx_streamer;
mod idx_data;
mod glyph_buckets;
mod glyph_sequences;

use cmn::{TractFrameMut, ParaHexArray};

const SQRT_3: f32 = 1.73205080756f32;

#[inline]
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
#[inline]
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
    scale_factor: f32, source: &[u8], /*target: &mut Sdr*/mut target: TractFrameMut) 
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


// [TODO]: Wire me up, Scotty.
pub fn encode_scalar() {
    // for v_id in 0..v_size {
    //     for u_id in 0..u_size {
    //         let (x, y, valid) = coord_hex_to_pixel(v_size, v_id, u_size, u_id, 
    //             self.image_height as usize, self.image_width as usize);
            
    //         if valid {
    //             let tar_idx = (v_id * u_size) + u_id;
    //             let src_idx = (y * self.image_width as usize) + x;

    //             target[tar_idx] = source[src_idx]; 
    //         }
    //         //target[tar_idx] = (x != 0 || y != 0) as u8; // SHOW INPUT SQUARE
    //     }
    // }
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
