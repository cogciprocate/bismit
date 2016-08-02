#![allow(unused_variables)]
extern crate find_folder;
extern crate bismit;

// use std::iter;
use bismit::GlyphBuckets;
use find_folder::Search;

fn main() {
    let label_file = Search::ParentsThenKids(3, 3).for_folder("tmp_data")
        .expect("ExternalSource::new(): 'label file folder (tmp_data)'")
        .join("train-labels-idx1-ubyte");
    let image_file = Search::ParentsThenKids(3, 3).for_folder("tmp_data")
        .expect("ExternalSource::new(): 'image file folder (tmp_data)'")
        .join("train-images-idx3-ubyte");

    let mut gb = GlyphBuckets::new(label_file, image_file);

    // let mut glyph_buf: Vec<u8> = iter::repeat(0).take(gb.glyph_len()).collect();

    for i in 0..20000 {
        for b_id in 0..gb.count() {
            let glyph = gb.next_glyph(b_id);
        }
    }
}
