use ocl::SpatialDims;
use ocl::traits::MemLen;
use cmn::{ParaHexArray, TractDims, CmnResult};

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
    v_size: u32, // in cell-edges (log2) (HxWxD: 1x1xN)
    u_size: u32, // in cell-edges (log2) (HxWxD: 1x1xN)
    depth: u8, // in cell-edges (NxMx1)
    // tfts_per_cel: u32, // dendritic tufts per cell
    // per_tft_l2: i8, // divisions per cell-tuft (log2)
    incr: Option<u32>,
}

impl CorticalDims {
    pub fn new(v_size: u32, u_size: u32, depth: u8, /*per_tft_l2: i8,*/ incr: Option<u32>) -> CorticalDims {
        //assert!(super::OPENCL_PREFERRED_VECTOR_MULTIPLE == 4);
        //println!("\n\n##### v_size: {}, u_size: {}", v_size, u_size);
        //let incr = resolve_incr(ocl);
        //assert!(v_size % 4 == 0, "CorticalDims::new(): Size of dimension 'v' must be a multiple of 4.");
        //assert!(u_size % 4 == 0, "CorticalDims::new(): Size of dimension 'u' must be a multiple of 4.");

        CorticalDims {
            v_size: v_size,
            u_size: u_size,
            /*u_size_l2: u_size_l2,
            v_size_l2: v_size_l2,*/
            depth: depth,
            // tfts_per_cel: 1,
            // per_tft_l2: per_tft_l2,
            incr: incr, // <<<<< PENDING RENAME
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

    // PHYSICAL_INCREMENT():
    //         TODO: improve this description
    #[inline]
    pub fn incr(&self) -> CmnResult<u32> {
        match self.incr {
            Some(pi) => Ok(pi),
            None => Err("CorticalDims::incr: Physical increment not set.".into()),
        }
    }

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
    pub fn v_size(&self) -> u32 {
        self.v_size
    }

    #[inline]
    pub fn u_size(&self) -> u32 {
        self.u_size
    }

    #[inline]
    pub fn depth(&self) -> u8 {
        self.depth
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
            Err(format!("Invalid subgroup size: {} % {} = {}", physical_len, subgroup_count,
                physical_len % subgroup_count).into())
        }
    }

    // #[inline]
    // pub fn clone_with_ptl2(&self, per_tft_l2: i8) -> CorticalDims {
    //     CorticalDims { per_tft_l2: per_tft_l2, .. *self }
    // }

    #[inline]
    pub fn clone_with_depth(&self, depth: u8) -> CorticalDims {
        CorticalDims { depth: depth, .. *self }
    }

    #[inline]
    pub fn clone_with_incr(&self, incr: usize) -> CorticalDims {
        CorticalDims { incr: Some(incr as u32), .. *self }
    }

    #[inline]
    pub fn set_incr(&mut self, incr: usize) {
        self.incr = Some(incr as u32);
    }

    #[inline]
    pub fn with_incr(mut self, incr: usize) -> CorticalDims {
        self.set_incr(incr);
        self
    }

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
    fn depth(&self) -> u8 {
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

// [KEEPME]: DO NOT IMPLEMENT THIS:
// impl Into<SpatialDims> for CorticalDims {
//     fn into(self) -> SpatialDims {
//         SpatialDims::Three(self.depth as usize, self.v_size as usize, self.u_size as usize)
//     }
// }


// fn resolve_incr(ocl: Option<&ProQue>) -> Option<u32> {
//     match ocl {
//         Some(ocl) => Some(ocl.max_wg_size()),
//         None => None,
//     }
// }
// #[inline]
// fn len_components(cells: u32, per_tft_l2: i8, tfts_per_cel: u32) -> u32 {
//     //println!("\n\n##### TOTAL_LEN(): cells: {}, pcl2: {}", cells, per_tft_l2);
//     let tufts = cells * tfts_per_cel;

//     if per_tft_l2 >= 0 {
//         tufts << per_tft_l2
//     } else {
//         tufts >> (0 - per_tft_l2)
//     }
// }


    /*pub fn u_size_l2(&self) -> u8 {
        self.u_size_l2
    }

    pub fn v_size_l2(&self) -> u8 {
        self.v_size_l2
    }

    pub fn u_size(&self) -> u32 {
        1 << self.u_size_l2 as u32
    }

    pub fn v_size(&self) -> u32 {
        1 << self.v_size_l2 as u32
    }
    */

// #[cfg(test)]
// pub mod tests {
//     use super::*;

//     #[test]
//     fn len() {

//         // Actually test the damn thing...

//     }
// }



    // LEN(): 4D Volume - Total linear length if stretched out - measured in cell-piece-whatevers
    /* TEMPORARY */
    /*pub fn len(&self) -> u32 {
        self.len()
    }*/

    /* CORTICAL_LEN(): 'VIRTUAL' CORTEX SIZE, NOT TO BE CONFUSED WITH THE PHYSICAL IN-MEMORY SIZE */
    /*fn cortical_len(&self) -> u32 {
        len_components(self.cells(), self.per_tuft_l2)
    }*/
