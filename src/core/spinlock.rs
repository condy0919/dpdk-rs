//! A spinlock & recursive spinlock.
//!
//! The `SpinLock&RecursiveSpinLock` provides the same interfaces except `is_locked`.
//!
//! The basic lock primitives:
//!
//! - `lock`
//! - `unlock`
//! - `trylock`
//!
//! # Example
//! ```
//! extern crate dpdk;
//!
//! use dpdk::core::spinlock;
//!
//! # fn main() {
//! let mut lk = spinlock::SpinLock::default();
//! let mut val = 0;
//!
//! lk.lock();
//! val += 1;
//! lk.unlock();
//!
//! assert_eq!(val, 1);
//!
//! # }
//! ```
//!
//! NOTE:
//! [TSX](https://gcc.gnu.org/onlinedocs/gcc-4.8.2/gcc/X86-transactional-memory-intrinsics.html)
//!
//!

use super::gettid;
use std::cell::UnsafeCell;
use std::os::raw::c_int;
use std::sync::atomic::{spin_loop_hint, AtomicI32, Ordering};
use std::thread::panicking;

extern "C" {
    fn rte_try_tm(lock: *mut i32) -> c_int;
    fn rte_xend();
}

/// The spinlock type
pub struct SpinLock {
    /// The lock state
    // 0 indicates unlocked; 1 indicates locked.
    // locked must be of 32bit size for RTM
    locked: UnsafeCell<AtomicI32>,
}

unsafe impl Sync for SpinLock {}
unsafe impl Send for SpinLock {}

impl Default for SpinLock {
    /// Construct the spinlock with unlocked state
    fn default() -> Self {
        SpinLock {
            locked: UnsafeCell::new(AtomicI32::new(0)),
        }
    }
}

impl Drop for SpinLock {
    fn drop(&mut self) {
        if self.is_locked() && !panicking() {
            panic!("spinlock still locked");
        }
    }
}

impl SpinLock {
    /// Take the spinlock
    pub fn lock(&self) {
        unsafe {
            if cfg!(feature = "tsx") {
                if rte_try_tm(self.locked.get() as *mut i32) == 1 {
                    return;
                }
            }

            while (*self.locked.get())
                .compare_exchange_weak(0, 1, Ordering::Acquire, Ordering::Relaxed)
                .is_err()
            {
                while (*self.locked.get()).load(Ordering::Relaxed) == 1 {
                    spin_loop_hint();
                }
            }
        }
    }

    /// Release the spinlock
    pub fn unlock(&self) {
        unsafe {
            if self.is_locked() {
                (*self.locked.get()).store(0, Ordering::Release);
            } else {
                rte_xend();
            }
        }
    }

    /// Try to take the spinlock
    pub fn trylock(&self) -> bool {
        unsafe {
            if cfg!(feature = "tsx") {
                if rte_try_tm(self.locked.get() as *mut i32) == 1 {
                    return true;
                }
            }

            (*self.locked.get())
                .compare_exchange_weak(0, 1, Ordering::Acquire, Ordering::Relaxed)
                .is_ok()
        }
    }

    /// Test if the lock is taked
    pub fn is_locked(&self) -> bool {
        unsafe { (*self.locked.get()).load(Ordering::Acquire) == 1 }
    }
}

/// The recursive spinlock type
pub struct RecursiveSpinLock {
    /// The actual spinlock
    lk: SpinLock,
    /// The thread id, -1 for unused
    tid: UnsafeCell<i32>,
    /// The count of times this lock has been called
    count: UnsafeCell<usize>,
}

unsafe impl Sync for RecursiveSpinLock {}
unsafe impl Send for RecursiveSpinLock {}

impl Default for RecursiveSpinLock {
    /// Construct the recursive spinlock with unlocked state
    fn default() -> Self {
        RecursiveSpinLock {
            lk: SpinLock::default(),
            tid: UnsafeCell::new(-1),
            count: UnsafeCell::new(0),
        }
    }
}

impl RecursiveSpinLock {
    /// Take the recursive spinlock
    pub fn lock(&self) {
        let id = gettid();

        unsafe {
            if cfg!(feature = "tsx") {
                if rte_try_tm(&self.lk as *const _ as *mut i32) == 1 {
                    return;
                }
            }

            if *self.tid.get() != id {
                self.lk.lock();
                *self.tid.get() = id;
            }

            *self.count.get() += 1;
        }
    }

    /// Release recursive spinlock
    pub fn unlock(&self) {
        unsafe {
            if self.lk.is_locked() {
                *self.count.get() -= 1;

                if *self.count.get() == 0 {
                    *self.tid.get() = -1;
                    self.lk.unlock();
                }
            } else {
                rte_xend();
            }
        }
    }

    /// Try to take the recursive spinlock
    pub fn trylock(&self) -> bool {
        let id = gettid();

        unsafe {
            if cfg!(feature = "tsx") {
                if rte_try_tm(&self.lk as *const _ as *mut i32) == 1 {
                    return true;
                }
            }

            if *self.tid.get() != id {
                if !self.lk.trylock() {
                    return false;
                }
                *self.tid.get() = id;
            }
            *self.count.get() += 1;
        }

        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic]
    fn spinlock_drop_when_locked() {
        let lk = SpinLock::default();

        lk.lock();
    }

    #[test]
    fn spinlock_lock_unlock() {
        let lk = SpinLock::default();
        assert!(!lk.is_locked());

        lk.lock();
        assert!(lk.is_locked());

        lk.unlock();
        assert!(!lk.is_locked());
    }

    #[test]
    fn spinlock_trylock() {
        let lk = SpinLock::default();
        assert!(!lk.is_locked());

        assert!(lk.trylock());
        assert!(lk.is_locked());

        assert!(!lk.trylock());

        lk.unlock();
        assert!(!lk.is_locked());
    }

    #[test]
    #[should_panic]
    fn recursive_spinlock_drop_when_locked() {
        let lk = RecursiveSpinLock::default();

        lk.lock();
    }

    #[test]
    fn recursive_spinlock_lock_unlock() {
        let lk = RecursiveSpinLock::default();

        lk.lock();
        lk.unlock();
    }
}
