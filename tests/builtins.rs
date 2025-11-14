#![cfg(feature = "help")]
use rust_args_parser as ap;
mod common;
use common::*;

#[test]
fn help_and_version_exit_zero() {
    let env = env_basic();
    let root =
        ap::CmdSpec::new("t").help("T").opt(ap::OptSpec::flag("verbose", inc_verbose).short('v'));

    let err = ap::parse(&env, &root, &argv(&["-h"]), &mut Ctx::default()).unwrap_err();
    match err {
        ap::Error::ExitMsg { code, .. } => assert_eq!(code, 0),
        _ => panic!("{err:?}"),
    }

    let err = ap::parse(&env, &root, &argv(&["--version"]), &mut Ctx::default()).unwrap_err();
    match err {
        ap::Error::ExitMsg { code, .. } => assert_eq!(code, 0),
        _ => panic!("{err:?}"),
    }
}
