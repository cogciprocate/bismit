
// use cmn::ScalarEncodable;
use cmn::{CmnError, CmnResult, TractDims};
use thalamus::{ExternalPathwayTract, ExternalPathwayFrame, TractFrameMut, LayerTags};
use encode::{ScalarGlyphWriter};

#[derive(Clone, Debug)]
pub struct VectorEncoder {
    ranges: Vec<(f32, f32)>,
    values: Vec<f32>,
    layer_tags: Vec<LayerTags>,
    writers: Vec<ScalarGlyphWriter<f32>>,
}

impl VectorEncoder {
    pub fn new(ranges: Vec<(f32, f32)>, layers: &[LayerTags], tract_dims: &[TractDims])
                -> CmnResult<VectorEncoder> {
        if ranges.len() != layers.len() || ranges.len() != tract_dims.len() {
            return CmnError::err(format!("VectorEncoder::new(): Range list length ('{}'), \
                layer count ('{}'), and/or tract count ('{}') are not equal.", ranges.len(),
                layers.len(), tract_dims.len()));
        }

        let mut writers = Vec::with_capacity(ranges.len());

        for (r, ref td) in ranges.iter().zip(tract_dims) {
            let writer = ScalarGlyphWriter::new(r.clone(), td);

            writers.push(writer);
        }

        // ScalarGlyphWriter::new(range.clone(), tract_dims);

        Ok(VectorEncoder {
            ranges: ranges,
            values: vec![Default::default(); layers.len()],
            layer_tags: Vec::from(layers),
            writers: writers,
        })
    }

    pub fn ext_frame_mut(&mut self) -> ExternalPathwayFrame {
        ExternalPathwayFrame::F32Slice(&mut self.values[..])
    }

    pub fn set_ranges(&mut self, new_ranges: &[(f32, f32)]) -> CmnResult<()> {
        if new_ranges.len() != self.ranges.len() {
            return CmnError::err(format!("VectorEncoder::set_ranges(): Incorrect number of ranges
                provided ('{}'/'{}').", new_ranges.len(), self.ranges.len()));
        }

        for (sr, nr) in self.ranges.iter_mut().zip(new_ranges.iter()) {
            *sr = *nr;
        }

        Ok(())
    }
}

impl ExternalPathwayTract for VectorEncoder {
    fn write_into(&mut self, tract_frame: &mut TractFrameMut, tags: LayerTags) {
        let l_idx = self.layer_tags.iter().position(|&t| t == tags)
            .expect(&format!("VectorEncoder::write_into(): No layers matching tags: {}", tags));

        // println!("Vector encoder: encoding value: {}...", self.values[l_idx]);
        // super::encode_scalar(self.values[l_idx], self.ranges[l_idx], tract_frame);
        self.writers[l_idx].encode(self.values[l_idx], tract_frame);

        // Default::default()
    }

    fn cycle_next(&mut self) {
        // self.increment_frame();
    }
}

