#![cfg(feature = "derive")]
#![allow(dead_code)]

use ap::{Args, Parser, Subcommand};
use rust_args_parser as ap;
use std::ffi::{OsStr, OsString};
use std::path::PathBuf;

fn argv(parts: &[&str]) -> Vec<OsString> {
    parts.iter().map(OsString::from).collect()
}

fn non_empty(v: &OsStr) -> Result<(), &'static str> {
    if v.is_empty() {
        Err("empty argument")
    } else {
        Ok(())
    }
}

#[derive(Debug, PartialEq, Parser)]
#[command(name = "all", help = "Typed parser coverage")]
struct AllFields {
    /// Verbosity can be clustered, as in -vvv.
    #[arg(short = 'v', action = "count")]
    verbose: u8,
    #[arg(short = 'j', long, value_name = "N", default_value = "4")]
    jobs: u32,
    #[arg(long, help = "Output JSON")]
    json: bool,
    #[arg(short = 'D', long = "define")]
    define: Vec<String>,
    #[arg(long)]
    maybe: Option<i64>,
    #[arg(positional, value_name = "INPUT", value_parser = non_empty)]
    input: OsString,
    #[arg(positional, value_name = "FILE")]
    files: Vec<PathBuf>,
}

#[test]
fn typed_flags_values_counts_positionals_and_defaults() {
    let parsed = AllFields::try_parse_from(argv(&[
        "-vvvj8",
        "--json",
        "-D",
        "x=1",
        "--define=y=2",
        "--maybe",
        "-3",
        "in.txt",
        "a.txt",
        "b.txt",
    ]))
    .unwrap();
    assert_eq!(parsed.verbose, 3);
    assert_eq!(parsed.jobs, 8);
    assert!(parsed.json);
    assert_eq!(parsed.define, vec!["x=1".to_owned(), "y=2".to_owned()]);
    assert_eq!(parsed.maybe, Some(-3));
    assert_eq!(parsed.input.as_os_str(), OsStr::new("in.txt"));
    assert_eq!(parsed.files, vec![PathBuf::from("a.txt"), PathBuf::from("b.txt")]);

    let parsed = AllFields::try_parse_from(["file"]).unwrap();
    assert_eq!(parsed.jobs, 4);
    assert_eq!(parsed.verbose, 0);
    assert!(!parsed.json);
    assert_eq!(parsed.define.len(), 0);
    assert_eq!(parsed.maybe, None);
}

#[test]
fn error_variants_and_atomic_validation() {
    assert!(matches!(AllFields::try_parse_from(Vec::<OsString>::new()), Err(ap::Error::User(_))));
    assert!(matches!(AllFields::try_parse_from(["--jobs", "wat", "in"]), Err(ap::Error::User(_))));
    assert!(matches!(AllFields::try_parse_from(["--maybe"]), Err(ap::Error::MissingValue { .. })));
    assert!(matches!(AllFields::try_parse_from(["--oops", "in"]), Err(ap::Error::UnknownOption { .. })));
    match AllFields::try_parse_from([""]) {
        Err(ap::Error::User(text)) => assert!(text.contains("empty argument")),
        other => panic!("unexpected: {other:?}"),
    }
}

#[derive(Debug, PartialEq, Parser)]
#[command(name = "svc", group(name = "output", mode = "xor"))]
struct Overlays {
    #[arg(long, env = "RAP_DERIVE_LIMIT", default_value = "7")]
    limit: u16,
    #[arg(long = "json", group = "output")]
    json: bool,
    #[arg(long, group = "output")]
    yaml: bool,
    #[arg(long, env = "RAP_DERIVE_FILES")]
    files: Vec<String>,
    #[arg(long, env = "RAP_DERIVE_LOG")]
    log: bool,
}

#[test]
fn overlay_precedence_repeats_and_flag_counts() {
    std::env::set_var("RAP_DERIVE_LIMIT", "8");
    std::env::set_var("RAP_DERIVE_FILES", "from-env");
    std::env::set_var("RAP_DERIVE_LOG", "1");
    let p = Overlays::try_parse_from(["--limit=9", "--files", "cli1", "--files", "cli2", "--log"]).unwrap();
    assert_eq!(p.limit, 9);
    assert_eq!(p.files, vec!["cli1".to_owned(), "cli2".to_owned()], "CLI must replace ENV, not append to it");
    assert!(p.log);

    let spec = Overlays::command();
    let mut ctx = ();
    let m = ap::parse(&ap::Env::default(), &spec, &argv(&["--log", "--files", "cli"]), &mut ctx).unwrap();
    assert_eq!(m.flag_count("log"), 1, "ENV flag must not count in addition to CLI flag");
    assert!(m.is_set_from("files", ap::Source::Cli));
    assert!(m.is_set_from("limit", ap::Source::Env));

    let p = Overlays::try_parse_from(Vec::<OsString>::new()).unwrap();
    assert_eq!(p.limit, 8);
    assert_eq!(p.files, vec!["from-env".to_owned()]);
    assert!(p.log);

    std::env::remove_var("RAP_DERIVE_LIMIT");
    std::env::remove_var("RAP_DERIVE_FILES");
    std::env::remove_var("RAP_DERIVE_LOG");
    let p = Overlays::try_parse_from(Vec::<OsString>::new()).unwrap();
    assert_eq!(p.limit, 7);
    assert_eq!(p.files, Vec::<String>::new());
}

