use std::convert::From;
use cmn::{ParaHexArray, CorticalDims};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TractDims {
    v_size: u32,
    u_size: u32,
    depth: u8,
}

impl TractDims {
    pub fn v_size(&self) -> u32 {
        self.v_size
    }

    pub fn u_size(&self) -> u32 {
        self.u_size
    }

    pub fn depth(&self) -> u8 {
        self.depth
    }

    pub fn new(v_size: u32, u_size: u32, depth: u8) -> TractDims {
        TractDims { v_size: v_size, u_size: u_size, depth: depth }
    }

    pub fn to_len(&self) -> usize {
        (self.v_size * self.u_size * self.depth as u32) as usize
    }
}

impl ParaHexArray for TractDims {
    fn v_size(&self) -> u32 {
        self.v_size
    }

    fn u_size(&self) -> u32 {
        self.u_size
    }

    fn depth(&self) -> u8 {
        self.depth
    }
}

impl From<(usize, usize, usize)> for TractDims {
    fn from(sizes: (usize, usize, usize)) -> TractDims {
        TractDims { v_size: sizes.0 as u32, u_size: sizes.1 as u32, depth: sizes.2 as u8 }
    }
}

impl From<(u32, u32, u8)> for TractDims {
    fn from(sizes: (u32, u32, u8)) -> TractDims {
        TractDims { v_size: sizes.0, u_size: sizes.1, depth: sizes.2 }
    }
}

impl<'c, P: ParaHexArray> From<&'c P> for TractDims {
    fn from(dims: &'c P) -> TractDims {
        TractDims { v_size: dims.v_size(), u_size: dims.u_size(), depth: dims.depth() }
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
        TractDims { v_size: cd.v_size(), u_size: cd.u_size(), depth: cd.depth() }
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