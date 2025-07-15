use safa_api::syscalls;

use super::unsupported;
use crate::ffi::CStr;
use crate::io;
use crate::num::NonZero;
use crate::time::Duration;

pub struct Thread(u32);

pub const DEFAULT_MIN_STACK_SIZE: usize = 64 * 1024;

impl Thread {
    // unsafe: see thread::Builder::spawn_unchecked for safety requirements
    pub unsafe fn new(_stack: usize, p: Box<dyn FnOnce()>) -> io::Result<Thread> {
        let raw_ptr = Box::into_raw(Box::new(p));
        fn thread_start(_cid: u32, main_fn: &'static Box<dyn FnOnce()>) -> ! {
            let main_fn: Box<Box<dyn FnOnce()>> =
                unsafe { Box::from_raw((main_fn as *const Box<dyn FnOnce()>).cast_mut()) };
            main_fn();
            syscalls::thread::exit(0);
        }

        let cid = syscalls::thread::spawn(thread_start, unsafe { &*raw_ptr }, None);
        match cid {
            Ok(cid) => Ok(Thread(cid)),
            Err(e) => {
                unsafe {
                    _ = Box::from_raw(raw_ptr);
                };
                Err(e.into())
            }
        }
    }

    pub fn yield_now() {
        syscalls::thread::yield_now();
    }

    pub fn set_name(_name: &CStr) {
        // nope
    }

    pub fn sleep(dur: Duration) {
        syscalls::thread::sleep(dur).expect("sleep duration is too long");
    }

    pub fn join(self) {
        syscalls::thread::wait(self.0).expect("failed to join on thread");
    }
}

pub fn available_parallelism() -> io::Result<NonZero<usize>> {
    unsupported()
}