#[test]
fn xor_group_respects_all_sources() {
    match Overlays::try_parse_from(["--yaml", "--json"]) {
        Err(ap::Error::User(text)) => assert!(text.contains("mutually exclusive")),
        other => panic!("unexpected: {other:?}"),
    }
}

#[derive(Debug, PartialEq, Parser)]
#[command(name = "fmt", group(name = "kind", mode = "req_one"))]
struct ReqOne {
    #[arg(long, group = "kind")]
    text: bool,
    #[arg(long, group = "kind")]
    binary: bool,
}

#[test]
fn required_group() {
    assert!(matches!(ReqOne::try_parse_from(Vec::<OsString>::new()), Err(ap::Error::User(_))));
    assert!(ReqOne::try_parse_from(["--text"]).unwrap().text);
}

#[derive(Debug, PartialEq, Args)]
struct Global {
    #[arg(short = 'C')]
    chdir: Option<PathBuf>,
    #[arg(short = 'v', action = "count")]
    verbose: usize,
}

#[derive(Debug, PartialEq, Parser)]
#[command(name = "tool")]
struct Tool {
    #[arg(flatten)]
    global: Global,
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, PartialEq, Args)]
struct Remote {
    #[command(subcommand)]
    action: RemoteAction,
}

#[derive(Debug, PartialEq, Subcommand)]
enum Command {
    #[command(alias = "r", help = "Remote operations")]
    Remote(Remote),
    Status,
}

#[derive(Debug, PartialEq, Subcommand)]
enum RemoteAction {
    Add {
        #[arg(positional, value_name = "NAME")]
        name: String,
        #[arg(positional, value_name = "URL")]
        url: String,
    },
    #[command(name = "ls", alias = "list")]
    List,
}

#[test]
fn nested_enum_aliases_flatten_and_scopes() {
    let parsed = Tool::try_parse_from(["-vv", "-C", "/tmp", "r", "add", "origin", "https://example"]).unwrap();
    assert_eq!(parsed.global.verbose, 2);
    assert_eq!(parsed.global.chdir, Some(PathBuf::from("/tmp")));
    assert_eq!(
        parsed.command,
        Command::Remote(Remote { action: RemoteAction::Add { name: "origin".into(), url: "https://example".into() } })
    );
    let parsed = Tool::try_parse_from(["remote", "list"]).unwrap();
    assert!(matches!(parsed.command, Command::Remote(Remote { action: RemoteAction::List })));
    assert!(matches!(Tool::try_parse_from(["r", "add", "only-one"]), Err(ap::Error::User(_))));
    assert!(matches!(Tool::try_parse_from(Vec::<OsString>::new()), Err(ap::Error::User(_))));
    assert!(matches!(Tool::try_parse_from(["status", "-v"]), Err(ap::Error::UnknownOption { .. })));
    assert!(matches!(Tool::try_parse_from(["statuz"]), Err(ap::Error::UnknownCommand { .. })));
}

#[test]
fn command_and_argument_help_are_consistent() {
    let root = AllFields::command();
    assert_eq!(root.get_help(), Some("Typed parser coverage"));
    let json = root.get_opts().iter().find(|o| o.get_name() == "json").unwrap();
    assert_eq!(json.get_help(), Some("Output JSON"));

    let tool = Tool::command();
    let remote = tool.get_subcommands().iter().find(|cmd| cmd.get_name() == "remote").unwrap();
    assert_eq!(remote.get_help(), Some("Remote operations"));
}

#[cfg(feature = "help")]
#[test]
fn help_output_contains_derived_command_descriptions() {
    let cases = [
        (AllFields::try_parse_from(["--help"]).unwrap_err(), "Typed parser coverage"),
        (Tool::try_parse_from(["remote", "--help"]).unwrap_err(), "Remote operations"),
    ];
    for (error, expected) in cases {
        match error {
            ap::Error::ExitMsg { code: 0, message: Some(help) } => assert!(help.contains(expected), "{help}"),
            other => panic!("unexpected: {other:?}"),
        }
    }
}

