// # Open Questions
//
// * Can we use mapped memory somewhere in all of this (for buffer* variants)?
// *
//

#![allow(dead_code, unused_imports)]

use std::fmt;
use std::error::Error as StdError;
use std::ops::{Range, Deref};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use futures::{Async, Poll};
use futures::sync::oneshot::{self, Sender, Receiver, Canceled};
use crossbeam::sync::AtomicOption;
use futures::{Future};
use ocl::{/*Buffer,*/ RwVec, /*FutureReader, FutureWriter,*/ OclPrm};
use cmn::CmnError;


const BUFFER_0_FRESH: usize = 0x00000001;
const BUFFER_1_FRESH: usize = 0x00000002;
const BUFFER_2_FRESH: usize = 0x00000004;


// #[derive(Debug)]
// enum FutureSendError {
//     BufferFresh,
// }

// impl fmt::Display for FutureSendError {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         write!(f, "FutureSendError")
//     }
// }

// impl StdError for FutureSendError {
//     fn description(&self) -> &str {
//         "FutureSendError"
//     }
// }


pub enum FutureSend {
    Send,
    Skip,
    Wait(Receiver<()>),
}

impl Future for FutureSend {
    type Item = bool;
    type Error = Canceled;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        match *self {
            FutureSend::Send => Ok(Async::Ready(true)),
            FutureSend::Skip => Ok(Async::Ready(false)),
            FutureSend::Wait(ref mut rx) => rx.poll().map(|status| status.map(|_| true)),
        }
    }
}


pub struct FutureRecv {

}


#[derive(Debug)]
pub enum TractBuffer<T: OclPrm> {
    // TractReader(FutureReader<T>),
    // TractWriter(FutureWriter<T>),
    // OclBufferReader(Buffer<T>),
    // OclBufferWriter(Buffer<T>),
    Single(RwVec<T>),
    Double,
    Triple,
}


#[derive(Debug)]
pub struct UntypedTractInner<T: OclPrm> {
    buffer: TractBuffer<T>,
    buffer_idx_range: Range<usize>,
    backpressure: bool,
    state: AtomicUsize,
    send_tx: AtomicOption<Sender<()>>,
    recv_tx: AtomicOption<Sender<()>>,
}

impl<T: OclPrm> UntypedTractInner<T> {
    fn new(buffer: TractBuffer<T>, buffer_idx_range: Range<usize>, backpressure: bool)
            -> UntypedTractInner<T>
    {
        UntypedTractInner {
            buffer,
            buffer_idx_range,
            backpressure,
            state: AtomicUsize::new(0),
            send_tx: AtomicOption::new(),
            recv_tx: AtomicOption::new(),
        }
    }

    fn send(&self) -> FutureSend {
        let cur_state = self.state.fetch_or(BUFFER_0_FRESH, Ordering::SeqCst);
        let buffer_is_fresh = (cur_state & BUFFER_0_FRESH) != 0;

        if buffer_is_fresh {
            if self.backpressure {
                let (tx, rx) = oneshot::channel();
                let old_tx = self.send_tx.swap(tx, Ordering::SeqCst);
                assert!(old_tx.is_none());
                FutureSend::Wait(rx)
            } else {
                FutureSend::Skip
            }
        } else {
            FutureSend::Send
        }
    }

    fn recv(&self) -> bool {
        let cur_state = self.state.fetch_and(!BUFFER_0_FRESH, Ordering::SeqCst);
        let buffer_is_fresh = (cur_state & BUFFER_0_FRESH) != 0;

        if buffer_is_fresh {
            return true;
        } else {
            return false;
        }
    }
}

unsafe impl<T: OclPrm> Send for UntypedTractInner<T> {}
unsafe impl<T: OclPrm> Sync for UntypedTractInner<T> {}



#[derive(Debug)]
pub enum TractInner {
    I8(UntypedTractInner<i8>),
    U8(UntypedTractInner<u8>),
}

impl TractInner {
    fn new_i8(buffer: TractBuffer<i8>, buffer_idx_range: Range<usize>, backpressure: bool) -> TractInner {
        TractInner::I8(UntypedTractInner::new(buffer, buffer_idx_range, backpressure))
    }

    fn new_u8(buffer: TractBuffer<u8>, buffer_idx_range: Range<usize>, backpressure: bool) -> TractInner {
        TractInner::U8(UntypedTractInner::new(buffer, buffer_idx_range, backpressure))
    }

    #[inline]
    fn send(&self) -> FutureSend {
        match *self {
            TractInner::I8(ref ti) => ti.send(),
            TractInner::U8(ref ti) => ti.send(),
        }
    }

    #[inline]
    pub fn buffer_idx_range(&self) -> Range<usize> {
        match *self {
            TractInner::I8(ref ti) => ti.buffer_idx_range.clone(),
            TractInner::U8(ref ti) => ti.buffer_idx_range.clone(),
        }
    }

    #[inline]
    pub fn backpressure(&self) -> bool {
        match *self {
            TractInner::I8(ref ti) => ti.backpressure,
            TractInner::U8(ref ti) => ti.backpressure,
        }
    }

