
/*

# Open Questions

* Can we use mapped memory somewhere in all of this (for buffer* variants)?
*



*/

#![allow(dead_code)]

use std::ops::{Range};
use std::sync::Arc;
use futures::sync::mpsc::{self, Sender, Receiver};
use ocl::{Buffer, RwVec, FutureReader, FutureWriter, OclPrm};


#[derive(Debug)]
pub enum TractBuffer<T: OclPrm> {
    TractReader(FutureReader<T>),
    TractWriter(FutureWriter<T>),
    OclBufferReader(Buffer<T>),
    OclBufferWriter(Buffer<T>),
    Single(RwVec<T>),
    Double,
    Triple,
}


#[derive(Debug)]
struct TractInner<T: OclPrm> {
    buffer_kind: TractBuffer<T>,
    buffer_idx_range: Range<usize>,
    backpressure: bool,
}

// impl<T: OclPrm> TractInner<T> {
//     pub fn buffer_idx_range(&self) -> Range<usize> {
//         self.buffer_idx_range.clone()
//     }
// }


#[derive(Debug)]
struct TractSenderUntyped<T: OclPrm> {
    inner: Arc<TractInner<T>>,
    tx: Sender<()>,
}

#[derive(Debug)]
struct TractReceiverUntyped<T: OclPrm> {
    inner: Arc<TractInner<T>>,
    rx: Receiver<()>,
}


#[derive(Debug)]
enum TractSenderKind {
    I8(TractSenderUntyped<i8>),
    U8(TractSenderUntyped<u8>),
}

#[derive(Debug)]
enum TractReceiverKind {
    I8(TractReceiverUntyped<i8>),
    U8(TractReceiverUntyped<u8>),
}


#[derive(Debug)]
pub struct TractSender(TractSenderKind);

impl TractSender {
    pub fn buffer_idx_range(&self) -> Range<usize> {
        match self.0 {
            TractSenderKind::I8(ref tsu) => tsu.inner.buffer_idx_range.clone(),
            TractSenderKind::U8(ref tsu) => tsu.inner.buffer_idx_range.clone(),
        }
    }
}

// impl Deref for TractSender {
//     type Target = TractInner<T>;

//     fn deref(&self) -> &TractInner<T> {
//         match self.0 {
//             TractSenderKind::I8(ref tsu) => &tsu.inner,
//             TractSenderKind::U8(ref tsu) => &tsu.inner,
//         }
//     }
// }

#[derive(Debug)]
pub struct TractReceiver(TractReceiverKind);

impl TractReceiver {
    pub fn buffer_idx_range(&self) -> Range<usize> {
        match self.0 {
            TractReceiverKind::I8(ref tsu) => tsu.inner.buffer_idx_range.clone(),
            TractReceiverKind::U8(ref tsu) => tsu.inner.buffer_idx_range.clone(),
        }
    }
}


fn tract_channel<T: OclPrm>(buffer_kind: TractBuffer<T>, buffer_idx_range: Range<usize>, backpressure: bool)
        -> (TractSenderUntyped<T>, TractReceiverUntyped<T>)
{
    let inner = Arc::new(TractInner {
        buffer_kind,
        buffer_idx_range,
        backpressure,
    });

    let (tx, rx) = mpsc::channel(0);

    let sender = TractSenderUntyped {
        inner: inner.clone(),
        tx,
    };

    let receiver = TractReceiverUntyped {
        inner: inner,
        rx,
    };

    (sender, receiver)
}

pub fn tract_channel_single_i8(buffer: RwVec<i8>, buffer_idx_range: Range<usize>, backpressure: bool)
        -> (TractSender, TractReceiver)
{
    let (tx, rx) = tract_channel(TractBuffer::Single(buffer), buffer_idx_range, backpressure);
    (TractSender(TractSenderKind::I8(tx)), TractReceiver(TractReceiverKind::I8(rx)))
}

pub fn tract_channel_single_u8(buffer: RwVec<u8>, buffer_idx_range: Range<usize>, backpressure: bool)
        -> (TractSender, TractReceiver)
{
    let (tx, rx) = tract_channel(TractBuffer::Single(buffer), buffer_idx_range, backpressure);
    (TractSender(TractSenderKind::U8(tx)), TractReceiver(TractReceiverKind::U8(rx)))
}





// enum Inner<T: OclPrm> {
//     TractReader(FutureReader<T>),
//     TractWriter(FutureWriter<T>),
//     BufferReader(Buffer<T>),
//     BufferWriter(Buffer<T>),
//     Single(RwVec<T>),
//     Double,
//     Triple,
// }

// // Alternative Names: IoChannel
// //
// //
// pub struct IoLink<T: OclPrm> {
//     inner: Inner<T>,
//     backpressure: bool,
// }

// impl<T: OclPrm> IoLink<T> {
//     pub fn direct_reader(reader: FutureReader<T>, backpressure: bool) -> IoLink<T> {
//         IoLink {
//             inner: Inner::TractReader(reader),
//             backpressure
//         }
//     }

//     pub fn direct_writer(writer: FutureWriter<T>, backpressure: bool) -> IoLink<T> {
//         IoLink {
//             inner: Inner::TractWriter(writer),
//             backpressure
//         }
//     }
// }
