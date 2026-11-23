//! `cargo run --features derive --example derive_subcommands -- remote add origin https://example.com`
use rust_args_parser::{Args, Parser, Subcommand};
use std::ffi::OsStr;

#[derive(Default, Debug)]
struct App {
    remote_name: Option<String>,
}

fn set_name(value: &OsStr, app: &mut App) {
    app.remote_name = Some(value.to_string_lossy().into_owned());
}
fn add(matches: &rust_args_parser::Matches, app: &mut App) {
    println!("added remote {:?}, selected={:?}", app.remote_name, matches.leaf_path());
}
fn list(_: &rust_args_parser::Matches, _: &mut App) {
    println!("listing remotes");
}
fn status(_: &rust_args_parser::Matches, _: &mut App) {
    println!("status");
}

#[derive(Debug, Parser)]
#[command(name = "git-mini", help = "Manage remotes", context = App)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
#[command(context = App)]
enum Command {
    #[command(alias = "r", help = "Remote operations")]
    Remote(Remote),
    #[command(help = "Show status", handler = status)]
    Status,
}

#[derive(Debug, Args)]
#[command(context = App)]
struct Remote {
    #[command(subcommand)]
    operation: RemoteCommand,
}

#[derive(Debug, Subcommand)]
#[command(context = App)]
enum RemoteCommand {
    #[command(help = "Add a remote", handler = add)]
    Add {
        #[arg(positional, value_name = "NAME", on_value = set_name)]
        name: String,
        #[arg(positional, value_name = "URL")]
        url: String,
    },
    #[command(help = "List remotes", handler = list)]
    List,
}

fn main() {
    let mut app = App::default();
    match Cli::try_parse_from_with_context(std::env::args_os().skip(1), &mut app) {
        Ok(Cli { command }) => match command {
            Command::Remote(Remote { operation }) => match operation {
                RemoteCommand::Add { name, url } => println!("parsed: remote add {name} {url}"),
                RemoteCommand::List => {}
            },
            Command::Status => {}
        },
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
