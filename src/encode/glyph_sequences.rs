use std::collections::HashMap;
use std::path::PathBuf;
use rand::distributions::{IndependentSample, Range};
use rand;
use cmn::{CorticalDims, TractFrameMut};
use map::{self, LayerAddress, AxonTags};
use subcortex::{InputGeneratorTract, InputGeneratorLayer, /*SubcorticalNucleusLayer*/};
use encode::GlyphBuckets;
// use map::AxonTopology;


/// The cursor containing the current position of the glyph sequence.
#[derive(Debug)]
pub struct SeqReader {
    sequences: Vec<Vec<usize>>,
    seq_idx: usize,
    gly_idx: usize,
}

impl SeqReader {
    fn new(sequences: Vec<Vec<usize>>) -> SeqReader {
         SeqReader { sequences: sequences, seq_idx: 0, gly_idx: 0 }
    }

    fn get(&self) -> (usize, usize) {
        (self.seq_idx, self.sequences[self.seq_idx][self.gly_idx])
    }

    fn advance(&mut self) {
        self.gly_idx += 1;

        if self.gly_idx >= self.sequences[self.seq_idx].len() {
            self.gly_idx = 0;
            self.seq_idx += 1;
            if self.seq_idx >= self.sequences.len() { self.seq_idx = 0; }
        }
    }
}



/// Sequences of 2D colorless images.
///
/// The sequences vary in length and number specified by `seq_lens` and
/// `seq_count` respectively.
///
#[derive(Debug)]
pub struct GlyphSequences {
    // sequences: Vec<Vec<usize>>,
    buckets: GlyphBuckets,
    img_layer_dims: CorticalDims,
    img_layer_addr: LayerAddress,
    val_layer_dims: CorticalDims,
    val_layer_addr: LayerAddress,
    cursor: SeqReader,
    scale: f32,
}

impl GlyphSequences {
    pub fn img_lyr_tags() -> AxonTags {
        // (&[map::EXT, map::GLY_SEQ_IMG]).into()
        AxonTags::new(&[map::EXT, map::GLY_SEQ_IMG])
    }

    pub fn val_lyr_tags() -> AxonTags {
        // (&[map::EXT, map::GLY_SEQ_VAL]).into()
        AxonTags::new(&[map::EXT, map::GLY_SEQ_VAL])
    }

    pub fn new(layers: &mut HashMap<LayerAddress, InputGeneratorLayer>, seq_lens: (usize, usize),
                seq_count: usize, scale: f32, hrz_dims: (u32, u32), label_file: PathBuf,
                image_file: PathBuf) -> GlyphSequences
    {
        assert!(seq_lens.1 >= seq_lens.0, "GlyphSequences::new(): Sequence length range ('seq_lens') \
            invalid. High end must at least be equal to low end: '{:?}'.", seq_lens);
        assert_eq!(layers.len(), 2);

        let mut img_layer_dims: Option<CorticalDims> = None;
        let mut img_layer_addr: Option<LayerAddress> = None;
        let mut val_layer_dims: Option<CorticalDims> = None;
        let mut val_layer_addr: Option<LayerAddress> = None;

        for layer in layers.iter_mut().map(|(_, layer)| layer) {
            // if layer.axn_kind() == AxonTopology::Spatial {
            //     assert!(tags.contains(map::FF_OUT));
            //     img_layer_dims = layer.dims().cloned();
            // } else if layer.axn_kind() == AxonTopology::Horizontal {
            //     assert!(tags.contains(map::NS_OUT));
            //     val_layer_dims = Some(CorticalDims::new(hrz_dims.0, hrz_dims.1, 1, /*0,*/ None));
            //     layer.set_dims(val_layer_dims.clone());
            // }

            if layer.axn_tags().is_superset(&GlyphSequences::img_lyr_tags()) {
                // assert!(tags.contains(map::FF_OUT));
                assert!(img_layer_dims.is_none());
                assert!(img_layer_addr.is_none());
                img_layer_dims = Some(layer.sub().dims().unwrap().clone());
                img_layer_addr = Some(layer.sub().addr().clone());
            } else if layer.axn_tags().is_superset(&GlyphSequences::val_lyr_tags()) {
                // assert!(tags.contains(map::NS_OUT));
                assert!(val_layer_dims.is_none());
                assert!(val_layer_addr.is_none());
                val_layer_dims = Some(CorticalDims::new(hrz_dims.0, hrz_dims.1, 1, /*0,*/ None));
                layer.set_dims(val_layer_dims.clone().unwrap());
                val_layer_addr = Some(layer.sub().addr().clone());
            }

        }

        let buckets = GlyphBuckets::new(label_file, image_file);
        let mut rng = rand::weak_rng();
        let mut sequences = Vec::with_capacity(seq_count);

        // Build sequences of bucket_ids:
        for _ in 0..seq_count {
            let seq_len = Range::new(seq_lens.0, seq_lens.1 + 1).ind_sample(&mut rng);
            let mut seq = Vec::<usize>::with_capacity(seq_len);

            for _ in 0..seq_len {
                let glyph_id = Range::new(0, buckets.count()).ind_sample(&mut rng);
                seq.push(glyph_id);
            }

            sequences.push(seq);
        }

        GlyphSequences {
            // sequences: sequences,
            buckets: buckets,
            // layer_dims: [layer_dims.clone(), layer_dims.clone()],
            img_layer_dims: img_layer_dims.expect("GlyphSequences::new(): Spatial dims not set."),
            img_layer_addr: img_layer_addr.expect("GlyphSequences::new(): Spatial layer address not set."),
            val_layer_dims: val_layer_dims.expect("GlyphSequences::new(): Horizontal dims not set."),
            val_layer_addr: val_layer_addr.expect("GlyphSequences::new(): Horizontal layer address not set."),
            cursor: SeqReader::new(sequences),
            scale: scale,
        }
    }

