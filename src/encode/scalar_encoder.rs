//! Encodes a scalar value.

use std::fmt::Debug;
use std::ops::AddAssign;
use num::{Num, NumCast};
use thalamus::{ExternalSourceTract, TractFrameMut, LayerTags};

#[derive(Clone, Debug)]
pub struct ScalarEncoder<T> {
    range: (T, T),
}

impl<T> ScalarEncoder<T> where T: Num + NumCast + PartialOrd + Debug + Clone + AddAssign + Copy {
    pub fn new(range: (T, T)) -> ScalarEncoder<T> {
        ScalarEncoder {
            range: range,
        }
    }
}

impl<T> ExternalSourceTract for ScalarEncoder<T>
            where T: Num + NumCast + PartialOrd + Debug + Clone + AddAssign + Copy {
    fn write_into(&mut self, tract_frame: &mut TractFrameMut, _: LayerTags) -> [usize; 3] {
        super::encode_scalar(Default::default(), self.range, tract_frame);

        Default::default()
    }

    fn cycle_next(&mut self) {
        // self.increment_frame();
    }
}

