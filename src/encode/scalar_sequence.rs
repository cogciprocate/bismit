//! Mostly for testing purposes.

use std::fmt::Debug;
use std::ops::AddAssign;
use num::{Num, NumCast};
use thalamus::{ExternalPathwayTract, TractFrameMut, LayerTags};

#[derive(Clone, Debug)]
pub struct ScalarSequence<T> {
    range: (T, T),
    next: T,
    incr: T,
}

impl<T> ScalarSequence<T> where T: Num + NumCast + PartialOrd + Debug + Clone + AddAssign + Copy {
    pub fn new(range: (T, T), incr: T) -> ScalarSequence<T> {
        let next = range.0;

        ScalarSequence {
            range: range,
            incr: incr,
            next: next,
        }
    }

    pub fn increment_frame(&mut self) {
        self.next += self.incr;
        if self.next >= self.range.1 { self.next = self.range.0; }
    }
}

impl<T> ExternalPathwayTract for ScalarSequence<T>
            where T: Num + NumCast + PartialOrd + Debug + Clone + AddAssign + Copy {
    fn write_into(&mut self, tract_frame: &mut TractFrameMut, _: LayerTags) -> [usize; 3] {
        super::encode_scalar(self.next, self.range, tract_frame);

        Default::default()
    }

    fn cycle_next(&mut self) {
        self.increment_frame();
    }
}

