use cmn::{TractFrameMut, TractDims};
use map::LayerAddress;
use ::{ExternalPathwayTract};
use encode;


#[derive(Debug, Clone)]
pub struct HexMoldTest {
    radius: i8,
    src_dims: [u32; 2],
    // src_dims:
    hex_mold: Vec<u8>,
}

impl HexMoldTest {
    pub fn new<D: Into<TractDims>>(radius: i8, src_dims: [u32; 2], dst_dims: D) -> HexMoldTest {
        assert!(radius >= 0);
        let dst_dims = dst_dims.into();
        let mut hex_mold = vec![0u8; dst_dims.to_len()];
        {
            let mut tract_frame = TractFrameMut::new(hex_mold.as_mut_slice(), 
                dst_dims);
            encode::encode_hex_mold_scaled(radius, src_dims, &mut tract_frame);
        }

        HexMoldTest {
            radius,
            src_dims,
            hex_mold,
        }
    }
}


impl ExternalPathwayTract for HexMoldTest {
    fn write_into(&mut self, tract_frame: &mut TractFrameMut, _: LayerAddress) {
        // assert!((self.radius as u32 * 2) + 1 <= tract_frame.dims().v_size() &&
        //     (self.radius as u32 * 2) + 1 <= tract_frame.dims().v_size(),
        //     format!("Radius too big for it's britches (radius: {}, dims: {:?})",
        //         self.radius, tract_frame.dims()));

        // let scales = [cmn::calc_scale(tract_frame.dims().v_size(), self.src_dims[0]).unwrap(),
        //     cmn::calc_scale(tract_frame.dims().u_size(), self.src_dims[1]).unwrap()];

        // let src_dims = [self.src_dims.0, self.src_dims.1];
        // let dst_dims = [tract_frame.dims().v_size(), tract_frame.dims().u_size()];
        // let dst_mid = [tract_frame.dims().v_size() / 2, tract_frame.dims().u_size() / 2];

        // let radii = [cmn::scale(self.radius as i32, scales[0]),
        //     cmn::scale(self.radius as i32, scales[1])];

        // println!("###### scales: {:?}, radii: {:?}", scales, radii);


        // encode::encode_hex_mold_scaled(self.radius, scales, mid, tract_frame);
        // encode::encode_hex_mold_scaled(self.radius, self.src_dims, tract_frame);
        let len = self.hex_mold.len();
        assert_eq!(tract_frame.dims().to_len(), len);
        unsafe {
            ::std::ptr::copy_nonoverlapping(self.hex_mold.as_ptr(), 
                tract_frame.frame_mut().as_mut_ptr(), len);
        }
    }

    fn cycle_next(&mut self) {
        // self.increment_frame();
    }
}
