#![cfg(feature = "derive")]

use ap::{Args, Parser, Subcommand};
use rust_args_parser as ap;
use std::ffi::{OsStr, OsString};

#[derive(Debug, Default)]
struct State {
    events: Vec<String>,
}

#[derive(Debug)]
struct AppError(&'static str);
impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.0)
    }
}
impl std::error::Error for AppError {}

fn verbose(state: &mut State) {
    state.events.push("verbose".into());
}
fn global_value(value: &OsStr, state: &mut State) {
    state.events.push(format!("global={}", value.to_string_lossy()));
}
fn set_path(value: &OsStr, state: &mut State) {
    state.events.push(format!("path={}", value.to_string_lossy()));
}
fn flag_try(state: &mut State) -> Result<(), AppError> {
    if state.events.iter().any(|event| event == "flag_try") {
        return Err(AppError("duplicate flag"));
    }
    state.events.push("flag_try".into());
    Ok(())
}
fn value_try(value: &OsStr, state: &mut State) -> Result<(), AppError> {
    if value == OsStr::new("reject") {
        Err(AppError("value rejected by callback"))
    } else {
        state.events.push(format!("value_try={}", value.to_string_lossy()));
        Ok(())
    }
}
fn leaf_handler(matches: &ap::Matches, state: &mut State) {
    assert_eq!(matches.leaf_path(), vec!["sync"]);
    assert!(matches.view().is_set("dry"));
    state.events.push("sync_handler".into());
}
fn root_handler(matches: &ap::Matches, state: &mut State) {
    assert!(matches.leaf_path().is_empty());
    state.events.push("root_handler".into());
}
fn failing_handler(_: &ap::Matches, state: &mut State) -> Result<(), AppError> {
    state.events.push("failing_handler".into());
    Err(AppError("handler failed"))
}

#[derive(Debug, Args)]
#[command(context = State)]
struct Common {
    #[arg(long, on_value = global_value)]
    global: Option<String>,
}

#[derive(Debug, Parser)]
#[command(name = "tool", context = State, handler = root_handler)]
struct Cli {
    #[arg(short = 'v', action = "count", on_flag = verbose)]
    verbosity: u8,
    #[command(flatten)]
    common: Common,
    #[command(subcommand)]
    cmd: Option<Command>,
}

#[derive(Debug, Subcommand)]
#[command(context = State)]
enum Command {
    #[command(alias = "s", handler = leaf_handler)]
    Sync {
        #[arg(long, on_flag_try = flag_try)]
        dry: bool,
        #[arg(long, on_value_try = value_try)]
        tag: Vec<String>,
        #[arg(positional, value_name = "PATH", on_value = set_path)]
        path: OsString,
    },
    #[command(handler_try = failing_handler)]
    Fail,
}

#[test]
fn callbacks_execute_in_spec_order_then_leaf_handler() {
    let mut state = State::default();
    let cli = Cli::try_parse_from_with_context(
        ["-vv", "--global", "g", "s", "--dry", "--tag", "a", "--tag=b", "src"],
        &mut state,
    )
    .unwrap();
    assert_eq!(cli.verbosity, 2);
    assert_eq!(cli.common.global.as_deref(), Some("g"));
    match cli.cmd {
        Some(Command::Sync { dry, tag, path }) => {
            assert!(dry);
            assert_eq!(tag, vec!["a", "b"]);
            assert_eq!(path, OsString::from("src"));
        }
        _ => panic!("expected sync"),
    }
    assert_eq!(
        state.events.iter().map(String::as_str).collect::<Vec<_>>(),
        ["verbose", "verbose", "global=g", "flag_try", "value_try=a", "value_try=b", "path=src", "sync_handler",]
    );
}

#[test]
fn root_handler_runs_only_when_no_subcommand_selected() {
    let mut state = State::default();
    let cli = Cli::try_parse_from_with_context(["-v"], &mut state).unwrap();
    assert!(cli.cmd.is_none());
    assert_eq!(state.events.iter().map(String::as_str).collect::<Vec<_>>(), ["verbose", "root_handler"]);
}

#[test]
fn typed_failures_do_not_dispatch_events() {
    let mut state = State::default();
    let err = Cli::try_parse_from_with_context(["-v", "sync", "--tag", "a"], &mut state);
    assert!(matches!(err, Err(ap::Error::User(_))), "missing required positional: {err:?}");
    assert!(state.events.is_empty());

    // count overflow is also detected before callbacks.
    let flags: Vec<OsString> = std::iter::repeat(OsString::from("-v")).take(256).collect();
    let err = Cli::try_parse_from_with_context(flags, &mut state);
    assert!(matches!(err, Err(ap::Error::User(_))));
    assert!(state.events.is_empty());
}

#[test]
fn fallible_value_handler_propagates_original_error_and_skips_leaf() {
    let mut state = State::default();
    let err = Cli::try_parse_from_with_context(["sync", "--tag", "reject", "src"], &mut state).unwrap_err();
    match err {
        ap::Error::UserAny(e) => assert_eq!(e.downcast_ref::<AppError>().unwrap().0, "value rejected by callback"),
        other => panic!("unexpected error: {other:?}"),
    }
    assert!(!state.events.iter().any(|e| e == "sync_handler"));
}

