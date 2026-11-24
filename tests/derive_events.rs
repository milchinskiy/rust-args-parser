#![cfg(feature = "derive")]

use rust_args_parser::{Args, Parser, Subcommand};
use std::cell::RefCell;
use std::ffi::{OsStr, OsString};

thread_local! {
    static EVENTS: RefCell<Vec<String>> = const { RefCell::new(Vec::new()) };
}

fn record(s: impl Into<String>) {
    EVENTS.with(|events| events.borrow_mut().push(s.into()));
}

fn take_events() -> Vec<String> {
    EVENTS.with(|events| std::mem::take(&mut *events.borrow_mut()))
}

#[derive(Debug)]
struct AppError(&'static str);
impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.0)
    }
}
impl std::error::Error for AppError {}

#[derive(Debug, Args)]
struct Common {
    #[arg(long, on_value = Self::on_global)]
    global: Option<String>,
    #[arg(skip)]
    seen: Vec<String>,
}

impl Common {
    fn on_global(&mut self, value: &OsStr) {
        self.seen.push(value.to_string_lossy().into_owned());
        record(format!("global={}", value.to_string_lossy()));
    }
}

#[derive(Debug, Parser)]
#[command(name = "tool", handler = Self::run)]
struct Cli {
    #[arg(short = 'v', action = "count", on_flag = Self::on_verbose)]
    verbose: u8,
    #[command(flatten)]
    common: Common,
    #[command(subcommand)]
    command: Option<Command>,
    #[arg(skip)]
    called: usize,
}

impl Cli {
    fn on_verbose(&mut self) {
        self.called += 1;
        record("verbose");
    }
    fn run(&mut self) {
        self.called += 100;
        record("root_handler");
    }
}

#[derive(Debug, Subcommand)]
enum Command {
    #[command(alias = "s", handler = Self::sync)]
    Sync {
        #[arg(long, on_flag_try = Self::dry)]
        dry: bool,
        #[arg(long, on_value_try = Self::tag)]
        tag: Vec<String>,
        #[arg(positional, value_name = "PATH", on_value = Self::path)]
        path: OsString,
        #[arg(skip)]
        completed: bool,
        #[arg(skip)]
        seen: Vec<String>,
    },
    #[command(handler_try = Self::fail)]
    Fail,
    #[command(handler = Self::status)]
    Status,
    Remote(Remote),
}

impl Command {
    fn dry(&mut self) -> Result<(), AppError> {
        if let Self::Sync { seen, .. } = self {
            if seen.iter().any(|event| event == "dry") {
                return Err(AppError("duplicate dry"));
            }
            seen.push("dry".into());
        }
        record("dry");
        Ok(())
    }

    fn tag(&mut self, value: &OsStr) -> Result<(), AppError> {
        if value == OsStr::new("reject") {
            return Err(AppError("tag rejected"));
        }
        if let Self::Sync { seen, .. } = self {
            seen.push(value.to_string_lossy().into_owned());
        }
        record(format!("tag={}", value.to_string_lossy()));
        Ok(())
    }

    fn path(_: &Self, value: &OsStr) {
        record(format!("path={}", value.to_string_lossy()));
    }

    fn sync(&mut self) {
        if let Self::Sync { dry, tag, path, completed, .. } = self {
            assert!(*dry);
            assert_eq!(tag.as_slice(), ["a", "b"]);
            assert_eq!(path, &OsString::from("src"));
            *completed = true;
        }
        record("sync_handler");
    }

    fn fail(_: &Self) -> Result<(), AppError> {
        record("failing_handler");
        Err(AppError("command failed"))
    }

    fn status(_: &Self) {
        record("status_handler");
    }
}

#[derive(Debug, Args)]
struct Remote {
    #[command(subcommand)]
    action: RemoteAction,
}

#[derive(Debug, Subcommand)]
enum RemoteAction {
    #[command(handler = Self::add)]
    Add {
        #[arg(positional, on_value = Self::on_name)]
        name: String,
        #[arg(skip)]
        invoked: bool,
    },
    List,
}

