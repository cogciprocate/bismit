

struct SeqGen {
	step: usize,
	seq_eles: Vec<u8>,
}

impl SeqGen {
	pub fn new() -> SeqGen {
		SeqGen {
			step: 0,
			seq_eles: vec![9, 200, 50, 4],
		}
	}

	pub fn next(&mut self) -> u8 {
		if self.step >= self.seq_eles.len() {
			self.step = 0;
		} else {
			self.step += 1;
		}

		self.seq_eles[self.step]
	}
}
