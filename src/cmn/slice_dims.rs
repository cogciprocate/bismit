
use ocl;
use ocl::traits::MemLen;
use cmn::{self, ParaHexArray, CorticalDims, CmnResult, CmnError};
use map::{AxonKind};


#[derive(Clone, Debug)]
pub struct SliceDims {
    v_size: u32,
    u_size: u32,
    v_scale: u32,
    u_scale: u32,
    v_mid: u32,
    u_mid: u32,
}

impl SliceDims {
    pub fn new(area_dims: &CorticalDims, src_lyr_dims_opt: Option<&CorticalDims>,
                axn_kind: AxonKind) -> CmnResult<SliceDims>
    {
        match axn_kind {
            AxonKind::Spatial | AxonKind::None => {
                match src_lyr_dims_opt {
                    Some(src_area_dims) => {
                        let src_scales_res = get_src_scales(src_area_dims, area_dims);

                        match src_scales_res {
                            Ok(ss) => {
                                let (v_scale, u_scale) = ss;

                                Ok(SliceDims {
                                    v_size: src_area_dims.v_size(),
                                    u_size: src_area_dims.u_size(),
                                    v_scale: v_scale,
                                    u_scale: u_scale,
                                    v_mid: 0,
                                    u_mid: 0,
                                })
                            },

                            Err(err) => Err(err),
                        }
                    },

                    None => {
                        Ok(SliceDims {
                            v_size: area_dims.v_size(),
                            u_size: area_dims.u_size(),
                            v_scale: 16,
                            u_scale: 16,
                            v_mid: 0,
                            u_mid: 0,
                        })
                    },
                }
            },

            AxonKind::Horizontal => {
                match src_lyr_dims_opt {
                    Some(src_area_dims) => {
                        if src_area_dims.v_size() > cmn::MAX_HRZ_DIM_SIZE ||
                                    src_area_dims.u_size() > cmn::MAX_HRZ_DIM_SIZE {
                            // [NOTE]: Can't remember why I set this to 252
                            // but I doubt there's a good reason why it can't
                            // be 255. [UPDATE]: Now set to 255, should be cool.
                            return Err(CmnError::from(format!("Dimensions size for horizontal layers may \
                                not exceed {}.", cmn::MAX_HRZ_DIM_SIZE)));
                        }

                        Ok(SliceDims {
                            v_size: src_area_dims.v_size(),
                            u_size: src_area_dims.u_size(),
                            v_scale: 0,
                            u_scale: 0,
                            v_mid: src_area_dims.v_size() / 2,
                            u_mid: src_area_dims.u_size() / 2,
                        })
                    },

                    None => {
                        // let side = cmn::DEFAULT_HORIZONTAL_SLICE_SIDE;
                        // assert!(side <= 255);
                        // let mid = side / 2;

                        Ok(SliceDims {
                            v_size: area_dims.v_size(),
                            u_size: area_dims.u_size(),
                            v_scale: 0,
                            u_scale: 0,
                            v_mid: area_dims.v_size() / 2,
                            u_mid: area_dims.u_size() / 2,
                        })
                    }
                }
            }
        }
    }

    /// Scales `idxs` (v, u) by the appropriate amount for this slice. This is
    /// precisely the same calculation done within a kernel for indexing.
    ///
    /// [OPEN QUESTION]: What will we do about out of range offsets?
    /// * continue to panic
    /// * clamp to min/max (-128, 127)
    /// * sparsify synapse source addresses (put gaps but maintain relative reach)
    ///
    pub fn scale_offs(&self, offs: (i8, i8)) -> CmnResult<(i8, i8)> {
        let sc_l2 = cmn::SLC_SCL_COEFF_L2;
        let v_off = (offs.0 as i32 * self.v_scale as i32) >> sc_l2;
        let u_off = (offs.1 as i32 * self.u_scale as i32) >> sc_l2;
        let r_min = cmn::SYNAPSE_REACH_MIN as i32;
        let r_max = cmn::SYNAPSE_REACH_MAX as i32;

        if v_off < r_min || v_off > r_max || u_off < r_min || u_off > r_max {
            CmnError::err(format!("Offsets ({}, {}) out of range for source layer \
                [scales: ({}, {}), results: ({}, {})]. [DEV NOTE]: THIS MAY BE BETTER \
                IF IT CLAMPS TO MAX. See 'SliceDims::scale_offs' doc for more.",
                offs.0, offs.1, self.v_scale, self.u_scale, v_off, u_off))
        } else {
            Ok((v_off as i8, u_off as i8))
        }
    }

