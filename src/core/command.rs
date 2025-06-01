use crate::Result;
use crate::core::Process;

/// Process the given input for a given [`Process`].
pub fn handle_command(proc: &mut Process, input: &str) -> Result<()> {
    let command = input.split(' ').next().unwrap();

    if "continue".starts_with(command) {
        proc.resume()?;
        proc.wait_on_signal()?;
    } else {
        return Err(format!("unrecognized command '{command}'").into());
    }

    Ok(())
}
