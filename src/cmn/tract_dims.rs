use std::convert::From;
use cmn::{ParaHexArray, CorticalDims};

#[derive(Clone, Debug, PartialEq)]
pub struct TractDims {
    v_size: u32,
    u_size: u32,
}

impl TractDims {
    #[inline]
    pub fn new(v_size: u32, u_size: u32) -> TractDims {
        TractDims { v_size: v_size, u_size: u_size }
    }

    pub fn to_len(&self) -> usize {
        (self.v_size * self.u_size) as usize
    }
}

impl ParaHexArray for TractDims {
    #[inline]
    fn v_size(&self) -> u32 {
        self.v_size
    }

    #[inline]
    fn u_size(&self) -> u32 {
        self.u_size
    }

    #[inline]
    fn depth(&self) -> u8 {
        1u8
    }
}

impl From<(usize, usize)> for TractDims {
    #[inline]
    fn from(sizes: (usize, usize)) -> TractDims {
        TractDims { v_size: sizes.0 as u32, u_size: sizes.1 as u32 }
    }
}

impl From<(u32, u32)> for TractDims {
    #[inline]
    fn from(sizes: (u32, u32)) -> TractDims {
        TractDims { v_size: sizes.0, u_size: sizes.1 }
    }
}

impl<'c, P: ParaHexArray> From<&'c P> for TractDims {
    #[inline]
    fn from(dims: &'c P) -> TractDims {
        TractDims { v_size: dims.v_size(), u_size: dims.u_size() }
    }
}

impl PartialEq<CorticalDims> for TractDims {
    fn eq(&self, other: &CorticalDims) -> bool {
        self.v_size == other.v_size() && 
            self.u_size == other.u_size() &&
            self.depth() == other.depth()
    }
}

// impl PartialEq<TractDims> for TractDims {
//     fn eq(&self, other: &TractDims) -> bool {
//         self.v_size == other.v_size() && self.u_size == other.u_size()
//     }
// }