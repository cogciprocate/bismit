use ocl::SpatialDims;
use ocl::traits::MemLen;
use cmn::{ParaHexArray, TractDims, CmnResult};
use SlcId;

// # CorticalDims: Dimensions of a cortical area in units of cells
// - Used to define both the volume and granularity of a cortical area.
// - Also contains extra information such as opencl kernel workgroup size
//
//
// <<<<< THIS DESCRIPTION IS WAY OUT OF DATE >>>>>
// <<<<< TODO: ADD INFORMATION ABOUT TUFTS (OUR 5TH DIMENSION?) >>>>>
//
// Stored in log base 2 as a constraint (for simplicity and computational
// efficiency within OpenCL kernels).
//
// Cells are hexagonal prisms
//
// Dimensions are in the context of bismit where:
//     - Column is 1 x 1 x N (a rod)
//     - Slice (unfortunately coincident with rust terminology) has
//       dimensions N x M x 1 (a plane)
//     - Row has no meaning
//
// So, v_size * u_size determines number of columns
//
// The 4th parameter, per_cel_l2, is basically components or divisions
// per cell and can also be thought of as a 4th dimension. It can be
// positive or negative reflecting whether or not it's bigger or smaller
// than a cell and it's stored inverted. Don't think too hard about it.


#[derive(Debug, Clone, PartialEq, Eq)]
/// Dimensions for a cortical, area, layer, slice, or other subdivison thereof.
pub struct CorticalDims {
    depth: SlcId, // in cell-edges (NxMx1)
    v_size: u32, // in cell-edges (log2) (HxWxD: 1x1xN)
    u_size: u32, // in cell-edges (log2) (HxWxD: 1x1xN)
}

impl CorticalDims {
    pub const fn new(depth: SlcId, v_size: u32, u_size: u32, /*per_tft_l2: i8,*/ /*incr: Option<u32>*/) -> CorticalDims {
        CorticalDims {
            depth: depth,
            v_size: v_size,
            u_size: u_size,
        }
    }

    // // TUFTS_PER_CEL(): Dendrite tufts per cell
    // #[inline]
    // pub fn tfts_per_cel(&self) -> u32 {
    //     self.tfts_per_cel
    // }

    // #[inline]
    // pub fn per_tft_l2(&self) -> i8 {
    //     self.per_tft_l2
    // }

    // // PHYSICAL_INCREMENT():
    // //         TODO: improve this description
    // #[deprecated]
    // pub fn incr(&self) -> CmnResult<u32> {
    //     match self.incr {
    //         Some(pi) => Ok(pi),
    //         None => Err("CorticalDims::incr: Physical increment not set.".into()),
    //     }
    // }

    // SCALED_PHYSICAL_INCREMENT(): Represents the increment of the columns, not simply the dens/syns/whatever
    //         i.e. if cel_phys_incr == 256, syns must have an phys_incr of cel_phys_incr * syns_per_cel
    //
    //         TODO: DEPRICATE
    // pub fn scaled_incr(&self) -> Result<u32, &'static str> {
    //     match self.incr {
    //         Some(pi) => {
    //             let phys_incr = (pi << self.per_tft_l2_left()) * self.tfts_per_cel;
    //             Ok(phys_incr)
    //         },

    //         None => Err("physical increment not set"),
    //     }
    // }

    #[inline]
    pub fn depth(&self) -> SlcId {
        self.depth
    }

    #[inline]
    pub fn v_size(&self) -> u32 {
        self.v_size
    }

    #[inline]
    pub fn u_size(&self) -> u32 {
        self.u_size
    }

    // COLUMNS(): 2D Area of a slc measured in cell sides
    #[inline]
    pub fn columns(&self) -> u32 {
        self.v_size * self.u_size
    }

    // CELLS(): 3D Volume of area measured in cells
    #[inline]
    pub fn cells(&self) -> u32 {
        self.columns() * self.depth as u32
    }

    // // TUFTS(): 4D Volume of area measured in (dendrite-tuft * cells)
    // #[inline]
    // pub fn cel_tfts(&self) -> u32 {
    //     self.cells() * self.tfts_per_cel
    // }

    // #[inline]
    // pub fn per_tft_l2_left(&self) -> u32 {
    //     if self.per_tft_l2 >= 0 {
    //         self.per_tft_l2 as u32
    //     } else {
    //         panic!("\nocl::CorticalDims::per_tft_l2_left(): may only be called if per_tft_l2 is positive");
    //     }
    // }

    // #[inline]
    // pub fn per_tft_l2_right(&self) -> u32 {
    //     if self.per_tft_l2 < 0 {
    //         (0 - self.per_tft_l2) as u32
    //     } else {
    //         panic!("\nocl::CorticalDims::per_tft_l2_right(): may only be called if per_tft_l2 is negative");
    //     }
    // }

    // #[inline]
    // pub fn per_cel(&self) -> u32 {
    //     len_components(1, self.per_tft_l2, self.tfts_per_cel)
    // }

    // #[inline]
    // pub fn per_tft(&self) -> u32 {
    //     len_components(1, self.per_tft_l2, 1)
    // }

    // #[inline]
    // pub fn per_slc_per_tft(&self) -> u32 {
    //     len_components(self.columns(), self.per_tft_l2, 1)
    // }

    // // PER_SLICE(): 2D Area of a slc measured in divisions/components/whatever
    // #[inline]
    // pub fn per_slc(&self) -> u32 {
    //     len_components(self.columns(), self.per_tft_l2, self.tfts_per_cel)
    // }

