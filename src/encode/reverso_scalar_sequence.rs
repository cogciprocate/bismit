//! Mostly for testing purposes.

// use std::collections::HashMap;
use std::fmt::Debug;
use std::ops::AddAssign;
use num::{Num, NumCast};
use thalamus::{ExternalPathwayTract, TractFrameMut, LayerTags};
use encode::ScalarEncodable;
// use map::LayerTags;

#[derive(Clone, Debug)]
pub struct ReversoScalarSequence<T> {
    range: (T, T),
    next: T,
    incr: T,
    layer_tags: Vec<LayerTags>,
}

impl<T> ReversoScalarSequence<T> where T: Num + NumCast + PartialOrd + Debug + Clone + AddAssign + Copy {
    pub fn new(range: (T, T), incr: T, layers: &[LayerTags])
                -> ReversoScalarSequence<T> {
        let next = range.0;

        ReversoScalarSequence {
            range: range,
            incr: incr,
            next: next,
            layer_tags: Vec::from(layers),
        }
    }

    pub fn increment_frame(&mut self) {
        self.next += self.incr;
        if self.next >= self.range.1 { self.next = self.range.0; }
    }
}

impl<T> ExternalPathwayTract for ReversoScalarSequence<T>
            where T: ScalarEncodable {
    fn write_into(&mut self, tract_frame: &mut TractFrameMut, tags: LayerTags) {
        let l_idx = self.layer_tags.iter().position(|&t| t == tags)
            .expect(&format!("ReversoScalarSequence::write_into(): No layers matching tags: {}", tags));

        if l_idx == 0 {
            super::encode_scalar(self.next, self.range, tract_frame);
        } else if l_idx == 1 {
            super::encode_scalar(self.range.1 - self.next, self.range, tract_frame);
        } else {
            panic!("ReversoScalarSequence::write_into(): Too many layers!");
        }

        // Default::default()
    }

    fn cycle_next(&mut self) {
        self.increment_frame();
    }
}

