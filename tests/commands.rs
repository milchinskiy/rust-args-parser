mod common;
use common::*;

#[test]
fn dispatch_into_sub_and_aliases() {
    let env = env_base();

    let pos_add = [PosSpec::new("X").one()];
    let aliases_add = ["create"];
    let sub_add =
        CmdSpec::new(Some("add"), Some(run_cb)).pos(pos_add).aliases(aliases_add).desc("add smth");

    let aliases_rm = ["remove", "delete"];
    let sub_rm = CmdSpec::new(Some("rm"), Some(run_cb)).aliases(aliases_rm).desc("remove smth");

    let subs = [sub_add, sub_rm];
    let root = CmdSpec::new(None, None).subs(subs);

    let mut u1 = U::default();
    let mut out1 = Vec::<u8>::new();
    let r1 = dispatch_to(&env, &root, &["add", "42"], &mut u1, &mut out1);
    assert!(r1.is_ok());
    assert!(u1.ran);
    assert_eq!(u1.received, vec!["42"]);

    let mut u2 = U::default();
    let mut out2 = Vec::<u8>::new();
    let r2 = dispatch_to(&env, &root, &["remove"], &mut u2, &mut out2);
    assert!(r2.is_ok());
    assert!(u2.ran);
}

#[test]
fn unknown_command_when_no_positionals() {
    let env = env_base();

    let sub_add = CmdSpec::new(Some("add"), Some(run_cb));
    let sub_rm = CmdSpec::new(Some("rm"), Some(run_cb));
    let subs = [sub_add, sub_rm];
    let root = CmdSpec::new(None, None).subs(subs);

    let mut u = U::default();
    let mut out = Vec::<u8>::new();
    let e = dispatch_to(&env, &root, &["zzz"], &mut u, &mut out).unwrap_err();
    match e {
        Error::UnknownCommand { token: s, .. } => assert_eq!(s, "zzz"),
        _ => panic!("wrong error"),
    }
}

#[test]
fn bare_token_becomes_positional_when_schema_present() {
    let env = env_base();

    let pos_sub = [PosSpec::new("P").range(0, usize::MAX)];
    let sub = CmdSpec::new(Some("sub"), Some(run_cb)).pos(pos_sub);
    let subs = [sub];

    let pos_root = [PosSpec::new("ARG").range(0, usize::MAX)];
    let root = CmdSpec::new(None, Some(run_cb)).subs(subs).pos(pos_root);

    let mut u = U::default();
    let mut out = Vec::<u8>::new();
    let r = dispatch_to(&env, &root, &["not-a-sub", "more"], &mut u, &mut out);
    assert!(r.is_ok());
    assert_eq!(u.received, vec!["not-a-sub", "more"]);
}
