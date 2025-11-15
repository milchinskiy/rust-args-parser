use rust_args_parser as ap;
mod common;
use common::*;

#[test]
fn range_min_is_enforced() {
    let env = env_basic();
    let root = ap::CmdSpec::new("d").pos(ap::PosSpec::new("FILE", push_file).range(2, 3));

    let mut ctx = Ctx::default();
    // Only 1 provided — expect an error
    let err = ap::parse(&env, &root, &argv(&["a.txt"]), &mut ctx).unwrap_err();
    match err {
        ap::Error::User(msg) => assert!(msg.contains("below minimum")),
        _ => panic!("{err:?}"),
    }

    // 2..=3 should succeed
    let mut ctx = Ctx::default();
    ap::parse(&env, &root, &argv(&["a", "b"]), &mut ctx).unwrap();
    let mut ctx = Ctx::default();
    ap::parse(&env, &root, &argv(&["a", "b", "c"]), &mut ctx).unwrap();
}

#[test]
fn read_root_and_leaf_scopes() {
    let env = env_basic();
    let root = ap::CmdSpec::new("t")
        .opt(ap::OptSpec::flag("json", set_json).long("json"))
        .subcmd(ap::CmdSpec::new("sub").pos(ap::PosSpec::new("X", push_file)));

    let mut ctx = Ctx::default();
    let m = ap::parse(&env, &root, &argv(&["--json", "sub", "f"]), &mut ctx).unwrap();

    let rootv = m.at(&[]);
    assert!(rootv.is_set_from("json", ap::Source::Cli));
    let leafv = m.view();
    assert!(leafv.pos_one("X").is_some());
}
