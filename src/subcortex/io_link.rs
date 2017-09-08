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
use ocl::{RwVec, FutureReadGuard, FutureWriteGuard, OclPrm};
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
    Send(Option<WriteBuffer>),
    Skip,
    Wait(Receiver<()>, Option<WriteBuffer>),
}

impl Future for FutureSend {
    type Item = Option<WriteBuffer>;
    type Error = Canceled;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        match *self {
            FutureSend::Send(ref mut tb) => Ok(Async::Ready(tb.take())),
            FutureSend::Skip => Ok(Async::Ready(None)),
            FutureSend::Wait(ref mut rx, ref mut tb) => rx.poll().map(|status| status.map(|_| tb.take())),
        }
    }
}


pub struct FutureRecv {

}


pub enum WriteBuffer {
    I8(RwVec<i8>),
    U8(RwVec<u8>),
}

impl WriteBuffer {
    pub fn write_i8(&self) -> FutureWriteGuard<i8> {
        match *self {
            WriteBuffer::I8(ref rwv) => rwv.clone().write(),
            _ => panic!("WriteBuffer::write_i8: This buffer is not a 'i8'."),
        }
    }

    pub fn write_u8(&self) -> FutureWriteGuard<u8> {
        match *self {
            WriteBuffer::U8(ref rwv) => rwv.clone().write(),
            _ => panic!("WriteBuffer::write_u8: This buffer is not a 'u8'."),
        }
    }
}


pub enum ReadBuffer {
    I8(RwVec<i8>),
    U8(RwVec<u8>),
}

impl ReadBuffer {
    pub fn read_i8(&self) -> FutureReadGuard<i8> {
        match *self {
            ReadBuffer::I8(ref rwv) => rwv.clone().read(),
            _ => panic!("ReadBuffer::read_i8: This buffer is not a 'i8'."),
        }
    }

    pub fn read_u8(&self) -> FutureReadGuard<u8> {
        match *self {
            ReadBuffer::U8(ref rwv) => rwv.clone().read(),
            _ => panic!("ReadBuffer::read_u8: This buffer is not a 'u8'."),
        }
    }
}


#[derive(Debug)]
pub enum TractBufferTyped<T: OclPrm> {
    Single(RwVec<T>),
    Double,
    Triple,
}

impl<T: OclPrm> TractBufferTyped<T> {
    fn next_write_buffer(&self) -> RwVec<T> {
        match *self {
            TractBufferTyped::Single(ref rwv) => rwv.clone(),
            TractBufferTyped::Double => unimplemented!(),
            TractBufferTyped::Triple => unimplemented!(),
        }
    }
}


#[derive(Debug)]
pub enum TractBuffer {
    I8(TractBufferTyped<i8>),
    U8(TractBufferTyped<u8>),
}

impl TractBuffer {
    fn next_write_buffer(&self) -> WriteBuffer {
        match *self {
            TractBuffer::I8(ref tbt) => WriteBuffer::I8(tbt.next_write_buffer()),
            TractBuffer::U8(ref tbt) => WriteBuffer::U8(tbt.next_write_buffer()),
        }
    }
}


#[derive(Debug)]
pub struct TractInner {
    buffer: TractBuffer,
    buffer_idx_range: Range<usize>,
    backpressure: bool,
    state: AtomicUsize,
    send_waiting_tx: AtomicOption<Sender<()>>,
    recv_waiting_tx: AtomicOption<Sender<()>>,
}

impl TractInner {
    fn new(buffer: TractBuffer, buffer_idx_range: Range<usize>, backpressure: bool)
            -> TractInner
    {
        TractInner {
            buffer,
            buffer_idx_range,
            backpressure,
            state: AtomicUsize::new(0),
            send_waiting_tx: AtomicOption::new(),
            recv_waiting_tx: AtomicOption::new(),
        }
    }

    fn send(&self) -> FutureSend {
        let cur_state = self.state.fetch_or(BUFFER_0_FRESH, Ordering::SeqCst);
        let buffer_is_fresh = (cur_state & BUFFER_0_FRESH) != 0;

        if buffer_is_fresh {
            if self.backpressure {
                let (tx, rx) = oneshot::channel();
                let old_tx = self.send_waiting_tx.swap(tx, Ordering::SeqCst);
                assert!(old_tx.is_none());
                FutureSend::Wait(rx, Some(self.buffer.next_write_buffer()))
            } else {
                FutureSend::Skip
            }
        } else {
            FutureSend::Send(Some(self.buffer.next_write_buffer()))
        }
    }

    fn recv(&self) -> bool {
        let mut tx_completed = false;
        if let Some(tx) = self.send_waiting_tx.take(Ordering::SeqCst) {
            tx.send(()).ok();
            tx_completed = true;
        }

        let cur_state = self.state.fetch_and(!BUFFER_0_FRESH, Ordering::SeqCst);
        let buffer_is_fresh = (cur_state & BUFFER_0_FRESH) != 0;

        if buffer_is_fresh {
            assert!(tx_completed);
            return true;
        } else {
            return false;
        }
    }

    #[inline]
    pub fn buffer_idx_range(&self) -> Range<usize> {
        self.buffer_idx_range.clone()
    }

    #[inline]
    pub fn backpressure(&self) -> bool {
        self.backpressure
    }
}

unsafe impl Send for TractInner {}
unsafe impl Sync for TractInner {}



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
    // let inner = Arc::new(TractInner::new_i8(TractBuffer::Single(buffer), buffer_idx_range, backpressure));
    let tract_buffer = TractBuffer::I8(TractBufferTyped::Single(buffer));
    let inner = Arc::new(TractInner::new(tract_buffer, buffer_idx_range, backpressure));
    (TractSender { inner: inner.clone() }, TractReceiver { inner })
}

pub fn tract_channel_single_u8(buffer: RwVec<u8>, buffer_idx_range: Range<usize>, backpressure: bool)
        -> (TractSender, TractReceiver)
{
    let tract_buffer = TractBuffer::U8(TractBufferTyped::Single(buffer));
    let inner = Arc::new(TractInner::new(tract_buffer, buffer_idx_range, backpressure));
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
//     TractReader(FutureReadGuard<T>),
//     TractWriter(FutureWriteGuard<T>),
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
//     pub fn direct_reader(reader: FutureReadGuard<T>, backpressure: bool) -> IoLink<T> {
//         IoLink {
//             inner: Inner::TractReader(reader),
//             backpressure
//         }
//     }

//     pub fn direct_writer(writer: FutureWriteGuard<T>, backpressure: bool) -> IoLink<T> {
//         IoLink {
//             inner: Inner::TractWriter(writer),
//             backpressure
//         }
//     }
// }
