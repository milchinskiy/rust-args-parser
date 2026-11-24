use rust_args_parser::{key_for, parse, pos_key_for, CmdSpec, Env, OptSpec, PosSpec, Source};
use std::ffi::{OsStr, OsString};

#[derive(Default)]
struct Events(Vec<String>);

#[test]
fn scoped_keys_are_unambiguous() {
    assert_ne!(key_for(&[], "child.file"), key_for(&["child"], "file"));
    assert_ne!(key_for(&["a.b"], "c"), key_for(&["a", "b"], "c"));
    assert_ne!(key_for(&["a"], "b.c"), key_for(&["a", "b"], "c"));
    assert_ne!(key_for(&[], "@pos.FILE"), pos_key_for(&[], "FILE"));
    assert_ne!(key_for(&[], "FILE"), pos_key_for(&[], "FILE"));
    assert_ne!(key_for(&["a|b"], "c"), key_for(&["a"], "b|c"));
    assert_ne!(key_for(&["a:", "b"], "c"), key_for(&["a", ":b"], "c"));
    assert_ne!(pos_key_for(&[], "child.FILE"), pos_key_for(&["child"], "FILE"));
}

#[test]
fn options_and_positionals_do_not_shadow_across_scopes() {
    let root = CmdSpec::new("root")
        .opt(
            OptSpec::value("child.file", |v, events: &mut Events| {
                events.0.push(format!("root={}", v.to_string_lossy()));
            })
            .long("root-file")
            .default(OsString::from("DEFAULT")),
        )
        .subcmd(
            CmdSpec::new("child")
                .opt(
                    OptSpec::value("file", |v, events: &mut Events| {
                        events.0.push(format!("child={}", v.to_string_lossy()));
                    })
                    .long("file"),
                )
                .opt(
                    OptSpec::value("@pos.FILE", |v, events: &mut Events| {
                        events.0.push(format!("named={}", v.to_string_lossy()));
                    })
                    .long("named"),
                )
                .pos(
                    PosSpec::new("FILE", |v, events: &mut Events| {
                        events.0.push(format!("pos={}", v.to_string_lossy()));
                    })
                    .required(),
                ),
        );
    let argv = ["--root-file", "ROOT", "child", "--file", "CHILD", "--named", "NAMED", "POSITION"].map(OsString::from);
    let mut events = Events::default();
    let matches = parse(&Env::default(), &root, &argv, &mut events).unwrap();
    assert_eq!(matches.at(&[]).value("child.file"), Some(OsStr::new("ROOT")));
    assert_eq!(matches.at(&["child"]).value("file"), Some(OsStr::new("CHILD")));
    assert_eq!(matches.view().value("@pos.FILE"), Some(OsStr::new("NAMED")));
    assert_eq!(matches.view().pos_one("FILE"), Some(OsStr::new("POSITION")));
    assert!(matches.at(&[]).is_set_from("child.file", Source::Cli));
    assert!(matches.view().is_set_from("file", Source::Cli));
    assert_eq!(events.0, ["root=ROOT", "child=CHILD", "named=NAMED", "pos=POSITION"]);
}

#[test]
fn overlays_and_flag_counts_remain_scoped() {
    let root = CmdSpec::new("root")
        .opt(OptSpec::flag("child.v", |(): &mut ()| {}).short('v'))
        .opt(OptSpec::value("child.x", |_, (): &mut ()| {}).long("x").default("root-default"))
        .subcmd(
            CmdSpec::new("child")
                .opt(OptSpec::flag("v", |(): &mut ()| {}).short('v'))
                .opt(OptSpec::value("x", |_, (): &mut ()| {}).long("x").default("child-default")),
        );
    let argv = ["-vv", "child", "-v"].map(OsString::from);
    let matches = parse(&Env::default(), &root, &argv, &mut ()).unwrap();
    assert_eq!(matches.at(&[]).flag_count("child.v"), 2);
    assert_eq!(matches.at(&["child"]).flag_count("v"), 1);
    assert_eq!(matches.at(&[]).value("child.x"), Some(OsStr::new("root-default")));
    assert_eq!(matches.view().value("x"), Some(OsStr::new("child-default")));
    assert!(matches.at(&[]).is_set_from("child.x", Source::Default));
    assert!(matches.view().is_set_from("x", Source::Default));
}
