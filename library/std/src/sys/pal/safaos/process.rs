pub use crate::ffi::OsString as EnvKey;
use crate::ffi::{OsStr, OsString};
use crate::num::NonZero;
use crate::os::safaos::api::errors::ErrorStatus;
use crate::os::safaos::api::syscalls;
use crate::path::Path;
use crate::sys::fs::File;
use crate::sys::pipe::AnonPipe;
use crate::sys_common::process::{CommandEnv, CommandEnvs};
use crate::{fmt, io};
use safa_api::errors::SysResult;
use safa_api::process::{sysmeta_stderr, sysmeta_stdin, sysmeta_stdout};

use super::resources::FileDesc;

////////////////////////////////////////////////////////////////////////////////
// Command
////////////////////////////////////////////////////////////////////////////////

pub struct Command {
    program: OsString,
    args: Vec<OsString>,
    env: CommandEnv,

    cwd: Option<OsString>,
    stdin: Option<Stdio>,
    stdout: Option<Stdio>,
    stderr: Option<Stdio>,
}

#[derive(Debug)]
// passed back to std::process with the pipes connected to the child, if any
// were requested
pub struct StdioPipes {
    pub stdin: Option<AnonPipe>,
    pub stdout: Option<AnonPipe>,
    pub stderr: Option<AnonPipe>,
}
impl StdioPipes {
    pub fn new() -> Self {
        Self { stdin: None, stdout: None, stderr: None }
    }
}

#[derive(Debug, PartialEq)]
pub enum Stdio {
    Inherit,
    Null,
    InheritStdout,
    InheritStderr,
    InheritStdin,
    MakePipe,
    InheritFile(FileDesc),
}

impl Stdio {
    // `&self` because Self::InheritFile has to live as long as the results
    fn into_raw(&self) -> Option<usize> {
        match self {
            Stdio::Inherit => None,
            Stdio::InheritStdout => Some(sysmeta_stdout()),
            Stdio::InheritStderr => Some(sysmeta_stderr()),
            Stdio::InheritStdin => Some(sysmeta_stdin()),
            Stdio::Null => todo!(),
            Stdio::InheritFile(f) => Some(f.fd()),
            s => unimplemented!("stdio: {:?}", s),
        }
    }

    fn into_anon_pipe(self) -> Option<AnonPipe> {
        match self {
            Stdio::Inherit => None,
            Stdio::InheritStdout => {
                Some(AnonPipe::from_fd(unsafe { FileDesc::from_raw_dup(sysmeta_stdout()) }))
            }
            Stdio::InheritStderr => {
                Some(AnonPipe::from_fd(unsafe { FileDesc::from_raw_dup(sysmeta_stderr()) }))
            }
            Stdio::InheritStdin => {
                Some(AnonPipe::from_fd(unsafe { FileDesc::from_raw_dup(sysmeta_stdin()) }))
            }
            Stdio::Null => None,
            Stdio::InheritFile(fd) => Some(AnonPipe::from_fd(fd)),
            s => unimplemented!("stdio: {:?}", s),
        }
    }
}

impl Command {
    pub fn new(program: &OsStr) -> Command {
        Command {
            program: program.to_owned(),
            args: vec![program.to_owned()],
            env: Default::default(),
            cwd: None,
            stdin: None,
            stdout: None,
            stderr: None,
        }
    }

    pub fn arg(&mut self, arg: &OsStr) {
        self.args.push(arg.to_owned());
    }

    pub fn env_mut(&mut self) -> &mut CommandEnv {
        &mut self.env
    }

    pub fn cwd(&mut self, dir: &OsStr) {
        self.cwd = Some(dir.to_owned());
    }

    pub fn stdin(&mut self, stdin: Stdio) {
        self.stdin = Some(stdin);
    }

    pub fn stdout(&mut self, stdout: Stdio) {
        self.stdout = Some(stdout);
    }

    pub fn stderr(&mut self, stderr: Stdio) {
        self.stderr = Some(stderr);
    }

    pub fn get_program(&self) -> &OsStr {
        &self.program
    }

