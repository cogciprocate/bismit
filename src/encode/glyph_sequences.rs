use cmn::{Sdr, CorticalDims};
use input_source::InputTract;
use encode::GlyphBuckets;

pub struct GlyphSequences {
	buckets: GlyphBuckets,
	dims: CorticalDims,
	seq_lens: usize,
	seq_count: usize,
	scale_factor: f32,
}

impl GlyphSequences {
	pub fn new(dims: CorticalDims, seq_lens: usize, seq_count: usize, scale_factor: f32) 
			-> GlyphSequences 
	{
		let buckets = GlyphBuckets::new();

		// Build sequences of bucket_ids as a vec of vecs probably

		GlyphSequences { 			
			buckets: buckets,
			dims: dims,
			seq_lens: seq_lens,
			seq_count: seq_count,
			scale_factor: scale_factor,
		}
	}
}

impl InputTract for GlyphSequences {
	fn cycle(&mut self, tract_frame: &mut Sdr) -> usize {
		let glyph_dims = self.buckets.glyph_dims();
		let glyph: &[u8] = self.buckets.next_glyph(5);

		super::encode_2d_image(self.dims.v_size() as usize, self.dims.u_size() as usize, 
			glyph_dims.0, glyph_dims.1, self.scale_factor, 
			glyph, tract_frame);

		0
	}
}
