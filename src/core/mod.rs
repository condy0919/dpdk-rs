use libc::{syscall, SYS_gettid};
use std::cell::RefCell;

pub mod cycles;
pub mod lcore;
pub mod log;
pub mod spinlock;

thread_local! {
    static CURRENT_TID: RefCell<i32> = RefCell::new(-1);
}

pub fn gettid() -> i32 {
    CURRENT_TID.with(|current| {
        if *current.borrow() == -1 {
            let tid = unsafe { syscall(SYS_gettid) as i32 };
            *current.borrow_mut() = tid;
        }

        *current.borrow()
    })
}