#[derive(Debug, PartialEq, Parser)]
#[command(name = "optional")]
struct Optional {
    #[command(subcommand)]
    cmd: Option<Command>,
}

#[test]
fn optional_subcommand_and_unit_variants() {
    assert_eq!(Optional::try_parse_from(Vec::<OsString>::new()).unwrap().cmd, None);
    assert_eq!(Optional::try_parse_from(["status"]).unwrap().cmd, Some(Command::Status));
}

#[derive(Debug, PartialEq, Parser)]
#[command(name = "take")]
struct Range {
    #[arg(positional, value_name = "NUM", min = 2, max = 3)]
    values: Vec<i32>,
}

#[test]
fn bounded_positionals_numeric_and_end_of_options() {
    assert_eq!(Range::try_parse_from(["-1", "-2"]).unwrap().values, vec![-1, -2]);
    assert_eq!(Range::try_parse_from(["--", "-1", "-2", "3"]).unwrap().values, vec![-1, -2, 3]);
    assert!(matches!(Range::try_parse_from(["1"]), Err(ap::Error::User(_))));
    assert!(matches!(Range::try_parse_from(["1", "2", "3", "4"]), Err(ap::Error::UnexpectedPositional { .. })));
}

#[derive(Debug, PartialEq, Parser)]
#[command(name = "typed")]
struct Custom {
    #[arg(long)]
    required: u32,
    #[arg(long, required = false)]
    optional_with_default: u8,
}

#[test]
fn required_scalar_options_and_defaultable_scalars() {
    assert!(matches!(Custom::try_parse_from(Vec::<OsString>::new()), Err(ap::Error::User(_))));
    let c = Custom::try_parse_from(["--required", "17"]).unwrap();
    assert_eq!(c.required, 17);
    assert_eq!(c.optional_with_default, 0);
}

#[cfg(feature = "help")]
#[test]
fn generated_help_has_metadata_and_builtins() {
    let env = ap::Env { color: ap::ColorMode::Never, ..ap::Env::default() };
    let err = Tool::try_parse_from_with_env(&env, ["remote", "--help"]).unwrap_err();
    match err {
        ap::Error::ExitMsg { code: 0, message: Some(help) } => {
            assert!(help.contains("Remote operations"));
            assert!(help.contains("Commands"));
            assert!(help.contains("add"));
            assert!(help.contains("ls"));
        }
        other => panic!("unexpected: {other:?}"),
    }
}

#[cfg(feature = "suggest")]
#[test]
fn generated_suggestions_use_core_parser() {
    let err = Tool::try_parse_from(["remot"]).unwrap_err();
    match err {
        ap::Error::UnknownCommand { suggestions, .. } => assert!(suggestions.contains(&"remote".to_owned())),
        other => panic!("unexpected: {other:?}"),
    }
}

#[cfg(unix)]
#[test]
fn native_non_utf8_is_preserved_while_typed_strings_are_rejected() {
    use std::os::unix::ffi::OsStringExt;
    let raw = OsString::from_vec(vec![0xff, b'x']);
    let cli = AllFields::try_parse_from([raw.clone()]).unwrap();
    assert_eq!(cli.input, raw);
    let err = Range::try_parse_from([OsString::from("1"), OsString::from_vec(vec![0xff])]).unwrap_err();
    assert!(matches!(err, ap::Error::User(_)));
}

#[derive(Debug)]
struct DomainError;
impl std::fmt::Display for DomainError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("domain validator rejected input")
    }
}
impl std::error::Error for DomainError {}

fn typed_validation(s: &OsStr) -> Result<(), DomainError> {
    if s == OsStr::new("bad") {
        Err(DomainError)
    } else {
        Ok(())
    }
}

#[derive(Debug, Parser)]
#[command(name = "validated")]
struct TypedValidator {
    #[arg(long, value_parser_try = typed_validation)]
    value: String,
}

#[test]
fn typed_validator_preserves_userany_error() {
    let err = TypedValidator::try_parse_from(["--value=bad"]).unwrap_err();
    match err {
        ap::Error::UserAny(error) => assert!(error.downcast_ref::<DomainError>().is_some()),
        other => panic!("unexpected: {other:?}"),
    }
}

fn needs_target(m: &ap::Matches) -> Result<(), &'static str> {
    if m.view().value("target").is_some() {
        Ok(())
    } else {
        Err("target required by command validator")
    }
}

#[derive(Debug, Parser)]
#[command(name = "command-validation")]
struct WithCommandValidator {
    #[command(subcommand)]
    command: ValidatedCommand,
}

#[derive(Debug, Subcommand)]
enum ValidatedCommand {
    #[command(validate = needs_target)]
    Edit {
        #[arg(long)]
        target: Option<String>,
    },
}

