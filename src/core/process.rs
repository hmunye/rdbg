use std::{ffi, ptr};

use crate::utils::log_err;
use crate::{Result, errno};

use libc::{
    PTRACE_ATTACH, PTRACE_CONT, PTRACE_DETACH, PTRACE_TRACEME, SIGCONT, SIGKILL, SIGSTOP,
    WEXITSTATUS, WIFEXITED, WIFSIGNALED, WIFSTOPPED, WSTOPSIG, WTERMSIG, c_char, c_int, c_void,
    pid_t,
};

/// Represents a tracee [`Process`] the debugger can interact with.
#[derive(Debug)]
pub struct Process {
    /// Process ID of the tracee.
    pid: pid_t,
    /// Flag to determine whether to clean up the tracee. Set to `true` if created
    /// through [`Process::launch`].
    terminate: bool,
    /// The current state of the tracee.
    state: ProcessState,
}

/// Represents the current state of a [`Process`].
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum ProcessState {
    Stopped,
    Running,
    Exited,
    Terminated,
}

/// Holds information on why a [`Process`] was stopped, whether due to an exit,
/// termination, or halt.
#[derive(Debug)]
pub struct StopReason {
    /// The current state of the [`Process`].
    pub reason: ProcessState,
    /// Additional code associated with the stop, such as a signal or exit code.
    pub info: c_int,
}

impl StopReason {
    /// Parse the `wait_status`, populated by [`libc::waitpid`], returning a new
    /// [`StopReason`].
    fn new(wait_status: c_int) -> Self {
        let reason: ProcessState;
        let info: c_int;

        if WIFEXITED(wait_status) {
            // Child process terminated normally.
            reason = ProcessState::Exited;
            info = WEXITSTATUS(wait_status);
        } else if WIFSIGNALED(wait_status) {
            // Child process was terminated by a signal.
            reason = ProcessState::Terminated;
            info = WTERMSIG(wait_status);
        } else if WIFSTOPPED(wait_status) {
            // Child process was stopped by delivery of a signal.
            reason = ProcessState::Stopped;
            info = WSTOPSIG(wait_status);
        } else {
            log_err(
                "rdbg",
                format!("could not retrieve reason and info for wait_status '{wait_status}'"),
            );

            reason = ProcessState::Stopped;
            info = -1;
        }

        Self { reason, info }
    }

    /// Log details of the [`StopReason`] for the given [`Process`].
    pub fn log_stop_reason(&self, proc: &Process) {
        match self.reason {
            ProcessState::Exited => {
                println!("process {} exited with status {}", proc.pid, self.info);
            }
            ProcessState::Terminated => {
                let signal = {
                    // Returns a string describing the signal number provided.
                    let ptr = unsafe { libc::strsignal(self.info) };

                    if ptr.is_null() {
                        "UNKNOWN"
                    } else {
                        let c_str = unsafe { ffi::CStr::from_ptr(ptr as *const c_char) };
                        c_str.to_str().unwrap_or("UNKNOWN")
                    }
                };

                println!("process {} terminated with signal {}", proc.pid, signal);
            }
            ProcessState::Stopped => {
                let signal = {
                    // Returns a string describing the signal number provided.
                    let ptr = unsafe { libc::strsignal(self.info) };

                    if ptr.is_null() {
                        "UNKNOWN"
                    } else {
                        let c_str = unsafe { ffi::CStr::from_ptr(ptr as *const c_char) };

                        c_str.to_str().unwrap_or("UNKNOWN")
                    }
                };

                println!("process {} stopped with signal {}", proc.pid, signal);
            }
            _ => {
                log_err(
                    "rdbg",
                    format!("provided invalid stop_reason reason '{:?}'", self.reason),
                );
            }
        }
    }
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
                    ptr::null_mut::<c_char>(),
                )
            } < 0
            {
                return Err(errno!("failed to exec within child process"));
            }
        }

        let mut proc = Self {
            pid,
            terminate: true,
            state: ProcessState::Stopped,
        };

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

        let mut proc = Self {
            pid,
            terminate: true,
            state: ProcessState::Stopped,
        };

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

        self.state = ProcessState::Running;

        Ok(())
    }

    /// Wait on a state change for the given [`Process`], returning a new [`StopReason`]
    pub fn wait_on_signal(&mut self) -> Result<StopReason> {
        let mut wait_status = 0;
        let options = 0;

        // Wait for state changes in the child process.
        if unsafe { libc::waitpid(self.pid, &mut wait_status, options) } < 0 {
            return Err(errno!("failed to wait on tracee"));
        }

        let reason = StopReason::new(wait_status);
        self.state = reason.reason;

        Ok(reason)
    }

    /// Return the process ID of the given [`Process`].
    pub fn pid(&self) -> pid_t {
        self.pid
    }

    /// Return the current state of the given [`Process`].
    pub fn state(&self) -> ProcessState {
        self.state
    }
}

impl Drop for Process {
    fn drop(&mut self) {
        if self.pid != 0 {
            let pid = self.pid;
            let mut status = 0;

            unsafe {
                if self.state == ProcessState::Running {
                    // Send a signal to the tracee to stop execution.
                    libc::kill(pid, SIGSTOP);
                    // Wait for a state change from the tracee.
                    libc::waitpid(pid, &mut status, 0);
                }

                // Detach from the tracee, then restart execution.
                libc::ptrace(
                    PTRACE_DETACH,
                    pid,
                    ptr::null_mut::<c_void>(),
                    ptr::null_mut::<c_void>(),
                );
                // Send a signal to tracee to ensure it continues execution.
                libc::kill(pid, SIGCONT);

                // Terminate tracee if it was spawned due to [`Process::launch`].
                if self.terminate {
                    libc::kill(pid, SIGKILL);
                    libc::waitpid(pid, &mut status, 0);
                }
            }
        }
    }
}
