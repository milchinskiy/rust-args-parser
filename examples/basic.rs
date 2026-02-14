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

fn inc_verbose(c: &mut Ctx) {
    c.verbose = c.verbose.saturating_add(1);
}
fn set_quiet(c: &mut Ctx) {
    c.quiet = true;
}
fn set_json(c: &mut Ctx) {
    c.json = true;
}

fn set_jobs(v: &OsStr, c: &mut Ctx) {
    // Safe because `jobs_is_u32` validates first.
    c.jobs = Some(v.to_string_lossy().parse::<u32>().unwrap());
}
fn set_input(v: &OsStr, c: &mut Ctx) {
    c.input = Some(v.to_os_string());
}
fn push_file(v: &OsStr, c: &mut Ctx) {
    c.files.push(v.to_os_string());
}

fn non_empty(v: &OsStr) -> Result<(), &'static str> {
    if v.is_empty() {
        Err("value must be non-empty")
    } else {
        Ok(())
    }
}

fn jobs_is_u32(v: &OsStr) -> Result<(), &'static str> {
    non_empty(v)?;
    v.to_string_lossy().parse::<u32>().map(|_| ()).map_err(|_| "invalid number for --jobs")
}

fn main() {
    let mut env = rapp::Env { version: Some("2.0.0"), ..Default::default() };
    env.wrap_cols = 120;

    let root = rapp::CmdSpec::new("demo")
        .help("Demo app showing flags, options, positionals, groups")
        .opt(
            rapp::OptSpec::flag("verbose", inc_verbose)
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
                .validator(jobs_is_u32),
        )
        // Mutually exclusive output mode
        .opt(rapp::OptSpec::flag("quiet", set_quiet).long("quiet").help("Suppress output").group("out"))
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
