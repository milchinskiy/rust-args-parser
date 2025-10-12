use rust_args_parser as ap;

#[allow(clippy::unnecessary_wraps)]
fn noop(_: &[&str], _: &mut ()) -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
}

fn mk_tree<'a>() -> (ap::Env<'a>, ap::CmdSpec<'a, ()>) {
    let env = ap::Env::new("glyphctl").auto_help(true).color(false);
    let roots = ap::CmdSpec::new(Some("roots"), None)
        .subs(vec![ap::CmdSpec::new(Some("list"), Some(noop))])
        .desc("roots management");
    let store = ap::CmdSpec::new(Some("store"), None).subs(vec![roots]);
    let root = ap::CmdSpec::new(None, None).subs(vec![store]);
    (env, root)
}

#[test]
fn help_includes_full_parent_path() {
    let (env, root) = mk_tree();
    let argv = ["store", "roots", "--help"]; // actual typed path
    let mut out = Vec::new();
    let mut ctx = ();
    let err = ap::dispatch_to(&env, &root, &argv, &mut ctx, &mut out).unwrap_err();
    let s = String::from_utf8(out).unwrap();
    assert!(s.lines().next().unwrap().contains("Usage: glyphctl store roots"));
    match err {
        ap::Error::Exit(0) => {}
        _ => panic!("expected Exit(0)"),
    }
}

#[test]
fn non_terminal_invocation_prints_help_and_exits_1() {
    let (env, root) = mk_tree();
    let argv = ["store", "roots"]; // no subcommand, and `roots` has no run
    let mut out = Vec::new();
    let mut ctx = ();
    let err = ap::dispatch_to(&env, &root, &argv, &mut ctx, &mut out).unwrap_err();
    let s = String::from_utf8(out).unwrap();
    assert!(s.starts_with("Usage: glyphctl store roots"));
    match err {
        ap::Error::Exit(1) => {}
        _ => panic!("expected Exit(1)"),
    }
}

#[test]
fn non_terminal_without_auto_help_exits_1_silently() {
    let (mut env, root) = mk_tree();
    env = env.auto_help(false);
    let argv = ["store", "roots"]; // no run
    let mut out = Vec::new();
    let mut ctx = ();
    let err = ap::dispatch_to(&env, &root, &argv, &mut ctx, &mut out).unwrap_err();
    assert!(out.is_empty());
    match err {
        ap::Error::Exit(1) => {}
        _ => panic!("expected Exit(1)"),
    }
}