    pub fn get_args(&self) -> CommandArgs<'_> {
        let mut iter = self.args.iter();
        iter.next();
        CommandArgs { iter }
    }

    pub fn get_envs(&self) -> CommandEnvs<'_> {
        self.env.iter()
    }

    pub fn get_current_dir(&self) -> Option<&Path> {
        self.cwd.as_ref().map(|cs| Path::new(cs))
    }

    pub fn spawn(
        &mut self,
        _default: Stdio,
        _needs_stdin: bool,
    ) -> io::Result<(Process, StdioPipes)> {
        use safa_api::raw::processes::SpawnFlags;
        assert_eq!(_default, Stdio::Inherit);

        let (stdin, stdout, stderr) = (
            self.stdin.take().unwrap_or(Stdio::InheritStdin),
            self.stdout.take().unwrap_or(Stdio::InheritStdout),
            self.stderr.take().unwrap_or(Stdio::InheritStderr),
        );

        let (stdinn, stdoutn, stderrn) = (stdin.into_raw(), stdout.into_raw(), stderr.into_raw());

        assert!(
            self.cwd.is_none() && self.env.is_unchanged(),
            "Spawning a process with custom env or cwd is not supported for SafaOS"
        );

        let name = unsafe { self.program.to_str().unwrap_unchecked() };
        let argv =
            self.args.iter().map(|s| unsafe { s.to_str().unwrap_unchecked() }).collect::<Vec<_>>();
        let path = name;

        let pid = syscalls::pspawn(
            Some(name),
            path,
            argv,
            SpawnFlags::CLONE_CWD,
            stdinn,
            stdoutn,
            stderrn,
        )?;

        let (stdin, stdout, stderr) =
            (stdin.into_anon_pipe(), stdout.into_anon_pipe(), stderr.into_anon_pipe());
        Ok((Process(pid), StdioPipes { stdin, stdout, stderr }))
    }

    pub fn output(&mut self) -> io::Result<(ExitStatus, Vec<u8>, Vec<u8>)> {
        let (mut proc, mut pipes) = self.spawn(Stdio::Inherit, false)?;
        let status = proc.wait()?;
        drop(pipes.stdin.take());

        let (mut stdout, mut stderr) = (Vec::new(), Vec::new());
        // TODO: properly add Pipes
        if let Some(out) = pipes.stdout.take() {
            out.read_to_end(&mut stdout)?;
        }

        if let Some(err) = pipes.stderr.take() {
            err.read_to_end(&mut stderr)?;
        }
        Ok((status, stdout, stderr))
    }
}

impl From<AnonPipe> for Stdio {
    fn from(pipe: AnonPipe) -> Stdio {
        Stdio::InheritFile(pipe.into_raw())
    }
}

impl From<io::Stdout> for Stdio {
    fn from(_: io::Stdout) -> Stdio {
        Self::InheritStdout
    }
}

impl From<io::Stderr> for Stdio {
    fn from(_: io::Stderr) -> Stdio {
        Self::InheritStderr
    }
}

impl From<File> for Stdio {
    fn from(file: File) -> Stdio {
        Stdio::InheritFile(file.into_raw())
    }
}

impl fmt::Debug for Command {
    // show all attributes
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if f.alternate() {
            let mut debug_command = f.debug_struct("Command");
            debug_command.field("program", &self.program).field("args", &self.args);
            if !self.env.is_unchanged() {
                debug_command.field("env", &self.env);
            }

            if self.cwd.is_some() {
                debug_command.field("cwd", &self.cwd);
            }

            if self.stdin.is_some() {
                debug_command.field("stdin", &self.stdin);
            }
            if self.stdout.is_some() {
                debug_command.field("stdout", &self.stdout);
            }
            if self.stderr.is_some() {
                debug_command.field("stderr", &self.stderr);
            }

            debug_command.finish()
        } else {
            if let Some(ref cwd) = self.cwd {
                write!(f, "cd {cwd:?} && ")?;
            }
            if self.env.does_clear() {
                write!(f, "env -i ")?;
                // Altered env vars will be printed next, that should exactly work as expected.
            } else {
                // Removed env vars need the command to be wrapped in `env`.
                let mut any_removed = false;
                for (key, value_opt) in self.get_envs() {
                    if value_opt.is_none() {
                        if !any_removed {
                            write!(f, "env ")?;
                            any_removed = true;
                        }
                        write!(f, "-u {} ", key.to_string_lossy())?;
                    }
                }
            }
            // Altered env vars can just be added in front of the program.
            for (key, value_opt) in self.get_envs() {
                if let Some(value) = value_opt {
                    write!(f, "{}={value:?} ", key.to_string_lossy())?;
                }
            }
            if self.program != self.args[0] {
                write!(f, "[{:?}] ", self.program)?;
            }
            write!(f, "{:?}", self.args[0])?;

            for arg in &self.args[1..] {
                write!(f, " {:?}", arg)?;
            }
            Ok(())
        }
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug, Default)]
#[non_exhaustive]
pub struct ExitStatus(u32);

