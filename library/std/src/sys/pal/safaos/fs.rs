use crate::ffi::{OsStr, OsString};
use crate::fmt;
use crate::hash::Hash;
use crate::io::{self, BorrowedCursor, IoSlice, IoSliceMut, SeekFrom};
use crate::io::{Read, Seek, Write};
use crate::path::{Path, PathBuf};
use crate::sys::resources::{DirIterResource, FileDesc, FileResource};
use crate::sys::time::SystemTime;
use crate::sys::unsupported;
use safa_api::errors::ErrorStatus;
use safa_api::raw;
use safa_api::syscalls;

use super::resources::path_to_str;

#[derive(Debug)]
pub struct File(FileDesc);

#[derive(Debug, Clone)]
pub struct FileAttr {
    size: usize,
    kind: FileType,
}

impl From<raw::io::FileAttr> for FileAttr {
    fn from(other: raw::io::FileAttr) -> Self {
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
    inner: raw::io::DirEntry,
    full_path: PathBuf,
}

impl DirEntry {
    fn name_bytes(&self) -> &[u8] {
        &self.inner.name[..self.inner.name_length]
    }

    fn from_raw(raw: raw::io::DirEntry, parent_path: &Path) -> Self {
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

impl From<raw::io::InodeType> for FileType {
    fn from(value: raw::io::InodeType) -> Self {
        match value {
            raw::io::InodeType::Directory => Self::Directory,
            raw::io::InodeType::File => Self::File,
            raw::io::InodeType::Device => Self::Device,
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
    pub fn into_raw(self) -> FileDesc {
        self.0
    }

    pub fn open(path: &Path, opts: &OpenOptions) -> io::Result<File> {
        let append = opts.append && opts.write;
        let truncate = opts.truncate && opts.write;

        let path = path_to_str!(path);
        let fd = match FileDesc::open(path, append, truncate) {
            Err(ErrorStatus::NoSuchAFileOrDirectory) if opts.create || opts.create_new => {
                syscalls::create(path)?;
                FileDesc::open(path, append, truncate)?
            }
            Err(other) => return Err(other.into()),
            Ok(_) if opts.create_new => return Err(ErrorStatus::AlreadyExists.into()),
            Ok(fd) => fd,
        };

        Ok(Self(fd))
    }

    pub fn file_attr(&self) -> io::Result<FileAttr> {
        Ok(self.0.fd_raw().attrs()?)
    }

    pub fn fsync(&self) -> io::Result<()> {
        self.0.fsync()
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
        Ok(self.0.fd_raw().truncate(size as usize)?)
    }

    pub fn read(&self, buf: &mut [u8]) -> io::Result<usize> {
        (&mut &self.0).read(buf)
    }

    pub fn read_vectored(&self, bufs: &mut [IoSliceMut<'_>]) -> io::Result<usize> {
        (&mut &self.0).read_vectored(bufs)
    }

    pub fn is_read_vectored(&self) -> bool {
        (&self.0).is_read_vectored()
    }

    pub fn read_buf(&self, cursor: BorrowedCursor<'_>) -> io::Result<()> {
        (&mut &self.0).read_buf(cursor)
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

    pub fn flush(&self) -> io::Result<()> {
        self.fsync()
    }

    pub fn seek(&self, pos: SeekFrom) -> io::Result<u64> {
        (&mut &self.0).seek(pos)
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
