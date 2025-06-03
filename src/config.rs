//! Collects, parses, and validates command-line arguments.

use std::{env, process};

use crate::utils::log_err;

use libc::pid_t;

/// Configuration options to control the behavior of the debugger.
#[derive(Debug, Default)]
pub struct Config {
    /// Name of the tracer program (debugger).
    pub tracer: String,
    /// Path of the tracee program (program being attached to).
    pub tracee: String,
    /// Process ID of the tracee.
    pub pid: pid_t,
}

impl Config {
    /// Parses command-line arguments and returns a new [`Config`].
    ///
    /// The function will terminate the program early if:
    ///
    /// - The `--help` or `-h` flag is provided
    /// - The `--version` or `-v` flag is provided
    /// - An unrecognized or invalid command-line option is encountered
    ///
    /// # Panics
    ///
    /// Panics if the program name is missing from the command-line arguments.
    pub fn parse() -> Self {
        let mut opts = Self::default();

        let mut args = env::args();
        let program = args.next().expect("missing program name");

        while let Some(arg) = args.next() {
            if arg.starts_with('-') {
                if let Some(flag) = FLAG_REGISTRY
                    .iter()
                    .find(|f| f.names.contains(&arg.as_str()))
                {
                    match arg.as_str() {
                        "-p" | "--pid" => {
                            let pid = match args.next() {
                                Some(val) => {
                                    match val.parse::<pid_t>() {
                                        Ok(pid) if pid > 0 => pid,
                                        Ok(_) => {
                                            log_err(
                                                &program,
                                                "invald pid: pid must be greater than 0",
                                            );
                                            print_usage(&program); // Exits
                                            unreachable!();
                                        }
                                        Err(err) => {
                                            log_err(&program, format!("invald pid: {err}"));
                                            print_usage(&program); // Exits
                                            unreachable!();
                                        }
                                    }
                                }
                                None => {
                                    log_err(&program, "pid must be provided");
                                    print_usage(&program); // Exits
                                    unreachable!();
                                }
                            };

                            (flag.run)(&program, &mut opts, Some(pid));
                        }
                        _ => (flag.run)(&program, &mut opts, None),
                    }
                } else {
                    log_err(
                        &program,
                        format!("unrecognized command-line option '{}'", arg),
                    );
                    print_usage(&program); // Exits
                }
            } else {
                // Any argument not beginning with `-` is assumed to be the
                // tracee program path.
                if opts.pid == 0 {
                    opts.tracee = arg;
                }
                break;
            }
        }

        if opts.tracee.is_empty() && opts.pid == 0 {
            log_err(&program, "program name or pid must be provided");
            print_usage(&program); // Exits
        }

        opts.tracer = program;

        opts
    }
}

struct Flag {
    names: &'static [&'static str],
    description: &'static str,
    run: fn(&str, &mut Config, Option<pid_t>),
}

const FLAG_REGISTRY: &[Flag] = &[
    Flag {
        names: &["--pid", "-p"],
        description: "process ID of a running process to attach to.",
        run: |_, args, val| args.pid = val.unwrap_or(0),
    },
    Flag {
        names: &["--help", "-h"],
        description: "displays this help message.",
        run: |program, _, _| print_usage(program),
    },
    Flag {
        names: &["--version", "-v"],
        description: "prints version information.",
        run: |program, _, _| print_version(program),
    },
];

fn print_usage(program: &str) {
    println!("Usage:");
    println!("      {program} <program name>");
    println!("  or");
    println!("      {program} -p <pid>");
    println!("Options:");

    for flag in FLAG_REGISTRY {
        println!("      {:<18} {}", flag.names.join(", "), flag.description);
    }

    process::exit(1);
}

fn print_version(program: &str) {
    println!("{} {}", program, env!("CARGO_PKG_VERSION"));
    process::exit(0);
}
