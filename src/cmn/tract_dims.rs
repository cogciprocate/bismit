use std::convert::From;
use cmn::{ParaHexArray, CorticalDims};
use SlcId;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
// TODO: Deprecate -- this is now redundant with cortical dims.
pub struct TractDims {
    depth: SlcId,
    v_size: u32,
    u_size: u32,
}

impl TractDims {
    pub fn new(depth: SlcId, v_size: u32, u_size: u32) -> TractDims {
        TractDims { v_size: v_size, u_size: u_size, depth: depth }
    }

    pub fn to_len(&self) -> usize {
        (self.v_size * self.u_size * self.depth as u32) as usize
    }

    pub fn depth(&self) -> SlcId { self.depth }
    pub fn v_size(&self) -> u32 { self.v_size }
    pub fn u_size(&self) -> u32 { self.u_size }
}

impl ParaHexArray for TractDims {
    fn depth(&self) -> SlcId {
        self.depth
    }

    fn v_size(&self) -> u32 {
        self.v_size
    }

    fn u_size(&self) -> u32 {
        self.u_size
    }
}

// impl From<(usize, usize, usize)> for TractDims {
//     fn from(sizes: (usize, usize, usize)) -> TractDims {
//         TractDims { v_size: sizes.0 as u32, u_size: sizes.1 as u32, depth: sizes.2 as SlcId }
//     }
// }

// impl From<(SlcId, u32, u32)> for TractDims {
//     fn from(sizes: (SlcId, u32, u32)) -> TractDims {
//         TractDims { depth: sizes.0, v_size: sizes.1, u_size: sizes.2 }
//     }
// }

impl<'c, P: ParaHexArray> From<&'c P> for TractDims {
    fn from(dims: &'c P) -> TractDims {
        TractDims { depth: dims.depth(), v_size: dims.v_size(), u_size: dims.u_size() }
    }
}

// default impl<P: ParaHexArray> From<P> for TractDims {
//     fn from(dims: P) -> TractDims {
//         TractDims { v_size: dims.v_size(), u_size: dims.u_size() }
//     }
// }

// impl From<TractDims> for TractDims {
//     fn from(dims: TractDims) -> TractDims {
//         // TractDims { v_size: dims.v_size(), u_size: dims.u_size() }
//         dims
//     }
// }

impl From<CorticalDims> for TractDims {
    fn from(cd: CorticalDims) -> TractDims {
        TractDims { depth: cd.depth(), v_size: cd.v_size(), u_size: cd.u_size() }
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