//! Mostly for testing purposes.

use cmn::{TractDims, TractFrameMut};
use map::LayerAddress;
use ::{ExternalPathwayTract};
use encode::{ScalarEncodable, ScalarSdrWriter};

#[derive(Clone, Debug)]
pub struct ScalarSdrGradiant<T> {
    range: (T, T),
    next: T,
    incr: T,
    writer: ScalarSdrWriter<T>,
}

impl<T> ScalarSdrGradiant<T> where T: ScalarEncodable {
    pub fn new(range: (T, T), way_span: T, incr: T, tract_dims: &TractDims) -> ScalarSdrGradiant<T> {
        let next = range.0;

        let writer = ScalarSdrWriter::new(range.clone(), way_span, tract_dims);

        ScalarSdrGradiant {
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

impl<T> ExternalPathwayTract for ScalarSdrGradiant<T>
            where T: ScalarEncodable {
    fn write_into(&mut self, tract_frame: &mut TractFrameMut, _: LayerAddress) {
        self.writer.encode(self.next, tract_frame);
    }

    fn cycle_next(&mut self) {
        self.increment_frame();
    }
}

