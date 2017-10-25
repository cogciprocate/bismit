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
use std::cell::UnsafeCell;
use std::thread;
use futures::{Async, Poll};
use futures::sync::oneshot::{self, Sender, Receiver, Canceled};
use crossbeam::sync::AtomicOption;
use futures::{Future};
use ocl::{RwVec, FutureReadGuard, FutureWriteGuard, OclPrm};
use cmn::CmnError;


const NEXT_READ_GUARD_READY: usize = 0x00000001;
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

/// Spins an ever increasing amount of time.
fn _spin(spins: &mut usize) {
    if *spins < 16 {
        for _ in 0..(2 << *spins) { fence(Ordering::SeqCst); }
    } else {
        thread::yield_now();
    }
    *spins += 1;
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
    send_waiting: AtomicOption<(Sender<()>, ReadBuffer)>,
    recv_waiting: AtomicOption<Sender<ReadBuffer>>,
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
            send_waiting: AtomicOption::new(),
            recv_waiting: AtomicOption::new(),
        }
    }

    fn send(&self) -> FutureSend {
        let cur_state = self.state.fetch_or(NEXT_READ_GUARD_READY, Ordering::SeqCst);
        let buffer_already_ready = (cur_state & NEXT_READ_GUARD_READY) != 0;
        if buffer_already_ready {
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
            match self.recv_waiting.take(Ordering::SeqCst) {
                Some(tx) => {
                    // println!("TractInner::send: Read guard stale. Sending new...");
                    tx.send(rg).unwrap();
                },
                None => {
                    // println!("TractInner::send: Read guard stale. Swapping in new.");
                    let old_rg = self.next_read_guard.swap(rg, Ordering::SeqCst);
                    assert!(old_rg.is_none());
                },
            }
            FutureSend::Send(Some(wg))
        }
    }

    // `wait_for_frame` untested.
    fn recv(&self, wait_for_frame: bool) -> FutureRecv {
        match self.next_read_guard.take(Ordering::SeqCst) {
            Some(next_read_guard) => {
                assert!(self.state.load(Ordering::SeqCst) & NEXT_READ_GUARD_READY != 0);
                // Rotate in the waiting guard if any:
                match self.send_waiting.take(Ordering::SeqCst) {
                    Some((tx, wrg)) => {
                        // println!("TractInner::recv: self.send_waiting => Some(tx, wrg)");
                        self.next_read_guard.swap(wrg, Ordering::SeqCst);
                        tx.send(()).ok();
                    },
                    None => {
                        // println!("TractInner::recv: self.send_waiting => None");
                        let prior_state = self.state.fetch_and(!NEXT_READ_GUARD_READY, Ordering::SeqCst);
                        assert!(prior_state & NEXT_READ_GUARD_READY != 0);
                    },
                }
                FutureRecv::Recv(Some(next_read_guard))
            },
            None => {
                // NOTE: self.state.load(Ordering::SeqCst) & NEXT_READ_GUARD_READY ??= 0 //
                if wait_for_frame {
                    // UNTESTED:
                    // println!("TractInner::recv: Waiting for next frame.");
                    let (tx, rx) = oneshot::channel();
                    let old_tx = self.recv_waiting.swap(tx, Ordering::SeqCst);
                    assert!(old_tx.is_none());
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
    /// Sends the next buffer frame.
    #[inline]
    pub fn send(&self) -> FutureSend {
        self.inner.send()
    }
}

impl Deref for TractSender {
    type Target = TractInner;

    #[inline]
    fn deref(&self) -> &TractInner {
        &self.inner
    }
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
}

impl Deref for TractReceiver {
    type Target = TractInner;

    #[inline]
    fn deref(&self) -> &TractInner {
        &self.inner
    }
}


pub fn tract_channel_single_i8(buffer: RwVec<i8>, buffer_idx_range: Range<usize>, backpressure: bool)
        -> (TractSender, TractReceiver)
{
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


#[cfg(test)]
mod tests {

    #[test]
    fn tract_channel() {

    }
}