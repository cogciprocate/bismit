#![allow(unused_variables)]
extern crate bismit;

use std::iter;
use bismit::encode::GlyphBuckets;

fn main() {
	let mut gb = GlyphBuckets::new();

	let mut glyph_buf: Vec<u8> = iter::repeat(0).take(gb.glyph_len()).collect();

	for i in 0..20000 {
		for b_id in 0..gb.bucket_count() {
			gb.next_glyph(b_id, &mut glyph_buf);
		}
	}
}
