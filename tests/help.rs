#![cfg(feature = "help")]
use rust_args_parser as ap;
mod common;
use common::*;

#[test]
fn help_emits_and_exits() {
    let mut env = env_basic();
    env.color = ap::ColorMode::Never; // predictable text
    let root = ap::CmdSpec::new("demo")
        .help("Demo tool")
        .opt(ap::OptSpec::flag("verbose", inc_verbose).short('v').help("Verbose"))
        .subcmd(ap::CmdSpec::new("init").help("Init repo"))
        .pos(ap::PosSpec::new("FILE", push_file));

    let mut ctx = Ctx::default();
    let err = ap::parse(&env, &root, &argv(&["--help"]), &mut ctx).unwrap_err();
    match err {
        ap::Error::ExitMsg { code, message } => {
            assert_eq!(code, 0);
            let msg = message.unwrap();
            assert!(msg.contains("Usage"));
            assert!(msg.contains("Options"));
            assert!(msg.contains("Commands"));
            assert!(msg.contains("FILE"));
        }
        other => panic!("unexpected: {other:?}"),
    }
}
