use std::convert::From;
use cmn::{ ParaHexArray, CorticalDims};

pub struct TractDims {
	v_size: u32,
	u_size: u32,
}

impl TractDims {
	pub fn new(v_size: u32, u_size: u32) -> TractDims {
		TractDims { v_size: v_size, u_size: u_size }
	}
}

impl From<(usize, usize)> for TractDims {
	fn from(sizes: (usize, usize)) -> TractDims {
		TractDims { v_size: sizes.0 as u32, u_size: sizes.1 as u32 }
	}
}

impl From<(u32, u32)> for TractDims {
	fn from(sizes: (u32, u32)) -> TractDims {
		TractDims { v_size: sizes.0, u_size: sizes.1 }
	}
}

impl<'c> From<&'c CorticalDims> for TractDims {
	fn from(dims: &'c CorticalDims) -> TractDims {
		TractDims { v_size: dims.v_size(), u_size: dims.u_size() }
	}
}
