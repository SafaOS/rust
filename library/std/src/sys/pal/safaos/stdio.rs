use crate::arch::asm;
use crate::io;

pub struct Stdin;
pub struct Stdout;
pub struct Stderr;

impl Stdin {
    pub const fn new() -> Stdin {
        Stdin
    }
}

impl io::Read for Stdin {
    fn read(&mut self, _buf: &mut [u8]) -> io::Result<usize> {
        Ok(0)
    }
}

impl Stdout {
    pub const fn new() -> Stdout {
        Stdout
    }
}

fn syswrite(fd: usize, offset: isize, buf: *const u8, len: usize, dest_wrote: &mut usize) -> u16 {
    let result;
    unsafe {
        asm!(
            "mov rax, 0x03",
            "int 0x80",
            in("rdi") fd,
            in("rsi") offset,
            in("rdx") buf,
            in("rcx") len,
            in("r8") dest_wrote,
            lateout("rax") result,
        );
    }

    result
}

fn syssync(fd: usize) -> u16 {
    let result;
    unsafe {
        asm!(
            "mov rax, 0x10",
            "int 0x80",
            in("rdi") fd,
            lateout("rax") result,
        );
    }
    result
}

impl io::Write for Stdout {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let mut dest_wrote = 0;
        unsafe {
            syswrite(1, -1, buf.as_ptr(), buf.len(), &mut dest_wrote);
        }
        Ok(dest_wrote)
    }

    fn flush(&mut self) -> io::Result<()> {
        syssync(1);
        Ok(())
    }
}

impl Stderr {
    pub const fn new() -> Stderr {
        Stderr
    }
}

impl io::Write for Stderr {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let mut dest_wrote = 0;
        unsafe {
            syswrite(2, -1, buf.as_ptr(), buf.len(), &mut dest_wrote);
        }
        Ok(dest_wrote)
    }

    fn flush(&mut self) -> io::Result<()> {
        syssync(2);
        Ok(())
    }
}

pub const STDIN_BUF_SIZE: usize = 0;

pub fn is_ebadf(_err: &io::Error) -> bool {
    true
}

pub fn panic_output() -> Option<impl io::Write> {
    Some(Stderr::new())
}
