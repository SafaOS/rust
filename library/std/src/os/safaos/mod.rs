#![stable(feature = "rust1", since = "1.0.0")]
#[unstable(feature = "rustc_private", issue = "27812")]
pub use safa_api as api;

use safa_api::errors::ErrorStatus;
#[stable(feature = "safa_api", since = "1.0.0")]
pub use safa_api::*;

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
        InvalidPath => IoErrorKind::InvalidInput,
        InvalidStr => IoErrorKind::InvalidData,
        OutOfMemory | MMapError => IoErrorKind::OutOfMemory,
        _ => IoErrorKind::Other,
    }
}
