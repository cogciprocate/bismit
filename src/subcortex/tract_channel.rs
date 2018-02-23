//! An asynchronous channel for cortical input and output which can interact
//! with OpenCL events and optionally apply back-pressure.
//!

// Open Questions:
// * Can we use mapped memory somewhere in all of this (for buffer* variants)?
// *
//
// TODO: Loosen ordering constraints where possible.
// TODO: Replace `AtomicOption`s with `UnsafeCell`s where possible.

#![allow(dead_code, unused_imports)]

use std::fmt;
use std::error::Error as StdError;
use std::ops::{Range, Deref};
use std::sync::Arc;
use std::sync::atomic::{fence, AtomicUsize, Ordering};
use std::sync::atomic::Ordering::SeqCst;
use std::cell::UnsafeCell;
use std::thread;
use futures::{Async, Poll};
use futures::sync::oneshot::{self, Sender, Receiver, Canceled};
use crossbeam::sync::AtomicOption;
use futures::{Future};
use ocl::{RwVec, FutureReadGuard, FutureWriteGuard, ReadGuard, WriteGuard, OclPrm};
use cmn::CmnError;


const NEXT_READ_GUARD_READY_FLAG: usize = 0b00000001;
// const BUFFER_1_FRESH: usize = 0x00000002;
// const BUFFER_2_FRESH: usize = 0x00000004;
const BACKPRESSURE_FLAG: usize = 0b00010000;


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

/// Spins an ever increasing amount of time.
fn _spin(spins: &mut usize) {
    if *spins < 16 {
        for _ in 0..(2 << *spins) { fence(SeqCst); }
    } else {
        thread::yield_now();
    }
    *spins += 1;
}


#[derive(Debug)]
pub enum ReadGuardVec {
    U8(ReadGuard<Vec<u8>>),
    I8(ReadGuard<Vec<i8>>),
}

impl ReadGuardVec {
    pub fn u8(&self) -> &ReadGuard<Vec<u8>> {
        match *self {
            ReadGuardVec::U8(ref rg) => rg,
            _ => panic!("ReadGuardVec::u8: This guard is not a 'u8'."),
        }
    }

    pub fn i8(&self) -> &ReadGuard<Vec<i8>> {
        match *self {
            ReadGuardVec::I8(ref rg) => rg,
            _ => panic!("ReadGuardVec::i8: This guard is not an 'i8'."),
        }
    }
}


#[derive(Debug)]
pub enum FutureReadGuardVec {
    U8(FutureReadGuard<Vec<u8>>),
    I8(FutureReadGuard<Vec<i8>>),
}

impl From<ReadBuffer> for FutureReadGuardVec {
    fn from(rb: ReadBuffer) -> FutureReadGuardVec {
        match rb {
            ReadBuffer::RwVecI8(vec_i8) => FutureReadGuardVec::I8(vec_i8.read()),
            ReadBuffer::RwVecU8(vec_u8) => FutureReadGuardVec::U8(vec_u8.read()),
            ReadBuffer::FutureReadGuardI8(frg_i8) => FutureReadGuardVec::I8(frg_i8),
            ReadBuffer::FutureReadGuardU8(frg_u8) => FutureReadGuardVec::U8(frg_u8),
        }
    }
}

impl Future for FutureReadGuardVec {
    type Item = ReadGuardVec;
    type Error = CmnError;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        match *self {
            FutureReadGuardVec::U8(ref mut frg_u8) => {
                frg_u8.poll()
                    .map(|rg_poll| rg_poll.map(|rg| ReadGuardVec::U8(rg)))
                    .map_err(|err| err.into())
            }
            FutureReadGuardVec::I8(ref mut frg_i8) => {
                frg_i8.poll()
                    .map(|rg_poll| rg_poll.map(|rg| ReadGuardVec::I8(rg)))
                    .map_err(|err| err.into())

            }
        }
    }
}



#[derive(Debug)]
pub enum WriteBuffer {
    RwVecI8(RwVec<i8>),
    RwVecU8(RwVec<u8>),
    FutureWriteGuardI8(FutureWriteGuard<Vec<i8>>),
    FutureWriteGuardU8(FutureWriteGuard<Vec<u8>>),
}

