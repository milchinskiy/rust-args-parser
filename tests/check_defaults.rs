#[test]
fn root_default_applies_when_argv_starts_with_sub() {
    use rust_args_parser::{dispatch, CmdSpec, Env, OptSpec};
    #[derive(Default)]
    struct Ctx {
        sock: Option<String>,
        seen: bool,
        another_seen: bool,
    }

    let root = CmdSpec::new(None, None)
        .opts(vec![
            OptSpec::new("socket", |v, c: &mut Ctx| {
                c.sock = v.map(std::string::ToString::to_string);
                Ok(())
            })
            .required() // value-taking option
            .default("/tmp/splicer.sock")
            .metavar("PATH")
            .help("Socket path"),
            OptSpec::new("another", |_, c: &mut Ctx| {
                c.another_seen = true;
                Ok(())
            }),
        ])
        .subs(vec![CmdSpec::new(
            Some("server"),
            Some(|_, c: &mut Ctx| {
                assert_eq!(c.sock.as_deref(), Some("/tmp/splicer.sock"));
                c.seen = true;
                Ok(())
            }),
        )]);

    let env = Env::new("mybin").auto_help(true);
    let mut ctx = Ctx::default();
    dispatch(&env, &root, &["server"], &mut ctx).unwrap();
    assert!(ctx.seen);
    assert!(!ctx.another_seen);
    assert_eq!(ctx.sock, Some("/tmp/splicer.sock".to_string()));
}
