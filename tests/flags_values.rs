use rust_args_parser as ap;
use std::ffi::OsStr;
mod common;
use common::*;

fn root_spec<'a>() -> ap::CmdSpec<'a, Ctx> {
    ap::CmdSpec::new("demo")
        .help("Demo root")
        .opt(ap::OptSpec::flag("verbose", inc_verbose).short('v').long("verbose").help("verbose"))
        .opt(ap::OptSpec::flag("json", set_json).long("json").help("json output"))
        .opt(
            ap::OptSpec::value("jobs", set_jobs)
                .short('j')
                .long("jobs")
                .metavar("N")
                .help("worker threads"),
        )
        .opt(
            ap::OptSpec::value("limit", set_limit)
                .long("limit")
                .metavar("N")
                .help("optional limit"),
        )
        .pos(ap::PosSpec::new("INPUT", set_input).range(0, 1))
        .pos(ap::PosSpec::new("FILE", push_file).many())
}

#[test]
fn short_clusters_and_inline_values() {
    let env = env_basic();
    let root = root_spec();
    let mut ctx = Ctx::default();
    let argv = argv(&["-vvj10", "-v", "--limit=5", "input.txt", "a", "b"]);
    let m = ap::parse(&env, &root, &argv, &mut ctx).expect("parse");

    assert_eq!(ctx.verbose, 3);
    assert_eq!(ctx.jobs, Some(10));
    assert_eq!(ctx.limit.as_deref(), Some("5"));
    assert_eq!(ctx.input.as_deref(), Some(OsStr::new("input.txt")));
    assert_eq!(ctx.files.len(), 2);

    let leaf = m.view();
    assert!(leaf.is_set("verbose"));
    assert!(m.is_set_from("limit", ap::Source::Cli));
}

#[test]
fn numeric_lookahead_treats_negative_as_value() {
    // No short option starting with '-' exists; "-0.25" must be positional/value, not an option error
    let env = env_basic();
    let root = ap::CmdSpec::new("demo").pos(ap::PosSpec::new("VAL", |v, c: &mut Ctx| {
        c.limit = Some(v.to_string_lossy().into());
        Ok(())
    }));
    let mut ctx = Ctx::default();
    let argv = argv(&["-0.25"]);
    let m = ap::parse(&env, &root, &argv, &mut ctx).expect("parse ok");
    assert_eq!(ctx.limit.as_deref(), Some("-0.25"));
    assert!(m.view().pos_one("VAL").is_some());
}

#[test]
fn end_of_options_marker() {
    let env = env_basic();
    // Use a minimal spec so that tokens after `--` go to FILEs, not to an optional INPUT.
    let root = ap::CmdSpec::new("demo").pos(ap::PosSpec::new("FILE", push_file).many());
    let mut ctx = Ctx::default();
    let argv = argv(&["--", "-vv", "file.txt"]);
    ap::parse(&env, &root, &argv, &mut ctx).expect("parse");
    // "-vv" must be treated as a file
    assert_eq!(
        ctx.files,
        vec![std::ffi::OsString::from("-vv"), std::ffi::OsString::from("file.txt")]
    );
}

#[test]
fn missing_required_value_errors() {
    let env = env_basic();
    let root = ap::CmdSpec::new("demo").opt(ap::OptSpec::value("jobs", set_jobs).short('j'));
    let mut ctx = Ctx::default();
    let argv = argv(&["-j"]);
    let err = ap::parse(&env, &root, &argv, &mut ctx).unwrap_err();
    match err {
        ap::Error::MissingValue { opt } => assert_eq!(opt, "-j"),
        _ => panic!("{err:?}"),
    }
}

#[test]
fn short_inline_negative_value() {
    let env = env_basic();
    let root =
        ap::CmdSpec::new("t").opt(ap::OptSpec::value("delta", set_limit).short('d').long("delta"));
    let mut ctx = Ctx::default();
    ap::parse(&env, &root, &argv(&["-d-3"]), &mut ctx).unwrap();
    assert_eq!(ctx.limit.as_deref(), Some("-3"));
}

#[test]
fn long_next_arg_negative_value() {
    let env = env_basic();
    let root = ap::CmdSpec::new("t").opt(ap::OptSpec::value("delta", set_limit).long("delta"));
    let mut ctx = Ctx::default();
    ap::parse(&env, &root, &argv(&["--delta", "-3"]), &mut ctx).unwrap();
    assert_eq!(ctx.limit.as_deref(), Some("-3"));
}

#[test]
fn non_utf8_after_double_dash_is_positional() {
    let env = env_basic();
    let root = ap::CmdSpec::new("t").pos(ap::PosSpec::new("FILE", push_file).many());
    let mut ctx = Ctx::default();
    let bad = unsafe { std::ffi::OsString::from_encoded_bytes_unchecked(vec![0xff, b'a']) };
    let argv = vec!["--".into(), bad, "ok".into()];
    ap::parse(&env, &root, &argv, &mut ctx).unwrap();
    assert_eq!(ctx.files.len(), 2);
}
