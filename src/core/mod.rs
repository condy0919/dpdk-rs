use libc::{syscall, SYS_gettid};
use std::cell::Cell;
use std::io;
use std::os::unix::io::RawFd;

pub mod cycles;
pub mod lcore;
pub mod log;
pub mod rwlock;
pub mod spinlock;
pub mod thread;

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

#[doc(hidden)]
pub trait IsMinusOne {
    fn is_minus_one(&self) -> bool;
}

macro_rules! impl_is_minus_one {
    ($($t:ident)*) => {
        $(impl IsMinusOne for $t {
            fn is_minus_one(&self) -> bool {
                *self == -1
            }
        })*
    }
}

impl_is_minus_one! { i8 i16 i32 i64 isize }

#[inline]
pub fn cvt<T: IsMinusOne>(t: T) -> io::Result<T> {
    if t.is_minus_one() {
        Err(io::Error::last_os_error())
    } else {
        Ok(t)
    }
}

#[inline]
pub fn cvt_r<T: IsMinusOne, F: FnMut() -> T>(mut f: F) -> io::Result<T> {
    loop {
        match cvt(f()) {
            Err(ref e) if e.kind() == io::ErrorKind::Interrupted => {}
            other => return other,
        }
    }
}

#[inline]
pub fn read(fd: RawFd, buf: &mut [u8]) -> io::Result<usize> {
    cvt(unsafe { libc::read(fd, buf.as_mut_ptr() as *mut _, buf.len() as libc::size_t) })
        .map(|r| r as usize)
}

#[inline]
pub fn read_r(fd: RawFd, buf: &mut [u8]) -> io::Result<usize> {
    cvt_r(|| unsafe { libc::read(fd, buf.as_mut_ptr() as *mut _, buf.len() as libc::size_t) })
        .map(|r| r as usize)
}

#[inline]
pub fn write(fd: RawFd, buf: &[u8]) -> io::Result<usize> {
    let result = unsafe { libc::write(fd, buf.as_ptr() as *const _, buf.len() as libc::size_t) };
    cvt(result).map(|r| r as usize)
}

#[inline]
pub fn write_r(fd: RawFd, buf: &[u8]) -> io::Result<usize> {
    cvt_r(|| unsafe { libc::write(fd, buf.as_ptr() as *const _, buf.len() as libc::size_t) })
        .map(|r| r as usize)
}

#[inline]
pub fn close(fd: RawFd) -> io::Result<()> {
    let result = unsafe { libc::close(fd) };
    cvt(result).map(drop)
}
