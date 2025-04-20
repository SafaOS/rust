use core::cell::UnsafeCell;
use core::mem::ManuallyDrop;

use crate::io::{self, SeekFrom};
use crate::sys::fs::FileAttr;
use safa_api::raw;
use safa_api::{errors::ErrorStatus, syscalls};

macro_rules! path_to_str {
    ($path: expr) => {
        unsafe { core::str::from_utf8_unchecked($path.as_os_str().as_encoded_bytes()) }
    };
}
pub(crate) use path_to_str;

pub type ResourceID = usize;

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct FileResource(ResourceID);

impl FileResource {
    pub(crate) fn open(path: &str) -> Result<Self, ErrorStatus> {
        Ok(Self(syscalls::open(path)?))
    }

    pub fn attrs(&self) -> Result<FileAttr, ErrorStatus> {
        let attr = syscalls::fattrs(self.0)?;
        Ok(attr.into())
    }

    pub fn diriter_open(&self) -> Result<DirIterResource, ErrorStatus> {
        let ri = syscalls::diriter_open(self.0)?;
        Ok(DirIterResource(ri))
    }

    pub fn truncate(&self, len: usize) -> Result<(), ErrorStatus> {
        syscalls::truncate(self.0, len)
    }

    pub fn sync(&self) -> Result<(), ErrorStatus> {
        syscalls::sync(self.0)
    }

    fn read(&self, offset: isize, buf: &mut [u8]) -> Result<usize, ErrorStatus> {
        syscalls::read(self.0, offset, buf)
    }

    fn write(&self, offset: isize, buf: &[u8]) -> Result<usize, ErrorStatus> {
        syscalls::write(self.0, offset, buf)
    }

    fn size(&self) -> usize {
        syscalls::fsize(self.0).unwrap()
    }
}

#[derive(Debug)]
pub(crate) struct DirIterResource(ResourceID);

impl DirIterResource {
    pub(crate) fn open(path: &str) -> Result<Self, ErrorStatus> {
        let file = FileResource::open(path)?;
        file.diriter_open()
    }

    pub(crate) fn next(&mut self) -> Option<raw::io::DirEntry> {
        // should never error expect if there is no more entries it returns ErrorStatus::Generic
        let raw = syscalls::diriter_next(self.0).ok()?;
        if raw == unsafe { core::mem::zeroed() } { None } else { Some(raw) }
    }
}

impl Drop for DirIterResource {
    fn drop(&mut self) {
        syscalls::diriter_close(self.0).unwrap()
    }
}

impl Drop for FileResource {
    fn drop(&mut self) {
        syscalls::close(self.0).unwrap()
    }
}

impl Clone for FileResource {
    fn clone(&self) -> Self {
        Self(syscalls::dup(self.0).unwrap())
    }
}

// FIXME: make seek_at a mutex?
#[derive(Debug)]
pub(crate) struct FileDesc {
    fd: FileResource,
    seek_at: UnsafeCell<isize>,
}

impl Clone for FileDesc {
    fn clone(&self) -> Self {
        unsafe { Self { fd: self.fd.clone(), seek_at: UnsafeCell::new(*self.seek_at.get()) } }
    }
}

impl PartialEq for FileDesc {
    fn eq(&self, other: &Self) -> bool {
        self.fd == other.fd
    }
}

unsafe impl Send for FileDesc {}
unsafe impl Sync for FileDesc {}

impl FileDesc {
    /// converts a raw resource id into a FileDesc
    /// this is unsafe because the resource id is not checked for validity
    pub unsafe fn from_raw(ri: usize) -> Self {
        Self { fd: FileResource(ri), seek_at: UnsafeCell::new(0) }
    }

    /// duplicates a raw resource id into a FileDesc
    /// this is unsafe because the resource id is not checked for validity
    /// the returned FileDesc is a duplicate of the original with a different resource id and therefore doesn't take ownership of the resource
    pub unsafe fn from_raw_dup(ri: usize) -> Self {
        let fd = unsafe { ManuallyDrop::new(Self::from_raw(ri)) };
        ManuallyDrop::into_inner(fd.clone())
    }

    pub(crate) fn fd(&self) -> usize {
        self.fd.0
    }

    pub fn fd_raw(&self) -> &FileResource {
        &self.fd
    }

    pub fn open(path: &str, append: bool, truncate: bool) -> Result<Self, ErrorStatus> {
        let fd = FileResource::open(path)?;
        let seek_at = if append { -1 } else { 0 };
        if truncate {
            fd.truncate(0)?;
        }
        Ok(Self { fd, seek_at: UnsafeCell::new(seek_at) })
    }

    pub fn fsync(&self) -> io::Result<()> {
        self.fd.sync()?;
        Ok(())
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl io::Read for &FileDesc {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let at = unsafe { &mut *self.seek_at.get() };

        let read = match self.fd.read(*at, buf) {
            Ok(amount) => amount,
            Err(ErrorStatus::InvalidOffset) => return Ok(0),
            Err(other) => return Err(other.into()),
        };
        *at += read as isize;

        Ok(read)
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl io::Write for &FileDesc {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let at = unsafe { &mut *self.seek_at.get() };

        let wrote = match self.fd.write(*at, buf) {
            Ok(amount) => amount,
            Err(ErrorStatus::InvalidOffset) => return Ok(0),
            Err(other) => return Err(other.into()),
        };
        *at += wrote as isize;

        Ok(wrote)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.fsync()
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl io::Seek for &FileDesc {
    fn seek(&mut self, pos: io::SeekFrom) -> io::Result<u64> {
        unsafe {
            let seek_at = &mut *self.seek_at.get();
            match pos {
                SeekFrom::Start(start) => (*seek_at) = start as isize,
                SeekFrom::End(end) => (*seek_at) = -(end as isize + 1),
                SeekFrom::Current(current) => (*seek_at) += current as isize,
            }

            if (*seek_at) >= 0 {
                Ok(*seek_at as u64)
            } else {
                let end_at = (-(*seek_at)) as usize;
                let size = self.fd.size();
                Ok((size - end_at + 1) as u64)
            }
        }
    }
}
