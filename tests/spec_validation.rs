use rust_args_parser::{parse, CmdSpec, Env, Error, GroupMode, OptSpec, PosSpec};
use std::ffi::OsString;

#[derive(Default)]
struct State {
    callbacks: usize,
}

fn flag(state: &mut State) {
    state.callbacks += 1;
}

fn invalid(spec: &CmdSpec<'_, State>, env: &Env) -> String {
    let mut state = State::default();
    // Even a built-in help request cannot mask an invalid specification.
    let error = parse(env, spec, &[OsString::from("--help")], &mut state).unwrap_err();
    assert_eq!(state.callbacks, 0);
    match error {
        Error::Parse(message) => message,
        other => panic!("expected invalid specification error, got {other:?}"),
    }
}

#[test]
fn duplicate_names_and_switches_are_rejected() {
    let cases = [
        (
            CmdSpec::new("root").opt(OptSpec::flag("a", flag).short('v')).opt(OptSpec::flag("b", flag).short('v')),
            "duplicate short",
        ),
        (
            CmdSpec::new("root")
                .opt(OptSpec::flag("a", flag).long("verbose"))
                .opt(OptSpec::flag("b", flag).long("verbose")),
            "duplicate long",
        ),
        (
            CmdSpec::new("root")
                .opt(OptSpec::flag("a", flag).long("first"))
                .opt(OptSpec::flag("a", flag).long("second")),
            "duplicate option",
        ),
        (
            CmdSpec::new("root")
                .pos(PosSpec::new("FILE", |_, _: &mut State| {}))
                .pos(PosSpec::new("FILE", |_, _: &mut State| {})),
            "duplicate positional",
        ),
        (CmdSpec::new("root").group("g", GroupMode::Xor).group("g", GroupMode::ReqOne), "duplicate group"),
        (
            CmdSpec::new("root").subcmd(CmdSpec::new("one").alias("two")).subcmd(CmdSpec::new("two")),
            "duplicate subcommand",
        ),
    ];
    for (spec, expected) in &cases {
        assert!(invalid(spec, &Env::default()).contains(expected), "expected {expected}");
    }
}

#[test]
fn invalid_unselected_subcommand_is_rejected_before_dispatch() {
    let root = CmdSpec::new("root")
        .opt(OptSpec::flag("ok", flag).long("ok"))
        .subcmd(CmdSpec::new("good"))
        .subcmd(CmdSpec::new("bad").opt(OptSpec::flag("a", flag).short('x')).opt(OptSpec::flag("b", flag).short('x')));
    let mut state = State::default();
    let error = parse(&Env::default(), &root, &["--ok".into(), "good".into()], &mut state).unwrap_err();
    assert!(matches!(error, Error::Parse(s) if s.contains("duplicate short")));
    assert_eq!(state.callbacks, 0);
}

#[test]
fn reserved_builtins_depend_on_environment_and_scope() {
    let root = CmdSpec::new("root").opt(OptSpec::flag("custom-help", flag).short('h'));
    assert!(invalid(&root, &Env::default()).contains("reserved short"));
    let env = Env { auto_help: false, ..Env::default() };
    let mut state = State::default();
    parse(&env, &root, &["-h".into()], &mut state).unwrap();
    assert_eq!(state.callbacks, 1);

    let env = Env { version: Some("1.0"), author: Some("A"), ..Env::default() };
    for long in ["version", "author"] {
        let root = CmdSpec::new("root").opt(OptSpec::flag("custom", flag).long(long));
        assert!(invalid(&root, &env).contains("reserved long"));
    }
    let root = CmdSpec::new("root").subcmd(CmdSpec::new("child").opt(OptSpec::flag("child-version", flag).short('V')));
    let mut state = State::default();
    parse(&env, &root, &["child".into(), "-V".into()], &mut state).unwrap();
    assert_eq!(state.callbacks, 1);
}

#[test]
fn malformed_positionals_and_groups_fail_early() {
    let specs = [
        CmdSpec::new("root")
            .pos(PosSpec::new("FILES", |_, _: &mut State| {}).many())
            .pos(PosSpec::new("TAIL", |_, _: &mut State| {})),
        CmdSpec::new("root").pos(PosSpec::new("FILE", |_, _: &mut State| {}).range(0, 0)),
        CmdSpec::new("root").pos(PosSpec::new("FILE", |_, _: &mut State| {}).range(3, 2)),
        CmdSpec::new("root").opt(OptSpec::flag("foo", flag).long("foo").group("unknown")),
        CmdSpec::new("root").group("empty", GroupMode::Xor),
        CmdSpec::new("root").opt(OptSpec::flag("foo", flag).long("")),
        CmdSpec::new("root").opt(OptSpec::flag("foo", flag).long("foo").default("bad")),
    ];
    for spec in &specs {
        assert_ne!(invalid(spec, &Env::default()), String::new());
    }
}

#[test]
fn valid_bounded_positionals_allow_following_argument() {
    let root = CmdSpec::new("root")
        .pos(PosSpec::new("FIRST", |_, _: &mut State| {}).range(1, 2))
        .pos(PosSpec::new("LAST", |_, _: &mut State| {}).required());
    let matches = parse(&Env::default(), &root, &["a".into(), "b".into(), "c".into()], &mut State::default()).unwrap();
    assert_eq!(matches.view().pos_many("FIRST").unwrap().len(), 2);
    assert!(matches.view().pos_one("LAST").is_some());
}
