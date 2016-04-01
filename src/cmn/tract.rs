
/// A pipeline or buffer of frames of Sdrs.
#[allow(dead_code)]
pub struct Tract {
	ptr: *mut u8,
	len: usize,
	cap: usize,
}
