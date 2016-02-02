use super::TractDims;

/// A view of a terminal of a tract at an instant in time.
pub struct TractFrame<'a> {
	frame: &'a [u8],
	dims: TractDims,
}
