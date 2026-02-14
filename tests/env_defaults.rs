use rust_args_parser as ap;
use std::ffi::OsString;
mod common;
use common::*;
use std::env;

#[test]
fn cli_over_env_over_default() {
    let envv = env_basic();
    let root = ap::CmdSpec::new("d")
        .opt(ap::OptSpec::flag("json", set_json).long("json").env("APP_JSON"))
        .opt(ap::OptSpec::value("limit", set_limit).long("limit").default(OsString::from("42")));

    // default applied
    let mut ctx = Ctx::default();
    let m = ap::parse(&envv, &root, &[], &mut ctx).unwrap();
    assert_eq!(ctx.limit.as_deref(), Some("42"));
    assert!(m.is_set_from("limit", ap::Source::Default));

    // env overrides default
    env::set_var("APP_JSON", "1");
    env::set_var("APP_LIMIT", "100"); // just to confirm it doesn't matter when not bound
    let mut ctx = Ctx::default();
    let m = ap::parse(&envv, &root, &[], &mut ctx).unwrap();
    assert!(ctx.json);
    assert!(m.is_set_from("json", ap::Source::Env));

    // CLI overrides env
    let mut ctx = Ctx::default();
    let argv = argv(&["--limit", "5"]);
    let m = ap::parse(&envv, &root, &argv, &mut ctx).unwrap();
    assert_eq!(ctx.limit.as_deref(), Some("5"));
    assert!(m.is_set_from("limit", ap::Source::Cli));
}

#[test]
fn env_defaults() {
    let env = rust_args_parser::Env::default();
    assert_eq!(env.wrap_cols, 0);
    assert_eq!(env.color, rust_args_parser::ColorMode::Auto);
    assert!(env.suggest);
    assert!(env.auto_help);
    assert_eq!(env.version, None);
    assert_eq!(env.author, None);
}
