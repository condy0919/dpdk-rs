use std::io;
use std::cmp;
use std::mem;
use std::ptr;
use std::ffi::CStr;

pub const DEFAULT_MIN_STACK_SIZE: usize = 2 * 1024 * 1024;

pub struct Thread {
    id: libc::pthread_t,
}

unsafe impl Send for Thread {}
unsafe impl Sync for Thread {}

impl Thread {
    extern "C" fn thread_start(main: *mut libc::c_void) -> *mut libc::c_void {
        unsafe {
            Box::from_raw(main as *mut Box<dyn FnOnce()>)();
        }
        ptr::null_mut()
    }

    pub unsafe fn new(stack: usize, p: Box<dyn FnOnce()>) -> io::Result<Thread> {
        let p = Box::new(p);

        let mut attr: libc::pthread_attr_t = mem::uninitialized();
        assert_eq!(libc::pthread_attr_init(&mut attr), 0);

        let stack_size = cmp::max(stack, DEFAULT_MIN_STACK_SIZE);
        assert_eq!(libc::pthread_attr_setstacksize(&mut attr, stack_size), 0);

        let mut native: libc::pthread_t = mem::uninitialized();
        let ret = libc::pthread_create(
            &mut native,
            &attr,
            Thread::thread_start,
            &*p as *const _ as *mut _,
        );
        assert_eq!(libc::pthread_attr_destroy(&mut attr), 0);

        return if ret != 0 {
            Err(io::Error::from_raw_os_error(ret))
        } else {
            mem::forget(p); // ownership passed to pthread_create
            Ok(Thread { id: native })
        };
    }

    pub fn set_name(s: &CStr) {
        unsafe {
            libc::pthread_setname_np(libc::pthread_self(), s.as_ptr());
        }
    }

    pub fn join(self) {
        let ret = unsafe {
            libc::pthread_join(self.id, ptr::null_mut())
        };
        mem::forget(self);
        assert!(ret == 0,
                "failed to join thread: {}", io::Error::from_raw_os_error(ret));
    }

    pub fn id(&self) -> libc::pthread_t {
        self.id
    }

    pub fn into_id(self) -> libc::pthread_t {
        let id = self.id;
        mem::forget(self);
        id
    }
}

impl Drop for Thread {
    fn drop(&mut self) {
        let ret = unsafe {
            libc::pthread_detach(self.id)
        };
        debug_assert_eq!(ret, 0);
    }
}
