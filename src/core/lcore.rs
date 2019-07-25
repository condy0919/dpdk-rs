//! A logical core based on threads.
//!
//! ## The lcore model
//! It's based on threads.
//!
//! ## Spawning a lcore
//!
//! A new lcore can be spawned using the [`lcore::spawn`] function:
//!
//! ```
//! use dpdk::core::lcore;
//!
//! lcore::spawn::<()>();
//! ```
//!
//! In this example, the spawned lcore runs in a endless loop receiving command
//! and executing.
//!
//! The parent thread can [`launch`] specific tasks and [`Wait::wait`] for result:
//!
//! ```
//! use dpdk::core::lcore;
//!
//! let lc = lcore::spawn::<String>();
//!
//! let res = lc.launch(|| {
//!     // some work here
//!     "result".into()
//! })
//! .unwrap()
//! .wait();
//! ```
//!
//! The [`launch`] method return a [`Wait`] containing the result produced by
//! the task or a failure.
//!
//! ## Configuring lcore
//!
//! A new lcore can be configured before it is spawned via the [`Builder`] type,
//! which currently allows you to set the name and thread affinity:
//!
//! ```
//! use dpdk::core::lcore;
//!
//! lcore::Builder::new()
//!     .name("lcore255".into())
//!     .affinity(&[1])
//!     .spawn::<()>()
//!     .unwrap();
//! ```
//!
//! [`lcore::spawn`]: fn.spawn.html
//! [`launch`]: struct.LCore.html#method.launch
//! [`Wait`]: struct.Wait.html
//! [`Wait::wait`]: struct.Wait.html#method.wait
//! [`Builder`]: struct.Builder.html

use crate::core::{cvt, read_r, write_r, thread};
use std::io;
use std::mem;
use std::panic;
use std::result;
use std::any::Any;
use std::sync::Arc;
use std::cell::UnsafeCell;
use std::os::unix::io::RawFd;
use std::sync::atomic::{AtomicUsize, Ordering, fence, spin_loop_hint};

/// A specialized `Result` type for lcore.
///
/// Indicates the manner in which a lcore task exited.
///
/// A lcore task that completes without panicking is considered to exit successfully.
///
/// # Exaples
///
/// ```no_run
/// use dpdk::core::lcore;
///
/// fn main() {
///     let lc = lcore::spawn::<()>();
///
///     let res = lc.launch(|| {
///         panic!("panic");
///     })
///     .unwrap()
///     .wait();
///
///     match res {
///         Ok(_) => println!("success"),
///         Err(_) => println!("panicked"),
///     }
/// }
/// ```
pub type Result<T> = result::Result<T, Box<dyn Any + Send + 'static>>;

/// A logical core may outlive the caller (unless the caller thread is the main
/// thread.
/// You can [`launch`] tasks on it and get the result via [`wait`].
///
/// [`launch`]: struct.LCore.html#method.launch
/// [`wait`]: struct.Wait.html#method.wait
pub struct LCore<R> {
    thread: thread::Thread,  // the native thread
    send_efd: RawFd,         // communication eventfd with master
    ack_efd: RawFd,          // communication eventfd with master
    state: Arc<AtomicUsize>, // thread state
    func: FuncPacket<R>,     // function to call
    packet: Packet<R>,       // return value of function
}

unsafe impl<R> Send for LCore<R> {}
unsafe impl<R> Sync for LCore<R> {}

impl<R> LCore<R> {
    /// Launch a task and returns an `io::Result`.
    pub fn launch<'a, F: FnOnce() -> R + 'static>(&'a self, f: F) -> io::Result<Wait<'a, R>> {
        if self.state.load(Ordering::Relaxed) != State::WAIT as usize {
            return Err(io::Error::from_raw_os_error(libc::EBUSY));
        }

        unsafe {
            (*self.func.0.get()).replace(Box::new(f));
        }

        // send message
        let mut dummy = [1u8, 0, 0, 0, 0, 0, 0, 0];
        write_r(self.send_efd, &dummy)
            .expect("cannot write on eventfd with slave");

        // wait ack
        read_r(self.ack_efd, &mut dummy)
            .expect("cannot read on eventfd with slave");

        Ok(Wait {
            lcore: self
        })
    }

    /// The native thread id
    ///
    /// # Examples
    ///
    /// ```
    /// use dpdk::core::lcore;
    ///
    /// let lc0 = lcore::spawn::<()>();
    /// let lc1 = lcore::spawn::<()>();
    /// assert_ne!(lc0.id(), lc1.id());
    /// ```
    pub fn id(&self) -> u64 {
        self.thread.id()
    }
}

/// The continuation of `launch`ed task.
pub struct Wait<'a, T> {
    lcore: &'a LCore<T>,
}

