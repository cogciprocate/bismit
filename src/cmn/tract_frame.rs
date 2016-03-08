use super::TractDims;

/// A view of a terminal of a tract at an instant in time.
pub struct TractFrame<'a> {
    frame: &'a [u8],
    dims: TractDims,
}

impl<'a> TractFrame<'a> {
    #[inline]
    pub fn new<D: Into<TractDims>>(frame: &'a [u8], dims: D) -> TractFrame<'a> {
        TractFrame { frame: frame, dims: dims.into() }
    }

    #[inline]
    pub unsafe fn get_unchecked(&self, idx: usize) -> *const u8 {
        self.frame.get_unchecked(idx)
    }

    pub fn dims(&self) -> &TractDims {
        &self.dims
    }
}


pub struct TractFrameMut<'a> {
    frame: &'a mut [u8],
    dims: TractDims,
}

impl<'a> TractFrameMut<'a> {
    #[inline]
    pub fn new<D: Into<TractDims>>(frame: &'a mut [u8], dims: D) -> TractFrameMut<'a> {
        TractFrameMut { frame: frame, dims: dims.into() }
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
