use std::io::{self, BufRead, Write};

use rdbg::Config;
use rdbg::core::{Process, handle_command};
use rdbg::utils::log_err;

fn main() {
    let opts = Config::parse();

    let mut proc = match opts.pid {
        // -- Process ID provided
        1.. => Process::attach(opts.pid).unwrap_or_else(|err| {
            log_err(&opts.tracer, err);
            std::process::exit(1);
        }),
        // -- Program path provided
        _ => Process::launch(opts.tracee).unwrap_or_else(|err| {
            log_err(&opts.tracer, err);
            std::process::exit(1);
        }),
    };

    let mut stdin = io::stdin().lock();
    let mut buffer = String::with_capacity(128);

    loop {
        print!("\x1b[1;32mrdbg\x1b[0m ❯ ");
        io::stdout().flush().expect("failed to flush stdout");

        let br = stdin.read_line(&mut buffer).unwrap_or_else(|err| {
            log_err(&opts.tracer, err);
            std::process::exit(1);
        });

        if br == 0 {
            break;
        }

        // Don't include line feed in buffer slice.
        if let Err(err) = handle_command(&mut proc, &buffer[..=buffer.len()]) {
            log_err(&opts.tracer, err);
        };

        // Need to manually clear buffer.
        buffer.clear();
    }
}
