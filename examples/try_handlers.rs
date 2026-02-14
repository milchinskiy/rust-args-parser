#![allow(clippy::unnecessary_wraps)]
//! Demonstrates the `*_try` APIs for fallible callbacks and typed user errors.
//!
//! The parser still reports its own parse-level errors (unknown option, missing value, etc.),
//! but user callbacks/validators/handlers can return arbitrary error types.

use rust_args_parser as rapp;
use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};

#[derive(Default, Debug)]
struct Ctx {
    verbose: u8,
    name: Option<String>,
    port: Option<u16>,
    path: Option<PathBuf>,
}

#[derive(Debug)]
struct AppErr(String);
impl AppErr {
    fn new(msg: impl Into<String>) -> Self {
        Self(msg.into())
    }
}
impl core::fmt::Display for AppErr {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(&self.0)
    }
}
impl std::error::Error for AppErr {}

fn inc_verbose(c: &mut Ctx) -> Result<(), AppErr> {
    c.verbose = c.verbose.saturating_add(1);
    Ok(())
}

fn set_name(v: &OsStr, c: &mut Ctx) -> Result<(), AppErr> {
    let s = v.to_str().ok_or_else(|| AppErr::new("--name must be valid UTF-8"))?;
    if s.trim().is_empty() {
        return Err(AppErr::new("--name must be non-empty"));
    }
    c.name = Some(s.to_owned());
    Ok(())
}

fn set_port(v: &OsStr, c: &mut Ctx) -> Result<(), AppErr> {
    let s = v.to_str().ok_or_else(|| AppErr::new("--port must be valid UTF-8"))?;
    let p: u16 = s.parse().map_err(|_| AppErr::new("--port must be an integer in 0..=65535"))?;
    c.port = Some(p);
    Ok(())
}

fn set_path(v: &OsStr, c: &mut Ctx) -> Result<(), AppErr> {
    if v.is_empty() {
        return Err(AppErr::new("PATH must be non-empty"));
    }
    c.path = Some(PathBuf::from(v));
    Ok(())
}

// Validator returning a displayable error: maps into `Error::User(String)`.
fn name_is_not_root(v: &OsStr) -> Result<(), &'static str> {
    if v == OsStr::new("root") {
        Err("'root' is not allowed")
    } else {
        Ok(())
    }
}

// Validator returning a typed error: maps into `Error::UserAny(Box<dyn Error>)`.
fn port_is_not_zero(v: &OsStr) -> Result<(), AppErr> {
    let s = v.to_str().ok_or_else(|| AppErr::new("--port must be valid UTF-8"))?;
    let p: u16 = s.parse().map_err(|_| AppErr::new("--port must be an integer"))?;
    if p == 0 {
        Err(AppErr::new("--port must not be 0"))
    } else {
        Ok(())
    }
}

fn validate_cmd(m: &rapp::Matches) -> Result<(), AppErr> {
    // The handler will run only after successful validation.
    if !m.is_set("name") {
        return Err(AppErr::new("--name is required"));
    }
    Ok(())
}

fn handler(_m: &rapp::Matches, c: &mut Ctx) -> Result<(), AppErr> {
    let name = c.name.as_deref().unwrap_or("<none>");
    let port = c.port.unwrap_or(0);
    let path = c.path.as_deref().unwrap_or_else(|| Path::new("."));
    eprintln!("name={name} port={port} path={path:?} verbose={}", c.verbose);
    Ok(())
}

fn main() {
    let env = rapp::Env { version: Some("2.0.0"), ..Default::default() };

    let root = rapp::CmdSpec::new("try-demo")
        .help("Demonstrate *_try callbacks with typed user errors")
        .validator_try(validate_cmd)
        .opt(
            rapp::OptSpec::flag_try("verbose", inc_verbose)
                .short('v')
                .long("verbose")
                .help("Increase verbosity")
                .repeatable(),
        )
        .opt(
            rapp::OptSpec::value_try("name", set_name)
                .long("name")
                .metavar("NAME")
                .help("User name")
                // This validator uses a Display-only error, mapped to `Error::User(String)`.
                .validator(name_is_not_root),
        )
        .opt(
            rapp::OptSpec::value_try("port", set_port)
                .long("port")
                .metavar("PORT")
                .help("TCP port")
                // This validator uses a typed error, mapped to `Error::UserAny`.
                .validator_try(port_is_not_zero),
        )
        .pos(rapp::PosSpec::new_try("PATH", set_path).help("Target path").required())
        .handler_try(handler);

    let argv: Vec<OsString> = std::env::args_os().skip(1).collect();
    let mut ctx = Ctx::default();
    match rapp::parse(&env, &root, &argv, &mut ctx) {
        Ok(_) => {}
        Err(rapp::Error::ExitMsg { code, message }) => {
            if let Some(msg) = message {
                print!("{msg}");
            }
            std::process::exit(code);
        }
        Err(rapp::Error::UserAny(e)) => {
            // Typed user errors can be downcast.
            if let Some(app) = e.downcast_ref::<AppErr>() {
                eprintln!("app error: {app}");
            } else {
                eprintln!("user error: {e}");
            }
            std::process::exit(2);
        }
        Err(e) => {
            // Parse-level errors (unknown option, missing value, etc.).
            eprintln!("error: {e}");
            std::process::exit(2);
        }
    }
}
