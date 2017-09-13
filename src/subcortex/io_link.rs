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
use std::cell::UnsafeCell;
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


#[derive(Debug)]
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
            FutureSend::Send(ref mut b) => Ok(Async::Ready(b.take())),
            FutureSend::Skip => Ok(Async::Ready(None)),
            FutureSend::Wait(ref mut rx, ref mut b) => rx.poll().map(|status| status.map(|_| b.take())),
        }
    }
}


#[derive(Debug)]
pub enum FutureRecv {
    Recv(Option<ReadBuffer>),
    Skip,
    Wait(Receiver<()>, Option<ReadBuffer>),
}

impl FutureRecv {
    pub fn wait(&mut self) -> Result<Option<ReadBuffer>, Canceled> {
        Future::wait(self)
    }
}

impl Future for FutureRecv {
    type Item = Option<ReadBuffer>;
    type Error = Canceled;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        match *self {
            FutureRecv::Recv(ref mut b) => Ok(Async::Ready(b.take())),
            FutureRecv::Skip => Ok(Async::Ready(None)),
            FutureRecv::Wait(ref mut rx, ref mut b) => rx.poll().map(|status| status.map(|_| b.take())),
        }
    }
}


#[derive(Debug)]
pub enum WriteBuffer {
    RwVecI8(RwVec<i8>),
    RwVecU8(RwVec<u8>),
    FutureWriteGuardI8(FutureWriteGuard<i8>),
    FutureWriteGuardU8(FutureWriteGuard<u8>),
}

impl WriteBuffer {
    pub fn write_i8(self) -> FutureWriteGuard<i8> {
        match self {
            WriteBuffer::RwVecI8(rwv) => rwv.write(),
            WriteBuffer::FutureWriteGuardI8(fwg) => fwg,
            _ => panic!("WriteBuffer::write_i8: This buffer is not a 'i8'."),
        }
    }

    pub fn write_u8(self) -> FutureWriteGuard<u8> {
        match self {
            WriteBuffer::RwVecU8(rwv) => rwv.write(),
            WriteBuffer::FutureWriteGuardU8(fwg) => fwg,
            _ => panic!("WriteBuffer::write_u8: This buffer is not a 'u8'."),
        }
    }
}


#[derive(Debug)]
pub enum ReadBuffer {
    RwVecI8(RwVec<i8>),
    RwVecU8(RwVec<u8>),
    FutureReadGuardI8(FutureReadGuard<i8>),
    FutureReadGuardU8(FutureReadGuard<u8>),
}

impl ReadBuffer {
    pub fn read_i8(self) -> FutureReadGuard<i8> {
        match self {
            ReadBuffer::RwVecI8(rwv) => rwv.read(),
            ReadBuffer::FutureReadGuardI8(frg) => frg,
            _ => panic!("ReadBuffer::read_i8: This buffer is not a 'i8'."),
        }
    }

