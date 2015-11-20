
#[derive(PartialEq, Debug, Clone)]
pub enum Protoinput {
	World,
	Stripes { stripe_size: usize, zeros_first: bool },
	Hexballs { edge_size: usize, invert: bool, fill: bool },
	Zeros,
	IdxReader { file_name: &'static str, cyc_per: usize, scale: f64 },
	IdxReaderLoop { file_name: &'static str, cyc_per: usize, scale: f64, loop_frames: u32 },
	None,
}

impl Protoinput {
	pub fn is_some(&self) -> bool {
		match self {
			&Protoinput::None => false,
			_ => true,
		}
	}

	pub fn is_none(&self) -> bool {
		!self.is_some()
	}
}