impl RemoteAction {
    fn on_name(_: &Self, raw: &OsStr) {
        record(format!("remote_name={}", raw.to_string_lossy()));
    }
    fn add(&mut self) {
        if let Self::Add { name, invoked } = self {
            assert_eq!(name, "origin");
            *invoked = true;
        }
        record("remote_add_handler");
    }
}

#[test]
fn event_order_and_state_are_owned_by_typed_values() {
    take_events();
    let cli = Cli::try_parse_from(["-vv", "--global", "g", "s", "--dry", "--tag", "a", "--tag=b", "src"]).unwrap();
    assert_eq!(cli.verbose, 2);
    assert_eq!(cli.called, 2);
    assert_eq!(cli.common.global.as_deref(), Some("g"));
    assert_eq!(cli.common.seen, vec!["g"]);
    match cli.command {
        Some(Command::Sync { completed, seen, .. }) => {
            assert!(completed);
            assert_eq!(seen, vec!["dry", "a", "b"]);
        }
        other => panic!("unexpected command: {other:?}"),
    }
    assert_eq!(take_events(), ["verbose", "verbose", "global=g", "dry", "tag=a", "tag=b", "path=src", "sync_handler"]);
}

#[test]
fn only_selected_leaf_handler_runs() {
    take_events();
    let cli = Cli::try_parse_from(["-v"]).unwrap();
    assert!(cli.command.is_none());
    assert_eq!(cli.called, 101);
    assert_eq!(take_events(), ["verbose", "root_handler"]);

    let cli = Cli::try_parse_from(["status"]).unwrap();
    assert!(matches!(cli.command, Some(Command::Status)));
    assert_eq!(cli.called, 0);
    assert_eq!(take_events(), ["status_handler"]);

    let cli = Cli::try_parse_from(["remote", "add", "origin"]).unwrap();
    match cli.command {
        Some(Command::Remote(Remote { action: RemoteAction::Add { invoked, .. } })) => assert!(invoked),
        other => panic!("unexpected command: {other:?}"),
    }
    assert_eq!(take_events(), ["remote_name=origin", "remote_add_handler"]);
}

#[test]
fn all_errors_precede_events_and_handlers() {
    take_events();
    let err = Cli::try_parse_from(["-v", "sync", "--tag", "a"]).unwrap_err();
    assert!(matches!(err, rust_args_parser::Error::User(_)));
    assert_eq!(take_events(), Vec::<String>::new());

    let many = vec![OsString::from("-v"); 256];
    assert!(matches!(Cli::try_parse_from(many), Err(rust_args_parser::Error::User(_))));
    assert_eq!(take_events(), Vec::<String>::new());

    let err = Cli::try_parse_from(["-v", "sync", "--no-such-option", "src"]).unwrap_err();
    assert!(matches!(err, rust_args_parser::Error::UnknownOption { .. }));
    assert_eq!(take_events(), Vec::<String>::new());
}

#[test]
fn fallible_callbacks_stop_dispatch_without_rollback() {
    take_events();
    let err = Cli::try_parse_from(["-v", "sync", "--dry", "--dry", "src"]).unwrap_err();
    match err {
        rust_args_parser::Error::UserAny(e) => assert_eq!(e.downcast_ref::<AppError>().unwrap().0, "duplicate dry"),
        other => panic!("unexpected error: {other:?}"),
    }
    assert_eq!(take_events(), ["verbose", "dry"]);

    let err = Cli::try_parse_from(["sync", "--dry", "--tag", "reject", "src"]).unwrap_err();
    match err {
        rust_args_parser::Error::UserAny(e) => assert_eq!(e.downcast_ref::<AppError>().unwrap().0, "tag rejected"),
        other => panic!("unexpected error: {other:?}"),
    }
    assert_eq!(take_events(), ["dry"]);

    let err = Cli::try_parse_from(["fail"]).unwrap_err();
    match err {
        rust_args_parser::Error::UserAny(e) => assert_eq!(e.downcast_ref::<AppError>().unwrap().0, "command failed"),
        other => panic!("unexpected error: {other:?}"),
    }
    assert_eq!(take_events(), ["failing_handler"]);
}

