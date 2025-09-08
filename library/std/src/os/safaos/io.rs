use safa_api::syscalls;

use crate::fs::File;

use super::{AsRawResource, FromRawResource};

#[stable(feature = "rust1", since = "1.0.0")]
pub trait IoUtils {
    #[stable(feature = "rust1", since = "1.0.0")]
    /// Sends command `command` with argument `arg` to the resource `self`
    fn send_command(&self, command: u16, arg: u64) -> crate::io::Result<()>;
}

#[stable(feature = "rust1", since = "1.0.0")]
impl IoUtils for File {
    fn send_command(&self, command: u16, arg: u64) -> crate::io::Result<()> {
        let ri = self.as_raw_resource();
        syscalls::io::io_command(ri, command, arg).map_err(|e| e.into())
    }
}

/// Creates a new VTTY pair of (mother, child) file descriptors.
#[stable(feature = "io_create_vtty", since = "1.75.0")]
pub fn create_vtty() -> crate::io::Result<(File, File)> {
    let (mother, child) = syscalls::io::vtty_alloc()?;
    unsafe { Ok((File::from_raw_resource(mother), File::from_raw_resource(child))) }
}
