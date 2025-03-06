use crate::arch::asm;

#[stable(feature = "syscalls", since = "1.0.0")]
// Keep in sync with the kernel implementition
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum ErrorStatus {
    // use when no ErrorStatus is avalible for xyz and you cannot add a new one
    Generic = 1,
    OperationNotSupported,
    // for example an elf class is not supported, there is a difference between NotSupported and
    // OperationNotSupported
    NotSupported,
    // for example a magic value is invaild
    Corrupted,
    InvaildSyscall,
    InvaildResource,
    InvaildPid,
    InvaildOffset,
    // instead of panicking syscalls will return this on null and unaligned pointers
    InvaildPtr,
    // for operations that requires a vaild utf8 str...
    InvaildStr,
    // for operations that requires a str that doesn't exceed a max length such as
    // file names (128 bytes)
    StrTooLong,
    InvaildPath,
    NoSuchAFileOrDirectory,
    NotAFile,
    NotADirectory,
    AlreadyExists,
    NotExecutable,
    // would be useful when i add remove related operations to the vfs
    DirectoryNotEmpty,
    // Generic premissions(protection) related error
    MissingPermissions,
    // memory allocations and mapping error, most likely that memory is full
    MMapError,
    Busy,
    // errors sent by processes
    NotEnoughArguments,
    OutOfMemory,
    Last,
}

#[stable(feature = "syscalls", since = "1.0.0")]
impl TryFrom<u16> for ErrorStatus {
    type Error = ();
    fn try_from(value: u16) -> Result<Self, Self::Error> {
        if value >= Self::Last as u16 || value < Self::Generic as u16 {
            Err(())
        } else {
            Ok(unsafe { core::mem::transmute(value) })
        }
    }
}

#[derive(Clone, Copy)]
pub(crate) struct SysSuccess;
impl ErrorStatus {
    #[inline(always)]
    pub(crate) unsafe fn from_u16(value: u16) -> Result<SysSuccess, ErrorStatus> {
        unsafe { if value == 0 { Ok(SysSuccess) } else { Err(core::mem::transmute(value)) } }
    }

    #[inline(always)]
    pub(crate) fn into_io_error_kind(self) -> crate::io::ErrorKind {
        use crate::io::ErrorKind as IoErrorKind;

        match self {
            Self::NoSuchAFileOrDirectory => IoErrorKind::NotFound,
            Self::AlreadyExists => IoErrorKind::AlreadyExists,
            Self::MissingPermissions => IoErrorKind::PermissionDenied,
            Self::Busy => IoErrorKind::ResourceBusy,
            Self::NotADirectory => IoErrorKind::NotADirectory,
            Self::NotAFile => IoErrorKind::IsADirectory,
            Self::InvaildPath => IoErrorKind::InvalidInput,
            Self::InvaildStr => IoErrorKind::InvalidData,
            Self::OutOfMemory | Self::MMapError => IoErrorKind::OutOfMemory,
            Self::Last => IoErrorKind::Uncategorized,
            _ => IoErrorKind::Other,
        }
    }

    #[stable(feature = "syscalls", since = "1.0.0")]
    #[inline(always)]
    /// Gives a string description of the error
    pub fn as_str(&self) -> &'static str {
        use ErrorStatus::*;
        match *self {
            Generic => "Generic Error",
            OperationNotSupported => "Operation Not Supported",
            NotSupported => "Object Not Supported",
            Corrupted => "Corrupted",
            InvaildSyscall => "Invaild Syscall",
            InvaildResource => "Invaild Resource",
            InvaildPid => "Invaild PID",
            InvaildOffset => "Invaild Offset",
            InvaildPtr => "Invaild Ptr (not aligned or null)",
            InvaildStr => "Invaild Str (not utf8)",
            StrTooLong => "Str too Long",
            InvaildPath => "Invaild Path",
            NoSuchAFileOrDirectory => "No Such a File or Directory",
            NotAFile => "Not a File",
            NotADirectory => "Not a Directory",
            AlreadyExists => "Already Exists",
            NotExecutable => "Not Executable",
            DirectoryNotEmpty => "Directory not Empty",
            MissingPermissions => "Missing Permissions",
            MMapError => "Memory Map Error (most likely out of memory)",
            Busy => "Resource Busy",
            NotEnoughArguments => "Not Enough Arguments",
            OutOfMemory => "Out of Memory",
            Last => unreachable!(),
        }
    }
}

#[inline(always)]
fn syscall1(num: usize, arg1: usize) -> Result<SysSuccess, ErrorStatus> {
    let result: u16;
    unsafe {
        asm!(
            "int 0x80",
            in("rax") num,
            in("rdi") arg1,
            lateout("rax") result,
        );

        ErrorStatus::from_u16(result)
    }
}

