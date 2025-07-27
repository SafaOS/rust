use safa_api::abi::process::AbiStructures;
use safa_api::ffi::{slice::Slice, str::Str};
use safa_api::syscalls;

unsafe extern "C" {
    fn main() -> u16;
}

unsafe fn _start_inner(
    argc: usize,
    argv: *mut Str,
    envc: usize,
    envp: *mut Slice<u8>,
    task_abi_structures: *const AbiStructures,
) -> ! {
    unsafe {
        let args = Slice::from_raw_parts(argv, argc);
        let env = Slice::from_raw_parts(envp, envc);
        safa_api::process::init::sysapi_init(args, env, *task_abi_structures);
        let results = main();

        syscalls::process::exit(results as usize);
    }
}

#[unsafe(no_mangle)]
#[allow(unused)]
pub extern "C" fn _start(
    argc: usize,
    argv: *mut Str,
    envc: usize,
    envp: *mut Slice<u8>,
    task_abi_structures: *const AbiStructures,
) {
    unsafe {
        #[cfg(target_arch = "x86_64")]
        core::arch::asm!(
            "
            xor rbp, rbp
            push rbp
            push rbp
        ",
        );
        #[cfg(target_arch = "aarch64")]
        core::arch::asm!(
            "
            mov fp, #0
            sub sp, sp, #16
            stp xzr, xzr, [sp]
            "
        );
        _start_inner(argc, argv, envc, envp, task_abi_structures);
    };
}
