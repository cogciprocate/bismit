//! Mostly for testing purposes.

use external_source::{ExternalSourceTract, TractFrameMut, LayerTags};

#[derive(Clone, Debug)]
pub struct ScalarSequence {
    range: (f32, f32),
    next: f32,
    incr: f32,
}

impl ScalarSequence {
    pub fn new() -> ScalarSequence {
        ScalarSequence {
            range: (0.0, 30000.0),
            next: 0.0,
            incr: 1.0,
        }
    }

    pub fn increment_frame(&mut self) {
        self.next += self.incr;
        if self.next >= self.range.1 { self.next = 0.0; }
    }
}

impl ExternalSourceTract for ScalarSequence {
    fn write_into(&mut self, tract_frame: &mut TractFrameMut, _: LayerTags) -> [usize; 3] {
        super::encode_scalar(self.next, self.range, tract_frame);

        Default::default()
    }

    fn cycle_next(&mut self) {
        self.increment_frame();
    }
}

