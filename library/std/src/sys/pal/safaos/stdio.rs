use super::syscalls;
use crate::io;
use crate::os::safaos::errors::ErrorStatus;

#[stable(feature = "stdio", since = "1.0.0")]
impl From<ErrorStatus> for io::Error {
    fn from(err: ErrorStatus) -> io::Error {
        let kind = err.into_io_error_kind();
        let error = err.as_str();

        io::Error::new(kind, error)
    }
}

pub struct Stdin;
pub struct Stdout;
pub struct Stderr;

impl Stdin {
    pub const FD: usize = 0;
    pub const fn new() -> Stdin {
        Stdin
    }
}

impl io::Read for Stdin {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        Ok(syscalls::read(Self::FD, -1, buf)?)
    }
}

impl Stdout {
    const FD: usize = 1;
    pub const fn new() -> Stdout {
        Stdout
    }
}

impl io::Write for Stdout {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        Ok(syscalls::write(Self::FD, -1, buf)?)
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(syscalls::sync(1)?)
    }
}

impl Stderr {
    // TODO: please add a seprate stderr fd
    const FD: usize = Stdout::FD;
    pub const fn new() -> Stderr {
        Stderr
    }
}

impl io::Write for Stderr {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let wrote = syscalls::write(Self::FD, -1, buf)?;
        self.flush()?;
        Ok(wrote)
    }
    #[inline]
    fn flush(&mut self) -> io::Result<()> {
        Ok(syscalls::sync(Self::FD)?)
    }
}

pub const STDIN_BUF_SIZE: usize = 128;

pub fn is_ebadf(_err: &io::Error) -> bool {
    true
}

pub fn panic_output() -> Option<impl io::Write> {
    Some(Stderr::new())
}
