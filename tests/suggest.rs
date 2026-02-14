#![cfg(feature = "suggest")]
use rust_args_parser as ap;
mod common;
use common::*;

#[test]
fn unknown_options_and_commands_suggest() {
    let mut env = env_basic();
    env.suggest = true;
    let root = ap::CmdSpec::new("demo")
        .opt(ap::OptSpec::flag("helpme", |_| {}).long("helpme"))
        .subcmd(ap::CmdSpec::new("remote"));

    // --helme ~> --helpme
    let mut ctx = Ctx::default();
    let err = ap::parse(&env, &root, &argv(&["--helme"]), &mut ctx).unwrap_err();
    match err {
        ap::Error::UnknownOption { token, suggestions } => {
            assert_eq!(token, "--helme");
            assert!(suggestions.iter().any(|s| s == "--helpme"));
        }
        _ => panic!("{err:?}"),
    }

    // remot ~> remote
    let mut ctx = Ctx::default();
    let err = ap::parse(&env, &root, &argv(&["remot"]), &mut ctx).unwrap_err();
    match err {
        ap::Error::UnknownCommand { token, suggestions } => {
            assert_eq!(token, "remot");
            assert!(suggestions.iter().any(|s| s == "remote"));
        }
        _ => panic!("{err:?}"),
    }
}

#[cfg(not(feature = "suggest"))]
#[test]
fn unknown_has_no_suggestions_when_feature_off() {
    let env = env_basic();
    let root = ap::CmdSpec::new("t").opt(ap::OptSpec::flag("helpme", |_| {}).long("helpme"));
    let err = ap::parse(&env, &root, &argv(&["--helme"]), &mut Ctx::default()).unwrap_err();
    match err {
        ap::Error::UnknownOption { token, suggestions } => {
            assert_eq!(token, "--helme");
            assert!(suggestions.is_empty());
        }
        _ => panic!("{err:?}"),
    }
}
