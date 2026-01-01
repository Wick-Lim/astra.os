// Process stub for ASTRA.OS
// This file goes in: rust/library/std/src/sys/astra_os/process.rs

use crate::ffi::OsStr;
use crate::fmt;
use crate::io;
use crate::sys::unsupported;

pub struct Command {
    program: OsString,
}

impl Command {
    pub fn new(program: &OsStr) -> Command {
        Command {
            program: program.to_owned(),
        }
    }

    pub fn arg(&mut self, _arg: &OsStr) {
        // Stub: Ignore arguments
    }

    pub fn env(&mut self, _key: &OsStr, _val: &OsStr) {
        // Stub: Ignore environment
    }

    pub fn cwd(&mut self, _dir: &OsStr) {
        // Stub: Ignore working directory
    }

    pub fn stdin(&mut self, _stdin: Stdio) {
        // Stub: Ignore stdin
    }

    pub fn stdout(&mut self, _stdout: Stdio) {
        // Stub: Ignore stdout
    }

    pub fn stderr(&mut self, _stderr: Stdio) {
        // Stub: Ignore stderr
    }

    pub fn spawn(&mut self) -> io::Result<Process> {
        unsupported!()
    }
}

impl fmt::Debug for Command {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Command")
            .field("program", &self.program)
            .finish()
    }
}

pub struct Process;

impl Process {
    pub fn id(&self) -> u32 {
        0
    }

    pub fn kill(&mut self) -> io::Result<()> {
        unsupported!()
    }

    pub fn wait(&mut self) -> io::Result<ExitStatus> {
        unsupported!()
    }

    pub fn try_wait(&mut self) -> io::Result<Option<ExitStatus>> {
        unsupported!()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ExitStatus(i32);

impl ExitStatus {
    pub fn success(&self) -> bool {
        self.0 == 0
    }

    pub fn code(&self) -> Option<i32> {
        Some(self.0)
    }
}

impl fmt::Display for ExitStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "exit code: {}", self.0)
    }
}

pub struct Stdio;

impl Stdio {
    pub fn piped() -> Stdio {
        Stdio
    }

    pub fn inherit() -> Stdio {
        Stdio
    }

    pub fn null() -> Stdio {
        Stdio
    }
}

use crate::ffi::OsString;

pub fn abort_internal() -> ! {
    loop {
        #[cfg(target_arch = "x86_64")]
        unsafe {
            core::arch::asm!("hlt");
        }
    }
}
