use crate::arch::asm;
use crate::os::safaos::errors::{ErrorStatus, SysSuccess};
use core::{ops, ptr};

/// Keep in sync with the kernel implementition
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
enum SyscallNum {
    SysExit = 0,
    SysYield = 1,

    SysOpen = 2,
    SysDirIterOpen = 8,
    SysClose = 5,
    SysDirIterClose = 9,
    SysDirIterNext = 10,
    SysWrite = 3,
    SysRead = 4,
    SysCreate = 6,
    SysCreateDir = 7,
    SysSync = 16,
    SysTruncate = 17,
    SysCtl = 12,
    SysFSize = 22,

    SysCHDir = 14,
    SysGetCWD = 15,
    SysSbrk = 18,

    SysPSpawn = 19,
    SysWait = 11,

    SysShutdown = 20,
    SysReboot = 21,
}

#[inline(always)]
fn syscall1(num: SyscallNum, arg1: usize) -> Result<SysSuccess, ErrorStatus> {
    let result: u16;
    unsafe {
        asm!(
            "int 0x80",
            in("rax") num as usize,
            in("rdi") arg1,
            lateout("rax") result,
        );

        ErrorStatus::from_u16(result)
    }
}

#[inline(always)]
fn syscall2(num: SyscallNum, arg1: usize, arg2: usize) -> Result<SysSuccess, ErrorStatus> {
    let result: u16;
    unsafe {
        asm!(
            "int 0x80",
            in("rax") num as usize,
            in("rdi") arg1,
            in("rsi") arg2,
            lateout("rax") result,
        );

        ErrorStatus::from_u16(result)
    }
}

#[inline(always)]
fn syscall3(
    num: SyscallNum,
    arg1: usize,
    arg2: usize,
    arg3: usize,
) -> Result<SysSuccess, ErrorStatus> {
    let result: u16;
    unsafe {
        asm!(
            "int 0x80",
            in("rax") num as usize,
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
    num: SyscallNum,
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
            in("rax") num as usize,
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

#[inline(always)]
fn syscall4(
    num: SyscallNum,
    arg1: usize,
    arg2: usize,
    arg3: usize,
    arg4: usize,
) -> Result<SysSuccess, ErrorStatus> {
    let result: u16;
    unsafe {
        asm!(
            "int 0x80",
            in("rax") num as usize,
            in("rdi") arg1,
            in("rsi") arg2,
            in("rdx") arg3,
            in("rcx") arg4,
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
    syscall5(
        SyscallNum::SysWrite,
        fd,
        offset as usize,
        buf as usize,
        len,
        dest_wrote as *mut _ as usize,
    )
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
    syscall5(
        SyscallNum::SysRead,
        fd,
        offset as usize,
        buf as usize,
        len,
        dest_read as *mut _ as usize,
    )
}

#[inline]
pub fn read(fd: usize, offset: isize, buf: &mut [u8]) -> Result<usize, ErrorStatus> {
    let mut dest_read = 0;
    sysread(fd, offset, buf.as_mut_ptr(), buf.len(), &mut dest_read).map(|SysSuccess| dest_read)
}

#[inline]
fn syssync(fd: usize) -> Result<SysSuccess, ErrorStatus> {
    syscall1(SyscallNum::SysSync, fd)
}

#[inline]
pub fn sync(fd: usize) -> Result<(), ErrorStatus> {
    syssync(fd).map(|SysSuccess| ())
}

#[inline]
fn syssbrk(size: isize, target_ptr: &mut *mut u8) -> Result<SysSuccess, ErrorStatus> {
    syscall2(SyscallNum::SysSbrk, size as usize, target_ptr as *mut _ as usize)
}

#[inline]
pub fn sbrk(size: isize) -> Result<*mut u8, ErrorStatus> {
    let mut target_ptr: *mut u8 = core::ptr::null_mut();
    syssbrk(size, &mut target_ptr).map(|SysSuccess| target_ptr)
}

#[inline(always)]
pub fn exit(code: usize) -> ! {
    let _ = syscall1(SyscallNum::SysExit, code);
    unreachable!()
}

/// Gets the current working directory
/// returns Err(ErrorStatus::Generic) if the buffer is too small to hold the cwd
#[inline(always)]
fn sysgetcwd(cwd_buf: &mut [u8], dest_len: &mut usize) -> Result<SysSuccess, ErrorStatus> {
    syscall3(
        SyscallNum::SysGetCWD,
        cwd_buf.as_mut_ptr() as usize,
        cwd_buf.len(),
        dest_len as *mut _ as usize,
    )
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

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct SpawnFlags(u8);
impl SpawnFlags {
    pub const CLONE_RESOURCES: Self = Self(1 << 0);
    pub const CLONE_CWD: Self = Self(1 << 1);
}

impl ops::BitOr for SpawnFlags {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

#[inline(always)]
fn syspspawn(
    name: Option<&str>,
    path: &str,
    argv: &[&str],
    flags: SpawnFlags,
    dest_pid: &mut usize,
) -> Result<SysSuccess, ErrorStatus> {
    /// the temporary config struct for the spawn syscall, passed to the syscall
    /// because if it was passed as a bunch of arguments it would be too big to fit
    /// inside the registers
    #[repr(C)]
    struct SpawnConfig {
        name: (*const u8, usize),
        argv: (*mut (*const u8, usize), usize),
        flags: SpawnFlags,
    }
    impl SpawnConfig {
        #[inline(always)]
        fn new(name: Option<&str>, argv: &[&str], flags: SpawnFlags) -> Self {
            let name = name.map(|s| (s.as_ptr(), s.len())).unwrap_or((ptr::null(), 0));
            let argv: (*mut (*const u8, usize), usize) = unsafe { core::mem::transmute(argv) };
            Self { name, argv, flags }
        }
    }

    let config = SpawnConfig::new(name, argv, flags);
    syscall4(
        SyscallNum::SysPSpawn,
        path.as_ptr() as usize,
        path.len(),
        (&raw const config) as usize,
        dest_pid as *mut _ as usize,
    )
}

#[inline]
pub fn pspawn(
    name: Option<&str>,
    path: &str,
    argv: &[&str],
    flags: SpawnFlags,
) -> Result<usize, ErrorStatus> {
    let mut pid = 0;
    syspspawn(name, path, argv, flags, &mut pid).map(|SysSuccess| pid)
}

#[inline]
pub fn wait(pid: usize) -> Result<usize, ErrorStatus> {
    let mut dest_exit_code = 0;
    syscall2(SyscallNum::SysWait, pid, (&raw mut dest_exit_code) as usize)
        .map(|SysSuccess| dest_exit_code)
}
