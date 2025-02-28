use crate::ffi::{c_char, c_int};
use crate::ptr;

extern "C" {
    fn main(argc: c_int, argv: *const *const c_char) -> c_int;
}

#[no_mangle]
#[allow(unused)]
pub extern "C" fn _start() {
    unsafe {
        core::arch::asm!(
            "
            xor rbp, rbp
            push rbp
            push rbp
        ",
            options(nostack)
        );
        main(0, ptr::null());
        core::arch::asm!(
            "
            xor rax, rax
            xor rdi, rdi
            int 0x80
        ",
            options(noreturn)
        );
    };
}
