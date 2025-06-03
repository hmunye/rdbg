use std::{ffi, ptr};

use super::Pipe;
use crate::Result;
use crate::utils::{errno, log_err};

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
    /// Indicates whether the process has been attached to (used during cleanup).
    is_attached: bool,
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
    ///
    /// When `true`, the `debug` parameter indicates that the process should be
    /// attached to, otherwise the program is just launched.
    pub fn launch(tracee_path: String, debug: bool) -> Result<Self> {
        // Create pipe before forking to communicate errors between debugger
        // process and child process. Pass `true` to ensure file descriptors are
        // automatically closed.
        let mut channel = Pipe::new(true)?;

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
            channel.close_read();

            // Guard the `PTRACE_TRACEME` call so it only runs when requested.
            if debug
                && unsafe {
                    // Indicate that the child process can be traced by the debugger
                    // process. `pid`, `addr`, and `data` arguments are ignored.
                    libc::ptrace(
                        PTRACE_TRACEME,
                        0,
                        ptr::null_mut::<c_void>(),
                        ptr::null_mut::<c_void>(),
                    )
                } < 0
            {
                let msg: String = errno!("failed to trace child process");
                // TODO: Write could fail...
                channel.write(msg.as_bytes())?;
                std::process::exit(1);
            }

            let program_path = ffi::CString::new(tracee_path).unwrap_or_else(|err| {
                let msg = format!("failed to convert tracee program path: {err}");
                let _ = channel.write(msg.as_bytes());
                std::process::exit(1);
            });

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
                let msg: String = errno!("failed to trace child process");
                // TODO: Write could fail...
                channel.write(msg.as_bytes())?;
                std::process::exit(1);
            }
        }

        channel.close_write();
        let (msg, bytes_read) = channel.read()?;
        channel.close_read();

        // An error occurred within the child process. Wait for the child process
        // to terminate, and return it's error message.
        if bytes_read > 0 {
            if unsafe { libc::waitpid(pid, ptr::null_mut::<c_int>(), 0) } < 0 {
                return Err(errno!("failed to wait on tracee"));
            }

            // Try converting bytes to UTF-8 string.
            let err = String::from_utf8_lossy(&msg[..bytes_read]);
            return Err(err.into());
        }

        let mut proc = Self {
            pid,
            terminate: true,
            state: ProcessState::Stopped,
            is_attached: debug,
        };

        // Guard the `wait_on_signal` call so it only runs when requested
        if debug {
            // Wait for the child process to halt.
            proc.wait_on_signal()?;
        }

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
            is_attached: true,
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
                if self.is_attached {
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
                }

                // Terminate tracee if it was spawned due to [`Process::launch`].
                if self.terminate {
                    libc::kill(pid, SIGKILL);
                    libc::waitpid(pid, &mut status, 0);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::io;

    use super::*;

    fn get_process_status(pid: pid_t) -> char {
        // The `/proc` directory in Linux is a virtual filesystem that stores
        // information about processes and other systems activities (Linux `procfs`),
        // organized as files.
        //
        // The `/proc/[pid]/stat` file gives high-level information on the
        // current state of a given process.
        let stats = fs::read_to_string(format!("/proc/{pid}/stat"))
            .unwrap_or_else(|err| panic!("failed to read proc stats for /proc/{pid}/stat': {err}"));

        let last_paren_idx = stats
            .rfind(')')
            .unwrap_or_else(|| panic!("unexpected format for /proc/{pid}/stat"));

        // Return the state indicator.
        stats
            .chars()
            .nth(last_paren_idx + 2) // located right after the closing parenthesis + 2
            .unwrap_or_else(|| panic!("process state indicator missing for /proc/{pid}/stat"))
    }

    fn check_pid(pid: pid_t) -> bool {
        // If signal is 0, then no signal is sent, but existence and permission
        // checks are still performed on the given `pid`.
        let ret = unsafe { libc::kill(pid, 0) };

        let errno = io::Error::last_os_error().to_string();

        ret != -1 && !errno.contains("No such process")
    }

    #[test]
    fn process_exists() {
        let proc = Process::launch("yes".to_string(), true);
        assert_eq!(proc.is_ok(), true);

        assert_eq!(check_pid(proc.unwrap().pid()), true);
    }

    #[test]
    fn process_not_exists() {
        let proc = Process::launch("this_program_does_not_exist".to_string(), true);
        assert_eq!(proc.is_err(), true);
    }

    #[test]
    fn process_attach_valid() {
        // Does not request to trace the process.
        let target = Process::launch("target/debug/run".to_string(), false);
        assert_eq!(target.is_ok(), true);

        let target = target.unwrap();

        let proc = Process::attach(target.pid());
        assert_eq!(proc.is_ok(), true);

        // 't' indicates tracing has stopped for the process
        // (since `attach` sends SIGSTOP signal).
        assert_eq!(get_process_status(target.pid()), 't')
    }

    #[test]
    fn process_attach_invalid_pid() {
        let proc = Process::attach(0);
        assert_eq!(proc.is_err(), true);
    }

    #[test]
    fn process_resume_valid() {
        // Test: launch process and trace, then resume.
        {
            let proc = Process::launch("target/debug/run".to_string(), true);
            assert_eq!(proc.is_ok(), true);

            let mut proc = proc.unwrap();

            assert_eq!(proc.resume().is_ok(), true);

            // 'R' indicates the process is running and 'S' indicates the process
            // is sleeping in an interruptible wait (waiting to be scheduled by OS).
            assert_eq!(
                get_process_status(proc.pid()) == 'R' || get_process_status(proc.pid()) == 'S',
                true
            )
        }

        // Test: launch process, attach, then resume.
        {
            // Does not request to trace the process.
            let target = Process::launch("target/debug/run".to_string(), false);
            assert_eq!(target.is_ok(), true);

            let target = target.unwrap();

            let proc = Process::attach(target.pid());
            assert_eq!(proc.is_ok(), true);

            let mut proc = proc.unwrap();

            assert_eq!(proc.resume().is_ok(), true);

            // 'R' indicates the process is running and 'S' indicates the process
            // is sleeping in an interruptible wait (waiting to be scheduled by OS).
            assert_eq!(
                get_process_status(proc.pid()) == 'R' || get_process_status(proc.pid()) == 'S',
                true
            )
        }
    }

    #[test]
    fn process_resume_invalid() {
        let proc = Process::launch("target/debug/end".to_string(), true);
        assert_eq!(proc.is_ok(), true);

        let mut proc = proc.unwrap();

        assert_eq!(proc.resume().is_ok(), true);
        assert_eq!(proc.wait_on_signal().is_ok(), true);

        assert_eq!(proc.resume().is_err(), true);
    }
}
