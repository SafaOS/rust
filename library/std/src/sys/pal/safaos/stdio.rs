use crate::io;
use crate::os::safaos::api::errors::ErrorStatus;
use safa_api::process::{sysmeta_stderr, sysmeta_stdin, sysmeta_stdout};
use safa_api::syscalls;
#[stable(feature = "stdio", since = "1.0.0")]
impl From<ErrorStatus> for io::Error {
    fn from(err: ErrorStatus) -> io::Error {
        let error = err.as_str();
        let kind = crate::os::safaos::into_io_error_kind(err);

        io::Error::new(kind, error)
    }
}

pub struct Stdin;
pub struct Stdout;
pub struct Stderr;

impl Stdin {
    pub const fn new() -> Stdin {
        Stdin
    }
}

impl io::Read for Stdin {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        Ok(syscalls::read(sysmeta_stdin(), -1, buf)?)
    }
}

impl Stdout {
    pub const fn new() -> Stdout {
        Stdout
    }
}

impl io::Write for Stdout {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        Ok(syscalls::write(sysmeta_stdout(), -1, buf)?)
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(syscalls::sync(sysmeta_stdout())?)
    }
}

impl Stderr {
    pub const fn new() -> Stderr {
        Stderr
    }
}

impl io::Write for Stderr {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let wrote = syscalls::write(sysmeta_stderr(), -1, buf)?;
        self.flush()?;
        Ok(wrote)
    }
    #[inline]
    fn flush(&mut self) -> io::Result<()> {
        Ok(syscalls::sync(sysmeta_stderr())?)
    }
}

pub const STDIN_BUF_SIZE: usize = 128;

pub fn is_ebadf(_err: &io::Error) -> bool {
    true
}

pub fn panic_output() -> Option<impl io::Write> {
    Some(Stderr::new())
}