#[inline(always)]
fn syscall2(num: usize, arg1: usize, arg2: usize) -> Result<SysSuccess, ErrorStatus> {
    let result: u16;
    unsafe {
        asm!(
            "int 0x80",
            in("rax") num,
            in("rdi") arg1,
            in("rsi") arg2,
            lateout("rax") result,
        );

        ErrorStatus::from_u16(result)
    }
}

#[inline(always)]
fn syscall3(num: usize, arg1: usize, arg2: usize, arg3: usize) -> Result<SysSuccess, ErrorStatus> {
    let result: u16;
    unsafe {
        asm!(
            "int 0x80",
            in("rax") num,
            in("rdi") arg1,
            in("rsi") arg2,
            in("rdx") arg3,
            lateout("rax") result,
        );

        ErrorStatus::from_u16(result)
    }
}

#[inline(always)]
fn syscall5(
    num: usize,
    arg1: usize,
    arg2: usize,
    arg3: usize,
    arg4: usize,
    arg5: usize,
) -> Result<SysSuccess, ErrorStatus> {
    let result: u16;
    unsafe {
        asm!(
            "int 0x80",
            in("rax") num,
            in("rdi") arg1,
            in("rsi") arg2,
            in("rdx") arg3,
            in("rcx") arg4,
            in("r8") arg5,
            lateout("rax") result,
        );

        ErrorStatus::from_u16(result)
    }
}

fn syswrite(
    fd: usize,
    offset: isize,
    buf: *const u8,
    len: usize,
    dest_wrote: &mut usize,
) -> Result<SysSuccess, ErrorStatus> {
    syscall5(0x03, fd, offset as usize, buf as usize, len, dest_wrote as *mut _ as usize)
}

#[inline]
pub fn write(fd: usize, offset: isize, buf: &[u8]) -> Result<usize, ErrorStatus> {
    let mut dest_wrote = 0;
    syswrite(fd, offset, buf.as_ptr(), buf.len(), &mut dest_wrote).map(|SysSuccess| dest_wrote)
}

#[inline]
fn sysread(
    fd: usize,
    offset: isize,
    buf: *mut u8,
    len: usize,
    dest_read: &mut usize,
) -> Result<SysSuccess, ErrorStatus> {
    syscall5(0x04, fd, offset as usize, buf as usize, len, dest_read as *mut _ as usize)
}

#[inline]
pub fn read(fd: usize, offset: isize, buf: &mut [u8]) -> Result<usize, ErrorStatus> {
    let mut dest_read = 0;
    sysread(fd, offset, buf.as_mut_ptr(), buf.len(), &mut dest_read).map(|SysSuccess| dest_read)
}

#[inline]
fn syssync(fd: usize) -> Result<SysSuccess, ErrorStatus> {
    syscall1(0x10, fd)
}

#[inline]
pub fn sync(fd: usize) -> Result<(), ErrorStatus> {
    syssync(fd).map(|SysSuccess| ())
}

#[inline]
fn syssbrk(size: isize, target_ptr: &mut *mut u8) -> Result<SysSuccess, ErrorStatus> {
    syscall2(0x12, size as usize, target_ptr as *mut _ as usize)
}

#[inline]
pub fn sbrk(size: isize) -> Result<*mut u8, ErrorStatus> {
    let mut target_ptr: *mut u8 = core::ptr::null_mut();
    syssbrk(size, &mut target_ptr).map(|SysSuccess| target_ptr)
}

#[inline(always)]
pub fn exit(code: usize) -> ! {
    let _ = syscall1(0, code);
    unreachable!()
}

/// Gets the current working directory
/// returns Err(ErrorStatus::Generic) if the buffer is too small to hold the cwd
#[inline(always)]
fn sysgetcwd(cwd_buf: &mut [u8], dest_len: &mut usize) -> Result<SysSuccess, ErrorStatus> {
    syscall3(0xF, cwd_buf.as_mut_ptr() as usize, cwd_buf.len(), dest_len as *mut _ as usize)
}

#[inline]
pub fn getcwd() -> Result<Vec<u8>, ErrorStatus> {
    let extend = |cwd_buf: &mut Vec<u8>| unsafe {
        cwd_buf.reserve(128);
        cwd_buf.set_len(cwd_buf.capacity());
    };

    let mut dest_len = 0;
    let mut cwd_buf = Vec::new();
    extend(&mut cwd_buf);

    loop {
        match sysgetcwd(&mut cwd_buf, &mut dest_len) {
            Ok(SysSuccess) => unsafe {
                cwd_buf.set_len(dest_len);
                return Ok(cwd_buf);
            },
            Err(err) => {
                if err == ErrorStatus::Generic {
                    extend(&mut cwd_buf);
                } else {
                    return Err(err);
                }
            }
        }
    }
}
