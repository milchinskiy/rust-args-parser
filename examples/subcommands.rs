use rust_args_parser as rapp;
use std::ffi::{OsStr, OsString};

#[derive(Default, Debug)]
struct Ctx {
    verbose: u8,
    base: Option<OsString>,
    remote_name: Option<OsString>,
    url: Option<OsString>,
}

fn inc_verbose(c: &mut Ctx) {
    c.verbose = c.verbose.saturating_add(1);
}
fn set_name(v: &OsStr, c: &mut Ctx) {
    c.remote_name = Some(v.to_os_string());
}
fn set_url(v: &OsStr, c: &mut Ctx) {
    c.url = Some(v.to_os_string());
}

fn handle_remote_add(m: &rapp::Matches, c: &mut Ctx) {
    eprintln!(
        "[handler] remote add: base={:?} name={:?} url={:?}",
        c.base.as_deref(),
        c.remote_name.as_deref(),
        c.url.as_deref()
    );
    // You can also read from Matches if you prefer keys:
    let _name = m.get_value("NAME");
}

fn main() {
    let env = rapp::Env {
        version: Some("2.0.0"),
        author: Some("Rust Args Parser"),
        ..Default::default()
    };

    let remote_add = rapp::CmdSpec::new("add")
        .help("Add a remote")
        .pos(rapp::PosSpec::new("NAME", set_name).help("Remote name").required())
        .opt(
            rapp::OptSpec::value("url", set_url)
                .long("url")
                .default("https://example.com")
                .metavar("URL")
                .help("Remote URL"),
        )
        .handler(handle_remote_add);

    let remote = rapp::CmdSpec::new("remote")
        .help("Remote management")
        .alias("rmt")
        .alias("add-remote")
        .opt(
            rapp::OptSpec::flag("verbose", inc_verbose)
                .short('v')
                .long("verbose")
                .help("Increase verbosity")
                .repeatable(),
        )
        .opt(
            rapp::OptSpec::value("base", |v, c: &mut Ctx| {
                c.base = Some(v.to_os_string());
            })
            .default("some base value")
            .metavar("PATH")
            .long("base")
            .help("Base path"),
        )
        .subcmd(remote_add);

    let root = rapp::CmdSpec::<'_, Ctx>::new("sc").help("Tool with nested subcommands").subcmd(remote);

    let argv: Vec<OsString> = std::env::args_os().skip(1).collect();
    let mut ctx = Ctx::default();
    match rapp::parse(&env, &root, &argv, &mut ctx) {
        Err(rapp::Error::ExitMsg { code, message }) => {
            if let Some(msg) = message {
                println!("{msg}");
            }
            std::process::exit(code);
        }
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(2);
        }
        Ok(_) => {}
    }
}
