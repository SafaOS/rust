use safa_api::raw::processes::AbiStructures;
use safa_api::raw::{NonNullSlice, RawSliceMut};
use safa_api::syscalls::exit;

extern "C" {
    fn main() -> u16;
}

unsafe fn _start_inner(
    argc: usize,
    argv: *mut NonNullSlice<u8>,
    envc: usize,
    envp: *mut NonNullSlice<u8>,
    task_abi_structures: *const AbiStructures,
) -> ! {
    unsafe {
        let args = RawSliceMut::from_raw_parts(argv, argc);
        let env = RawSliceMut::from_raw_parts(envp, envc);
        safa_api::process::init::sysapi_init(args, env, *task_abi_structures);
        let results = main();

        exit(results as usize);
    }
}

#[no_mangle]
#[allow(unused)]
pub extern "C" fn _start(
    argc: usize,
    argv: *mut NonNullSlice<u8>,
    envc: usize,
    envp: *mut NonNullSlice<u8>,
    task_abi_structures: *const AbiStructures,
) {
    unsafe {
        core::arch::asm!(
            "
            xor rbp, rbp
            push rbp
            push rbp
        ",
        );
        _start_inner(argc, argv, envc, envp, task_abi_structures);
    };
}
