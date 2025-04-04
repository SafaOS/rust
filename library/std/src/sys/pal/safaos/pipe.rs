use crate::io::{self, BorrowedCursor, IoSlice, IoSliceMut};
use crate::sys::resources::FileDesc;
use io::{Read, Write};

use super::unsupported;

#[derive(Debug)]
pub struct AnonPipe(FileDesc);

impl AnonPipe {
    pub fn into_raw(self) -> FileDesc {
        self.0
    }

    pub fn from_fd(fd: FileDesc) -> Self {
        Self(fd)
    }

    pub fn try_clone(&self) -> io::Result<Self> {
        unsupported()
    }

    pub fn read(&self, buf: &mut [u8]) -> io::Result<usize> {
        (&mut &self.0).read(buf)
    }

    pub fn read_buf(&self, buf: BorrowedCursor<'_>) -> io::Result<()> {
        (&mut &self.0).read_buf(buf)
    }

    pub fn read_vectored(&self, bufs: &mut [IoSliceMut<'_>]) -> io::Result<usize> {
        (&self.0).read_vectored(bufs)
    }

    pub fn is_read_vectored(&self) -> bool {
        (&self.0).is_read_vectored()
    }

    pub fn read_to_end(&self, buf: &mut Vec<u8>) -> io::Result<usize> {
        (&mut &self.0).read_to_end(buf)
    }

    pub fn write(&self, buf: &[u8]) -> io::Result<usize> {
        (&mut &self.0).write(buf)
    }

    pub fn write_vectored(&self, bufs: &[IoSlice<'_>]) -> io::Result<usize> {
        (&mut &self.0).write_vectored(bufs)
    }

    pub fn is_write_vectored(&self) -> bool {
        (&self.0).is_write_vectored()
    }

    pub fn diverge(&self) -> ! {
        unimplemented!()
    }
}

/// FIXME: This is a temporary implementation I have no idea what an AnonPipe is forgive my naiveness
/// BUT I NEED IT FOR [`crate::sys::process::Command::output`]
pub fn read2(p1: AnonPipe, v1: &mut Vec<u8>, p2: AnonPipe, v2: &mut Vec<u8>) -> io::Result<()> {
    p1.read_to_end(v1)?;
    p2.read_to_end(v2)?;
    Ok(())
}