#[test]
fn nested_command_validator_has_leaf_scoped_matches() {
    match WithCommandValidator::try_parse_from(["edit"]) {
        Err(ap::Error::User(msg)) => assert!(msg.contains("target required")),
        other => panic!("unexpected: {other:?}"),
    }
    assert!(WithCommandValidator::try_parse_from(["edit", "--target", "okay"]).is_ok());
}

#[test]
fn core_flag_inline_value_rule_is_preserved() {
    let parsed = AllFields::try_parse_from(["--json=false", "in"]).unwrap();
    assert!(parsed.json, "core recognizes --flag=... as flag presence");
}

#[derive(Debug, PartialEq)]
enum Encoding {
    Utf8,
    Ascii,
}
impl std::str::FromStr for Encoding {
    type Err = &'static str;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "utf8" => Ok(Self::Utf8),
            "ascii" => Ok(Self::Ascii),
            _ => Err("expected utf8 or ascii"),
        }
    }
}

#[derive(Debug, Parser)]
#[command(name = "encoding")]
struct CustomFromStr {
    #[arg(long)]
    encoding: Encoding,
}

#[test]
fn custom_fromstr_conversion_and_validation_error() {
    assert!(matches!(
        CustomFromStr::try_parse_from(["--encoding=utf8"]),
        Ok(CustomFromStr { encoding: Encoding::Utf8 })
    ));
    match CustomFromStr::try_parse_from(["--encoding", "unknown"]) {
        Err(ap::Error::User(message)) => assert!(message.contains("expected utf8 or ascii")),
        other => panic!("unexpected: {other:?}"),
    }
}

#[test]
fn alias_is_canonicalized_in_scoped_matches() {
    let spec = Tool::command();
    let mut ctx = ();
    let m = ap::parse(&ap::Env::default(), &spec, &argv(&["r", "list"]), &mut ctx).unwrap();
    assert_eq!(m.leaf_path(), vec!["remote", "ls"]);
}

#[derive(Debug, Parser)]
#[command(name = "env-flag")]
struct EnvironmentFlag {
    #[arg(long, env = "RAP_DERIVE_PRESENCE_FALSE")]
    flag: bool,
}

#[test]
fn env_flag_means_presence_even_when_set_to_false() {
    std::env::set_var("RAP_DERIVE_PRESENCE_FALSE", "false");
    assert!(EnvironmentFlag::try_parse_from(Vec::<OsString>::new()).unwrap().flag);
    std::env::remove_var("RAP_DERIVE_PRESENCE_FALSE");
    assert!(!EnvironmentFlag::try_parse_from(Vec::<OsString>::new()).unwrap().flag);
}

#[derive(Debug, PartialEq, Parser)]
#[command(name = "required-option")]
struct RequiredOption {
    #[arg(long, required)]
    token: Option<String>,
    #[arg(long, required)]
    tags: Vec<String>,
}

#[test]
fn explicit_required_optional_and_repeated_fields() {
    assert!(matches!(RequiredOption::try_parse_from(Vec::<OsString>::new()), Err(ap::Error::User(_))));
    assert!(matches!(RequiredOption::try_parse_from(["--token", "ok"]), Err(ap::Error::User(_))));
    let parsed = RequiredOption::try_parse_from(["--token", "ok", "--tags", "a", "--tags", "b"]).unwrap();
    assert_eq!(parsed.token, Some("ok".into()));
    assert_eq!(parsed.tags, vec!["a".to_owned(), "b".to_owned()]);
}

#[derive(Debug, PartialEq, Parser)]
#[command(name = "singular")]
struct Singular {
    #[arg(long, name = "internal", value_name = "N")]
    setting: i32,
}

#[test]
fn logical_name_remains_distinct_from_long_name_and_last_value_wins() {
    let parsed = Singular::try_parse_from(["--setting=4", "--setting=9"]).unwrap();
    assert_eq!(parsed.setting, 9);
    let spec = Singular::command();
    let mut ctx = ();
    let m = ap::parse(&ap::Env::default(), &spec, &["--setting=42".into()], &mut ctx).unwrap();
    assert_eq!(m.get_value("internal"), Some(OsStr::new("42")));
    assert_eq!(m.get_value("setting"), None);
}

#[test]
fn count_field_reports_integer_overflow() {
    let args = vec![OsString::from("-v"); usize::from(u8::MAX) + 1];
    let mut args = args;
    args.push("input".into());
    match AllFields::try_parse_from(args) {
        Err(ap::Error::User(msg)) => assert!(msg.contains("overflow")),
        other => panic!("unexpected result: {other:?}"),
    }
}
