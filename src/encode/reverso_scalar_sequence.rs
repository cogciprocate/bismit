//! Mostly for testing purposes.

use std::fmt::Debug;
use std::ops::AddAssign;
use num::{Num, NumCast};
use cmn::TractFrameMut;
use ::{ExternalPathwayTract};
use encode::ScalarEncodable;
use map::LayerAddress;


// enum ScalarEncoder {

// }

#[derive(Clone, Debug)]
pub struct ReversoScalarSequence<T> {
    range: (T, T),
    next: T,
    incr: T,
    layer_addrs: Vec<LayerAddress>,
}

impl<T> ReversoScalarSequence<T> where T: Num + NumCast + PartialOrd + Debug + Clone + AddAssign + Copy {
    pub fn new(range: (T, T), incr: T, layer_addrs: &[LayerAddress])
                -> ReversoScalarSequence<T> {
        let next = range.0;
        let layer_addrs = layer_addrs.into_iter().cloned().collect();

        ReversoScalarSequence {
            range: range,
            incr: incr,
            next: next,
            layer_addrs: layer_addrs,
        }
    }

    pub fn increment_frame(&mut self) {
        self.next += self.incr;
        if self.next >= self.range.1 { self.next = self.range.0; }
    }
}

impl<T> ExternalPathwayTract for ReversoScalarSequence<T>
            where T: ScalarEncodable {
    fn write_into(&mut self, tract_frame: &mut TractFrameMut, addr: LayerAddress) {
        let l_idx = self.layer_addrs.iter().position(|&t| t == addr)
            .expect(&format!("ReversoScalarSequence::write_into(): No layers with address: {:?}", addr));

        if l_idx == 0 {
            super::encode_scalar(self.next, self.range, tract_frame);
        } else if l_idx == 1 {
            super::encode_scalar(self.range.1 - self.next, self.range, tract_frame);
        } else {
            panic!("ReversoScalarSequence::write_into(): Too many layers!");
        }
    }

    fn cycle_next(&mut self) {
        self.increment_frame();
    }
}