#[test]
fn fallible_command_handler_propagates_original_error() {
    let mut state = State::default();
    let err = Cli::try_parse_from_with_context(["fail"], &mut state).unwrap_err();
    match err {
        ap::Error::UserAny(e) => assert_eq!(e.downcast_ref::<AppError>().unwrap().0, "handler failed"),
        other => panic!("unexpected error: {other:?}"),
    }
    assert_eq!(state.events.iter().map(String::as_str).collect::<Vec<_>>(), ["failing_handler"]);
}

#[test]
fn generated_command_still_works_with_manual_core_parser() {
    let spec = Cli::command();
    let mut state = State::default();
    let matches =
        ap::parse(&ap::Env::default(), &spec, &["sync".into(), "--dry".into(), "f".into()], &mut state).unwrap();
    assert_eq!(matches.leaf_path(), vec!["sync"]);
    assert_eq!(state.events.iter().map(String::as_str).collect::<Vec<_>>(), ["flag_try", "path=f", "sync_handler"]);
}

#[test]
fn invalid_field_value_does_not_run_callbacks() {
    #[derive(Debug, Parser)]
    #[command(context = State)]
    struct Numeric {
        #[arg(long, on_value = global_value)]
        count: u16,
    }
    let mut state = State::default();
    let err = Numeric::try_parse_from_with_context(["--count", "not-int"], &mut state);
    assert!(matches!(err, Err(ap::Error::User(_))));
    assert!(state.events.is_empty());

    let parsed = Numeric::try_parse_from_with_context(["--count", "42"], &mut state).unwrap();
    assert_eq!(parsed.count, 42);
    assert_eq!(state.events, vec!["global=42".to_string()]);
}

#[test]
fn explicit_value_validator_blocks_events_and_handler() {
    fn validate(v: &OsStr) -> Result<(), &'static str> {
        if v == OsStr::new("invalid") {
            Err("invalid value")
        } else {
            Ok(())
        }
    }
    #[derive(Debug, Parser)]
    #[command(context = State, handler = root_handler)]
    struct Validated {
        #[arg(long, value_parser = validate, on_value = global_value)]
        name: String,
    }
    let mut state = State::default();
    let err = Validated::try_parse_from_with_context(["--name", "invalid"], &mut state).unwrap_err();
    assert!(matches!(err, ap::Error::User(_)));
    assert!(state.events.is_empty());

    let parsed = Validated::try_parse_from_with_context(["--name", "valid"], &mut state).unwrap();
    assert_eq!(parsed.name, "valid");
    assert_eq!(state.events, vec!["global=valid".to_string(), "root_handler".to_string()]);
}

#[cfg(unix)]
#[test]
fn non_utf8_positional_reaches_callback_unchanged() {
    use std::os::unix::ffi::{OsStrExt, OsStringExt};
    #[derive(Default)]
    struct Native {
        bytes: Vec<u8>,
    }
    fn set_native(value: &OsStr, state: &mut Native) {
        state.bytes = value.as_bytes().to_vec();
    }
    #[derive(Debug, Parser)]
    #[command(context = Native)]
    struct NativeCli {
        #[arg(positional, on_value = set_native)]
        file: OsString,
    }
    let invalid = OsString::from_vec(vec![0xff, b'a']);
    let mut state = Native::default();
    let parsed = NativeCli::try_parse_from_with_context(vec![invalid.clone()], &mut state).unwrap();
    assert_eq!(parsed.file, invalid);
    assert_eq!(state.bytes, vec![0xff, b'a']);
}

#[test]
fn fallible_flag_callback_propagates_typed_error() {
    fn reject(_: &mut State) -> Result<(), AppError> {
        Err(AppError("flag rejected"))
    }
    #[derive(Debug, Parser)]
    #[command(context = State, handler = root_handler)]
    struct Flag {
        #[arg(long, on_flag_try = reject)]
        reject: bool,
    }
    let mut state = State::default();
    let err = Flag::try_parse_from_with_context(["--reject"], &mut state).unwrap_err();
    match err {
        ap::Error::UserAny(e) => assert_eq!(e.downcast_ref::<AppError>().unwrap().0, "flag rejected"),
        other => panic!("unexpected error: {other:?}"),
    }
    assert!(state.events.is_empty(), "leaf handler must not execute after callback error");

    let parsed = Flag::try_parse_from_with_context(std::iter::empty::<&str>(), &mut state).unwrap();
    assert!(!parsed.reject);
    assert_eq!(state.events, vec!["root_handler".to_string()]);
}

#[test]
fn callbacks_honor_cli_over_environment_and_default() {
    const ENV_KEY: &str = "RAP_210_EVENT_OVERLAY";
    #[derive(Debug, Parser)]
    #[command(context = State, handler = root_handler)]
    struct Overlay {
        #[arg(long, env = "RAP_210_EVENT_OVERLAY", default_value = "fallback", on_value = global_value)]
        tag: Vec<String>,
    }
    std::env::set_var(ENV_KEY, "environment");
    let mut state = State::default();
    let parsed = Overlay::try_parse_from_with_context(["--tag=a", "--tag=b"], &mut state).unwrap();
    assert_eq!(parsed.tag, vec!["a", "b"]);
    assert_eq!(state.events.iter().map(String::as_str).collect::<Vec<_>>(), ["global=a", "global=b", "root_handler"]);
    std::env::remove_var(ENV_KEY);
    let mut state = State::default();
    let parsed = Overlay::try_parse_from_with_context(Vec::<OsString>::new(), &mut state).unwrap();
    assert_eq!(parsed.tag, vec!["fallback"]);
    assert_eq!(state.events.iter().map(String::as_str).collect::<Vec<_>>(), ["global=fallback", "root_handler"]);
}
