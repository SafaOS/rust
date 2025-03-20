#![stable(feature = "rust1", since = "1.0.0")]
#[unstable(feature = "rustc_private", issue = "27812")]
pub use safa_api as api;
use safa_api::errors::ErrorStatus;

#[inline(always)]
pub(crate) fn into_io_error_kind(err: ErrorStatus) -> crate::io::ErrorKind {
    use crate::io::ErrorKind as IoErrorKind;
    use ErrorStatus::*;

    match err {
        NoSuchAFileOrDirectory => IoErrorKind::NotFound,
        AlreadyExists => IoErrorKind::AlreadyExists,
        MissingPermissions => IoErrorKind::PermissionDenied,
        Busy => IoErrorKind::ResourceBusy,
        NotADirectory => IoErrorKind::NotADirectory,
        NotAFile => IoErrorKind::IsADirectory,
        InvaildPath => IoErrorKind::InvalidInput,
        InvaildStr => IoErrorKind::InvalidData,
        OutOfMemory | MMapError => IoErrorKind::OutOfMemory,
        _ => IoErrorKind::Other,
    }
}

#[stable(feature = "syscalls", since = "1.0.0")]
pub fn from_io_error_kind(kind: crate::io::ErrorKind) -> ErrorStatus {
    use crate::io::ErrorKind as IoErrorKind;
    use ErrorStatus::*;

    match kind {
        IoErrorKind::NotFound => NoSuchAFileOrDirectory,
        IoErrorKind::AlreadyExists => AlreadyExists,
        IoErrorKind::PermissionDenied => MissingPermissions,
        IoErrorKind::ResourceBusy => Busy,
        IoErrorKind::NotADirectory => NotADirectory,
        IoErrorKind::IsADirectory => NotAFile,
        IoErrorKind::InvalidInput => InvaildPath,
        IoErrorKind::InvalidData => InvaildStr,
        IoErrorKind::OutOfMemory => OutOfMemory,
        IoErrorKind::Other => Generic,
        _ => Generic,
    }
}
