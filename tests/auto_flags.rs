mod common;
use common::*;

#[test]
fn short_and_long_triggers_exit_with_output() {
    let env = env_base();

    let opts = [
        OptSpec::new("verbose", cb_verbose).short('v').flag().help("increase verbosity"),
        OptSpec::new("output", cb_output).short('o').required().metavar("FILE").help("output file"),
    ];
    let pos = [PosSpec::new("INPUT").one(), PosSpec::new("EXTRA").range(0, usize::MAX)];
    let cmd = CmdSpec::new(None, None).desc("demo").opts(opts).pos(pos);

    let mut u1 = U::default();
    let mut out1 = Vec::<u8>::new();
    match dispatch_to(&env, &cmd, &["--help"], &mut u1, &mut out1) {
        Err(Error::Exit(0)) => {}
        other => panic!("expected Exit(0), got {other:?}"),
    }
    let text = s(&out1);
    assert!(text.contains("Usage: prog"));
    assert!(text.contains("Options"));
    assert!(text.contains("Positionals"));

    let mut u2 = U::default();
    let mut out2 = Vec::<u8>::new();
    match dispatch_to(&env, &cmd, &["-h"], &mut u2, &mut out2) {
        Err(Error::Exit(0)) => {}
        _ => panic!("expected Exit(0) via -h"),
    }
}

#[test]
fn version_and_author_printed() {
    let env = env_base();

    let opts = [
        OptSpec::new("verbose", cb_verbose).short('v').flag(),
        OptSpec::new("output", cb_output).short('o').required().metavar("FILE"),
    ];
    let pos = [PosSpec::new("INPUT").one(), PosSpec::new("EXTRA").range(0, usize::MAX)];
    let cmd = CmdSpec::new(None, None).opts(opts).pos(pos);

    let mut u1 = U::default();
    let mut out1 = Vec::<u8>::new();
    match dispatch_to(&env, &cmd, &["--version"], &mut u1, &mut out1) {
        Err(Error::Exit(0)) => {}
        _ => panic!("expected Exit(0) via --version"),
    }
    assert_eq!(s(&out1).trim(), "1.2.3");

    let mut u2 = U::default();
    let mut out2 = Vec::<u8>::new();
    match dispatch_to(&env, &cmd, &["-V"], &mut u2, &mut out2) {
        Err(Error::Exit(0)) => {}
        _ => panic!("expected Exit(0) via -V"),
    }

    let mut u3 = U::default();
    let mut out3 = Vec::<u8>::new();
    match dispatch_to(&env, &cmd, &["--author"], &mut u3, &mut out3) {
        Err(Error::Exit(0)) => {}
        _ => panic!("expected Exit(0) via --author"),
    }
    assert_eq!(s(&out3).trim(), "Alice Example");

    let mut u4 = U::default();
    let mut out4 = Vec::<u8>::new();
    match dispatch_to(&env, &cmd, &["-A"], &mut u4, &mut out4) {
        Err(Error::Exit(0)) => {}
        _ => panic!("expected Exit(0) via -A"),
    }
}
