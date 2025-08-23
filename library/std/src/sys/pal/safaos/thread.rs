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
    pub unsafe fn new(stack: usize, p: Box<dyn FnOnce()>) -> io::Result<Thread> {
        let stack_size = NonZero::new(stack.max(DEFAULT_MIN_STACK_SIZE));

        let raw_ptr = Box::into_raw(Box::new(p));
        #[unsafe(no_mangle)]
        extern "C" fn thread_start_inner(_cid: u32, main_fn: &'static Box<dyn FnOnce()>) -> ! {
            let main_fn: Box<Box<dyn FnOnce()>> =
                unsafe { Box::from_raw((main_fn as *const Box<dyn FnOnce()>).cast_mut()) };
            main_fn();
            // run all destructors
            unsafe { crate::sys::thread_local::destructors::run() };
            crate::rt::thread_cleanup();
            syscalls::thread::exit(0);
        }

        #[unsafe(naked)]
        extern "C" fn thread_start(cid: u32, main_fn: &'static Box<dyn FnOnce()>) -> ! {
            unsafe {
                #[cfg(target_arch = "x86_64")]
                core::arch::naked_asm!(
                    "
                and rsp, ~0xf
                push rbp
                push rbp
                mov rbp, rsp
                call thread_start_inner
                "
                );
                #[cfg(target_arch = "aarch64")]
                core::arch::naked_asm!(
                    "
                mov fp, #0
                sub sp, sp, #16
                stp xzr, xzr, [sp]
                bl thread_start_inner
                "
                );
            }
        }

        let cid = syscalls::thread::spawn(
            thread_start,
            unsafe { &*raw_ptr },
            safa_api::abi::process::RawContextPriority::Default,
            stack_size,
        );
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