impl<'a, T> Wait<'a, T> {
    /// Wait for lcore task to complete
    pub fn wait(&self) -> Result<T> {
        if self.lcore.state.load(Ordering::Relaxed) == State::WAIT as usize {
            return Err(Box::new("lcore in WAIT state"));
        }

        while self.lcore.state.load(Ordering::Relaxed) == State::RUNNING as usize {
            spin_loop_hint();
        }

        fence(Ordering::Acquire);
        self.lcore.state.store(State::WAIT as usize, Ordering::Relaxed);

        unsafe {
            (*self.lcore.packet.0.get())
                .take()
                .unwrap()
        }
    }
}


/// LCore factory, wihch can be used in order to conigure the properties of
/// a new lcore.
///
/// Methods can be chained on it in order to configured it.
///
/// The two configuratoins available are:
///
/// - [`name`]: specifies an associated name for the lcore
/// - [`affinity`]: specifies the cpu cores which a lcore runs on
///
/// The [`spawn`] method will take ownership of the builder and create an
/// `io::Result`.
///
/// The [`lcore::spawn`] free function uses a `Builder` with default
/// configuration and [`unwrap`]s its return value.
///
/// You may want to use [`spawn`] instead of [`lcore::spawn`], when you want to
/// recover from a failure to launch a lcore, indeed the free function panic
/// where the `Builder` method will return an `io::Result`.
///
/// # Examples
///
/// ```
/// use dpdk::core::lcore;
///
/// let builder = lcore::Builder::new();
///
/// let lc = builder.spawn::<()>().unwrap();
/// ```
///
/// [`name`]: struct.Builder.html#method.name
/// [`affinity`]: struct.Builder.html#method.affinity
/// [`spawn`]: struct.Builder.html#method.spawn
/// [`lcore::spawn`]: fn.spawn.html
pub struct Builder {
    name: Option<String>,            // thread's name. Guaranteed to be UTF-8
    cpuset: Option<libc::cpu_set_t>, // cpu set which the thread affinity to
}

impl Builder {
    /// Generates the base configuration for spawning a lcore, from which
    /// configuration methods can be chained.
    ///
    /// # Examples
    ///
    /// ```
    /// use dpdk::core::lcore;
    ///
    /// let builder = lcore::Builder::new()
    ///     .name("foo".into())
    ///     .affinity(&[0]);
    ///
    /// let lc = builder.spawn::<()>().unwrap();
    /// ```
    pub fn new() -> Builder {
        Builder {
            name: None,
            cpuset: None,
        }
    }

    /// Names the lcore-to-be. The name can be used for identification when
    /// listing all threads (`ps -efL` in unix-like platforms).
    ///
    /// The name must not contain null bytes (`\0`).
    ///
    /// # Examples
    ///
    /// ```
    /// use std::mem;
    /// use dpdk::core::lcore;
    ///
    /// let builder = lcore::Builder::new()
    ///     .name("lcore0".into());
    ///
    /// let lc = builder.spawn::<String>().unwrap();
    ///
    /// let res = lc.launch(|| unsafe {
    ///     let mut buff = [0u8; 32];
    ///     libc::pthread_getname_np(
    ///         libc::pthread_self(),
    ///         buff.as_mut_ptr() as *mut _,
    ///         mem::size_of_val(&buff),
    ///     );
    ///     String::from_utf8_unchecked(buff[..6].to_vec())
    /// })
    /// .unwrap()
    /// .wait()
    /// .unwrap();
    ///
    /// assert_eq!(res, "lcore0");
    /// ```
    pub fn name(mut self, name: String) -> Builder {
        self.name = Some(name);
        self
    }

    /// Sets the cpu affinity for the new lcore.
    ///
    /// No assumption shoule be made about particular CPUs being available, or
    /// the set of CPUs being contiguous, since CPUs can be taken offline
    /// dynyamically or be otherwise absent.
    ///
    /// Any value larger than CPU_SETSIZE (currently 1024) makes no effect.
    ///
    /// ```
    /// use dpdk::core::lcore;
    ///
    /// let lc = lcore::Builder::new()
    ///     .affinity(&[0])
    ///     .spawn::<i32>()
    ///     .unwrap();
    ///
    /// let res = lc.launch(|| unsafe {
    ///     libc::sched_getcpu()
    /// })
    /// .unwrap()
    /// .wait()
    /// .unwrap();
    ///
    /// assert_eq!(res, 0);
    /// ```
    pub fn affinity(mut self, cpuvec: &[usize]) -> Builder {
        let mut cpuset: libc::cpu_set_t = unsafe {
            mem::MaybeUninit::zeroed().assume_init()
        };
        unsafe {
            libc::CPU_ZERO(&mut cpuset);

            for &cpu in cpuvec.iter() {
                libc::CPU_SET(cpu, &mut cpuset);
            }
        }

        self.cpuset = Some(cpuset);
        self
    }

