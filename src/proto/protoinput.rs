
#[derive(PartialEq, Debug, Clone)]
pub enum Protoinput {
	World,
	Stripes { stripe_size: usize, zeros_first: bool },
	Hexballs { edge_size: usize, invert: bool, fill: bool },
	Exp1,
	IdxReader { file_name: &'static str, cyc_per: usize, scale: f64 },
	IdxReaderLoop { file_name: &'static str, cyc_per: usize, scale: f64, loop_frames: u32 },
	None,
}
