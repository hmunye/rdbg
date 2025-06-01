use std::{ffi, ptr};

use crate::{Result, errno};

use libc::{PTRACE_ATTACH, PTRACE_CONT, PTRACE_TRACEME, c_void, pid_t};

/// Represents a tracee [`Process`] the debugger can interact with.
#[derive(Debug)]
pub struct Process {
    /// Process ID of the tracee.
    pid: pid_t,
}

impl Process {
    /// Begin tracing a program given it's path, returning a new [`Process`].
    pub fn launch(tracee_path: String) -> Result<Self> {
        let pid = {
            // Creates a new child process by duplicating the debugger process.
            // On success, the PID of the child process is returned in the debugger
            // process, and 0 is returned in the child.
            let ret = unsafe { libc::fork() };

            if ret < 0 {
                return Err(errno!("failed to fork parent process"));
            }

            ret
        };

        if pid == 0 {
            // Within child process...

            // Indicate that the child process can be traced by the debugger
            // process. `pid`, `addr`, and `data` arguments are ignored.
            if unsafe {
                libc::ptrace(
                    PTRACE_TRACEME,
                    0,
                    ptr::null_mut::<c_void>(),
                    ptr::null_mut::<c_void>(),
                )
            } < 0
            {
                return Err(errno!("failed to trace child process"));
            }

            let program_path = ffi::CString::new(tracee_path)
                .map_err(|err| format!("failed to convert tracee program path: {err}"))?;

            // Replaces the debugger process image with a new process image.
            // `execlp` searches for the program in the same way as the current
            // shell if it does not contain a slash (/). Arguments are accepted
            // individually instead of as an array.
            if unsafe {
                libc::execlp(
                    program_path.as_ptr(),
                    program_path.as_ptr(),
                    ptr::null_mut::<c_void>(),
                )
            } < 0
            {
                return Err(errno!("failed to exec within child process"));
            }
        }

        let mut proc = Self { pid };

        // Wait for the child process to halt.
        proc.wait_on_signal()?;

        Ok(proc)
    }

    /// Attach to a process with the specified `pid`, returning a new [`Process`].
    pub fn attach(pid: pid_t) -> Result<Self> {
        // Attach to the process specified by `pid`, making it a tracee
        // of the debugger process. The tracee is sent a SIGSTOP signal.
        if unsafe {
            libc::ptrace(
                PTRACE_ATTACH,
                pid,
                ptr::null_mut::<c_void>(),
                ptr::null_mut::<c_void>(),
            )
        } < 0
        {
            return Err(errno!("failed to attach to provided pid '{}'", pid));
        }

        let mut proc = Self { pid };

        // Wait for the child process to halt.
        proc.wait_on_signal()?;

        Ok(proc)
    }

    /// Continue execution for the halted [`Process`].
    pub fn resume(&mut self) -> Result<()> {
        // Restart the stopped tracee process. `addr` argument is ignored.
        if unsafe {
            libc::ptrace(
                PTRACE_CONT,
                self.pid,
                ptr::null_mut::<c_void>(),
                ptr::null_mut::<c_void>(),
            )
        } < 0
        {
            return Err(errno!("failed to resume execution for tracee"));
        }

        Ok(())
    }

    /// Wait on a state change for the given [`Process`].
    pub fn wait_on_signal(&mut self) -> Result<()> {
        let mut wait_status = 0;
        let options = 0;

        // Wait for state changes in the child process.
        if unsafe { libc::waitpid(self.pid, &mut wait_status, options) } < 0 {
            return Err(errno!("failed to wait on tracee"));
        }

        Ok(())
    }

    /// Return the process ID of the given [`Process`].
    pub fn pid(&self) -> pid_t {
        self.pid
    }
}
