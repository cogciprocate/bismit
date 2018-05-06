#![allow(unused_imports, unused_variables, dead_code)]

use std::sync::Arc;
use std::rc::{Rc, Weak};
use std::cell::RefCell;
use std::thread::{self, JoinHandle, Thread};
use futures::{executor, SinkExt, StreamExt, Future, Never, Poll, Async, Stream, FutureExt};
use futures::stream::FuturesUnordered;
use futures::task::{Context, Waker, LocalMap, Wake};
use futures::executor::{enter, Executor, SpawnError, ThreadPool};
use futures::channel::mpsc::{self, Sender};


/// An error associated with `CompletionPool`.
#[derive(Debug, Fail)]
pub enum CompletionPoolError {
    #[fail(display = "{}", _0)]
    StdIo(#[cause] ::std::io::Error),
    #[fail(display = "{}", _0)]
    FuturesMpscSend(#[cause] ::futures::channel::mpsc::SendError),
}

impl From<::std::io::Error> for CompletionPoolError {
    fn from(err: ::std::io::Error) -> CompletionPoolError {
        CompletionPoolError::StdIo(err)
    }
}

impl From<::futures::channel::mpsc::SendError> for CompletionPoolError {
    fn from(err: ::futures::channel::mpsc::SendError) -> CompletionPoolError {
        CompletionPoolError::FuturesMpscSend(err)
    }
}


struct ThreadNotify {
    thread: Thread,
}

thread_local! {
    static CURRENT_THREAD_NOTIFY: Arc<ThreadNotify> = Arc::new(ThreadNotify {
        thread: thread::current(),
    });
}

impl ThreadNotify {
    fn with_current<R, F>(f: F) -> R
            where F: FnOnce(&Arc<ThreadNotify>) -> R {
        CURRENT_THREAD_NOTIFY.with(f)
    }

    fn park(&self) {
        thread::park();
    }
}

impl Wake for ThreadNotify {
    fn wake(arc_self: &Arc<Self>) {
        arc_self.thread.unpark();
    }
}


/// A work pool task.
struct Task {
    fut: Box<Future<Item = (), Error = Never>>,
    map: LocalMap,
}

impl Future for Task {
    type Item = ();
    type Error = Never;

    fn poll(&mut self, cx: &mut Context) -> Poll<(), Never> {
        self.fut.poll(&mut cx.with_locals(&mut self.map))
    }
}


/// The event loop components of a `CompletionPool`.
struct CompletionPoolCore {
    pool: FuturesUnordered<Task>,
    incoming: Rc<RefCell<Vec<Task>>>,
    thread_pool: ThreadPool,
}

impl CompletionPoolCore {
    /// Create a new, empty work pool.
    pub fn new() -> Result<CompletionPoolCore, CompletionPoolError> {
        Ok(CompletionPoolCore {
            pool: FuturesUnordered::new(),
            incoming: Default::default(),
            thread_pool: ThreadPool::builder()
                .name_prefix("completion_pool_thread-")
                .create()?,
        })
    }

    // Make maximal progress on the entire pool of spawned task, returning `Ready`
    // if the pool is empty and `Pending` if no further progress can be made.
    fn poll_pool(&mut self, waker: &Waker) -> Async<()> {
        // state for the FuturesUnordered, which will never be used
        let mut pool_map = LocalMap::new();
        let mut pool_cx = Context::new(&mut pool_map, waker, &mut self.thread_pool);

        loop {
            // empty the incoming queue of newly-spawned tasks
            {
                let mut incoming = self.incoming.borrow_mut();
                for task in incoming.drain(..) {
                    self.pool.push(task)
                }
            }

            if let Ok(ret) = self.pool.poll_next(&mut pool_cx) {
                // we queued up some new tasks; add them and poll again
                if !self.incoming.borrow().is_empty() {
                    continue;
                }

                // no queued tasks; we may be done
                match ret {
                    Async::Pending => return Async::Pending,
                    Async::Ready(None) => return Async::Ready(()),
                    _ => {}
                }
            }
        }
    }

    pub fn run(&mut self) {
        let _enter = enter().expect("cannot execute `CompletionPool` \
            executor from within another executor");

        ThreadNotify::with_current(|thread| {
            let waker = &Waker::from(thread.clone());
            loop {
                if let Async::Ready(t) = self.poll_pool(waker) {
                    return t;
                }
                thread.park();
            }
        })
    }

    fn spawn(&mut self, f: Box<Future<Item = (), Error = Never> + Send>) -> Result<(), SpawnError> {
        let task = Task {
            fut: f,
            map: LocalMap::new(),
        };

        self.incoming.borrow_mut().push(task);
        Ok(())
    }
}


/// A general purpose work completion pool.
///
/// Contains elements of a single-threaded event loop and a thread pool.
///
/// Runs in and manages its own threads. Dropping the `CompletionPool` will block
/// the dropping thread until all submitted and spawned work is complete.
pub struct CompletionPool {
    core_tx: Option<Sender<Box<Future<Item = (), Error = Never> + Send>>>,
    core_thread: Option<JoinHandle<()>>,
}

impl CompletionPool {
    /// Create a new, empty work pool.
    pub fn new(buffer_size: usize) -> Result<CompletionPool, CompletionPoolError> {
        // Allowing the channel size to take the place of buffer causes
        // deadlocks:
        let (core_tx, core_rx) = mpsc::channel(0);
        // let (core_tx, core_rx) = mpsc::channel(buffer_size);
        let core_thread_pre = "completion_pool_core-".to_owned();

        let core_thread: JoinHandle<_> = thread::Builder::new()
                .name(core_thread_pre).spawn(move || {
            let mut core = CompletionPoolCore::new().unwrap();
            // Allowing the channel size to take the place of buffer causes
            // deadlocks:
            let work = Box::new(core_rx.buffer_unordered(buffer_size).for_each(|_| Ok(())).map(|_| ()));
            core.spawn(work).unwrap();
            core.run();
        }).unwrap();

        Ok(CompletionPool {
            core_tx: Some(core_tx),
            core_thread: Some(core_thread),
        })
    }

    /// Submits a future which need only be polled to completion and that
    /// contains no intensive CPU work (including memcpy).
    pub fn complete<F>(&mut self, future: F) -> Result<(), CompletionPoolError>
            where F: Future<Item = (), Error = Never> + Send + 'static {
        let tx = self.core_tx.take().unwrap();
        self.core_tx.get_or_insert(executor::block_on(tx.send(Box::new(future)))?);
        Ok(())
    }

    /// Polls a future which may contain non-trivial CPU work to completion.
    pub fn complete_work<F>(&mut self, work: F) -> Result<(), CompletionPoolError>
            where F: Future<Item = (), Error = Never> + Send + 'static {
        // let future = self.cpu_pool.spawn(work);
        // self.complete(future)
        self.complete(work)
    }
}

impl Drop for CompletionPool {
    /// Blocks the dropping thread until all submitted *and* all spawned work
    /// is complete.
    //
    // TODO: Guarantee above.
    fn drop(&mut self) {
        self.core_tx.take().unwrap().close_channel();
        self.core_thread.take().unwrap().join().expect("Error joining `CompletionPool` thread");
    }
}