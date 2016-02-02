use super::TractDims;

/// A view of a terminal of a tract at an instant in time.
pub struct TractFrame<'a> {
	frame: &'a [u8],
	dims: TractDims,
}

impl<'a> TractFrame<'a> {
	pub fn new<D: Into<TractDims>>(frame: &'a [u8], dims: D) -> TractFrame<'a> {
		TractFrame { frame: frame, dims: dims.into() }
	}
}
