#![stable(feature = "rust1", since = "1.0.0")]
#[unstable(feature = "rustc_private", issue = "27812")]
pub use safa_api as api;

#[stable(feature = "safa_api", since = "1.0.0")]
pub use safa_api::*;
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