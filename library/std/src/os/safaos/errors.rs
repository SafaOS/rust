#![stable(feature = "syscalls", since = "1.0.0")]

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

#[stable(feature = "syscalls", since = "1.0.0")]
impl TryFrom<i32> for ErrorStatus {
    type Error = ();
    fn try_from(value: i32) -> Result<Self, Self::Error> {
        if value > u16::MAX as i32 { Err(()) } else { Self::try_from(value as u16) }
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
    pub fn from_io_error_kind(kind: crate::io::ErrorKind) -> Self {
        use crate::io::ErrorKind as IoErrorKind;
        match kind {
            IoErrorKind::NotFound => Self::NoSuchAFileOrDirectory,
            IoErrorKind::AlreadyExists => Self::AlreadyExists,
            IoErrorKind::PermissionDenied => Self::MissingPermissions,
            IoErrorKind::ResourceBusy => Self::Busy,
            IoErrorKind::NotADirectory => Self::NotADirectory,
            IoErrorKind::IsADirectory => Self::NotAFile,
            IoErrorKind::InvalidInput => Self::InvaildPath,
            IoErrorKind::InvalidData => Self::InvaildStr,
            IoErrorKind::OutOfMemory => Self::OutOfMemory,
            IoErrorKind::Other => Self::Generic,
            _ => Self::Generic,
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
