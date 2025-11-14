#![allow(clippy::unnecessary_wraps)]

use rust_args_parser as rapp;
use std::ffi::{OsStr, OsString};

#[derive(Default, Debug)]
struct Ctx {
    socket: Option<std::path::PathBuf>,
}

fn set_socket(v: &OsStr, c: &mut Ctx) -> rapp::Result<()> {
    c.socket = Some(std::path::PathBuf::from(v));
    Ok(())
}
fn non_empty(v: &OsStr) -> rapp::Result<()> {
    if v.is_empty() {
        Err(rapp::Error::User("empty value".into()))
    } else {
        Ok(())
    }
}

fn main() -> rapp::Result<()> {
    let env = rapp::Env::default();

    let root =
        rapp::CmdSpec::<'_, Ctx>::new("svc").help("Service tool with ENV/DEFAULT overlays").opt(
            rapp::OptSpec::value("socket", set_socket)
                .long("socket")
                .short('s')
                .metavar("PATH")
                .help("Control socket path")
                .env("SVC_SOCKET")
                .default_os("/run/svc.sock")
                .validator(non_empty),
        );

    let argv: Vec<OsString> = std::env::args_os().skip(1).collect();
    let mut ctx = Ctx::default();
    match rapp::parse(&env, &root, &argv, &mut ctx) {
        Err(rapp::Error::ExitMsg { code, message }) => {
            if let Some(msg) = message {
                print!("{msg}");
            }
            std::process::exit(code);
        }
        Err(e) => {
            eprintln!("error: {e}");
            std::process::exit(2);
        }
        Ok(_) => {}
    }

    eprintln!("socket = {:?}", ctx.socket);
    Ok(())
}
