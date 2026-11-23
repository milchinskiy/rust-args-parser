//! `cargo run --features derive --example derive_basic -- -vv --jobs 4 input.txt`
#[allow(dead_code)]
use rust_args_parser::{Parser, Subcommand};
use std::ffi::{OsStr, OsString};

#[derive(Default)]
struct App {
    verbose: usize,
    last_input: Option<OsString>,
}

fn verbose(ctx: &mut App) {
    ctx.verbose += 1;
}
fn input(value: &OsStr, ctx: &mut App) {
    ctx.last_input = Some(value.to_os_string());
}
fn run(matches: &rust_args_parser::Matches, ctx: &mut App) {
    println!("input={:?}, verbosity={}, command={:?}", ctx.last_input, ctx.verbose, matches.leaf_path());
}

#[derive(Debug, Parser)]
#[command(name = "demo", help = "Parse files", context = App, handler = run)]
struct Cli {
    #[arg(short = 'v', action = "count", on_flag = verbose)]
    verbose: usize,
    #[arg(short = 'j', long, default_value = "2", env = "DEMO_JOBS")]
    jobs: u32,
    #[arg(positional, value_name = "INPUT", on_value = input)]
    input: OsString,
    #[command(subcommand)]
    command: Option<Command>,
}

fn show(_: &rust_args_parser::Matches, _: &mut App) {
    println!("show command");
}

#[derive(Debug, Subcommand)]
#[command(context = App)]
enum Command {
    #[command(help = "Show input details", handler = show)]
    Show,
}

fn main() {
    let mut app = App::default();
    match Cli::try_parse_from_with_context(std::env::args_os().skip(1), &mut app) {
        Ok(cli) => {
            println!("parsed: verbose={}, jobs={}, input={:?}", cli.verbose, cli.jobs, cli.input);
            if matches!(cli.command, Some(Command::Show)) {
                println!("selected: show");
            }
        }
        Err(rust_args_parser::Error::ExitMsg { code, message }) => {
            if let Some(text) = message {
                print!("{text}");
            }
            std::process::exit(code);
        }
        Err(e) => {
            eprintln!("error: {e}");
            std::process::exit(2);
        }
    }
}