impl ExitStatus {
    pub fn exit_ok(&self) -> Result<(), ExitStatusError> {
        match SysResult::try_from(self.0 as u16) {
            Ok(SysResult::Success) => Ok(()),
            Ok(SysResult::Error(e)) => Err(ExitStatusError::ErrorStatus(e)),
            Err(_) => Err(ExitStatusError::Unknown(self.0 as u32)),
        }
    }

    pub fn code(&self) -> Option<i32> {
        Some(self.0 as i32)
    }
}

impl fmt::Display for ExitStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.exit_ok() {
            Ok(()) => write!(f, "{}", "Success"),
            Err(err) => write!(f, "{}", err),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExitStatusError {
    ErrorStatus(ErrorStatus),
    Unknown(u32),
}

impl core::fmt::Display for ExitStatusError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ErrorStatus(err) => write!(f, "{}", err.as_str()),
            Self::Unknown(code) => write!(f, "<unknown exit status {}>", code),
        }
    }
}

impl ExitStatusError {
    pub fn code(self) -> Option<NonZero<i32>> {
        let i32 = Into::<ExitStatus>::into(self).code()?;
        NonZero::new(i32)
    }
}

impl Into<ExitStatus> for ExitStatusError {
    fn into(self) -> ExitStatus {
        match self {
            Self::ErrorStatus(err) => ExitStatus(err as u32),
            Self::Unknown(code) => ExitStatus(code),
        }
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct ExitCode(usize);

impl ExitCode {
    pub const SUCCESS: ExitCode = ExitCode(0);
    pub const FAILURE: ExitCode = ExitCode(1);

    pub fn as_i32(&self) -> i32 {
        self.0 as i32
    }
}

impl From<u8> for ExitCode {
    fn from(code: u8) -> Self {
        Self(code as usize)
    }
}

impl From<usize> for ExitCode {
    fn from(code: usize) -> Self {
        Self(code)
    }
}

pub struct Process(usize);

impl Process {
    pub fn id(&self) -> u32 {
        self.0 as u32
    }

    pub fn kill(&mut self) -> io::Result<()> {
        todo!("pkill is not yet implemented")
    }

    pub fn wait(&mut self) -> io::Result<ExitStatus> {
        let exit_code = syscalls::wait(self.0)?;
        Ok(ExitStatus(exit_code as u32))
    }

    pub fn try_wait(&mut self) -> io::Result<Option<ExitStatus>> {
        todo!("try_wait is not yet implemented for SafaOS, use wait instead")
    }
}

pub struct CommandArgs<'a> {
    iter: crate::slice::Iter<'a, OsString>,
}

impl<'a> Iterator for CommandArgs<'a> {
    type Item = &'a OsStr;
    fn next(&mut self) -> Option<&'a OsStr> {
        self.iter.next().map(|os| &**os)
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<'a> ExactSizeIterator for CommandArgs<'a> {
    fn len(&self) -> usize {
        self.iter.len()
    }
    fn is_empty(&self) -> bool {
        self.iter.is_empty()
    }
}

impl<'a> fmt::Debug for CommandArgs<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.iter.clone()).finish()
    }
}
