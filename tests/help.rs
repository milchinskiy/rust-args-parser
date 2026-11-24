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

#[test]
fn render_help_uses_command_name_and_respects_never_color() {
    let env = env_basic();
    let root = ap::CmdSpec::<()>::new("tool")
        .help("A tool")
        .opt(ap::OptSpec::flag("verbose", |(): &mut ()| {}).long("verbose"));
    let help = ap::render_help(&env, &root);
    assert!(help.contains("tool [options]"), "{help}");
    assert!(!help.contains('\x1b'), "ColorMode::Never must not emit ANSI escapes");
}

#[cfg(feature = "color")]
#[test]
fn always_color_overrides_terminal_detection() {
    let env = ap::Env { color: ap::ColorMode::Always, ..env_basic() };
    let root = ap::CmdSpec::<()>::new("tool");
    assert!(ap::render_help(&env, &root).contains('\x1b'));
}
