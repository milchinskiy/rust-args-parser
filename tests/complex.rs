// tests/golden_complex.rs
use rust_args_parser as ap;
mod common;
use common::*;
use std::env;
use std::ffi::{OsStr, OsString};

fn golden_spec(env_name: &str) -> ap::CmdSpec<'_, Ctx> {
    // Root (global) options
    ap::CmdSpec::new("tool")
        .help("Multitool root")
        .opt(ap::OptSpec::flag("verbose", inc_verbose).short('v').long("verbose").help("Increase verbosity"))
        .opt(ap::OptSpec::flag("json", set_json).long("json").help("JSON output"))
        .opt(ap::OptSpec::value("jobs", set_jobs).short('j').long("jobs").metavar("N").help("worker threads"))
        .opt(
            ap::OptSpec::value("limit", set_limit)
                .long("limit")
                .metavar("N")
                .env(env_name)
                .default(OsString::from("42")),
        )
        .subcmd(
            // tool repo push <REMOTE> <BRANCH> [FILE...]
            ap::CmdSpec::new("repo").help("Repository ops").subcmd(
                ap::CmdSpec::new("push")
                    .help("Push to remote")
                    .pos(ap::PosSpec::new("REMOTE", set_input).required())
                    // BRANCH is validated only via view below; we don't mutate Ctx for it
                    .pos(ap::PosSpec::new("BRANCH", |_, _: &mut Ctx| {}).required())
                    .pos(ap::PosSpec::new("FILE", push_file).many()),
            ),
        )
}

#[test]
fn golden_complex_cli() {
    let envv = env_basic();
    // Prove overlay order: default(42) < env(99) < cli(5)
    env::set_var("APP_LIMIT_1", "99");

    let root = golden_spec("APP_LIMIT_1");
    let mut ctx = Ctx::default();
    let argv = argv(&[
        "-vv",       // short cluster ⇒ verbose twice
        "--json",    // global flag
        "-j8",       // inline short value
        "--limit=5", // CLI overrides env (and default)
        "repo",
        "push",   // nested subcommands
        "origin", // REMOTE ⇒ ctx.input
        "main",   // BRANCH (verified via view)
        "--",     // end-of-options; rest must be positional FILEs
        "-nasty.rs",
        "a.rs",
        "b.rs",
    ]);

    let m = ap::parse(&envv, &root, &argv, &mut ctx).expect("parse ok");

    // Effects on Ctx via callbacks
    assert_eq!(ctx.verbose, 2);
    assert!(ctx.json);
    assert_eq!(ctx.jobs, Some(8));
    assert_eq!(ctx.limit.as_deref(), Some("5")); // CLI beats env (99) and default (42)
    assert_eq!(ctx.input.as_deref(), Some(OsStr::new("origin")));
    assert_eq!(ctx.files, vec![OsString::from("-nasty.rs"), OsString::from("a.rs"), OsString::from("b.rs"),]);

    // Matches / leaf scoping
    assert_eq!(m.leaf_path(), vec!["repo", "push"]);
    let v = m.view();
    assert_eq!(v.pos_one("BRANCH").unwrap(), OsStr::new("main"));

    // Provenance checks
    let rootv = m.at(&[]);
    assert!(rootv.is_set_from("limit", ap::Source::Cli));
    assert!(rootv.is_set_from("json", ap::Source::Cli));
    assert!(rootv.is_set_from("verbose", ap::Source::Cli));
}

#[test]
fn golden_complex_cli_env_overlay() {
    let envv = env_basic();
    std::env::set_var("APP_LIMIT_2", "77");

    let root = golden_spec("APP_LIMIT_2");
    let mut ctx = Ctx::default();
    let argv = argv(&["-v", "repo", "push", "o", "m", "--", "f1"]);
    let m = ap::parse(&envv, &root, &argv, &mut ctx).unwrap();

    assert_eq!(ctx.verbose, 1);
    assert_eq!(ctx.limit.as_deref(), Some("77"));
    let rootv = m.at(&[]);
    assert!(rootv.is_set_from("limit", ap::Source::Env));
}
