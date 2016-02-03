use cmn::{Sdr, CorticalDims, TractFrameMut};
use input_source::InputTract;
use encode::GlyphBuckets;

pub struct GlyphSequences {
	buckets: GlyphBuckets,
	layer_dims: CorticalDims,
	seq_lens: usize,
	seq_count: usize,
	scale_factor: f32,
}

impl GlyphSequences {
	#[inline]
	pub fn new(layer_dims: &CorticalDims, seq_lens: usize, seq_count: usize, scale_factor: f32) 
			-> GlyphSequences 
	{
		let buckets = GlyphBuckets::new();

		// Build sequences of bucket_ids as a vec of vecs probably

		GlyphSequences { 			
			buckets: buckets,
			layer_dims: layer_dims.clone(),
			seq_lens: seq_lens,
			seq_count: seq_count,
			scale_factor: scale_factor,
		}
	}
}

impl InputTract for GlyphSequences {
	#[inline]
	fn cycle(&mut self, sdr: &mut Sdr) -> usize {
		let glyph_dims = self.buckets.glyph_dims();
		let glyph: &[u8] = self.buckets.next_glyph(5);

		let tract_frame = TractFrameMut::new(sdr, &self.layer_dims);

		super::encode_2d_image(self.layer_dims.v_size() as usize, self.layer_dims.u_size() as usize, 
			glyph_dims.0, glyph_dims.1, self.scale_factor, 
			glyph, tract_frame);

		0
	}
}
