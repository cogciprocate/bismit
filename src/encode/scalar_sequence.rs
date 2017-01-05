//! Mostly for testing purposes.

use cmn::TractDims;
use map::LayerAddress;
use thalamus::{ExternalPathwayTract, TractFrameMut};
use encode::{ScalarEncodable, ScalarGlyphWriter};

#[derive(Clone, Debug)]
pub struct ScalarSequence<T> {
    range: (T, T),
    next: T,
    incr: T,
    writer: ScalarGlyphWriter<T>,
}

impl<T> ScalarSequence<T> where T: ScalarEncodable {
    pub fn new(range: (T, T), incr: T, tract_dims: &TractDims) -> ScalarSequence<T> {
        let next = range.0;

        let writer = ScalarGlyphWriter::new(range.clone(), tract_dims);

        ScalarSequence {
            range: range,
            incr: incr,
            next: next,
            writer: writer,
        }
    }

    pub fn increment_frame(&mut self) {
        self.next += self.incr;
        if self.next >= self.range.1 { self.next = self.range.0; }
    }
}

impl<T> ExternalPathwayTract for ScalarSequence<T>
            where T: ScalarEncodable {
    fn write_into(&mut self, tract_frame: &mut TractFrameMut, _: &LayerAddress) {
        // super::encode_scalar(self.next, self.range, tract_frame);
        self.writer.encode(self.next, tract_frame);

        // Default::default()
    }

    fn cycle_next(&mut self) {
        self.increment_frame();
    }
}

