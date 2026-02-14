use rust_args_parser as ap;

mod common;
use common::*;

use core::fmt;

#[derive(Debug)]
struct AppErr(&'static str);

impl fmt::Display for AppErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for AppErr {}

#[test]
fn cmd_validator_and_handler_variants() {
    let env = env_basic();

    // validator (Display-only) => Error::User
    let root = ap::CmdSpec::new("t").validator(|_| -> Result<(), &'static str> { Err("nope") });
    let err = ap::parse(&env, &root, &[], &mut Ctx::default()).unwrap_err();
    match err {
        ap::Error::User(msg) => assert!(msg.contains("nope")),
        other => panic!("unexpected: {other:?}"),
    }

    // validator_try (typed error) => Error::UserAny(AppErr)
    let root = ap::CmdSpec::new("t").validator_try(|_| -> Result<(), AppErr> { Err(AppErr("bad")) });
    let err = ap::parse(&env, &root, &[], &mut Ctx::default()).unwrap_err();
    match err {
        ap::Error::UserAny(e) => {
            let e = e.downcast_ref::<AppErr>().expect("must preserve AppErr");
            assert_eq!(e.0, "bad");
        }
        other => panic!("unexpected: {other:?}"),
    }

    // handler (infallible) executes on leaf
    let root = ap::CmdSpec::new("t").subcmd(ap::CmdSpec::new("ok").handler(|_m, ctx: &mut Ctx| {
        ctx.json = true;
    }));
    let mut ctx = Ctx::default();
    ap::parse(&env, &root, &argv(&["ok"]), &mut ctx).unwrap();
    assert!(ctx.json);

    // handler_try (typed error) => Error::UserAny(AppErr)
    let root = ap::CmdSpec::new("t").subcmd(
        ap::CmdSpec::new("fail").handler_try(|_m, _ctx: &mut Ctx| -> Result<(), AppErr> { Err(AppErr("boom")) }),
    );
    let err = ap::parse(&env, &root, &argv(&["fail"]), &mut Ctx::default()).unwrap_err();
    match err {
        ap::Error::UserAny(e) => {
            let e = e.downcast_ref::<AppErr>().expect("must preserve AppErr");
            assert_eq!(e.0, "boom");
        }
        other => panic!("unexpected: {other:?}"),
    }
}

#[test]
fn opt_flag_and_value_variants() {
    let env = env_basic();

    // flag/value (infallible)
    let root = ap::CmdSpec::new("t")
        .opt(ap::OptSpec::flag("json", set_json).long("json"))
        .opt(ap::OptSpec::value("limit", set_limit).long("limit"));
    let mut ctx = Ctx::default();
    ap::parse(&env, &root, &argv(&["--json", "--limit", "5"]), &mut ctx).unwrap();
    assert!(ctx.json);
    assert_eq!(ctx.limit.as_deref(), Some("5"));

    // flag_try => Error::UserAny(AppErr)
    let root = ap::CmdSpec::new("t").opt(
        ap::OptSpec::flag_try("json", |_ctx: &mut Ctx| -> Result<(), AppErr> { Err(AppErr("no-json")) }).long("json"),
    );

    let err = ap::parse(&env, &root, &argv(&["--json"]), &mut Ctx::default()).unwrap_err();
    match err {
        ap::Error::UserAny(e) => {
            let e = e.downcast_ref::<AppErr>().expect("must preserve AppErr");
            assert_eq!(e.0, "no-json");
        }
        other => panic!("unexpected: {other:?}"),
    }

    // value_try => Error::UserAny(AppErr)
    let root = ap::CmdSpec::new("t").opt(
        ap::OptSpec::value_try("limit", |_v, _ctx: &mut Ctx| -> Result<(), AppErr> { Err(AppErr("no-limit")) })
            .long("limit"),
    );

    let err = ap::parse(&env, &root, &argv(&["--limit", "5"]), &mut Ctx::default()).unwrap_err();
    match err {
        ap::Error::UserAny(e) => {
            let e = e.downcast_ref::<AppErr>().expect("must preserve AppErr");
            assert_eq!(e.0, "no-limit");
        }
        other => panic!("unexpected: {other:?}"),
    }
}

#[test]
fn pos_new_and_new_try_variants() {
    let env = env_basic();

    // new (infallible)
    let root = ap::CmdSpec::new("t").pos(ap::PosSpec::new("FILE", push_file));
    let mut ctx = Ctx::default();
    ap::parse(&env, &root, &argv(&["a.txt"]), &mut ctx).unwrap();
    assert_eq!(ctx.files, vec!["a.txt"]);

    // new_try => Error::UserAny(AppErr)
    let root = ap::CmdSpec::new("t")
        .pos(ap::PosSpec::new_try("FILE", |_v, _ctx: &mut Ctx| -> Result<(), AppErr> { Err(AppErr("no-file")) }));

    let err = ap::parse(&env, &root, &argv(&["a.txt"]), &mut Ctx::default()).unwrap_err();
    match err {
        ap::Error::UserAny(e) => {
            let e = e.downcast_ref::<AppErr>().expect("must preserve AppErr");
            assert_eq!(e.0, "no-file");
        }
        other => panic!("unexpected: {other:?}"),
    }
}
