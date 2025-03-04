use super::syscalls;
use super::syscalls::ErrorStatus;
use crate::io as std_io;

// SAFETY: must be called only once during runtime initialization.
// NOTE: this is not guaranteed to run, for example when Rust code is called externally.
pub unsafe fn init(_argc: isize, _argv: *const *const u8, _sigpipe: u8) {}

// SAFETY: must be called only once during runtime cleanup.
// NOTE: this is not guaranteed to run, for example when the program aborts.
pub unsafe fn cleanup() {}

pub fn unsupported<T>() -> std_io::Result<T> {
    Err(unsupported_err())
}

pub fn unsupported_err() -> std_io::Error {
    std_io::Error::UNSUPPORTED_PLATFORM
}

pub fn is_interrupted(_code: i32) -> bool {
    false
}

pub fn decode_error_kind(code: i32) -> crate::io::ErrorKind {
    if code > u16::MAX as i32 {
        crate::io::ErrorKind::Uncategorized
    } else {
        let status = ErrorStatus::try_from(code as u16).unwrap_or(ErrorStatus::Last);
        status.into_io_error_kind()
    }
}

pub fn abort_internal() -> ! {
    syscalls::exit(1)
}
