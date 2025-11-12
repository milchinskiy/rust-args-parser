use rust_args_parser::{dispatch, CmdSpec, Env, OptSpec, PosSpec};

#[derive(Default)]
struct Ctx {
    json: bool,
    ran: bool,
}

#[test]
fn root_flag_then_sub_long_ok() {
    let env = Env::new("mybin");
    let mut ctx = Ctx::default();

    let root = CmdSpec::new(None, None)
        .opts(vec![OptSpec::new("json", |_, c: &mut Ctx| {
            c.json = true;
            Ok(())
        })
        .flag()])
        .subs(vec![CmdSpec::new(
            Some("test"),
            Some(|pos, c: &mut Ctx| {
                assert!(pos.is_empty());
                assert!(c.json);
                c.ran = true;
                Ok(())
            }),
        )]);

    dispatch(&env, &root, &["--json", "test"], &mut ctx).unwrap();
    assert!(ctx.ran);
}

#[test]
fn root_short_then_sub_ok() {
    let env = Env::new("mybin");
    let mut ctx = Ctx::default();

    let root = CmdSpec::new(None, None)
        .opts(vec![OptSpec::new("v", |_, c: &mut Ctx| {
            c.json = true;
            Ok(())
        })
        .short('v')
        .flag()])
        .subs(vec![CmdSpec::new(
            Some("go"),
            Some(|_, c: &mut Ctx| {
                assert!(c.json);
                Ok(())
            }),
        )]);

    dispatch(&env, &root, &["-v", "go"], &mut ctx).unwrap();
}

#[test]
fn double_dash_still_blocks_descent() {
    let env = Env::new("mybin");

    // Root accepts positionals; ensure we DO NOT descend to subcommand after `--`.
    let root = CmdSpec::new(
        None,
        Some(|pos, _c: &mut Ctx| {
            assert_eq!(pos, &["test"]);
            Ok(())
        }),
    )
    .subs(vec![CmdSpec::new(
        Some("test"),
        Some(|_, _c: &mut Ctx| {
            panic!("should not descend after --");
            #[allow(unreachable_code)]
            Ok(())
        }),
    )])
    .pos(vec![PosSpec::new("test").one()]);

    let mut ctx = Ctx::default();
    dispatch(&env, &root, &["--", "test"], &mut ctx).unwrap();
}
