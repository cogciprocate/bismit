
// use cmn::ScalarEncodable;
use cmn::{CmnError, CmnResult, TractDims};
use map::LayerAddress;
use thalamus::{ExternalPathwayTract, ExternalPathwayFrame, TractFrameMut};
use encode::{ScalarGlyphWriter};

// [TODO]: Convert into a multi-layer/multi-slice system. Plumbing should be in place.
//
//
#[derive(Clone, Debug)]
pub struct VectorEncoder {
    ranges: Vec<(f32, f32)>,
    values: Vec<f32>,
    // layer_tags: Vec<LayerTags>,
    layer_addrs: Vec<LayerAddress>,
    tract_dims: Vec<TractDims>,
    writers: Vec<ScalarGlyphWriter<f32>>,
}

impl VectorEncoder {
    pub fn new(ranges: Vec<(f32, f32)>, layer_addrs: &[LayerAddress], tract_dims: &[TractDims])
                -> CmnResult<VectorEncoder> {
        if ranges.len() != layer_addrs.len() || ranges.len() != tract_dims.len() {
            return CmnError::err(format!("VectorEncoder::new(): Range list length ('{}'), \
                layer count ('{}'), and/or tract count ('{}') are not equal.", ranges.len(),
                layer_addrs.len(), tract_dims.len()));
        }

        let mut writers = Vec::with_capacity(ranges.len());

        for (r, ref td) in ranges.iter().zip(tract_dims) {
            writers.push(ScalarGlyphWriter::new(r.clone(), td));
        }

        // ScalarGlyphWriter::new(range.clone(), tract_dims);

        Ok(VectorEncoder {
            ranges: ranges,
            values: vec![Default::default(); layer_addrs.len()],
            layer_addrs: Vec::from(layer_addrs),
            tract_dims: Vec::from(tract_dims),
            writers: writers,
        })
    }

    pub fn ext_frame_mut(&mut self) -> ExternalPathwayFrame {
        ExternalPathwayFrame::F32Slice(&mut self.values[..])
    }

    /// Resets the ranges and number of scalars this encoder will encode.
    pub fn set_ranges(&mut self, new_ranges: &[(f32, f32)]) -> CmnResult<()> {
        // if new_ranges.len() != self.ranges.len() {
        //     return CmnError::err(format!("VectorEncoder::set_ranges(): Incorrect number of ranges
        //         provided ('{}'/'{}').", new_ranges.len(), self.ranges.len()));
        // }
        if new_ranges.len() > self.tract_dims.len() {
            return CmnError::err(format!("VectorEncoder::set_ranges(): Too many ranges
                provided ('{}'/'{}').", new_ranges.len(), self.tract_dims.len()));
        }

        self.ranges.clear();

        for nr in new_ranges.iter() {
            self.ranges.push(*nr);
        }

        self.writers.clear();

        for (r, td) in self.ranges.iter().zip(self.tract_dims.iter()) {
            self.writers.push(ScalarGlyphWriter::new(r.clone(), td));
        }

        self.values = vec![0.0; self.ranges.len()];

        // println!("VectorEncoder::set_ranges(): Ranges now set to: {:?}", self.ranges);

        Ok(())
    }
}

impl ExternalPathwayTract for VectorEncoder {
    fn write_into(&mut self, tract_frame: &mut TractFrameMut, addr: &LayerAddress) {
        let l_idx = self.layer_addrs.iter().position(|t| t == addr)
            .expect(&format!("VectorEncoder::write_into(): No layers with address: {:?}", addr));

        // println!("Vector encoder: encoding value: {}...", self.values[l_idx]);
        // super::encode_scalar(self.values[l_idx], self.ranges[l_idx], tract_frame);
        match self.writers.get(l_idx) {
            Some(w) => w.encode(self.values[l_idx], tract_frame),
            None => (),
        }

        // Default::default()
    }

    fn cycle_next(&mut self) {
        // self.increment_frame();
    }
}