impl WriteBuffer {
    pub fn write_i8(self) -> FutureWriteGuard<Vec<i8>> {
        match self {
            WriteBuffer::RwVecI8(rwv) => rwv.write(),
            WriteBuffer::FutureWriteGuardI8(fwg) => fwg,
            _ => panic!("WriteBuffer::write_i8: This buffer is not a 'i8'."),
        }
    }

    pub fn write_u8(self) -> FutureWriteGuard<Vec<u8>> {
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
    FutureReadGuardI8(FutureReadGuard<Vec<i8>>),
    FutureReadGuardU8(FutureReadGuard<Vec<u8>>),
}

impl ReadBuffer {
    pub fn read_i8(self) -> FutureReadGuard<Vec<i8>> {
        match self {
            ReadBuffer::RwVecI8(rwv) => rwv.read(),
            ReadBuffer::FutureReadGuardI8(frg) => frg,
            _ => panic!("ReadBuffer::read_i8: This buffer is not a 'i8'."),
        }
    }

    pub fn read_u8(self) -> FutureReadGuard<Vec<u8>> {
        match self {
            ReadBuffer::RwVecU8(rwv) => rwv.read(),
            ReadBuffer::FutureReadGuardU8(frg) => frg,
            _ => panic!("ReadBuffer::read_u8: This buffer is not a 'u8'."),
        }
    }
}


#[derive(Debug)]
pub enum FutureSend {
    Send(Option<WriteBuffer>),
    Skip,
    Wait(Receiver<()>, Option<WriteBuffer>),
}

impl FutureSend {
    pub fn wait(self) -> Result<Option<WriteBuffer>, Canceled> {
        <Self as Future>::wait(self)
    }
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
    Wait(Receiver<ReadBuffer>),
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
            FutureRecv::Wait(ref mut rx) => rx.poll().map(|status| status.map(|b| Some(b))),
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

