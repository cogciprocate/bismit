use cmn::{TractFrameMut, TractDims};
// use encode::ScalarEncodable;

#[derive(Clone, Debug)]
pub struct Vector2dWriter {
    raw_offs: f32,
    raw_scale: f32,
    // dim_count: u32,
    scale_level_count: u32,
    precision_redundancy: u32,
    scale_level_mid: u32,
}

impl Vector2dWriter {
    pub fn new(tract_dims: TractDims) -> Vector2dWriter {
        assert!(tract_dims.u_size() >= 7, "Vector2dWriter::new: 'u' dimension too small.");
        let scale_level_count = tract_dims.v_size() / 7;
        let precision_redundancy = tract_dims.u_size();

        let scale_level_mid = (scale_level_count / 2) + (scale_level_count % 2);

        Vector2dWriter {
            raw_offs: 0.,
            raw_scale: 1.,
            // dim_count: 2,
            scale_level_count,
            precision_redundancy,
            scale_level_mid,
        }
    }

    // fn xform(&self, raw: T) -> f32 {
    //     (raw.to_f32().unwrap() + self.raw_offs) * self.raw_scale
    // }

    #[inline]
    fn xform(&self, raw: f32) -> f32 {
        (raw + self.raw_offs) * self.raw_scale
    }

    fn decompose(&self, a_raw: [f32; 2]) -> [f32; 6] {
        let a = [self.xform(a_raw[0]),
            self.xform(a_raw[1])];



        let
    }

    pub fn encode(&mut self, a_raw: [f32; 2], tract: &mut TractFrameMut) {
        // let a = [self.xform(a_raw[0]),
        //     self.xform(a_raw[1])];

        let a = decompose(a_raw);


        for scale_level_idx in 0..self.scale_level_count {
            let unit = if scale_level_idx < self.scale_level_mid {
                0.
            } else if scale_level_idx > self.scale_level_mid {
                0.
            } else {
                1.0f32
            };
        }

    }
}