#[test]
fn all_field_conversions_precede_dispatch() {
    #[derive(Debug, Parser)]
    struct Typed {
        #[arg(long, on_value = Self::on_name)]
        name: String,
        #[arg(long)]
        count: u16,
    }
    impl Typed {
        fn on_name(_: &Self, _raw: &OsStr) {
            record("name");
        }
    }
    take_events();
    let err = Typed::try_parse_from(["--name", "ok", "--count", "bad"]).unwrap_err();
    assert!(matches!(err, rust_args_parser::Error::User(_)));
    assert_eq!(take_events(), Vec::<String>::new());
    let typed = Typed::try_parse_from(["--name", "ok", "--count", "42"]).unwrap();
    assert_eq!(typed.name, "ok");
    assert_eq!(typed.count, 42);
    assert_eq!(take_events(), ["name"]);
}

#[test]
fn explicit_validator_runs_before_events() {
    fn validate(value: &OsStr) -> Result<(), &'static str> {
        if value == OsStr::new("bad") {
            Err("invalid")
        } else {
            Ok(())
        }
    }
    #[derive(Debug, Parser)]
    #[command(handler = Self::run)]
    struct Validated {
        #[arg(long, value_parser = validate, on_value = Self::set)]
        tag: String,
        #[arg(skip)]
        seen: Vec<String>,
    }
    impl Validated {
        fn set(&mut self, _: &OsStr) {
            self.seen.push(self.tag.clone());
            record("tag");
        }
        fn run(_: &Self) {
            record("handler");
        }
    }
    take_events();
    assert!(matches!(Validated::try_parse_from(["--tag=bad"]), Err(rust_args_parser::Error::User(_))));
    assert_eq!(take_events(), Vec::<String>::new());
    let parsed = Validated::try_parse_from(["--tag=good"]).unwrap();
    assert_eq!(parsed.seen, vec!["good"]);
    assert_eq!(take_events(), ["tag", "handler"]);
}

#[test]
fn no_context_and_explicit_skipped_field_initializer() {
    #[derive(Debug, Parser)]
    #[command(handler = Self::run)]
    struct Simple {
        #[arg(long)]
        number: i32,
        #[arg(skip = 10usize)]
        result: usize,
    }
    impl Simple {
        fn run(&mut self) {
            self.result += usize::try_from(self.number).expect("nonnegative input");
        }
    }
    let parsed = Simple::try_parse_from(["--number=2"]).unwrap();
    assert_eq!(parsed.result, 12);
}

#[test]
fn events_follow_specification_order_options_then_positionals() {
    #[derive(Debug, Args)]
    struct Shared {
        #[arg(long, on_value = Self::option)]
        shared: Option<String>,
    }
    impl Shared {
        fn option(_: &Self, _: &OsStr) {
            record("shared");
        }
    }
    #[derive(Debug, Parser)]
    struct Ordering {
        #[arg(positional, on_value = Self::pos)]
        file: OsString,
        #[arg(long, on_flag = Self::flag)]
        flag: bool,
        #[command(flatten)]
        shared: Shared,
    }
    impl Ordering {
        fn pos(_: &Self, _: &OsStr) {
            record("pos");
        }
        fn flag(_: &Self) {
            record("flag");
        }
    }
    take_events();
    let result = Ordering::try_parse_from(["--shared", "x", "file", "--flag"]).unwrap();
    assert!(result.flag);
    assert_eq!(result.file, OsString::from("file"));
    assert_eq!(result.shared.shared.as_deref(), Some("x"));
    assert_eq!(take_events(), ["flag", "shared", "pos"]);
}

