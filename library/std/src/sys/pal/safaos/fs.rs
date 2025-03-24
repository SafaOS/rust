use core::cell::UnsafeCell;

use crate::ffi::{OsStr, OsString};
use crate::fmt;
use crate::hash::Hash;
use crate::io::{self, BorrowedCursor, IoSlice, IoSliceMut, SeekFrom};
use crate::path::{Path, PathBuf};
use crate::sys::time::SystemTime;
use crate::sys::unsupported;
use safa_api::errors::ErrorStatus;
use safa_api::raw;
use safa_api::syscalls;

macro_rules! path_to_str {
    ($path: expr) => {
        unsafe { core::str::from_utf8_unchecked($path.as_os_str().as_encoded_bytes()) }
    };
}

pub type ResourceID = usize;

#[derive(Debug)]
struct FileResource(ResourceID);

impl FileResource {
    fn open(path: &str) -> Result<Self, ErrorStatus> {
        Ok(Self(syscalls::open(path)?))
    }

    fn attrs(&self) -> Result<FileAttr, ErrorStatus> {
        let attr = syscalls::fattrs(self.0)?;
        Ok(attr.into())
    }

    fn diriter_open(&self) -> Result<DirIterResource, ErrorStatus> {
        let ri = syscalls::diriter_open(self.0)?;
        Ok(DirIterResource(ri))
    }

    fn truncate(&self, len: usize) -> Result<(), ErrorStatus> {
        syscalls::truncate(self.0, len)
    }

