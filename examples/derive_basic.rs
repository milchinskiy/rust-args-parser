//! `cargo run --features derive --example derive_basic -- -vv --jobs 4 input.txt`

use rust_args_parser::{Parser, Subcommand};
use std::ffi::{OsStr, OsString};

#[derive(Debug, Parser)]
#[command(name = "demo", help = "Process an input file", handler = Self::run)]
struct Cli {
    #[arg(short = 'v', action = "count", on_flag = Self::on_verbose, help = "Verbosity level")]
    verbose: usize,
    #[arg(short = 'j', long, default_value = "2", env = "DEMO_JOBS")]
    jobs: u32,
    #[arg(positional, value_name = "INPUT", on_value = Self::on_input, help = "input help")]
    input: OsString,
    #[command(subcommand)]
    command: Option<Command>,
    #[arg(skip)]
    observed_flags: usize,
    #[arg(skip)]
    observed_input: Option<OsString>,
}

impl Cli {
    fn on_verbose(&mut self) {
        self.observed_flags += 1;
    }

    fn on_input(&mut self, value: &OsStr) {
        self.observed_input = Some(value.to_os_string());
    }

    fn run(&self) {
        println!("processing {:?} with {} jobs (verbosity {})", self.input, self.jobs, self.verbose);
    }
}

#[derive(Debug, Subcommand)]
enum Command {
    #[command(help = "Show input information", handler = Self::show)]
    Show,
}

impl Command {
    fn show(_: &Self) {
        println!("show command selected");
    }
}

fn main() {
    match Cli::try_parse() {
        Ok(cli) => {
            println!("observed {} flags, input {:?}", cli.observed_flags, cli.observed_input);
            if cli.command.is_some() {
                println!("subcommand completed (jobs: {})", cli.jobs);
            }
        }
        Err(rust_args_parser::Error::ExitMsg { code, message }) => {
            if let Some(text) = message {
                print!("{text}");
            }
            std::process::exit(code);
        }
        Err(error) => {
            eprintln!("error: {error}");
            std::process::exit(2);
        }
    }
}
