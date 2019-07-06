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
//! [TSX](https://gcc.gnu.org/onlinedocs/gcc-4.8.2/gcc/X86-transactional-memory-intrinsics.html) supports will be available whem `asm!` is stable.
//!
//!

use std::sync::atomic::{spin_loop_hint, AtomicBool, Ordering};
use std::cell::UnsafeCell;
use super::gettid;

/// The spinlock type
pub struct SpinLock {
    /// The lock state
    ///
    /// false indicates unlocked
    /// true indicates locked
    locked: UnsafeCell<AtomicBool>,
}

unsafe impl Sync for SpinLock {}
unsafe impl Send for SpinLock {}

impl Default for SpinLock {
    /// Construct the spinlock with unlocked state
    fn default() -> Self {
        SpinLock {
            locked: UnsafeCell::new(AtomicBool::new(false)),
        }
    }
}

impl Drop for SpinLock {
    fn drop(&mut self) {
        if self.is_locked() {
            panic!("spinlock still locked");
        }
    }
}

impl SpinLock {
    /// Take the spinlock
    pub fn lock(&self) {
        while self
            .get_lock_mut()
            .compare_exchange_weak(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_err()
        {
            while self.get_lock_mut().load(Ordering::Relaxed) {
                spin_loop_hint();
            }
        }
    }

    /// Release the spinlock
    pub fn unlock(&self) {
        self.get_lock_mut().store(false, Ordering::Release);
    }

    /// Try to take the spinlock
    pub fn trylock(&self) -> bool {
        self.get_lock_mut()
            .compare_exchange_weak(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_ok()
    }

    /// Test if the lock is taked
    pub fn is_locked(&self) -> bool {
        self.get_lock_mut().load(Ordering::Acquire)
    }

    fn get_lock_mut(&self) -> &mut AtomicBool {
        unsafe {
            &mut *self.locked.get()
        }
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

        if *self.get_tid_mut() != id {
            self.lk.lock();
            *self.get_tid_mut() = id;
        }
        *self.get_count_mut() += 1;
    }

    /// Release recursive spinlock
    pub fn unlock(&self) {
        *self.get_count_mut() -= 1;

        if *self.get_count_mut() == 0 {
            *self.get_tid_mut() = -1;
            self.lk.unlock();
        }
    }

    /// Try to take the recursive spinlock
    pub fn trylock(&self) -> bool {
        let id = gettid();

        if *self.get_tid_mut() != id {
            if !self.lk.trylock() {
                return false;
            }
            *self.get_tid_mut() = id;
        }
        *self.get_count_mut() += 1;

        true
    }

    fn get_tid_mut(&self) -> &mut i32 {
        unsafe {
            &mut *self.tid.get()
        }
    }

    fn get_count_mut(&self) -> &mut usize {
        unsafe {
            &mut *self.count.get()
        }
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
