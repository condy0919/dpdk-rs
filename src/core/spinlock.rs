use std::sync::atomic::{spin_loop_hint, AtomicBool, Ordering};
use super::gettid;

/// The spinlock type
pub struct SpinLock {
    /// The lock state
    ///
    /// false indicates unlocked
    /// true indicates locked
    locked: AtomicBool,
}

impl Default for SpinLock {
    /// Construct the spinlock with unlocked state
    fn default() -> Self {
        SpinLock {
            locked: AtomicBool::new(false),
        }
    }
}

impl Drop for SpinLock {
    fn drop(&mut self) {
        if self.is_locked() {
            panic!("spinlock unlocked");
        }
    }
}

impl SpinLock {
    /// Take the spinlock
    pub fn lock(&mut self) {
        while self
            .locked
            .compare_exchange_weak(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_err()
        {
            while self.locked.load(Ordering::Relaxed) {
                spin_loop_hint();
            }
        }
    }

    /// Release the spinlock
    pub fn unlock(&mut self) {
        self.locked.store(false, Ordering::Release);
    }

    /// Try to take the spinlock
    pub fn trylock(&mut self) -> bool {
        self.locked
            .compare_exchange_weak(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_ok()
    }

    /// Test if the lock is taked
    pub fn is_locked(&self) -> bool {
        self.locked.load(Ordering::Acquire)
    }
}


/// The recursive spinlock type
pub struct RecursiveSpinLock {
    /// The actual spinlock
    lk: SpinLock,
    /// The thread id, -1 for unused
    tid: i32,
    /// The count of times this lock has been called
    count: usize,
}

impl Default for RecursiveSpinLock {
    /// Construct the recursive spinlock with unlocked state
    fn default() -> Self {
        RecursiveSpinLock {
            lk: SpinLock::default(),
            tid: -1,
            count: 0,
        }
    }
}

impl RecursiveSpinLock {
    /// Take the recursive spinlock
    pub fn lock(&mut self) {
        let id = gettid();

        if self.tid != id {
            self.lk.lock();
            self.tid = id;
        }
        self.count += 1;
    }

    /// Release recursive spinlock
    pub fn unlock(&mut self) {
        self.count -= 1;

        if self.count == 0 {
            self.tid = -1;
            self.lk.unlock();
        }
    }

    /// Try to take the recursive spinlock
    pub fn trylock(&mut self) -> bool {
        let id = gettid();

        if self.tid != id {
            if !self.lk.trylock() {
                return false;
            }
            self.tid = id;
        }
        self.count += 1;

        true
    }
}

#[cfg(test)]
mod tests {

}