    fn sync(&self) -> Result<(), ErrorStatus> {
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
struct DirIterResource(ResourceID);

impl DirIterResource {
    fn open(path: &str) -> Result<Self, ErrorStatus> {
        let file = FileResource::open(path)?;
        file.diriter_open()
    }

    fn next(&mut self) -> Option<raw::DirEntry> {
        let raw = syscalls::diriter_next(self.0).unwrap();
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

// FIXME: make seek_at a mutex?
#[derive(Debug)]
pub struct File {
    fd: FileResource,
    seek_at: UnsafeCell<isize>,
}

unsafe impl Sync for File {}
unsafe impl Send for File {}

#[derive(Debug, Clone)]
pub struct FileAttr {
    size: usize,
    kind: FileType,
}

impl From<raw::FileAttr> for FileAttr {
    fn from(other: raw::FileAttr) -> Self {
        Self { size: other.size, kind: other.kind.into() }
    }
}

#[derive(Debug)]
pub struct ReadDir {
    /// the resource id of the directory iterator
    ri: DirIterResource,
    /// the path of the directory
    path: PathBuf,
}

pub struct DirEntry {
    inner: raw::DirEntry,
    full_path: PathBuf,
}

impl DirEntry {
    fn name_bytes(&self) -> &[u8] {
        &self.inner.name[..self.inner.name_length]
    }

    fn from_raw(raw: raw::DirEntry, parent_path: &Path) -> Self {
        let name = unsafe { OsStr::from_encoded_bytes_unchecked(&raw.name[..raw.name_length]) };
        let full_path = parent_path.join(name);
        Self { inner: raw, full_path }
    }
}

#[derive(Clone, Debug)]
pub struct OpenOptions {
    truncate: bool,
    create: bool,
    create_new: bool,
    read: bool,
    write: bool,
    append: bool,
}

#[derive(Copy, Clone, Debug, Default)]
pub struct FileTimes {}

pub struct FilePermissions(!);

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum FileType {
    File,
    Directory,
    Device,
}

impl From<raw::InodeType> for FileType {
    fn from(value: raw::InodeType) -> Self {
        match value {
            raw::InodeType::Directory => Self::Directory,
            raw::InodeType::File => Self::File,
            raw::InodeType::Device => Self::Device,
        }
    }
}

#[derive(Debug)]
pub struct DirBuilder {}

impl FileAttr {
    pub fn size(&self) -> u64 {
        self.size as u64
    }

    pub fn perm(&self) -> FilePermissions {
        unimplemented!("File Permissions is not yet implemented for SafaOS")
    }

    pub fn file_type(&self) -> FileType {
        self.kind
    }

    pub fn modified(&self) -> io::Result<SystemTime> {
        unsupported()
    }

    pub fn accessed(&self) -> io::Result<SystemTime> {
        unsupported()
    }

    pub fn created(&self) -> io::Result<SystemTime> {
        unsupported()
    }
}

impl FilePermissions {
    pub fn readonly(&self) -> bool {
        self.0
    }

    pub fn set_readonly(&mut self, _readonly: bool) {
        self.0
    }
}

impl Clone for FilePermissions {
    fn clone(&self) -> FilePermissions {
        self.0
    }
}

impl PartialEq for FilePermissions {
    fn eq(&self, _other: &FilePermissions) -> bool {
        self.0
    }
}

impl Eq for FilePermissions {}

impl fmt::Debug for FilePermissions {
    fn fmt(&self, _f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0
    }
}

impl FileTimes {
    pub fn set_accessed(&mut self, _t: SystemTime) {}
    pub fn set_modified(&mut self, _t: SystemTime) {}
}

impl FileType {
    pub fn is_dir(&self) -> bool {
        *self == Self::Directory
    }

    pub fn is_file(&self) -> bool {
        *self == Self::File || *self == Self::Device
    }

    pub fn is_symlink(&self) -> bool {
        false
    }
}

impl Iterator for ReadDir {
    type Item = io::Result<DirEntry>;

    fn next(&mut self) -> Option<io::Result<DirEntry>> {
        let raw = self.ri.next()?;
        Some(Ok(DirEntry::from_raw(raw, &self.path)))
    }
}

impl DirEntry {
    pub fn path(&self) -> PathBuf {
        self.full_path.clone()
    }

    pub fn file_name(&self) -> OsString {
        unsafe { OsString::from_encoded_bytes_unchecked(self.name_bytes().to_vec()) }
    }

    pub fn metadata(&self) -> io::Result<FileAttr> {
        Ok(self.inner.attrs.clone().into())
    }

    pub fn file_type(&self) -> io::Result<FileType> {
        Ok(self.inner.attrs.kind.into())
    }
}

impl OpenOptions {
    pub fn new() -> OpenOptions {
        OpenOptions {
            truncate: false,
            create: false,
            read: false,
            write: false,
            create_new: false,
            append: false,
        }
    }

    pub fn read(&mut self, read: bool) {
        self.read = read
    }
    pub fn write(&mut self, write: bool) {
        self.write = write
    }
    pub fn append(&mut self, append: bool) {
        self.append = append
    }
    pub fn truncate(&mut self, truncate: bool) {
        self.truncate = truncate
    }
    pub fn create(&mut self, create: bool) {
        self.create = create
    }
    pub fn create_new(&mut self, create_new: bool) {
        self.create_new = create_new
    }
}

impl File {
    pub fn open(path: &Path, opts: &OpenOptions) -> io::Result<File> {
        let create_from_fd = move |fd: FileResource| {
            if opts.write && opts.truncate {
                fd.truncate(0)?;
            }

            let seek_at = if opts.append && opts.write { -1 } else { 0 };

            Ok(Self { fd, seek_at: UnsafeCell::new(seek_at) })
        };

        let path = path_to_str!(path);
        match FileResource::open(path) {
            Err(ErrorStatus::NoSuchAFileOrDirectory) if opts.create => {
                syscalls::create(path)?;
                let fd = FileResource::open(path)?;
                create_from_fd(fd)
            }
            Err(other) => Err(other.into()),
            Ok(_) if opts.create_new => Err(ErrorStatus::AlreadyExists.into()),
            Ok(fd) => create_from_fd(fd),
        }
    }

    pub fn file_attr(&self) -> io::Result<FileAttr> {
        Ok(self.fd.attrs()?)
    }

    pub fn fsync(&self) -> io::Result<()> {
        self.fd.sync()?;
        Ok(())
    }

    pub fn datasync(&self) -> io::Result<()> {
        self.fsync()
    }

    pub fn lock(&self) -> io::Result<()> {
        unsupported()
    }

    pub fn lock_shared(&self) -> io::Result<()> {
        unsupported()
    }

    pub fn try_lock(&self) -> io::Result<bool> {
        unsupported()
    }

    pub fn try_lock_shared(&self) -> io::Result<bool> {
        unsupported()
    }

    pub fn unlock(&self) -> io::Result<()> {
        unsupported()
    }

    pub fn truncate(&self, size: u64) -> io::Result<()> {
        Ok(self.fd.truncate(size as usize)?)
    }

    pub fn read(&self, buf: &mut [u8]) -> io::Result<usize> {
        let at = unsafe { *self.seek_at.get() };
        Ok(self.fd.read(at, buf)?)
    }

    pub fn read_vectored(&self, _bufs: &mut [IoSliceMut<'_>]) -> io::Result<usize> {
        todo!()
    }

    pub fn is_read_vectored(&self) -> bool {
        false
    }

    pub fn read_buf(&self, _cursor: BorrowedCursor<'_>) -> io::Result<()> {
        todo!()
    }

    pub fn write(&self, buf: &[u8]) -> io::Result<usize> {
        let at = unsafe { *self.seek_at.get() };
        Ok(self.fd.write(at, buf)?)
    }

    pub fn write_vectored(&self, _bufs: &[IoSlice<'_>]) -> io::Result<usize> {
        todo!()
    }

    pub fn is_write_vectored(&self) -> bool {
        false
    }

    pub fn flush(&self) -> io::Result<()> {
        self.fsync()
    }

    pub fn seek(&self, pos: SeekFrom) -> io::Result<u64> {
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

    pub fn duplicate(&self) -> io::Result<File> {
        unsupported()
    }

    pub fn set_permissions(&self, _perm: FilePermissions) -> io::Result<()> {
        unsupported()
    }

    pub fn set_times(&self, _times: FileTimes) -> io::Result<()> {
        unsupported()
    }
}

impl DirBuilder {
    pub fn new() -> DirBuilder {
        DirBuilder {}
    }

    pub fn mkdir(&self, p: &Path) -> io::Result<()> {
        Ok(syscalls::createdir(path_to_str!(p))?)
    }
}

pub fn readdir(p: &Path) -> io::Result<ReadDir> {
    let path = path_to_str!(p);
    let diriter = DirIterResource::open(path)?;
    Ok(ReadDir { ri: diriter, path: p.to_path_buf() })
}

pub fn unlink(_p: &Path) -> io::Result<()> {
    unsupported()
}

pub fn rename(_old: &Path, _new: &Path) -> io::Result<()> {
    unsupported()
}

pub fn set_perm(_p: &Path, perm: FilePermissions) -> io::Result<()> {
    match perm.0 {}
}

pub fn rmdir(_p: &Path) -> io::Result<()> {
    unsupported()
}

pub fn remove_dir_all(_path: &Path) -> io::Result<()> {
    unsupported()
}

pub fn exists(path: &Path) -> io::Result<bool> {
    let path = path_to_str!(path);
    // fastest syscall to verify the existence of a path
    match syscalls::getdirentry(path) {
        Err(ErrorStatus::NoSuchAFileOrDirectory) => Ok(false),
        Err(other) => Err(other.into()),
        Ok(_) => Ok(true),
    }
}

pub fn readlink(_p: &Path) -> io::Result<PathBuf> {
    unsupported()
}

pub fn symlink(_original: &Path, _link: &Path) -> io::Result<()> {
    unsupported()
}

pub fn link(_src: &Path, _dst: &Path) -> io::Result<()> {
    unsupported()
}

pub fn stat(p: &Path) -> io::Result<FileAttr> {
    let path = path_to_str!(p);
    let fd = FileResource::open(path)?;
    Ok(fd.attrs()?)
}

pub fn lstat(p: &Path) -> io::Result<FileAttr> {
    // always correct because there is no symlinks :D
    stat(p)
}

pub fn canonicalize(_p: &Path) -> io::Result<PathBuf> {
    unsupported()
}

pub fn copy(_from: &Path, _to: &Path) -> io::Result<u64> {
    unsupported()
}
