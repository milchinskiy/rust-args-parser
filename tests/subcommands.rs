#![allow(clippy::unnecessary_wraps)]
use rust_args_parser as ap;
mod common;
use common::*;
use std::ffi::OsStr;

fn set_name(v: &OsStr, c: &mut Ctx) -> ap::Result<()> {
    c.input = Some(v.to_os_string());
    Ok(())
}
fn set_url(v: &OsStr, c: &mut Ctx) -> ap::Result<()> {
    c.limit = Some(v.to_string_lossy().into());
    Ok(())
}

#[test]
fn subcommand_path_and_view_scoping() {
    let env = env_basic();
    let root = ap::CmdSpec::new("git").subcmd(
        ap::CmdSpec::new("remote").alias("rm").subcmd(
            ap::CmdSpec::new("add")
                .pos(ap::PosSpec::new("NAME", set_name).required())
                .pos(ap::PosSpec::new("URL", set_url).required()),
        ),
    );

    let mut ctx = Ctx::default();
    let argv = argv(&["remote", "add", "origin", "https://example"]);
    let m = ap::parse(&env, &root, &argv, &mut ctx).expect("parse");
    assert_eq!(m.leaf_path(), vec!["remote", "add"]);

    let v = m.view();
    assert_eq!(v.pos_one("NAME").unwrap(), OsStr::new("origin"));
    assert_eq!(v.pos_one("URL").unwrap().to_string_lossy(), "https://example");
}

#[cfg(feature = "suggest")]
#[test]
fn unknown_subcommand_suggests() {
    let mut env = env_basic();
    env.suggest = true;
    let root = ap::CmdSpec::new("git").subcmd(ap::CmdSpec::new("remote"));
    let mut ctx = Ctx::default();
    let argv = argv(&["remot"]);
    let err = ap::parse(&env, &root, &argv, &mut ctx).unwrap_err();
    match err {
        ap::Error::UnknownCommand { token, suggestions } => {
            assert_eq!(token, "remot");
            assert!(suggestions.iter().any(|s| s == "remote"));
        }
        _ => panic!("{err:?}"),
    }
}

#[test]
fn root_option_not_recognized_after_descent() {
    let env = env_basic();
    let root = ap::CmdSpec::new("t")
        .opt(ap::OptSpec::value("limit", set_limit).long("limit"))
        .subcmd(ap::CmdSpec::new("sub").pos(ap::PosSpec::new("X", push_file)));
    let mut ctx = Ctx::default();
    let argv = argv(&["sub", "--limit=9", "z"]);
    let err = ap::parse(&env, &root, &argv, &mut ctx).unwrap_err();
    match err {
        ap::Error::UnknownOption { token, .. } => assert_eq!(token, "--limit"),
        _ => panic!("{err:?}"),
    }
}
