use std::ops::{Deref, DerefMut};
use super::TractDims;

/// A view of a terminal of a tract at an instant in time.
pub struct TractFrame<'a> {
    frame: &'a [u8],
    dims: TractDims,
}

impl<'a> TractFrame<'a> {
    #[inline]
    pub fn new<D: Into<TractDims>>(frame: &'a [u8], dims: D) -> TractFrame<'a> {
        let dims = dims.into();
        assert_eq!(dims.to_len(), frame.len());
        TractFrame { frame: frame, dims: dims }
    }

    #[inline]
    pub unsafe fn get_unchecked(&self, idx: usize) -> *const u8 {
        self.frame.get_unchecked(idx)
    }

    pub fn dims(&self) -> &TractDims {
        &self.dims
    }
}

impl<'a> Deref for TractFrame<'a> {
    type Target = [u8];

    fn deref<'b>(&'b self) -> &'b [u8] {
        self.frame
    }
}


pub struct TractFrameMut<'a> {
    frame: &'a mut [u8],
    dims: TractDims,
}

impl<'a> TractFrameMut<'a> {
    #[inline]
    pub fn new<D: Into<TractDims>>(frame: &'a mut [u8], dims: D) -> TractFrameMut<'a> {
        let dims = dims.into();
        assert_eq!(dims.to_len(), frame.len());
        TractFrameMut { frame: frame, dims: dims }
    }

    #[inline]
    pub unsafe fn get_unchecked(&self, idx: usize) -> *const u8 {
        self.frame.get_unchecked(idx)
    }

    #[inline]
    pub unsafe fn get_unchecked_mut(&mut self, idx: usize) -> *mut u8 {
        self.frame.get_unchecked_mut(idx)
    }

    pub fn dims(&self) -> &TractDims {
        &self.dims
    }
}

impl<'a> Deref for TractFrameMut<'a> {
    type Target = [u8];

    fn deref<'b>(&'b self) -> &'b [u8] {
        self.frame
    }
}

impl<'a> DerefMut for TractFrameMut<'a> {
    fn deref_mut<'b>(&'b mut self) -> &'b mut [u8] {
        self.frame
    }
}