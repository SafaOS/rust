use crate::io::{self, BorrowedCursor, IoSlice, IoSliceMut};
use crate::sys::resources::FileDesc;
use io::{Read, Write};

use super::unsupported;

#[derive(Debug)]
pub struct AnonPipe(FileDesc);

impl AnonPipe {
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

pub fn read2(_p1: AnonPipe, _v1: &mut Vec<u8>, _p2: AnonPipe, _v2: &mut Vec<u8>) -> io::Result<()> {
    unimplemented!()
}
