use core::ptr::NonNull;

use super::args::RAW_ARGS;
use super::args::RawArgs;
use super::os::exit;
use crate::mem::MaybeUninit;

extern "C" {
    fn main() -> u16;
}

unsafe fn _start_inner(argc: usize, argv: *mut (NonNull<u8>, usize)) -> ! {
    unsafe {
        let raw_args = match argc == 0 || argv.is_null() {
            true => None,
            false => {
                let args_slice = core::slice::from_raw_parts_mut(argv, argc);
                let args = NonNull::new_unchecked(args_slice as *mut _);
                Some(RawArgs { args })
            }
        };

        RAW_ARGS.get().write(MaybeUninit::new(raw_args));
        let results = main();

        exit(results as i32)
    }
}

#[no_mangle]
#[allow(unused)]
pub extern "C" fn _start(argc: usize, argv: *mut (NonNull<u8>, usize)) {
    unsafe {
        core::arch::asm!(
            "
            xor rbp, rbp
            push rbp
            push rbp
        ",
            options(nostack)
        );
        _start_inner(argc, argv);
    };
}