    #[inline]
    /// [FIXME]: Return a proper result type, wrap the OclError from `::padded_buffer_len`.
    pub fn per_subgrp(&self, subgroup_count: u32) -> CmnResult<u32> {
        // let physical_len = self.to_len_padded(ocl_pq.max_wg_size()) as u32;
        let physical_len = self.to_len() as u32;

        // println!("\nCORTICAL_DIMS: per_subgrp: max_wg_size: {}, physical_len: {}",
        //     ocl_pq.max_wg_size(), physical_len);

        if physical_len % subgroup_count == 0 {
            Ok(physical_len / subgroup_count)
        } else {
            // Err(format!("Invalid subgroup size: {} % {} = {}", physical_len, subgroup_count,
            //     physical_len % subgroup_count).into())
            panic!("Invalid subgroup size: {} % {} = {}", physical_len, subgroup_count,
                physical_len % subgroup_count);
        }
    }

    // #[inline]
    // pub fn clone_with_ptl2(&self, per_tft_l2: i8) -> CorticalDims {
    //     CorticalDims { per_tft_l2: per_tft_l2, .. *self }
    // }

    #[inline]
    pub fn clone_with_depth(&self, depth: SlcId) -> CorticalDims {
        CorticalDims { depth: depth, .. *self }
    }

    // #[inline]
    // #[deprecated]
    // pub fn clone_with_incr(&self, incr: usize) -> CorticalDims {
    //     CorticalDims { incr: Some(incr as u32), .. *self }
    // }

    // #[inline]
    // #[deprecated]
    // pub fn set_incr(&mut self, incr: usize) {
    //     self.incr = Some(incr as u32);
    // }

    // #[inline]
    // #[deprecated]
    // pub fn with_incr(mut self, incr: usize) -> CorticalDims {
    //     #[allow(deprecated)]
    //     self.set_incr(incr);
    //     self
    // }

    // #[inline]
    // pub fn with_tfts(mut self, tfts_per_cel: u32) -> CorticalDims {
    //     self.tfts_per_cel = tfts_per_cel;
    //     self
    // }

    pub fn to_len(&self) -> usize {
        // len_components(self.v_size * self.u_size * self.depth as u32,
        //     self.per_tft_l2, self.tfts_per_cel) as usize
        // (self.v_size * self.u_size * self.depth as u32) as usize
        self.cells() as usize
    }

    /// Length of the buffer required to properly represent this section of cortex.
    ///
    /// Rounded based on columns for versatility's sake.
    pub fn to_len_padded(&self, incr: usize) -> usize {
        let cols = self.columns();
        // let phys_incr = ocl_pq.max_wg_size();

        let len_mod = cols % incr as u32;

        if len_mod == 0 {
            self.to_len()
        } else {
            let pad = incr as u32 - len_mod;
            debug_assert_eq!((cols + pad) % incr as u32, 0);
            // len_components((cols + pad) * self.depth as u32, self.per_tft_l2, self.tfts_per_cel) as usize
            ((cols + pad) * self.depth as u32) as usize
        }
    }

    /// Returns `true` if the the `v_size`, `u_size` and `depth` of `at_least`
    /// are less than or equal to the dimensions of this `CorticalDims`.
    pub fn are_at_least(&self, at_least: &CorticalDims) -> bool {
        at_least.v_size <= self.v_size &&
            at_least.u_size <= self.u_size &&
            at_least.depth <= self.depth
    }
}

impl Copy for CorticalDims {}

impl ParaHexArray for CorticalDims {
    #[inline]
    fn v_size(&self) -> u32 {
        self.v_size
    }

    #[inline]
    fn u_size(&self) -> u32 {
        self.u_size
    }

    #[inline]
    fn depth(&self) -> SlcId {
        self.depth
    }
}

impl MemLen for CorticalDims {
    fn to_len(&self) -> usize {
        self.to_len()
    }

    fn to_len_padded(&self, incr: usize) -> usize {
        self.to_len_padded(incr) as usize
    }

    fn to_lens(&self) -> [usize; 3] {
        [self.to_len(), 1, 1]
    }
}

impl PartialEq<TractDims> for CorticalDims {
    fn eq(&self, other: &TractDims) -> bool {
        self.v_size == other.v_size() &&
            self.u_size == other.u_size() &&
            self.depth() == other.depth()
    }
}

impl Into<SpatialDims> for CorticalDims {
    fn into(self) -> SpatialDims {
        self.to_lens().into()
    }
}

impl<'a> Into<SpatialDims> for &'a CorticalDims {
    fn into(self) -> SpatialDims {
        self.to_lens().into()
    }
}

// impl From<(u32, u32, SlcId)> for CorticalDims {
//     fn from(tuple: (u32, u32, SlcId)) -> CorticalDims {
//         CorticalDims::new(tuple.0, tuple.1, tuple.2)
//     }
// }

// [KEEPME]: DO NOT IMPLEMENT THIS:
// impl Into<SpatialDims> for CorticalDims {
//     fn into(self) -> SpatialDims {
//         SpatialDims::Three(self.depth as usize, self.v_size as usize, self.u_size as usize)
//     }
// }



// #[cfg(any(test, feature = "eval"))]
// pub mod tests {
//     use super::*;

//     #[test]
//     fn len() {

//         // Actually test the damn thing...

//     }
// }

