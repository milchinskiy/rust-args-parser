use rust_args_parser as ap;
mod common;
use common::*;

#[test]
fn xor_group_enforced() {
    let env = env_basic();
    let root = ap::CmdSpec::new("demo")
        .opt(ap::OptSpec::flag("json", set_json).long("json").group("fmt"))
        .opt(
            ap::OptSpec::flag("yaml", |c: &mut Ctx| {
                c.limit = Some("yaml".into());
            })
            .long("yaml")
            .group("fmt"),
        )
        .group("fmt", ap::GroupMode::Xor);

    let mut ctx = Ctx::default();
    // setting both must error
    let argv = argv(&["--json", "--yaml"]);
    let err = ap::parse(&env, &root, &argv, &mut ctx).unwrap_err();
    match err {
        ap::Error::User(msg) => assert!(msg.contains("mutually exclusive")),
        _ => panic!("{err:?}"),
    }
}

#[test]
fn reqone_group_enforced() {
    let env = env_basic();
    let root = ap::CmdSpec::new("demo")
        .opt(ap::OptSpec::flag("quiet", |_| {}).long("quiet").group("out"))
        .opt(ap::OptSpec::flag("verbose", inc_verbose).long("verbose").group("out"))
        .group("out", ap::GroupMode::ReqOne);

    let mut ctx = Ctx::default();
    let err = ap::parse(&env, &root, &[], &mut ctx).unwrap_err();
    match err {
        ap::Error::User(msg) => assert!(msg.contains("is required")),
        _ => panic!("{err:?}"),
    }
}

#[test]
fn xor_group_env_vs_cli_conflict() {
    let envv = env_basic();
    let root = ap::CmdSpec::new("t")
        .opt(ap::OptSpec::flag("json", set_json).long("json").env("FMT_JSON").group("fmt"))
        .opt(ap::OptSpec::flag("yaml", |_| {}).long("yaml").group("fmt"))
        .group("fmt", ap::GroupMode::Xor);

    std::env::set_var("FMT_JSON", "1");
    let mut ctx = Ctx::default();
    let err = ap::parse(&envv, &root, &argv(&["--yaml"]), &mut ctx).unwrap_err();
    match err {
        ap::Error::User(msg) => assert!(msg.contains("mutually exclusive")),
        _ => panic!("{err:?}"),
    }
}