    /// Spawns a new lcore by taking ownership of the [`Builder`], and return an
    /// `io::Result` to [`LCore`].
    ///
    /// The spawned lcore may outlive the caller (unless the caller thread
    /// is the main thread. There's no `join` method because the `lcore` runs
    /// in a endless loop until the main thread finished).
    ///
    /// # Errors
    ///
    /// Unlike the [`spawn`] free function, this method yeilds an
    /// `io::Result` to capture any failure when creating the thread at the OS level.
    ///
    /// [`Builder`]: struct.Builder.html
    /// [`LCore`]: struct.LCore.html
    /// [`spawn`]: fn.spawn.html
    ///
    /// # Panics
    ///
    /// Panics if a lcore name was set and it contained null bytes.
    ///
    /// # Examples
    ///
    /// ```
    /// use dpdk::core::lcore;
    ///
    /// let builder = lcore::Builder::new();
    ///
    /// let lc = builder.spawn::<()>().unwrap();
    /// ```
    pub fn spawn<R: Send + 'static>(self) -> io::Result<LCore<R>> {
        unsafe { self.spawn_unchecked() }
    }

    unsafe fn spawn_unchecked<R: Send + 'static>(self) -> io::Result<LCore<R>> {
        let Builder { name, cpuset } = self;

        let send_efd = cvt(libc::eventfd(0, libc::EFD_CLOEXEC))?;
        let ack_efd = cvt(libc::eventfd(0, libc::EFD_CLOEXEC))?;

        let my_state = Arc::new(AtomicUsize::new(State::WAIT as usize));
        let their_state = my_state.clone();

        let my_func = Arc::new(UnsafeCell::new(None));
        let their_func = my_func.clone();

        let my_packet = Arc::new(UnsafeCell::new(None));
        let their_packet = my_packet.clone();

        let main = move || {
            if let Some(s) = name {
                libc::pthread_setname_np(libc::pthread_self(), s.as_ptr() as *const _);
            }

            if let Some(cpuset) = cpuset {
                libc::pthread_setaffinity_np(
                    libc::pthread_self(),
                    mem::size_of_val(&cpuset),
                    &cpuset as *const _,
                );
            }

            let mut dummy = [0u8; 8];
            loop {
                let func = their_func.clone();
                let packet = their_packet.clone();

                // wait command
                read_r(send_efd, &mut dummy)
                    .expect("cannot read on eventfd with master");

                their_state.store(State::RUNNING as usize, Ordering::Relaxed);

                // send ack
                write_r(ack_efd, &dummy)
                    .expect("cannot write on eventfd with master");

                // call the function and store the return value
                if let Some(f) = (*func.get()).take() {
                    let result = panic::catch_unwind(panic::AssertUnwindSafe(f));
                    *packet.get() = Some(result);
                }

                fence(Ordering::Release);

                their_state.store(State::FINISHED as usize, Ordering::Relaxed);
            }
        };

        Ok(LCore {
            thread: thread::Thread::new(
                thread::DEFAULT_MIN_STACK_SIZE,
                Box::new(main)
            )?,
            send_efd: send_efd,
            ack_efd: ack_efd,
            state: my_state,
            func: FuncPacket(my_func),
            packet: Packet(my_packet),
        })
    }
}

/// Spawns a new lcore, returning a [`LCore`].
///
/// [`LCore`]: struct.LCore.html
pub fn spawn<R: Send + 'static>() -> LCore<R> {
    Builder::new().spawn().expect("failed to spawn lcore")
}

/// State of an lcore
#[repr(usize)]
enum State {
    /// waiting a new command
    WAIT,
    /// executing command
    RUNNING,
    /// command executed
    FINISHED,
}

// This packet is used to communicate the return value between the child thread
// and the parent thread. Memory is shared through the `Arc` within.
// There is no need for a mutex because synchronization happens with `wait()`
//
struct Packet<T>(Arc<UnsafeCell<Option<Result<T>>>>);

unsafe impl<T: Send> Send for Packet<T> {}
unsafe impl<T: Sync> Sync for Packet<T> {}

// This packet is used to communicate the lcore task (`FnOnce() -> R`) between
// the child thread and the parent thread. Memory is shared through the `Arc`
// within.
// Synchronization via `eventfd`.
struct FuncPacket<R>(Arc<UnsafeCell<Option<Box<dyn FnOnce() -> R>>>>);

unsafe impl<R> Send for FuncPacket<R> {}
unsafe impl<R> Sync for FuncPacket<R> {}