    pub fn read_u8(self) -> FutureReadGuard<u8> {
        match self {
            ReadBuffer::RwVecU8(rwv) => rwv.read(),
            ReadBuffer::FutureReadGuardU8(frg) => frg,
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

    fn next_read_buffer(&self) -> RwVec<T> {
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
            TractBuffer::I8(ref tbt) => WriteBuffer::RwVecI8(tbt.next_write_buffer()),
            TractBuffer::U8(ref tbt) => WriteBuffer::RwVecU8(tbt.next_write_buffer()),
        }
    }

    fn next_read_buffer(&self) -> ReadBuffer {
        match *self {
            TractBuffer::I8(ref tbt) => ReadBuffer::RwVecI8(tbt.next_read_buffer()),
            TractBuffer::U8(ref tbt) => ReadBuffer::RwVecU8(tbt.next_read_buffer()),
        }
    }

    fn next_wr_guard_pair(&self) -> (WriteBuffer, ReadBuffer) {
        match *self {
            TractBuffer::I8(ref tbt) => {
                let wg = WriteBuffer::FutureWriteGuardI8(tbt.next_write_buffer().write());
                let rg = ReadBuffer::FutureReadGuardI8(tbt.next_read_buffer().read());
                (wg, rg)
            },
            TractBuffer::U8(ref tbt) => {
                let wg = WriteBuffer::FutureWriteGuardU8(tbt.next_write_buffer().write());
                let rg = ReadBuffer::FutureReadGuardU8(tbt.next_read_buffer().read());
                (wg, rg)
            },
        }
    }
}


#[derive(Debug)]
pub struct TractInner {
    buffer: TractBuffer,
    buffer_idx_range: Range<usize>,
    backpressure: bool,
    state: AtomicUsize,
    next_read_guard: AtomicOption<ReadBuffer>,
    // next_read_guard: UnsafeCell<Option<ReadBuffer>>,
    send_waiting: AtomicOption<(Sender<()>, ReadBuffer)>,
    // send_waiting_read_buffer: AtomicOption<ReadBuffer>,
    // send_waiting_read_buffer: UnsafeCell<Option<ReadBuffer>>,
    // recv_waiting_tx: AtomicOption<Sender<()>>,
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
            next_read_guard: AtomicOption::new(),
            // next_read_guard: UnsafeCell::new(None),
            send_waiting: AtomicOption::new(),
            // send_waiting_read_buffer: AtomicOption::new(),
            // send_waiting_read_buffer: UnsafeCell::new(None),
            // recv_waiting_tx: AtomicOption::new(),
        }
    }

    fn send(&self) -> FutureSend {
        // if let Some(tx) = self.recv_waiting_tx.take(Ordering::SeqCst) {
        //     tx.send(()).ok();
        // }

        let cur_state = self.state.fetch_or(BUFFER_0_FRESH, Ordering::SeqCst);
        let buffer_already_fresh = (cur_state & BUFFER_0_FRESH) != 0;

        if buffer_already_fresh {
            if self.backpressure {
                let (tx, rx) = oneshot::channel();
                let (wg, rg) = self.buffer.next_wr_guard_pair();
                let old_tx = self.send_waiting.swap((tx, rg), Ordering::SeqCst);
                assert!(old_tx.is_none());
                FutureSend::Wait(rx, Some(wg))
            } else {
                FutureSend::Skip
            }
        } else {
            let (wg, rg) = self.buffer.next_wr_guard_pair();
            let old_rg = self.next_read_guard.swap(rg, Ordering::SeqCst);
            assert!(old_rg.is_none());
            FutureSend::Send(Some(wg))
        }
    }

    fn recv(&self, _wait_for_frame: bool) -> FutureRecv {
        let cur_state = self.state.fetch_and(!BUFFER_0_FRESH, Ordering::SeqCst);
        let buffer_is_fresh = (cur_state & BUFFER_0_FRESH) != 0;

        if buffer_is_fresh {
            let next_read_guard = match self.send_waiting.take(Ordering::SeqCst) {
                Some((tx, wrg)) => {
                    let state = self.state.fetch_or(BUFFER_0_FRESH, Ordering::SeqCst);
                    // Ensure no one has tampered with the state (the
                    // sender(s) should be blocked by backpressure):
                    assert!(state & BUFFER_0_FRESH == 0);
                    tx.send(()).ok();
                    self.next_read_guard.swap(wrg, Ordering::SeqCst)
                },
                None => self.next_read_guard.take(Ordering::SeqCst),
            };

            assert!(next_read_guard.is_some());
            FutureRecv::Recv(next_read_guard)
        } else {
            debug_assert!(self.send_waiting.take(Ordering::SeqCst).is_none());
            // if wait_for_frame {
            //     let (tx, rx) = oneshot::channel();
            //     let old_tx = self.recv_waiting_tx.swap(tx, Ordering::SeqCst);
            //     assert!(old_tx.is_none());

            //     FutureRecv::Wait(rx, Some(self.buffer.next_read_guard()))
            // } else {
            //     FutureRecv::Skip
            // }
            FutureRecv::Skip
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
    pub fn recv(&self, wait_for_frame: bool) -> FutureRecv {
        self.inner.recv(wait_for_frame)
    }
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
