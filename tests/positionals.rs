mod common;
use common::*;

#[test]
fn missing_and_too_many_positionals() {
    let env = env_base();

    let pos1 = [PosSpec::new("SRC").one(), PosSpec::new("DST").one()];
    let cmd = CmdSpec::new(None, Some(run_cb)).pos(pos1);

    let mut u1 = U::default();
    let mut out1 = Vec::<u8>::new();
    let e1 = dispatch_to(&env, &cmd, &["only"], &mut u1, &mut out1).unwrap_err();
    match e1 {
        Error::MissingPositional(n) => assert_eq!(n, "DST"),
        _ => panic!("wrong error"),
    }

    let pos2 = [PosSpec::new("FILE").range(1, 2), PosSpec::new("EXTRA").range(0, 1)];
    let cmd2 = CmdSpec::new(None, Some(run_cb)).pos(pos2);

    let mut u2 = U::default();
    let mut out2 = Vec::<u8>::new();
    let e2 = dispatch_to(&env, &cmd2, &["a", "b", "c", "d"], &mut u2, &mut out2).unwrap_err();
    match e2 {
        Error::TooManyPositional(n) => assert_eq!(n, "EXTRA"),
        _ => panic!("wrong error"),
    }
}

#[test]
fn run_receives_positionals_in_order() {
    let env = env_base();

    let pos = [PosSpec::new("X").range(1, 3)];
    let cmd = CmdSpec::new(None, Some(run_cb)).pos(pos);

    let mut u = U::default();
    let mut out = Vec::<u8>::new();
    let r = dispatch_to(&env, &cmd, &["1", "2", "3"], &mut u, &mut out);
    assert!(r.is_ok());
    assert_eq!(u.received, vec!["1", "2", "3"]);
}

#[test]
fn unexpected_argument_when_no_schema() {
    let env = env_base();

    let cmd = CmdSpec::new(None, None).opts(&[]);

    let mut u = U::default();
    let mut out = Vec::<u8>::new();
    let e = dispatch_to(&env, &cmd, &["dangling"], &mut u, &mut out).unwrap_err();
    match e {
        Error::UnexpectedArgument(s) => assert_eq!(s, "dangling"),
        _ => panic!("wrong error"),
    }
}
