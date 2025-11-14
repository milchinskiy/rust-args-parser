#![allow(clippy::unnecessary_wraps)]

use rust_args_parser as rapp;
use std::ffi::{OsStr, OsString};

#[derive(Default, Debug)]
struct Ctx {
    verbose: u8,
    jobs: Option<u32>,
    input: Option<OsString>,
    files: Vec<OsString>,
    quiet: bool,
    json: bool,
}

fn set_verbose(c: &mut Ctx) -> rapp::Result<()> {
    c.verbose = c.verbose.saturating_add(1);
    Ok(())
}
fn set_quiet(c: &mut Ctx) -> rapp::Result<()> {
    c.quiet = true;
    Ok(())
}
fn set_json(c: &mut Ctx) -> rapp::Result<()> {
    c.json = true;
    Ok(())
}

fn set_jobs(v: &OsStr, c: &mut Ctx) -> rapp::Result<()> {
    let s = v.to_string_lossy();
    let n: u32 = s.parse().map_err(|_| rapp::Error::User("invalid number for --jobs".into()))?;
    c.jobs = Some(n);
    Ok(())
}
fn set_input(v: &OsStr, c: &mut Ctx) -> rapp::Result<()> {
    c.input = Some(v.to_os_string());
    Ok(())
}
fn push_file(v: &OsStr, c: &mut Ctx) -> rapp::Result<()> {
    c.files.push(v.to_os_string());
    Ok(())
}

fn non_empty(v: &OsStr) -> rapp::Result<()> {
    if v.is_empty() {
        Err(rapp::Error::User("value must be non-empty".into()))
    } else {
        Ok(())
    }
}

fn main() {
    let mut env = rapp::Env { version: Some("0.1.0"), ..Default::default() };
    env.wrap_cols = 120;

    let root = rapp::CmdSpec::<'_, Ctx>::new("demo")
        .help("Demo app showing flags, options, positionals, groups")
        .opt(
            rapp::OptSpec::flag("verbose", set_verbose)
                .short('v')
                .long("verbose")
                .help("Increase verbosity")
                .repeatable(),
        )
        .opt(
            rapp::OptSpec::value("jobs", set_jobs)
                .short('j')
                .long("jobs")
                .metavar("N")
                .help("Number of jobs")
                .validator(non_empty),
        )
        // Mutually exclusive output mode
        .opt(
            rapp::OptSpec::flag("quiet", set_quiet)
                .long("quiet")
                .help("Suppress output")
                .group("out"),
        )
        .opt(rapp::OptSpec::flag("json", set_json).long("json").help("JSON output").group("out"))
        .group("out", rapp::GroupMode::Xor)
        // Positionals: INPUT (required), then FILE... (zero or more)
        .pos(rapp::PosSpec::new("INPUT", set_input).help("Primary input").required())
        .pos(rapp::PosSpec::new("FILE", push_file).help("Additional files").many());

    let argv: Vec<OsString> = std::env::args_os().skip(1).collect();
    let mut ctx = Ctx::default();
    let matches = match rapp::parse(&env, &root, &argv, &mut ctx) {
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
        Ok(m) => m,
    };

    // Your app logic
    if matches.is_set_from("json", rapp::Source::Cli) {
        eprintln!("json explicitly set via CLI");
    }
}