    pub fn sequences(&self) -> &Vec<Vec<usize>> {
        &self.cursor.sequences
    }
}

impl InputGeneratorTract for GlyphSequences {
    fn write_into(&mut self, tract_frame: &mut TractFrameMut, addr: LayerAddress) {
        let glyph_dims = self.buckets.glyph_dims();
        let (_, next_glyph_id) = self.cursor.get();
        let glyph: &[u8] = self.buckets.next_glyph(next_glyph_id);

        if addr == self.img_layer_addr {
            assert!(&self.img_layer_dims == tract_frame.dims());
            super::encode_2d_image(glyph_dims, &self.img_layer_dims, self.scale,
                glyph, tract_frame);
        } else if addr == self.val_layer_addr {
            assert!(&self.val_layer_dims == tract_frame.dims());
            // * TODO: ENCODE THE HRZ BUSINESS
            // super::encode_2d_image(glyph_dims, &self.val_layer_dims, self.scale,
            //     glyph, tract_frame);
            for idx in 0..tract_frame.dims().to_len() {
                unsafe { *tract_frame.get_unchecked_mut(idx) = (idx & 0xFF) as u8; }
            }
        } else {
            panic!("GlyphSequences::write_into(): Layer address not found: '{:?}'.", addr);
        }

        // [next_glyph_id, next_seq_idx, 0]
    }

    fn cycle_next(&mut self) {
        self.cursor.advance();
    }
}


mod tests {
    // #[test]
    // /// Huge pain in the ass to re-implement this test now that a hashmap of
    // /// `InputGeneratorLayer`s is req'd.
    // fn glyph_sequences_FIXME() {
    //     use std::collections::HashMap;
    //     use encode::GlyphSequences;
    //     use cmn::CorticalDims;
    //     use map::LayerTags;
    //     use thalamus::InputGeneratorLayer;

    //     let dims = CorticalDims::new(32, 32, 1, 0, None);

    //     for i in 0..6 {
    //         let seq_lens = (i, (i * 2) + 11);
    //         let seq_count = 79 - i;

    //         let mut layers: HashMap<LayerTags, InputGeneratorLayer> = HashMap::with_capacity(2);

    //         let gss = GlyphSequences::new(&mut area_map, seq_lens, seq_count, 1.0, (16, 16));

    //         assert!(gss.sequences.len() == seq_count);

    //         for seq in gss.sequences() {
    //             assert!(seq.len() >= seq_lens.0);
    //             assert!(seq.len() <= seq_lens.1);
    //         }
    //     }
    // }
}
