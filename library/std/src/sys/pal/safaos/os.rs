use super::resources::path_to_str;
use super::unsupported;
use crate::error::Error as StdError;
use crate::ffi::{OsStr, OsString};
use crate::marker::PhantomData;
use crate::os::safaos::api::errors::ErrorStatus;
use crate::os::safaos::api::syscalls;
use crate::path::{self, PathBuf};
use crate::{fmt, io};

pub fn errno() -> i32 {
    0
}

pub fn error_string(errno: i32) -> String {
    if errno == 0 {
        return "operation successful".to_string();
    }

    if errno <= u16::MAX as i32 {
        if let Ok(err) = ErrorStatus::try_from(errno as u16) {
            return err.as_str().to_string();
        }
    }

    "operation failed".to_string()
}

pub fn getcwd() -> io::Result<PathBuf> {
    let cwd = syscalls::process_misc::getcwd()?;
    let path = unsafe { OsString::from_encoded_bytes_unchecked(cwd.into_bytes()) };
    let path = PathBuf::from(path);
    Ok(path)
}

pub fn chdir(path: &path::Path) -> io::Result<()> {
    let path = path_to_str!(path);
    syscalls::process_misc::chdir(path)?;
    Ok(())
}

pub struct SplitPaths<'a>(!, PhantomData<&'a ()>);

pub fn split_paths(_unparsed: &OsStr) -> SplitPaths<'_> {
    panic!("unsupported")
}

impl<'a> Iterator for SplitPaths<'a> {
    type Item = PathBuf;
    fn next(&mut self) -> Option<PathBuf> {
        self.0
    }
}

#[derive(Debug)]
pub struct JoinPathsError;

pub fn join_paths<I, T>(_paths: I) -> Result<OsString, JoinPathsError>
where
    I: Iterator<Item = T>,
    T: AsRef<OsStr>,
{
    Err(JoinPathsError)
}

impl fmt::Display for JoinPathsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        "not supported on this platform yet".fmt(f)
    }
}

impl StdError for JoinPathsError {
    #[allow(deprecated)]
    fn description(&self) -> &str {
        "not supported on this platform yet"
    }
}

pub fn current_exe() -> io::Result<PathBuf> {
    unsupported()
}

pub fn temp_dir() -> PathBuf {
    panic!("no filesystem on this platform")
}

pub fn home_dir() -> Option<PathBuf> {
    None
}

pub fn exit(code: i32) -> ! {
    syscalls::process::exit(code as usize)
}

#[unsafe(no_mangle)]
pub extern "C" fn abort() -> ! {
    exit(1)
}

// TODO: define an ErrorStatus for panics? or just go with Generic?
#[unsafe(no_mangle)]
pub extern "C" fn __rust_abort() -> ! {
    exit(1)
}

pub fn getpid() -> u32 {
    panic!("no pids on this platform")
}
