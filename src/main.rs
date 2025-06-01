use std::{ffi, ptr};

use rdbg::utils::log_err;
use rdbg::{Config, Result, errno};

use libc::{PTRACE_ATTACH, PTRACE_TRACEME, c_void, pid_t};

fn main() {
    let opts = Config::parse();

    let pid = attach(opts.pid, opts.tracee).unwrap_or_else(|err| {
        log_err(&opts.tracer, err);
        std::process::exit(1);
    });

    let mut wait_status = 0;
    let options = 0;

    // Wait for state changes in the child process.
    if unsafe { libc::waitpid(pid, &mut wait_status, options) } < 0 {
        log_err(
            &opts.tracer,
            format!(
                "failed to wait on child process: {}",
                std::io::Error::last_os_error()
            ),
        );
        std::process::exit(1);
    }
}

fn attach(pid: pid_t, tracee: String) -> Result<pid_t> {
    match pid {
        // -- Process ID provided
        1.. => {
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
                return Err(errno!("failed to attach to provided pid"));
            }

            Ok(pid)
        }
        // -- Program path provided
        _ => {
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

                let program_path = ffi::CString::new(tracee)
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

            // Process ID of the child process
            Ok(pid)
        }
    }
}
