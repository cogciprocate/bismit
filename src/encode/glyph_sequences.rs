use std::collections::HashMap;
use rand::distributions::{IndependentSample, Range};
use rand;
use cmn::{CorticalDims, TractFrameMut};
use map::{self, LayerTags};
use external_source::{ExternalSourceTract, ExternalSourceLayer};
use encode::GlyphBuckets;
use proto::AxonKind;
// use map::proto::Protoinput;

pub struct SeqCursor {
    seq_idx: usize,
    gly_idx: usize,
}

impl SeqCursor {
    fn next(&mut self, sequences: &Vec<Vec<usize>>) -> (usize, usize) {
        let next_seq_idx = self.seq_idx;
        let next_glyph_id = sequences[self.seq_idx][self.gly_idx];
        self.gly_idx += 1;

        if self.gly_idx >= sequences[self.seq_idx].len() { 
            self.gly_idx = 0;
            self.seq_idx += 1;
            if self.seq_idx >= sequences.len() { self.seq_idx = 0; }
        }

        (next_seq_idx, next_glyph_id)
    }
}


pub struct GlyphSequences {
    sequences: Vec<Vec<usize>>,
    buckets: GlyphBuckets,
    spt_layer_dims: CorticalDims,
    hrz_layer_dims: CorticalDims,
    cursor: SeqCursor,
    scale: f32,
}

impl GlyphSequences {
    #[inline]
    pub fn new(layers: &mut HashMap<LayerTags, ExternalSourceLayer>, seq_lens: (usize, usize), 
                seq_count: usize, scale: f32, hrz_dims: (u32, u32)) -> GlyphSequences
    {
        assert!(seq_lens.1 >= seq_lens.0, "GlyphSequences::new(): Sequence length range ('seq_lens') \
            invalid. High end must at least be equal to low end: '{:?}'.", seq_lens);
        assert_eq!(layers.len(), 2);

        let mut spt_layer_dims: Option<CorticalDims> = None;
        let mut hrz_layer_dims: Option<CorticalDims> = None;

        for (tags, layer) in layers.iter_mut() {
            if layer.axn_kind() == AxonKind::Spatial {
                assert!(tags.contains(map::FF_OUT));
                spt_layer_dims = layer.dims().cloned();
            } else if layer.axn_kind() == AxonKind::Horizontal {                
                assert!(tags.contains(map::NS_OUT));
                hrz_layer_dims = Some(CorticalDims::new(hrz_dims.0, hrz_dims.1, 1, 0, None));
                layer.set_dims(hrz_layer_dims.clone());
            }
        }

        let buckets = GlyphBuckets::new();
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
            sequences: sequences,
            buckets: buckets,
            // layer_dims: [layer_dims.clone(), layer_dims.clone()],
            spt_layer_dims: spt_layer_dims.expect("GlyphSequences::new(): Spatial dims not set."),
            hrz_layer_dims: hrz_layer_dims.expect("GlyphSequences::new(): Horizontal dims not set."),
            cursor: SeqCursor { seq_idx: 0, gly_idx: 0 },
            scale: scale,
        }
    }

    pub fn sequences(&self) -> &Vec<Vec<usize>> {
        &self.sequences
    }
}

impl ExternalSourceTract for GlyphSequences {
    fn read_into(&mut self, tags: LayerTags, tract_frame: &mut TractFrameMut) 
            -> [usize; 3]
    {
        let glyph_dims = self.buckets.glyph_dims();
        let (next_seq_idx, next_glyph_id) = self.cursor.next(&self.sequences);
        let glyph: &[u8] = self.buckets.next_glyph(next_glyph_id);

        if tags.contains(map::FF_OUT) {
            assert!(&self.spt_layer_dims == tract_frame.dims());           
            super::encode_2d_image(glyph_dims, &self.spt_layer_dims, self.scale,
                glyph, tract_frame);
        } else if tags.contains(map::NS_OUT) {
            assert!(&self.hrz_layer_dims == tract_frame.dims());
            // ENCODE THE HRZ BUSINESS
            // super::encode_2d_image(glyph_dims, &self.hrz_layer_dims, self.scale,
            //     glyph, tract_frame);
        } else {
            panic!("GlyphSequences::read_into(): Invalid tags: tags: '{:?}' must mesh with {:?}", 
                tags, map::NS_OUT);
        }

        [next_glyph_id, next_seq_idx, 0]
    }

    fn cycle_next(&mut self) {
        
    }
}


mod tests {
    #[test]
    fn glyph_sequences_FIXME() {
        use std::collections::HashMap;
        use encode::GlyphSequences;
        use cmn::CorticalDims;
        use map::LayerTags;
        use external_source::ExternalSourceLayer;

        let dims = CorticalDims::new(32, 32, 1, 0, None);

        for i in 0..6 {
            let seq_lens = (i, (i * 2) + 11);
            let seq_count = 79 - i;

            // [FIXME]: Bring the rest of this back:

            // let extern_src_lyr = ExternalSourceLayer::

            // // &mut dims
            // let area_map: HashMap<LayerTags, ExternalSourceLayer> = HashMap::with_capacity(2);

            // let gss = GlyphSequences::new(&mut area_map, seq_lens, seq_count, 1.0, (16, 16));

            // assert!(gss.sequences.len() == seq_count);

            // for seq in gss.sequences() {
            //     assert!(seq.len() >= seq_lens.0);
            //     assert!(seq.len() <= seq_lens.1);
            // }
        }        
    }
}
