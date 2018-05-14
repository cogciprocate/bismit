//! A work completion pool.
//
// Some of this is blatantly plagiarized from `futures-rs`.

// #![allow(unused_imports, unused_variables, dead_code)]

use std::sync::{Arc, Mutex};
use std::sync::mpsc::{Sender as StdSender, Receiver as StdReceiver};
use std::rc::Rc;
use std::cell::RefCell;
use std::thread::{self, JoinHandle, Thread};
use futures::{executor, SinkExt, StreamExt, Future, Never, Poll, Async, Stream, FutureExt};
use futures::stream::FuturesUnordered;
use futures::task::{self, Context, Waker, LocalMap, Wake};
use futures::executor::{enter, Executor, SpawnError};
use futures::channel::mpsc::{Sender as FuturesSender};

use cmn::completion_pool::unpark_mutex::UnparkMutex;

/// An error associated with `CompletionPool`.
#[derive(Debug, Fail)]
pub enum CompletionPoolError {
    #[fail(display = "{}", _0)]
    StdIo(#[cause] ::std::io::Error),
    #[fail(display = "{}", _0)]
    FuturesMpscSend(#[cause] ::futures::channel::mpsc::SendError),
    #[fail(display = "{:?}", _0)]
    FuturesSpawnError(::futures::executor::SpawnError)
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

impl From<::futures::executor::SpawnError> for CompletionPoolError {
    fn from(err: ::futures::executor::SpawnError) -> CompletionPoolError {
        CompletionPoolError::FuturesSpawnError(err)
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


/// Units of work submitted to a `WorkerPool`.
struct WorkerTask {
    spawn: Box<Future<Item = (), Error = Never> + Send>,
    map: LocalMap,
    exec: WorkerPool,
    wake_handle: Arc<WakeHandle>,
}


/// A wake handle.
struct WakeHandle {
    mutex: UnparkMutex<WorkerTask>,
    exec: WorkerPool,
}

impl Wake for WakeHandle {
    fn wake(arc_self: &Arc<Self>) {
        match arc_self.mutex.notify() {
            Ok(task) => arc_self.exec.inner.send(Command::Run(task)),
            Err(()) => {}
        }
    }
}


impl WorkerTask {
    /// Actually run the task (invoking `poll` on its future) on the current
    /// thread.
    pub fn run(self) {
        let WorkerTask { mut spawn, wake_handle, mut map, mut exec } = self;
        let waker = Waker::from(wake_handle.clone());

        // SAFETY: the ownership of this `WorkerTask` object is evidence that
        // we are in the `POLLING`/`REPOLL` state for the mutex.
        unsafe {
            wake_handle.mutex.start_poll();

            loop {
                let res = {
                    let mut cx = task::Context::new(&mut map, &waker, &mut exec);
                    spawn.poll(&mut cx)
                };
                match res {
                    Ok(Async::Pending) => {}
                    Ok(Async::Ready(())) => return wake_handle.mutex.complete(),
                    Err(never) => match never {},
                }
                let task = WorkerTask {
                    spawn,
                    map,
                    wake_handle: wake_handle.clone(),
                    exec: exec
                };
                match wake_handle.mutex.wait(task) {
                    Ok(()) => return,            // we've waited
                    Err(r) => { // someone's notified us
                        spawn = r.spawn;
                        map = r.map;
                        exec = r.exec;
                    }
                }
            }
        }
    }
}


/// A message request to a thread.
enum Command {
    Run(WorkerTask),
    Stop,
}


#[derive(Debug)]
struct WorkerPoolInner {
    tx: Mutex<StdSender<Command>>,
    rx: Mutex<StdReceiver<Command>>,
    size: usize,
    threads: Mutex<Vec<JoinHandle<()>>>,
}

impl WorkerPoolInner {
    fn new(size: usize) -> WorkerPoolInner {
        assert!(size > 0);
        let (tx, rx) = ::std::sync::mpsc::channel();

        WorkerPoolInner {
            tx: Mutex::new(tx),
            rx: Mutex::new(rx),
            size,
            threads: Mutex::new(Vec::with_capacity(size)),
        }
    }

    fn send(&self, cmd: Command) {
        self.tx.lock().unwrap().send(cmd).unwrap();
    }

    fn work(&self, _idx: usize) {
        let _scope = enter().unwrap();
        loop {
            let msg = self.rx.lock().unwrap().recv().unwrap();
            match msg {
                Command::Run(r) => r.run(),
                Command::Stop => break,
            }
        }
    }

    fn stop(&self) {
        for _ in 0..self.size {
            self.send(Command::Stop);
        }
    }

    fn join(&self) {
        let mut threads = self.threads.lock().unwrap();
        for thread in threads.drain(..) {
            thread.join().unwrap()
        }
    }
}


/// A bunch of threads that do stuff.
///
/// Blocks the dropping thread until work is complete.
#[derive(Clone, Debug)]
struct WorkerPool {
    inner: Arc<WorkerPoolInner>,
}

impl WorkerPool {
    pub fn new(size: usize) -> Result<WorkerPool, CompletionPoolError> {
        let pool = WorkerPool { inner: Arc::new(WorkerPoolInner::new(size)) };

        for idx in 0..size {
            let inner = pool.inner.clone();
            let mut thread : JoinHandle<_> = thread::Builder::new()
                .name(format!("completion_pool_thread-{}", idx))
                .spawn(move || inner.work(idx))?;

            let mut threads = pool.inner.threads.lock().unwrap();
            threads.push(thread);
        }

        Ok(pool)
    }
}

impl Executor for WorkerPool {
    fn spawn(&mut self, f: Box<Future<Item = (), Error = Never> + Send>) -> Result<(), SpawnError> {
        let task = WorkerTask {
            spawn: f,
            map: LocalMap::new(),
            wake_handle: Arc::new(WakeHandle {
                exec: self.clone(),
                mutex: UnparkMutex::new(),
            }),
            exec: self.clone(),
        };
        self.inner.send(Command::Run(task));
        Ok(())
    }
}


/// A work pool task.
struct CompletionTask {
    fut: Box<Future<Item = (), Error = Never> + Send>,
    map: LocalMap,
}

impl Future for CompletionTask {
    type Item = ();
    type Error = Never;

    fn poll(&mut self, cx: &mut Context) -> Poll<(), Never> {
        self.fut.poll(&mut cx.with_locals(&mut self.map))
    }
}


/// The event loop components of a `CompletionPool`.
struct CompletionPoolCore {
    pool: FuturesUnordered<CompletionTask>,
    incoming: Rc<RefCell<Vec<CompletionTask>>>,
    worker_pool: WorkerPool,
}

impl CompletionPoolCore {
    /// Create a new, empty work pool.
    fn new(worker_pool: WorkerPool) -> Result<CompletionPoolCore, CompletionPoolError> {
        Ok(CompletionPoolCore {
            pool: FuturesUnordered::new(),
            incoming: Default::default(),
            // FIXME: Use `num_cpus`.
            worker_pool,
        })
    }

    // Make maximal progress on the entire pool of spawned task, returning `Ready`
    // if the pool is empty and `Pending` if no further progress can be made.
    fn poll_pool(&mut self, waker: &Waker) -> Async<()> {
        // state for the FuturesUnordered, which will never be used
        let mut pool_map = LocalMap::new();
        let mut pool_cx = Context::new(&mut pool_map, waker, &mut self.worker_pool);

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

    fn run(&mut self) {
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
        let task = CompletionTask {
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
//
// TODO: Add a note comparing this to the tokio threadpool (lack of work
// stealing, performance, etc.).
pub struct CompletionPool {
    core_tx: Option<FuturesSender<Box<Future<Item = (), Error = Never> + Send>>>,
    core_thread: Option<JoinHandle<()>>,
    worker_pool: WorkerPool,
}

impl CompletionPool {
    /// Create a new, empty work pool.
    pub fn new(buffer_size: usize) -> Result<CompletionPool, CompletionPoolError> {
        // Allowing the channel to be the 'buffer' can cause deadlocks because
        // the tasks are polled one at a time, in order. Being out of order is
        // crucial:
        let (core_tx, core_rx) = ::futures::channel::mpsc::channel(0);
        let core_thread_pre = "completion_pool_thread-core".to_owned();
        let worker_pool = WorkerPool::new(4)?;
        let worker_pool_ref = worker_pool.clone();

        let core_thread: JoinHandle<_> = thread::Builder::new()
                .name(core_thread_pre)
                .spawn(move || {
            let mut core = CompletionPoolCore::new(worker_pool_ref).unwrap();
            let work = Box::new(core_rx.buffer_unordered(buffer_size).for_each(|_| Ok(())).map(|_| ()));
            core.spawn(work).unwrap();
            core.run();
        }).unwrap();

        Ok(CompletionPool {
            core_tx: Some(core_tx),
            core_thread: Some(core_thread),
            worker_pool,
        })
    }

    /// Submits a future which need only be polled to completion and that
    /// contains only trivial CPU work. Futures containing intensive work
    /// should be completed using `::complete_work` instead.
    // pub fn complete<F>(&mut self, future: F) -> Result<(), CompletionPoolError>
    //         where F: Future<Item = (), Error = Never> + Send + 'static {
    pub fn complete(&mut self, future: Box<Future<Item = (), Error = Never> + Send>)
            -> Result<(), CompletionPoolError> {
        let tx = self.core_tx.take().unwrap();
        // FIXME: Sending work should be done using `::try_send` and a loop
        // instead. (Loop while checking
        // `futures::channel::mpsc::TrySendError` `::is_full` and
        // `::is_disconnected`, yielding the thread if full, etc...)
        self.core_tx.get_or_insert(executor::block_on(tx.send(Box::new(future)))?);
        Ok(())
    }

    /// Polls a future which may contain non-trivial CPU work to completion.
    pub fn complete_work(&mut self, work: Box<Future<Item = (), Error = Never> + Send>)
            -> Result<(), CompletionPoolError> {
        self.worker_pool.spawn(work).map_err(|err| err.into())
    }
}

impl Drop for CompletionPool {
    /// Blocks the dropping thread until all submitted *and* all spawned work
    /// is complete.
    //
    // TODO: Guarantee above.
    fn drop(&mut self) {
        self.worker_pool.inner.stop();
        self.core_tx.take().unwrap().close_channel();
        self.core_thread.take().unwrap().join().expect("Error joining `CompletionPool` thread");
        self.worker_pool.inner.join();
    }
}