#[test]
fn overlay_callbacks_see_only_the_effective_source() {
    #[derive(Debug, Parser)]
    struct Overlay {
        #[arg(long, env = "RAP_300_DERIVE_EVENT_SOURCE", default_value = "fallback", on_value = Self::on_tag)]
        tags: Vec<String>,
    }
    impl Overlay {
        fn on_tag(_: &Self, value: &OsStr) {
            record(value.to_string_lossy());
        }
    }
    take_events();
    std::env::set_var("RAP_300_DERIVE_EVENT_SOURCE", "env");
    let typed = Overlay::try_parse_from(["--tags", "cli1", "--tags=cli2"]).unwrap();
    assert_eq!(typed.tags, vec!["cli1", "cli2"]);
    assert_eq!(take_events(), ["cli1", "cli2"]);
    let typed = Overlay::try_parse_from(Vec::<OsString>::new()).unwrap();
    assert_eq!(typed.tags, vec!["env"]);
    assert_eq!(take_events(), ["env"]);
    std::env::remove_var("RAP_300_DERIVE_EVENT_SOURCE");
    let typed = Overlay::try_parse_from(Vec::<OsString>::new()).unwrap();
    assert_eq!(typed.tags, vec!["fallback"]);
    assert_eq!(take_events(), ["fallback"]);
}

#[cfg(unix)]
#[test]
fn non_utf8_callback_and_field_are_byte_preserving() {
    use std::os::unix::ffi::OsStringExt;
    #[derive(Debug, Parser)]
    struct Native {
        #[arg(positional, on_value = Self::set)]
        input: OsString,
        #[arg(skip)]
        seen: OsString,
    }
    impl Native {
        fn set(&mut self, value: &OsStr) {
            self.seen = value.to_os_string();
        }
    }
    let raw = OsString::from_vec(vec![0xff, b'a']);
    let typed = Native::try_parse_from([raw.clone()]).unwrap();
    assert_eq!(typed.input, raw);
    assert_eq!(typed.seen, raw);
}

#[test]
fn generated_spec_is_usable_with_core_parser() {
    let spec = Cli::command();
    let m = rust_args_parser::parse(
        &rust_args_parser::Env::default(),
        &spec,
        &["s".into(), "--dry".into(), "src".into()],
        &mut (),
    )
    .unwrap();
    assert_eq!(m.leaf_path(), vec!["sync"]);
    assert!(m.view().is_set("dry"));
}

#[test]
fn nested_conversion_failure_prevents_parent_callbacks() {
    #[derive(Debug, Parser)]
    struct Parent {
        #[arg(short = 'v', on_flag = Self::on_verbose)]
        verbose: bool,
        #[command(subcommand)]
        command: Child,
    }
    impl Parent {
        fn on_verbose(_: &Self) {
            record("parent flag");
        }
    }
    #[derive(Debug, Subcommand)]
    enum Child {
        Run {
            #[arg(long)]
            attempts: u32,
        },
    }
    take_events();
    let err = Parent::try_parse_from(["-v", "run", "--attempts", "not-a-number"]).unwrap_err();
    assert!(matches!(err, rust_args_parser::Error::User(_)));
    assert_eq!(take_events(), Vec::<String>::new());
    let parsed = Parent::try_parse_from(["run", "--attempts=3"]).unwrap();
    assert!(!parsed.verbose);
    assert!(matches!(parsed.command, Child::Run { attempts: 3 }));
}

#[test]
fn singular_option_callback_sees_last_value_only() {
    #[derive(Debug, Parser)]
    struct Last {
        #[arg(long, on_value = Self::set)]
        value: String,
        #[arg(skip)]
        observed: Vec<String>,
    }
    impl Last {
        fn set(&mut self, raw: &OsStr) {
            self.observed.push(raw.to_string_lossy().into_owned());
        }
    }
    let value = Last::try_parse_from(["--value=first", "--value=second"]).unwrap();
    assert_eq!(value.value, "second");
    assert_eq!(value.observed, ["second"]);
}

#[test]
fn flag_count_from_environment_does_not_add_to_cli_count() {
    #[derive(Debug, Parser)]
    struct Presence {
        #[arg(long, env = "RAP_300_DERIVE_FLAG_SOURCE", on_flag = Self::bump)]
        enabled: bool,
        #[arg(skip)]
        seen: usize,
    }
    impl Presence {
        fn bump(&mut self) {
            self.seen += 1;
        }
    }
    std::env::set_var("RAP_300_DERIVE_FLAG_SOURCE", "false");
    let parsed = Presence::try_parse_from(Vec::<OsString>::new()).unwrap();
    assert!(parsed.enabled);
    assert_eq!(parsed.seen, 1);
    let parsed = Presence::try_parse_from(["--enabled", "--enabled"]).unwrap();
    assert!(parsed.enabled);
    assert_eq!(parsed.seen, 2);
    std::env::remove_var("RAP_300_DERIVE_FLAG_SOURCE");
}

