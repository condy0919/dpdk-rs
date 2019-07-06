use std::cell::UnsafeCell;
use std::sync::atomic::{spin_loop_hint, AtomicI32, Ordering};

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
        if self.get_cnt_mut().load(Ordering::Relaxed) != 0 {
            panic!("rwlock still locked");
        }
    }
}

impl RwLock {
    /// Take a read lock. Loop until the lock is held.
    pub fn read_lock(&self) {
        let mut success = false;
        while !success {
            let x = self.get_cnt_mut().load(Ordering::Relaxed);

            // write lock is held
            if x < 0 {
                spin_loop_hint();
                continue;
            }

            success = self
                .get_cnt_mut()
                .compare_exchange_weak(x, x + 1, Ordering::Acquire, Ordering::Relaxed)
                .is_ok();
        }
    }

    /// Release a read lock
    pub fn read_unlock(&self) {
        self.get_cnt_mut().fetch_sub(1, Ordering::Release);
    }

    /// Take a write lock. Loop until the lock is held.
    pub fn write_lock(&self) {
        let mut success = false;
        while !success {
            let x = self.get_cnt_mut().load(Ordering::Relaxed);

            // a lock is held
            if x != 0 {
                spin_loop_hint();
                continue;
            }

            success = self
                .get_cnt_mut()
                .compare_exchange_weak(x, -1, Ordering::Acquire, Ordering::Relaxed)
                .is_ok();
        }
    }

    /// Release a write lock
    pub fn write_unlock(&self) {
        self.get_cnt_mut().store(0, Ordering::Release);
    }

    fn get_cnt_mut(&self) -> &mut AtomicI32 {
        unsafe { &mut *self.cnt.get() }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::sync::Arc;

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
                println!("global write lock taken on thread {}", i);
                global_lk.write_unlock();

                local_lk.write_lock();
                println!("hello from thread {}", i);
                local_lk.write_unlock();
            }));
        }

        global.write_unlock();
        for h in threads {
            let _ = h.join();
        }
    }
}
