use std::mem;

use crate::{Result, errno};

use libc::{O_CLOEXEC, c_int, c_void};

/// Wrapper around [`libc::pipe`] API.
#[derive(Debug)]
pub struct Pipe {
    /// Read file descriptor of the pipe.
    read_fd: c_int,
    /// Write file descriptor of the pipe.
    write_fd: c_int,
    // Array of file descriptors for the pipe.
    fds: [c_int; 2],
}

impl Pipe {
    /// Creates a new [`Pipe`] instance.
    ///
    /// The `close_on_exec` parameter determines whether the pipe should
    /// automatically close if the process makes a call to [`libc::execlp`].
    ///
    /// This is for cases where the child process is expected to replace the
    /// process image, and duplicate file handles are not wanted.
    pub fn new(close_on_exec: bool) -> Result<Self> {
        let mut fds = [0i32; 2];

        // Set the close-on-exec (FD_CLOEXEC) flag on the two new file
        // descriptors.
        let flags = if close_on_exec { O_CLOEXEC } else { 0 };

        // Creates a unidirectional data channel that can be used for communication
        // between the tracer and tracee.
        if unsafe { libc::pipe2(fds.as_mut_ptr(), flags) } < 0 {
            return Err(errno!("failed to create pipe"));
        }

        Ok(Self {
            read_fd: 0,
            write_fd: 1,
            fds,
        })
    }

    /// Reads a 1024 byte chunk from the given [`Pipe`], returning the fixed-size
    /// buffer, and number of bytes read as a tuple.
    pub fn read(&self) -> Result<([u8; 1024], usize)> {
        let mut buffer = [0u8; 1024];

        // Read up to `buffer.len()` bytes from the file descriptor into `buffer`.
        let bytes_read = unsafe {
            libc::read(
                self.fds[self.read_fd as usize],
                buffer.as_mut_ptr() as *mut c_void,
                buffer.len(),
            )
        };

        if bytes_read < 0 {
            return Err(errno!("failed to read from pipe"));
        }

        Ok((buffer, bytes_read as usize))
    }

    /// Write the given byte buffer into [`Pipe`].
    pub fn write(&self, buffer: &[u8]) -> Result<()> {
        // Writes up to `buffer.len()` bytes from `buffer` to the file descriptor.
        if unsafe {
            libc::write(
                self.fds[self.write_fd as usize],
                buffer.as_ptr() as *const c_void,
                buffer.len(),
            )
        } < 0
        {
            return Err(errno!("failed to write to pipe"));
        }

        Ok(())
    }

    /// Close the `read` end of the given [`Pipe`].
    pub fn close_read(&mut self) {
        if self.fds[self.read_fd as usize] != -1 {
            unsafe {
                // Closes the file descriptor, so that it no longer refers to any
                // file and may be reused.
                libc::close(self.fds[self.read_fd as usize]);
            }

            let _ = self.release_read();
        }
    }

    /// Close the `write` end of the given [`Pipe`].
    pub fn close_write(&mut self) {
        if self.fds[self.write_fd as usize] != -1 {
            unsafe {
                // Closes the file descriptor, so that it no longer refers to any
                // file and may be reused.
                libc::close(self.fds[self.write_fd as usize]);
            }

            let _ = self.release_write();
        }
    }

    fn release_read(&mut self) -> c_int {
        mem::replace(&mut self.fds[self.read_fd as usize], -1)
    }

    fn release_write(&mut self) -> c_int {
        mem::replace(&mut self.fds[self.write_fd as usize], -1)
    }
}

impl Drop for Pipe {
    // Ensure both read and write ends of the pipe are closed.
    fn drop(&mut self) {
        self.close_read();
        self.close_write();
    }
}
