use rand::distributions::{IndependentSample, Range};
use rand;
use cmn::{Sdr, CorticalDims, TractFrameMut};
use input_source::InputTract;
use encode::GlyphBuckets;
// use map::proto::Protoinput;

pub struct SeqCursor {
	seq: usize,
	gly: usize,
}

impl SeqCursor {
	fn next(&mut self, sequences: &Vec<Vec<usize>>) -> usize {
		let next_glyph_id = sequences[self.seq][self.gly];
		self.gly += 1;

		if self.gly >= sequences[self.seq].len() { 
			self.gly = 0;
			self.seq += 1;
			if self.seq >= sequences.len() { self.seq = 0; }
		}

		next_glyph_id
	}
}


pub struct GlyphSequences {
	sequences: Vec<Vec<usize>>,
	buckets: GlyphBuckets,
	layer_dims: CorticalDims,
	cursor: SeqCursor,
	scale: f32,
}

impl GlyphSequences {
	#[inline]
	pub fn new(layer_dims: &CorticalDims, seq_lens: (usize, usize), seq_count: usize, 
				scale: f32) -> GlyphSequences 
	{
		assert!(seq_lens.1 >= seq_lens.0, "GlyphSequences::new(): Sequence length range ('seq_lens') \
			invalid. High end must at least be equal to low end: '{:?}'.", seq_lens);

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
			layer_dims: layer_dims.clone(),
			cursor: SeqCursor { seq: 0, gly: 0 },
			scale: scale,
		}
	}

	pub fn sequences(&self) -> &Vec<Vec<usize>> {
		&self.sequences
	}
}

impl InputTract for GlyphSequences {
	#[inline]
	fn cycle(&mut self, sdr: &mut Sdr) -> usize {
		let glyph_dims = self.buckets.glyph_dims();
		let next_glyph_id = self.cursor.next(&self.sequences);
		let glyph: &[u8] = self.buckets.next_glyph(next_glyph_id);

		let tract_frame = TractFrameMut::new(sdr, &self.layer_dims);

		super::encode_2d_image(glyph_dims, &self.layer_dims, self.scale,
			glyph, tract_frame);

		0
	}
}


mod tests {
	#[test]
	fn test_glyph_sequences() {
		use encode::GlyphSequences;
		use cmn::CorticalDims;

		let dims = CorticalDims::new(32, 32, 1, 0, None);

		for i in 0..20 {
			let seq_lens = (i, (i * 2) + 11);
			let seq_count = 79 - i;

			let gss = GlyphSequences::new(&dims, seq_lens, seq_count, 1.0);

			assert!(gss.sequences.len() == seq_count);

			for seq in gss.sequences() {
				assert!(seq.len() >= seq_lens.0);
				assert!(seq.len() <= seq_lens.1);
			}
		}		
	}
}