#[test]
fn ambiguous_generated_specs_are_rejected_before_parse() {
    #[allow(dead_code)]
    #[derive(Debug, Args)]
    struct Shared {
        #[arg(long = "dup")]
        first: bool,
    }
    #[allow(dead_code)]
    #[derive(Debug, Parser)]
    struct Collide {
        #[command(flatten)]
        shared: Shared,
        #[arg(long = "dup")]
        second: bool,
    }
    #[allow(dead_code)]
    #[derive(Debug, Parser)]
    struct AliasCollision {
        #[command(subcommand)]
        command: Aliased,
    }
    #[derive(Debug, Subcommand)]
    enum Aliased {
        #[command(alias = "name")]
        First,
        #[command(name = "name")]
        Second,
    }
    let err = Collide::try_parse_from(Vec::<OsString>::new()).unwrap_err();
    assert!(matches!(err, rust_args_parser::Error::Parse(s) if s.contains("duplicate long")));

    let err = AliasCollision::try_parse_from(["name"]).unwrap_err();
    assert!(matches!(err, rust_args_parser::Error::Parse(s) if s.contains("duplicate subcommand")));
}

#[test]
fn changed_subcommand_variant_cannot_dispatch_wrong_handler() {
    #[derive(Debug, Parser)]
    struct Root {
        #[arg(long, on_flag = Self::replace_command)]
        replace: bool,
        #[command(subcommand)]
        command: Commands,
    }
    impl Root {
        fn replace_command(&mut self) {
            self.command = Commands::Other;
        }
    }
    #[derive(Debug, Subcommand)]
    enum Commands {
        #[command(handler = Self::run)]
        Run,
        #[command(handler = Self::other)]
        Other,
    }
    impl Commands {
        fn run(_: &Self) {
            record("run");
        }
        fn other(_: &Self) {
            record("other");
        }
    }
    take_events();
    let err = Root::try_parse_from(["--replace", "run"]).unwrap_err();
    assert!(matches!(err, rust_args_parser::Error::Parse(message) if message.contains("changed")));
    assert_eq!(take_events(), Vec::<String>::new());
    let parsed = Root::try_parse_from(["run"]).unwrap();
    assert!(!parsed.replace);
    assert!(matches!(parsed.command, Commands::Run));
    assert_eq!(take_events(), ["run"]);
}

#[test]
fn single_variant_nested_commands_dispatch_without_irrefutable_patterns() {
    #[derive(Debug, Parser)]
    struct RootTuple {
        #[command(subcommand)]
        command: Tuple,
    }
    #[derive(Debug, Subcommand)]
    enum Tuple {
        Wrap(Wrapped),
    }
    #[derive(Debug, Args)]
    struct Wrapped {
        #[command(subcommand)]
        action: Action,
    }
    #[derive(Debug, Subcommand)]
    enum Action {
        #[command(handler = Self::run)]
        Run,
    }
    impl Action {
        fn run(_: &Self) {
            record("single tuple");
        }
    }
    #[derive(Debug, Parser)]
    struct RootNamed {
        #[command(subcommand)]
        command: Named,
    }
    #[derive(Debug, Subcommand)]
    enum Named {
        Wrap {
            #[command(subcommand)]
            action: Action,
        },
    }
    take_events();
    let parsed = RootTuple::try_parse_from(["wrap", "run"]).unwrap();
    assert!(matches!(parsed.command, Tuple::Wrap(Wrapped { action: Action::Run })));
    assert_eq!(take_events(), ["single tuple"]);

    let parsed = RootNamed::try_parse_from(["wrap", "run"]).unwrap();
    assert!(matches!(parsed.command, Named::Wrap { action: Action::Run }));
    assert_eq!(take_events(), ["single tuple"]);
}
