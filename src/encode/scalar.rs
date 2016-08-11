use std::ops::Range;
use cmn::{TractDims, TractFrameMut};

pub struct ScalarEncoder2d {
    tract_dims: TractDims,
    range: Range<usize>,
}

impl ScalarEncoder2d {
    pub fn new(tract_dims: TractDims, range: Range<usize>) -> ScalarEncoder2d {
        ScalarEncoder2d {
            tract_dims: tract_dims,
            range: range,
        }
    }

    pub fn encode(val: usize, target: &mut TractFrameMut) {

    }
}


#[allow(dead_code)]
pub struct CoordEncoder2d {
    tract_dims: TractDims,
    coord_ranges: (usize, usize),
    blip_size: (usize, usize),
}

impl CoordEncoder2d {
    pub fn new(tract_dims: TractDims, coord_ranges: (usize, usize)) -> CoordEncoder2d {
        let blip_size = (
            (tract_dims.u_size() as usize / coord_ranges.0) + 1,
            (tract_dims.v_size() as usize / coord_ranges.1) + 1,
        );

        CoordEncoder2d {
            tract_dims: tract_dims,
            coord_ranges: coord_ranges,
            blip_size: blip_size,
        }
    }

    pub fn encode(coord: (u32, u32), target: &mut TractFrameMut) {

    }
}



// pub struct CoordEncoder1d {
//     size: i32,
// }

// impl CoordEncoder1d {
//     pub fn new(size: i32) -> CoordEncoder1d {
//         CoordEncoder1d { size: size }
//     }
// }
