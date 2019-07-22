//! A rwlock
//!
//! The basic lock primitives:
//!
//! - `read_lock`
//! - `read_unlock`
//! - `write_lock`
//! - `write_unlock`
//!
//! # Example
//! ```
//! extern crate dpdk;
//!
//! use dpdk::core::rwlock;
//!
//! # fn main() {
//! let mut lk = rwlock::RwLock::default();
//! let mut val = 0;
//!
//! lk.read_lock();
//! println!("val = {}", val);
//! lk.read_unlock();
//!
//! lk.write_lock();
//! val += 10;
//! lk.write_unlock();
//!
//! lk.read_lock();
//! assert_eq!(val, 10);
//! lk.read_unlock();
//!
//! # }
//!
//!

use std::cell::UnsafeCell;
use std::os::raw::c_int;
use std::sync::atomic::{spin_loop_hint, AtomicI32, Ordering};
use std::thread::panicking;

extern "C" {
    fn rte_try_tm(lock: *mut i32) -> c_int;
    fn rte_xend();
}

/// The read/write lock type
///
/// cnt is -1 when write lock is held, and > 0 when read locks are held.
pub struct RwLock {
    cnt: UnsafeCell<AtomicI32>,
}

unsafe impl Sync for RwLock {}
unsafe impl Send for RwLock {}

impl Default for RwLock {
    fn default() -> Self {
        RwLock {
            cnt: UnsafeCell::new(AtomicI32::new(0)),
        }
    }
}

impl Drop for RwLock {
    fn drop(&mut self) {
        unsafe {
            if (*self.cnt.get()).load(Ordering::Relaxed) != 0 && !panicking() {
                panic!("rwlock still locked");
            }
        }
    }
}

impl RwLock {
    /// Take a read lock. Loop until the lock is held.
    pub fn read_lock(&self) {
        unsafe {
            if cfg!(feature = "tsx") {
                if rte_try_tm(self.cnt.get() as *mut i32) == 1 {
                    return;
                }
            }

            let mut success = false;
            while !success {
                let x = (*self.cnt.get()).load(Ordering::Relaxed);

                // write lock is held
                if x < 0 {
                    spin_loop_hint();
                    continue;
                }

                success = (*self.cnt.get())
                    .compare_exchange_weak(x, x + 1, Ordering::Acquire, Ordering::Relaxed)
                    .is_ok();
            }
        }
    }

    /// Release a read lock
    pub fn read_unlock(&self) {
        unsafe {
            if (*self.cnt.get()).load(Ordering::Relaxed) != 0 {
                (*self.cnt.get()).fetch_sub(1, Ordering::Release);
            } else {
                rte_xend();
            }
        }
    }

    /// Take a write lock. Loop until the lock is held.
    pub fn write_lock(&self) {
        unsafe {
            if cfg!(feature = "tsx") {
                if rte_try_tm(self.cnt.get() as *mut i32) == 1 {
                    return;
                }
            }

            let mut success = false;
            while !success {
                let x = (*self.cnt.get()).load(Ordering::Relaxed);

                // a lock is held
                if x != 0 {
                    spin_loop_hint();
                    continue;
                }

                success = (*self.cnt.get())
                    .compare_exchange_weak(x, -1, Ordering::Acquire, Ordering::Relaxed)
                    .is_ok();
            }
        }
    }

    /// Release a write lock
    pub fn write_unlock(&self) {
        unsafe {
            if (*self.cnt.get()).load(Ordering::Relaxed) != 0 {
                (*self.cnt.get()).store(0, Ordering::Release);
            } else {
                rte_xend();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::thread;

    #[test]
    fn rwlock_lock_unlock() {
        const NWORKER: usize = 2;

        let global = Arc::from(RwLock::default());
        let rwlks = vec![Arc::from(RwLock::default()); NWORKER];

        global.write_lock();

        let mut threads = Vec::new();
        for i in 0..NWORKER {
            let global_lk = global.clone();
            let local_lk = rwlks[i].clone();

            threads.push(thread::spawn(move || {
                global_lk.write_lock();
                global_lk.write_unlock();

                local_lk.write_lock();
                local_lk.write_unlock();
            }));
        }

        global.write_unlock();
        for h in threads {
            let _ = h.join();
        }
    }
}
