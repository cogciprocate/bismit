
use ocl::{ self, Result as OclResult };
use ocl::traits::MemDims;
use cmn::{ self, ParaHexArray, CorticalDims, CmnResult, CmnError };
use proto::{ AxonKind };


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
                        if src_area_dims.v_size() > 252 || src_area_dims.u_size() > 252 { 
                            return Err(CmnError::from("Dimensions size for horizontal layers may \
                                not exceed 252."));
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
                        let side = cmn::DEFAULT_HORIZONTAL_SLICE_SIDE;
                        assert!(side <= 255);
                        let mid = side / 2;

                        Ok(SliceDims { 
                            v_size: side,
                            u_size: side,
                            v_scale: 0,
                            u_scale: 0,
                            v_mid: mid,
                            u_mid: mid,
                        })
                    }
                }
            }
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

impl MemDims for SliceDims {
    // [FIXME]: TODO: ROUND CORTICAL_LEN() UP TO THE NEXT PHYSICAL_INCREMENT
    #[inline]
    fn padded_buffer_len(&self, incr: usize) -> OclResult<usize> {
        Ok(ocl::util::padded_len(self.columns() as usize, incr))
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

    let v_res = match calc_scale(src_area_dims.v_size(), tar_area_dims.v_size()) {
        Ok(vr) => vr,
        Err(err) => return Err(err),
    };

    let u_res = match calc_scale(src_area_dims.u_size(), tar_area_dims.u_size()) {
        Ok(vr) => vr,
        Err(err) => return Err(err),
    };

    Ok((v_res, u_res))
}

#[inline]
pub fn calc_scale(src_dim: u32, tar_dim: u32) -> CmnResult<u32> {
    // let scale_incr = if src_dim >= 16 { src_dim / 16 } 
    //     else if src_dim > 0 { 1 }
    //     else { panic!("area_map::calc_scale(): Source dimension cannot be zero.") };

    let src_dim = (src_dim as usize) * 1024;
    let tar_dim = (tar_dim as usize) * 1024;

    let scale_incr = match tar_dim {
        0 => return Err(CmnError::new("Target area dimension cannot be zero.".to_string())),
        1...15 => 1,
        _ => tar_dim / 16,
    };

    return match src_dim / scale_incr {
        0 => return Err(CmnError::new("Source area dimension cannot be zero.".to_string())),
        s @ 1...255 => Ok(s as u32),
        _ => return Err(CmnError::new("Source area cannot have a dimension more than 16 times \
            target area dimension.".to_string())),
    }
}
