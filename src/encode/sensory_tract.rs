use cmn::{TractFrameMut, TractDims};
use map::LayerTags;
use thalamus::{ExternalPathwayTract, ExternalPathwayFrame};

#[derive(Debug)]
pub struct SensoryTract {
    buf: Vec<u8>,
    dims: TractDims,
}

impl SensoryTract {
    // /// Generates a number of SensoryTracts.
    // pub fn gen(dim_list: Vec<(u32, u32)>) -> Vec<SensoryTract> {
    //     dim_list.iter().map(|dims| SensoryTract::new(TractDims::from(dims.clone()))).collect()
    // }

    pub fn new<T: Into<TractDims>>(dims: T) -> SensoryTract {
        let dims = dims.into();
        SensoryTract {
            buf: vec![0; dims.to_len()],
            dims: dims,
        }
    }

    pub fn ext_frame_mut(&mut self) -> ExternalPathwayFrame {
        ExternalPathwayFrame::Tract(TractFrameMut::new(&mut self.buf[..], self.dims.clone()))
    }
}

impl ExternalPathwayTract for SensoryTract {
    fn write_into(&mut self, tract_frame: &mut TractFrameMut, _: LayerTags) {
        assert!(tract_frame.dims() == &self.dims);
        tract_frame.frame_mut().clone_from_slice(&self.buf[..]);
        // [0, 0, 0]
    }

    fn cycle_next(&mut self) {

    }
}
