/*

# Open Questions

* Can we use mapped memory somewhere in all of this (for buffer* variants)?
*



*/

#![allow(dead_code)]

use std::ops::Range;
use std::sync::Arc;
use futures::sync::mpsc::{self, Sender, Receiver};
use ocl::{Buffer, RwVec, FutureReader, FutureWriter, OclPrm};



struct TractInner<T> {
    tract_buffer: RwVec<T>,
    tract_range: Range<usize>,
}



// (for now) the `Thalamus` end:
pub struct TractSender<T> {
    inner: Arc<TractInner<T>>,
    tx: Sender<()>,
}


// (for now) the external end:
pub struct TractReceiver<T> {
    inner: Arc<TractInner<T>>,
    rx: Receiver<()>,
}


pub fn tract_channel<T>(tract_buffer: RwVec<T>, tract_range: Range<usize>)
        -> (TractSender<T>, TractReceiver<T>)
{
    let inner = Arc::new(TractInner {
        tract_buffer,
        tract_range,
    });

    let (tx, rx) = mpsc::channel(0);

    let sender = TractSender {
        inner: inner.clone(),
        tx,
    };

    let receiver = TractReceiver {
        inner: inner,
        rx,
    };

    (sender, receiver)
}





enum Inner<T: OclPrm> {
    TractReader(FutureReader<T>),
    TractWriter(FutureWriter<T>),
    BufferReader(Buffer<T>),
    BufferWriter(Buffer<T>),
    Single(RwVec<T>),
    Double,
    Triple,
}

// Alternative Names: IoChannel
//
//
pub struct IoLink<T: OclPrm> {
    inner: Inner<T>,
    backpressure: bool,
}

impl<T: OclPrm> IoLink<T> {
    pub fn direct_reader(reader: FutureReader<T>, backpressure: bool) -> IoLink<T> {
        IoLink {
            inner: Inner::TractReader(reader),
            backpressure
        }
    }

    pub fn direct_writer(writer: FutureWriter<T>, backpressure: bool) -> IoLink<T> {
        IoLink {
            inner: Inner::TractWriter(writer),
            backpressure
        }
    }
}
