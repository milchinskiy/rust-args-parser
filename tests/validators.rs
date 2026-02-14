use rust_args_parser as ap;
mod common;
use common::*;
use std::ffi::OsStr;

fn non_empty(v: &OsStr) -> Result<(), &'static str> {
    if v.is_empty() {
        Err("empty")
    } else {
        Ok(())
    }
}

#[test]
fn option_and_positional_validators() {
    let env = env_basic();
    let root = ap::CmdSpec::new("d")
        .opt(ap::OptSpec::value("name", set_limit).long("name").validator(non_empty))
        .pos(ap::PosSpec::new("FILE", push_file).validator(non_empty));

    // ok
    let mut ctx = Ctx::default();
    ap::parse(&env, &root, &argv(&["--name=a", "f.txt"]), &mut ctx).unwrap();

    // bad option
    let mut ctx = Ctx::default();
    let err = ap::parse(&env, &root, &argv(&["--name="]), &mut ctx).unwrap_err();
    match err {
        ap::Error::User(msg) => assert!(msg.contains("empty")),
        _ => panic!("{err:?}"),
    }

    // bad positional
    let mut ctx = Ctx::default();
    let err = ap::parse(&env, &root, &argv(&[""]), &mut ctx).unwrap_err();
    match err {
        ap::Error::User(msg) => assert!(msg.contains("empty")),
        _ => panic!("{err:?}"),
    }
}

#[test]
fn validator_runs_on_env_before_callbacks() {
    let envv = env_basic();
    let root = ap::CmdSpec::new("t")
        .opt(ap::OptSpec::value("name", set_limit).long("name").env("APP_NAME").validator(non_empty));

    std::env::set_var("APP_NAME", ""); // invalid via validator
    let mut ctx = Ctx::default();
    let err = ap::parse(&envv, &root, &[], &mut ctx).unwrap_err();
    match err {
        ap::Error::User(msg) => assert!(msg.contains("empty")),
        _ => panic!("{err:?}"),
    }
    assert!(ctx.limit.is_none(), "callback must not fire on invalid env value");
}
