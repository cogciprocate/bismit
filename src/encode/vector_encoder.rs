
// use cmn::ScalarEncodable;
use thalamus::{ExternalPathwayTract, ExternalPathwayFrame, TractFrameMut, LayerTags};

#[derive(Clone, Debug)]
pub struct VectorEncoder {
    ranges: Vec<(f32, f32)>,
    values: Vec<f32>,
}

impl VectorEncoder {
    pub fn new(ranges: Vec<(f32, f32)>) -> VectorEncoder {
        VectorEncoder {
            ranges: ranges,
            values: vec![Default::default(); 16],
        }
    }

    pub fn ext_frame_mut(&mut self) -> ExternalPathwayFrame {
        ExternalPathwayFrame::F32Slice16(&mut self.values[0..16])
    }
}

impl ExternalPathwayTract for VectorEncoder {
    fn write_into(&mut self, tract_frame: &mut TractFrameMut, _: LayerTags) -> [usize; 3] {
        // super::encode_scalar(Default::default(), self.range, tract_frame);
        // println!("Vector encoder frame: {:?}", tract_frame);

        Default::default()
    }

    fn cycle_next(&mut self) {
        // self.increment_frame();
    }
}

