use cmn;
use thalamus::{ExternalPathwayTract, LayerTags, TractFrameMut};
use encode;


#[derive(Debug, Clone)]
pub struct HexMoldTest {
    radius: i8,
    src_dims: (u32, u32),
    // src_dims:
}

impl HexMoldTest {
    pub fn new(radius: i8, src_dims: (u32, u32)) -> HexMoldTest {
        assert!(radius >= 0);

        HexMoldTest {
            radius: radius,
            src_dims: src_dims,
        }
    }
}


impl ExternalPathwayTract for HexMoldTest {
    fn write_into(&mut self, tract_frame: &mut TractFrameMut, _: LayerTags) {
        // assert!((self.radius as u32 * 2) + 1 <= tract_frame.dims().v_size() &&
        //     (self.radius as u32 * 2) + 1 <= tract_frame.dims().v_size(),
        //     format!("Radius too big for it's britches (radius: {}, dims: {:?})",
        //         self.radius, tract_frame.dims()));

        let scales = [cmn::calc_scale(tract_frame.dims().v_size(), self.src_dims.0).unwrap(),
            cmn::calc_scale(tract_frame.dims().u_size(), self.src_dims.1).unwrap()];

        let src_dims = [self.src_dims.0, self.src_dims.1];
        let dst_dims = [tract_frame.dims().v_size(), tract_frame.dims().u_size()];
        let dst_mid = [tract_frame.dims().v_size() / 2, tract_frame.dims().u_size() / 2];

        // let radii = [cmn::scale(self.radius as i32, scales[0]),
        //     cmn::scale(self.radius as i32, scales[1])];

        // println!("###### scales: {:?}, radii: {:?}", scales, radii);


        // encode::encode_hex_mold_scaled(self.radius, scales, mid, tract_frame);
        encode::encode_hex_mold_scaled(self.radius, src_dims, dst_dims, dst_mid, tract_frame);
    }

    fn cycle_next(&mut self) {
        // self.increment_frame();
    }
}
