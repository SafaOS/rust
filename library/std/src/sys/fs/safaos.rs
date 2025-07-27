use crate::ffi::{OsStr, OsString};
use crate::hash::Hash;
use crate::io::{self, BorrowedCursor, IoSlice, IoSliceMut, SeekFrom};
use crate::io::{Read, Seek, Write};
use crate::path::{Path, PathBuf};
use crate::sys::resources::{DirIterResource, FileDesc, FileResource};
use crate::sys::time::SystemTime;
use crate::sys::{unsupported, unsupported_err};
use crate::sys_common::ignore_notfound;
use crate::{fmt, fs};
use safa_api::abi::fs as raw_fs;
use safa_api::abi::fs::FSObjectType;
use safa_api::errors::ErrorStatus;
use safa_api::syscalls;

use crate::fs::TryLockError;
use crate::sys::pal::resources::path_to_str;

#[derive(Debug)]
pub struct File(FileDesc);

#[derive(Debug, Clone)]
pub struct FileAttr {
    size: usize,
    kind: FileType,
}

impl From<raw_fs::FileAttr> for FileAttr {
    fn from(other: raw_fs::FileAttr) -> Self {
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
    inner: raw_fs::DirEntry,
    full_path: PathBuf,
}

impl DirEntry {
    fn name_bytes(&self) -> &[u8] {
        &self.inner.name[..self.inner.name_length]
    }

    fn from_raw(raw: raw_fs::DirEntry, parent_path: &Path) -> Self {
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

impl From<raw_fs::FSObjectType> for FileType {
    fn from(value: raw_fs::FSObjectType) -> Self {
        use raw_fs::FSObjectType as Raw;
        match value {
            Raw::Directory => Self::Directory,
            Raw::File => Self::File,
            Raw::Device => Self::Device,
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
        let open_f = || FileDesc::open(path, opts.write, opts.read, append, opts.create, truncate);
        let fd = match open_f() {
            Err(ErrorStatus::NoSuchAFileOrDirectory) if opts.create_new => {
                syscalls::fs::create(path)?;
                open_f()?
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

    pub fn try_lock(&self) -> Result<(), TryLockError> {
        Err(TryLockError::Error(unsupported_err()))
    }

    pub fn try_lock_shared(&self) -> Result<(), TryLockError> {
        Err(TryLockError::Error(unsupported_err()))
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

    pub fn tell(&self) -> io::Result<u64> {
        Ok(self.0.tell())
    }

    pub fn duplicate(&self) -> io::Result<File> {
        Ok(Self(self.0.clone()))
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
        Ok(syscalls::fs::createdir(path_to_str!(p))?)
    }
}

pub fn readdir(p: &Path) -> io::Result<ReadDir> {
    let path = path_to_str!(p);
    let diriter = DirIterResource::open(path)?;
    Ok(ReadDir { ri: diriter, path: p.to_path_buf() })
}

pub fn unlink(p: &Path) -> io::Result<()> {
    let str = path_to_str!(p);
    let fattrs = syscalls::fs::getdirentry(str)?;
    if fattrs.attrs.kind != FSObjectType::File {
        return Err(ErrorStatus::NotAFile.into());
    }
    syscalls::fs::remove_path(str)?;
    Ok(())
}

pub fn rename(old: &Path, new: &Path) -> io::Result<()> {
    let old_str = path_to_str!(old);
    let new_str = path_to_str!(new);

    let old_attrs = syscalls::fs::getdirentry(old_str)?;
    // TODO: implement native rename syscall
    match old_attrs.attrs.kind {
        FSObjectType::File => {
            copy(old, new)?;
            syscalls::fs::remove_path(old_str)?;
            Ok(())
        }
        FSObjectType::Directory => {
            // create the new directory if it doesn't exist
            if let Err(e) = syscalls::fs::createdir(new_str)
                && e != ErrorStatus::AlreadyExists
            {
                return Err(e.into());
            }

            for entry in crate::fs::read_dir(old)? {
                let entry = entry?;
                let entry_name = entry.file_name();
                // special cases for '.' and '..' to avoid infinite recursion
                if entry_name.as_encoded_bytes() == b"." || entry_name.as_encoded_bytes() == b".." {
                    continue;
                }

                let old_entry_path = entry.path();

                let new_entry_path = new.join(entry_name);
                rename(&old_entry_path, &new_entry_path)?;
            }

            syscalls::fs::remove_path(old_str)?;
            Ok(())
        }
        _ => unsupported(),
    }
}

pub fn set_perm(_p: &Path, perm: FilePermissions) -> io::Result<()> {
    match perm.0 {}
}

pub fn rmdir(p: &Path) -> io::Result<()> {
    let path = path_to_str!(p);
    let fattrs = syscalls::fs::getdirentry(path)?;
    if fattrs.attrs.kind != FSObjectType::Directory {
        return Err(ErrorStatus::NotADirectory.into());
    }
    syscalls::fs::remove_path(path)?;
    Ok(())
}

pub fn remove_dir_all(path: &Path) -> io::Result<()> {
    for child in crate::fs::read_dir(path)? {
        let result: io::Result<()> = try {
            let child = child?;
            let name = child.file_name();
            // special cases for '.' and '..' to avoid infinite recursion
            if name.as_encoded_bytes() == b"." || name.as_encoded_bytes() == b".." {
                continue;
            }

            let path = child.path();

            if child.file_type()?.is_dir() {
                remove_dir_all(&path)?;
            } else {
                fs::remove_file(&path)?;
            }
        };
        // ignore internal NotFound errors to prevent race conditions
        if let Err(err) = &result
            && err.kind() != io::ErrorKind::NotFound
        {
            return result;
        }
    }
    ignore_notfound(fs::remove_dir(path))
}

pub fn exists(path: &Path) -> io::Result<bool> {
    let path = path_to_str!(path);
    // fastest syscall to verify the existence of a path
    match syscalls::fs::getdirentry(path) {
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
    let fd = FileResource::open(path, raw_fs::OpenOptions::READ)?;
    Ok(fd.attrs()?)
}

pub fn lstat(p: &Path) -> io::Result<FileAttr> {
    // always correct because there is no symlinks :D
    stat(p)
}

pub fn canonicalize(_p: &Path) -> io::Result<PathBuf> {
    unsupported()
}

pub fn copy(from: &Path, to: &Path) -> io::Result<u64> {
    let mut reader = fs::File::open(from)?;
    let metadata = reader.metadata()?;

    if !metadata.is_file() {
        return Err(ErrorStatus::NotAFile.into());
    }

    let mut writer = fs::File::create(to)?;
    let ret = io::copy(&mut reader, &mut writer)?;
    Ok(ret)
}
