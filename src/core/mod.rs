use libc::{syscall, SYS_gettid};
use std::cell::Cell;

pub mod cycles;
pub mod lcore;
pub mod log;
pub mod spinlock;
pub mod rwlock;

thread_local! {
    static CURRENT_TID: Cell<i32> = Cell::new(-1);
}

pub fn gettid() -> i32 {
    CURRENT_TID.with(|current| {
        if current.get() == -1 {
            let tid = unsafe { syscall(SYS_gettid) as i32 };
            current.set(tid);
        }

        current.get()
    })
}
