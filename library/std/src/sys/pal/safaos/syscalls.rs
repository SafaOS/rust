use crate::arch::asm;

// Keep in sync with the kernel implementition
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum ErrorStatus {
    None = 0,
    // use when no ErrorStatus is avalible for xyz and you cannot add a new one
    Generic,
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
}

fn syscall1(num: usize, arg1: usize) -> ErrorStatus {
    let result: u16;
    unsafe {
        asm!(
            "int 0x80",
            in("rax") num,
            in("rdi") arg1,
            lateout("rax") result,
        );

        crate::mem::transmute(result)
    }

}

fn syscall5(
    num: usize,
    arg1: usize,
    arg2: usize,
    arg3: usize,
    arg4: usize,
    arg5: usize,
) -> ErrorStatus {
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

        crate::mem::transmute(result)
    }
}

pub fn syswrite(
    fd: usize,
    offset: isize,
    buf: *const u8,
    len: usize,
    dest_wrote: &mut usize,
) -> ErrorStatus {
    syscall5(0x03, fd, offset as usize, buf as usize, len, dest_wrote as *const _ as usize)
}

pub fn syssync(fd: usize) -> ErrorStatus {
    syscall1(0x10, fd)
}


pub fn syssbrk(size: isize) -> Result<*mut u8, ErrorStatus> {
    todo!()
}