    /// Panics if not a u8.
    #[inline]
    pub fn buffer_u8(&self) -> &RwVec<u8> {
        match *self {
            TractInner::U8(ref ti) => {
                match ti.buffer {
                    TractBuffer::Single(ref b) => b,
                    _ => unimplemented!(),
                }
            },
            _ => panic!("TractSender::single_u8: This buffer is not a 'u8'."),
        }
    }
}


#[derive(Debug)]
pub struct TractSender {
    inner: Arc<TractInner>,
}

impl TractSender {
    pub fn send(&self) -> FutureSend {
        self.inner.send()
    }
}

impl Deref for TractSender {
    type Target = TractInner;

    fn deref(&self) -> &TractInner {
        &self.inner
    }
}


#[derive(Debug)]
pub struct TractReceiver {
    inner: Arc<TractInner>,
}

impl TractReceiver {

}

impl Deref for TractReceiver {
    type Target = TractInner;

    fn deref(&self) -> &TractInner {
        &self.inner
    }
}


pub fn tract_channel_single_i8(buffer: RwVec<i8>, buffer_idx_range: Range<usize>, backpressure: bool)
        -> (TractSender, TractReceiver)
{
    // let (tx, rx) = tract_channel(TractBuffer::Single(buffer), buffer_idx_range, backpressure);
    // (TractSender(TractSenderKind::I8(tx)), TractReceiver(TractReceiverKind::I8(rx)))
    let inner = Arc::new(TractInner::new_i8(TractBuffer::Single(buffer), buffer_idx_range, backpressure));
    (TractSender { inner: inner.clone() }, TractReceiver { inner })
}

pub fn tract_channel_single_u8(buffer: RwVec<u8>, buffer_idx_range: Range<usize>, backpressure: bool)
        -> (TractSender, TractReceiver)
{
    let inner = Arc::new(TractInner::new_u8(TractBuffer::Single(buffer), buffer_idx_range, backpressure));
    (TractSender { inner: inner.clone() }, TractReceiver { inner })
}







// #[derive(Debug)]
// struct TractSenderUntyped<T: OclPrm> {
//     inner: Arc<TractInner<T>>,
//     // tx: Sender<()>,
// }

// #[derive(Debug)]
// struct TractReceiverUntyped<T: OclPrm> {
//     inner: Arc<TractInner<T>>,
//     // rx: Receiver<()>,
// }


// #[derive(Debug)]
// enum TractSenderKind {
//     I8(TractSenderUntyped<i8>),
//     U8(TractSenderUntyped<u8>),
// }

// #[derive(Debug)]
// enum TractReceiverKind {
//     I8(TractReceiverUntyped<i8>),
//     U8(TractReceiverUntyped<u8>),
// }


// impl TractSender {
//     pub fn buffer_idx_range(&self) -> Range<usize> {
//         self.inner.buffer_idx_range()
//     }

//     pub fn backpressure(&self) -> bool {
//         match self.0 {
//             TractSenderKind::I8(ref ts) => ts.inner.backpressure,
//             TractSenderKind::U8(ref ts) => ts.inner.backpressure,
//         }
//     }

//     pub fn buffer_single_u8(&self) -> &RwVec<u8> {
//         match self.0 {
//             TractSenderKind::U8(ref ts) => {
//                 match ts.inner.buffer {
//                     TractBuffer::Single(ref b) => b,
//                     _ => panic!("TractSender::single_u8: This buffer is not a 'Single'."),
//                 }
//             },
//             _ => panic!("TractSender::single_u8: This buffer is not a 'u8'."),
//         }
//     }
// }



// impl TractReceiver {
//     pub fn buffer_idx_range(&self) -> Range<usize> {
//         match self.0 {
//             TractReceiverKind::I8(ref tr) => tr.inner.buffer_idx_range.clone(),
//             TractReceiverKind::U8(ref tr) => tr.inner.buffer_idx_range.clone(),
//         }
//     }

//     pub fn backpressure(&self) -> bool {
//         match self.0 {
//             TractReceiverKind::I8(ref tr) => tr.inner.backpressure,
//             TractReceiverKind::U8(ref tr) => tr.inner.backpressure,
//         }
//     }

//     pub fn buffer_single_u8(&self) -> &RwVec<u8> {
//         match self.0 {
//             TractReceiverKind::U8(ref ts) => {
//                 match ts.inner.buffer {
//                     TractBuffer::Single(ref b) => b,
//                     _ => panic!("TractReceiver::single_u8: This buffer is not a 'Single'."),
//                 }
//             },
//             _ => panic!("TractReceiver::single_u8: This buffer is not a 'u8'."),
//         }
//     }
// }


// fn tract_channel<T: OclPrm>(buffer: TractBuffer<T>, buffer_idx_range: Range<usize>, backpressure: bool)
//         -> (TractSenderUntyped<T>, TractReceiverUntyped<T>)
// {
//     let inner = Arc::new(TractInner {
//         buffer,
//         buffer_idx_range,
//         backpressure,
//         state: AtomicUsize::new(0),
//     });

//     // let (tx, rx) = mpsc::channel(0);

//     let sender = TractSenderUntyped {
//         inner: inner.clone(),
//         // tx,
//     };

//     let receiver = TractReceiverUntyped {
//         inner: inner,
//         // rx,
//     };

//     (sender, receiver)
// }


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
