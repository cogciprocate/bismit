
pub struct TractDims {
	v_size: u32,
	u_size: u32,
}

impl TractDims {
	pub fn new(v_size: u32, u_size: u32) -> TractDims {
		TractDims { v_size: v_size, u_size: u_size }
	}
}
