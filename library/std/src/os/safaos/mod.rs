#![stable(feature = "rust1", since = "1.0.0")]
#[stable(feature = "rust1", since = "1.0.0")]
pub mod io;
#[stable(feature = "rust1", since = "1.0.0")]
pub mod process;

#[unstable(feature = "rustc_private", issue = "27812")]
pub use safa_api as api;

use safa_api::errors::ErrorStatus;
#[stable(feature = "safa_api", since = "1.0.0")]
pub use safa_api::*;

use crate::{
    fs::File,
    sys::resources::FileDesc,
    sys_common::{AsInner, FromInner},
};

#[inline(always)]
pub(crate) const fn into_io_error_kind(err: ErrorStatus) -> crate::io::ErrorKind {
    const fn unknown_err() -> crate::io::ErrorKind {
        crate::io::ErrorKind::Uncategorized
    }
    safa_api::err_into_io_error_kind!(err, crate::io::ErrorKind);
}

#[stable(feature = "rust1", since = "1.0.0")]
pub use crate::sys::resources::ResourceID;

#[stable(feature = "rust1", since = "1.0.0")]
/// A trait to express something that can be converted into a raw resource
pub trait AsRawResource {
    #[stable(feature = "rust1", since = "1.0.0")]
    /// Returns the raw resource ID for this object.
    fn as_raw_resource(&self) -> ResourceID;
}

#[stable(feature = "rust1", since = "1.0.0")]
/// A trait to express something that can be converted from a raw resource
pub trait FromRawResource {
    #[stable(feature = "rust1", since = "1.0.0")]
    /// Returns the object for this raw resource ID, can take ownership of the resource.
    /// # Safety
    /// resource must be valid and this may take ownership of the resource.
    unsafe fn from_raw_resource(resource: ResourceID) -> Self;
}

#[stable(feature = "rust1", since = "1.0.0")]
impl AsRawResource for File {
    fn as_raw_resource(&self) -> ResourceID {
        self.as_inner().0.fd.0
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl FromRawResource for File {
    unsafe fn from_raw_resource(resource: ResourceID) -> Self {
        let inner = crate::sys::fs::File::from_raw(FileDesc::from_raw(resource));
        FromInner::from_inner(inner)
    }
}
