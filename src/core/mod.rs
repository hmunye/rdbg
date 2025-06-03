//! Core library implementation of the debugger.

mod command;
pub use command::handle_command;

mod process;
pub use process::{Process, StopReason};

mod pipe;
pub(crate) use pipe::Pipe;

mod register;
#[allow(unused_imports)]
pub(crate) use register::{RegisterFormat, RegisterInfo, RegisterType};