    fn len(&self) -> usize {
        match *self {
            TractBufferTyped::Single(ref rwv) => rwv.len_stale(),
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

    fn next_write_guard(&self) -> WriteBuffer {
        match *self {
            TractBuffer::I8(ref tbt) => {
                WriteBuffer::FutureWriteGuardI8(tbt.next_write_buffer().write())
            },
            TractBuffer::U8(ref tbt) => {
                WriteBuffer::FutureWriteGuardU8(tbt.next_write_buffer().write())
            },
        }
    }

    fn next_read_guard(&self) -> ReadBuffer {
        match *self {
            TractBuffer::I8(ref tbt) => {
                ReadBuffer::FutureReadGuardI8(tbt.next_read_buffer().read())
            },
            TractBuffer::U8(ref tbt) => {
                ReadBuffer::FutureReadGuardU8(tbt.next_read_buffer().read())
            },
        }
    }

    pub fn len(&self) -> usize {
        match *self {
            TractBuffer::I8(ref tbt) => tbt.len(),
            TractBuffer::U8(ref tbt) => tbt.len(),
        }
    }
}


#[derive(Clone, Copy, Debug)]
enum Direction {
    SendRecv,
    SendOnly,
    RecvOnly,
}


#[derive(Debug)]
pub struct TractInner {
    buffer: TractBuffer,
    buffer_idx_range: Range<usize>,
    // backpressure: bool,
    direction: Direction,
    state: AtomicUsize,
    next_read_guard: AtomicOption<ReadBuffer>,
    send_waiting: AtomicOption<(Sender<()>, ReadBuffer)>,
    recv_waiting: AtomicOption<Sender<ReadBuffer>>,
}

impl TractInner {
    fn new(buffer: TractBuffer, buffer_idx_range: Option<Range<usize>>, backpressure: bool,
            direction: Direction) -> TractInner {
        let buffer_idx_range = buffer_idx_range.unwrap_or(0..buffer.len());
        let backpressure_state = if backpressure { BACKPRESSURE_FLAG } else { 0 };

        TractInner {
            buffer,
            buffer_idx_range,
            // backpressure,
            // send_only,
            // recv_only,
            direction,
            state: AtomicUsize::new(backpressure_state),
            next_read_guard: AtomicOption::new(),
            send_waiting: AtomicOption::new(),
            recv_waiting: AtomicOption::new(),
        }
    }

    fn send(&self) -> FutureSend {
        match self.direction {
            Direction::SendOnly => return FutureSend::Send(Some(self.buffer.next_write_guard())),
            Direction::RecvOnly => return FutureSend::Skip,
            _ => ()
        }

        let prior_state = self.state.fetch_or(NEXT_READ_GUARD_READY_FLAG, SeqCst);
        let backpressure = (prior_state & BACKPRESSURE_FLAG) != 0;
        let buffer_already_ready = (prior_state & NEXT_READ_GUARD_READY_FLAG) != 0;

        if buffer_already_ready {
            if backpressure {
                let (tx, rx) = oneshot::channel();
                let (wg, rg) = self.buffer.next_wr_guard_pair();
                let old_tx = self.send_waiting.swap((tx, rg), SeqCst);
                assert!(old_tx.is_none());
                FutureSend::Wait(rx, Some(wg))
            } else {
                FutureSend::Skip
            }
        } else {
            let (wg, rg) = self.buffer.next_wr_guard_pair();
            match self.recv_waiting.take(SeqCst) {
                Some(tx) => {
                    // println!("TractInner::send: Read guard stale. Sending new...");
                    tx.send(rg).unwrap();
                },
                None => {
                    // println!("TractInner::send: Read guard stale. Swapping in new.");
                    let old_rg = self.next_read_guard.swap(rg, SeqCst);
                    assert!(old_rg.is_none());
                },
            }
            FutureSend::Send(Some(wg))
        }
    }

    fn recv(&self, wait_for_frame: bool) -> FutureRecv {
        match self.direction {
            Direction::SendOnly => return FutureRecv::Skip,
            Direction::RecvOnly => return FutureRecv::Recv(Some(self.buffer.next_read_guard())),
            _ => ()
        }

        match self.next_read_guard.take(SeqCst) {
            Some(next_read_guard) => {
                assert!(self.state.load(SeqCst) & NEXT_READ_GUARD_READY_FLAG != 0);
                // Rotate in the waiting guard if any:
                match self.send_waiting.take(SeqCst) {
                    Some((tx, wrg)) => {
                        // println!("TractInner::recv: self.send_waiting => Some(tx, wrg)");
                        self.next_read_guard.swap(wrg, SeqCst);
                        tx.send(()).ok();
                    },
                    None => {
                        // println!("TractInner::recv: self.send_waiting => None");
                        let prior_state = self.state.fetch_and(!NEXT_READ_GUARD_READY_FLAG, SeqCst);
                        assert!(prior_state & NEXT_READ_GUARD_READY_FLAG != 0);
                    },
                }
                FutureRecv::Recv(Some(next_read_guard))
            },
            None => {
                // NOTE: self.state.load(SeqCst) & NEXT_READ_GUARD_READY_FLAG ?= 0
                if wait_for_frame {
                    // UNTESTED:
                    // println!("TractInner::recv: Waiting for next frame.");
                    let (tx, rx) = oneshot::channel();

                    let old_tx = self.recv_waiting.swap(tx, SeqCst);

                    assert!(old_tx.is_none(), "TractInner::recv: TractReceiver::recv has been \
                        called too many times in succession. If `wait_for_frame` is `true`, the \
                        caller MUST block its thread each call and must not send the `FutureRecv` \
                        to an event loop to be resolved.");

                    FutureRecv::Wait(rx)
                } else {
                    FutureRecv::Skip
                }
            }
        }
    }

    #[inline]
    pub fn buffer_idx_range(&self) -> Range<usize> {
        self.buffer_idx_range.clone()
    }

    /// Sets a new backpressure state and returns the prior (though
    /// immediately out-of-date) state.
    ///
    /// It is may be advisable to check the prior state to ensure that it
    /// matches up with expectations (another thread may also modifying
    /// it -- this may or may not be desirable).
    pub fn set_backpressure(&self, bp: bool) -> bool {
        let prior_state = if bp {
            self.state.fetch_or(BACKPRESSURE_FLAG, SeqCst)
        } else {
            self.state.fetch_and(!BACKPRESSURE_FLAG, SeqCst)
        };
        (prior_state & BACKPRESSURE_FLAG) != 0
    }

    /// Returns the current (immediately out-of-date) backpressure state.
    pub fn backpressure_stale(&self) -> bool {
        (self.state.load(SeqCst) & BACKPRESSURE_FLAG) != 0
    }
}

unsafe impl Send for TractInner {}
unsafe impl Sync for TractInner {}



#[derive(Debug)]
pub struct TractSender {
    inner: Arc<TractInner>,
}

impl TractSender {
    /// Sends the next buffer frame.
    #[inline]
    pub fn send(&self) -> FutureSend {
        self.inner.send()
    }

    #[inline] pub fn buffer_idx_range(&self) -> Range<usize> { self.inner.buffer_idx_range() }
    #[inline] pub fn backpressure_stale(&self) -> bool { self.inner.backpressure_stale() }
    #[inline] pub fn set_backpressure(&self, bp: bool) -> bool { self.inner.set_backpressure(bp) }
}


#[derive(Debug)]
pub struct TractReceiver {
    inner: Arc<TractInner>,
}

impl TractReceiver {
    /// Returns the next buffer frame.
    #[inline]
    pub fn recv(&self, wait_for_frame: bool) -> FutureRecv {
        self.inner.recv(wait_for_frame)
    }

    #[inline] pub fn buffer_idx_range(&self) -> Range<usize> { self.inner.buffer_idx_range() }
    #[inline] pub fn backpressure_stale(&self) -> bool { self.inner.backpressure_stale() }
    #[inline] pub fn set_backpressure(&self, bp: bool) -> bool { self.inner.set_backpressure(bp) }
}



pub fn tract_channel_single_i8(buffer: RwVec<i8>, buffer_idx_range: Option<Range<usize>>, backpressure: bool)
        -> (TractSender, TractReceiver) {
    let tract_buffer = TractBuffer::I8(TractBufferTyped::Single(buffer));
    let inner = Arc::new(TractInner::new(tract_buffer, buffer_idx_range, backpressure, Direction::SendRecv));
    (TractSender { inner: inner.clone() }, TractReceiver { inner })
}

pub fn tract_channel_single_u8(buffer: RwVec<u8>, buffer_idx_range: Option<Range<usize>>, backpressure: bool)
        -> (TractSender, TractReceiver) {
    let tract_buffer = TractBuffer::U8(TractBufferTyped::Single(buffer));
    let inner = Arc::new(TractInner::new(tract_buffer, buffer_idx_range, backpressure, Direction::SendRecv));
    (TractSender { inner: inner.clone() }, TractReceiver { inner })
}

pub fn tract_channel_single_u8_send_only(buffer: RwVec<u8>, buffer_idx_range: Option<Range<usize>>, backpressure: bool)
        -> (TractSender, TractReceiver) {
    let tract_buffer = TractBuffer::U8(TractBufferTyped::Single(buffer));
    let inner = Arc::new(TractInner::new(tract_buffer, buffer_idx_range, backpressure, Direction::SendOnly));
    (TractSender { inner: inner.clone() }, TractReceiver { inner })
}

pub fn tract_channel_single_u8_recv_only(buffer: RwVec<u8>, buffer_idx_range: Option<Range<usize>>, backpressure: bool)
        -> (TractSender, TractReceiver) {
    let tract_buffer = TractBuffer::U8(TractBufferTyped::Single(buffer));
    let inner = Arc::new(TractInner::new(tract_buffer, buffer_idx_range, backpressure, Direction::RecvOnly));
    (TractSender { inner: inner.clone() }, TractReceiver { inner })
}


// #[cfg(any(test, feature = "eval"))]
// mod tests {

//     #[test]
//     #[allow(non_snake_case)]

//     fn tract_channel_UNIMPLEMENTED() {

//     }
// }