    #[inline]
    pub fn v_size(&self) -> u32 {
        self.v_size
    }

    #[inline]
    pub fn u_size(&self) -> u32 {
        self.u_size
    }

    #[inline]
    pub fn v_scale(&self) -> u32 {
        self.v_scale
    }

    #[inline]
    pub fn u_scale(&self) -> u32 {
        self.u_scale
    }

    #[inline]
    pub fn v_mid(&self) -> u32 {
        self.v_mid
    }

    #[inline]
    pub fn u_mid(&self) -> u32 {
        self.u_mid
    }

    #[inline]
    pub fn columns(&self) -> u32 {
        self.v_size * self.u_size
    }

    // pub fn depth(&self) -> u8 {
    //     1u8
    // }
}

impl ParaHexArray for SliceDims {
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

impl MemLen for SliceDims {
    // [FIXME]: TODO: ROUND CORTICAL_LEN() UP TO THE NEXT PHYSICAL_INCREMENT
    #[inline]
    fn to_len(&self) -> usize {
        self.columns() as usize
    }

    fn to_len_padded(&self, incr: usize) -> usize {
        ocl::util::padded_len(self.columns() as usize, incr)
    }

    fn to_lens(&self) -> [usize; 3] {
        [1, self.v_size as usize, self.u_size as usize]
    }
}

#[inline]
pub fn get_src_scales(src_area_dims: &CorticalDims, tar_area_dims: &CorticalDims,
        ) -> CmnResult<(u32, u32)>
{
    // let v_res = calc_scale(src_area_dims.v_size(), tar_area_dims.v_size());
    // let u_res = calc_scale(src_area_dims.u_size(), tar_area_dims.u_size());

    // if v_res.is_err() || u_res.is_err() {
    //     let mut err_str = String::new();

    //     match &v_res {
    //         &Err(e) => err_str.push_str(&format!("v_size: {}. ", e)),
    //         _ => (),
    //     }

    //     match &u_res {
    //         &Err(e) => err_str.push_str(&format!("u_size: {}. ", e)),
    //         _ => (),
    //     }

    //     Err(err_str)
    // } else {
    //     Ok((v_res.unwrap(), u_res.unwrap()))
    // }

    let v_res = try!(calc_scale(src_area_dims.v_size(), tar_area_dims.v_size()));
    let u_res = try!(calc_scale(src_area_dims.u_size(), tar_area_dims.u_size()));

    Ok((v_res, u_res))
}

#[inline]
pub fn calc_scale(src_dim: u32, tar_dim: u32) -> CmnResult<u32> {
    // let scale_incr = if src_dim >= 16 { src_dim / 16 }
    //     else if src_dim > 0 { 1 }
    //     else { panic!("area_map::calc_scale(): Source dimension cannot be zero.") };

    let arbitrary_coeff = 1024;
    const SRC_SCL_COEFF_M1: usize = cmn::SLC_SCL_COEFF - 1;

    let src_dim = (src_dim as usize) * arbitrary_coeff;
    let tar_dim = (tar_dim as usize) * arbitrary_coeff;

    let scale_incr = match tar_dim {
        0 => return Err(CmnError::new("Target area dimension cannot be zero.".to_owned())),
        1...SRC_SCL_COEFF_M1 => 1,
        _ => tar_dim / cmn::SLC_SCL_COEFF,
    };

    return match src_dim / scale_incr {
        0 => return Err(CmnError::new("Source area dimension cannot be zero.".to_owned())),
        s @ 1...255 => Ok(s as u32),
        _ => return Err(CmnError::new("Source area cannot have a dimension more than 16 times \
            target area dimension.".to_owned())),
    }
}


#[cfg(test)]
mod tests {
    #[test]
    fn slc_dim_scl_offs_unimplemented() {

    }
}