//! `cargo run --features derive --example derive_subcommands -- remote add origin https://example.com`

use rust_args_parser::{Args, Parser, Subcommand};
use std::ffi::OsStr;

#[derive(Debug, Parser)]
#[command(name = "git-mini", help = "Manage remotes")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    #[command(alias = "r", help = "Remote operations")]
    Remote(Remote),
    #[command(help = "Show status", handler = Self::status)]
    Status,
}

impl Command {
    fn status(_: &Self) {
        println!("status: clean");
    }
}

#[derive(Debug, Args)]
struct Remote {
    #[command(subcommand)]
    operation: RemoteCommand,
}

#[derive(Debug, Subcommand)]
enum RemoteCommand {
    #[command(help = "Add a remote", handler = Self::add)]
    Add {
        #[arg(positional, value_name = "NAME", on_value = Self::on_name)]
        name: String,
        #[arg(positional, value_name = "URL")]
        url: String,
        #[arg(skip)]
        name_checked: bool,
    },
    #[command(help = "List remotes", handler = Self::list)]
    List,
}

impl RemoteCommand {
    fn on_name(&mut self, value: &OsStr) {
        if let Self::Add { name_checked, .. } = self {
            *name_checked = !value.is_empty();
        }
    }

    fn add(&mut self) {
        if let Self::Add { name, url, name_checked } = self {
            assert!(*name_checked);
            println!("added remote {name} -> {url}");
        }
    }

    fn list(_: &Self) {
        println!("listing remotes");
    }
}

fn main() {
    match Cli::try_parse() {
        Ok(Cli { command }) => match command {
            Command::Remote(Remote { operation }) => match operation {
                RemoteCommand::Add { name, url, name_checked } => {
                    println!("completed: {name} {url} (checked: {name_checked})");
                }
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
        Err(error) => {
            eprintln!("error: {error}");
            std::process::exit(2);
        }
    }
}
