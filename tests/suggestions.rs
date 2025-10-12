use rust_args_parser as ap;

struct Ctx;
#[allow(clippy::unnecessary_wraps)]
fn ok(_: Option<&str>, _: &mut Ctx) -> std::result::Result<(), Box<dyn std::error::Error>> {
    Ok(())
}

#[test]
fn suggests_for_long_typo_help() {
    let env = ap::Env::new("demo").auto_help(true);
    let root = ap::CmdSpec::new(None, None);
    let mut ctx = Ctx;
    let err = ap::dispatch(&env, &root, &["--hlep"], &mut ctx).unwrap_err();
    match err {
        ap::Error::UnknownOption { token, suggestions } => {
            assert_eq!(token, "--hlep");
            assert!(suggestions.iter().any(|s| s == "--help"));
        }
        _ => panic!("unexpected error: {err}"),
    }
}

#[test]
fn suggests_for_short_typo_upper_lower_case() {
    let env = ap::Env::new("demo").auto_help(true);
    let root = ap::CmdSpec::new(None, None);
    let err = ap::dispatch(&env, &root, &["-H"], &mut ()).unwrap_err();
    match err {
        ap::Error::UnknownOption { token, suggestions } => {
            assert_eq!(token, "-H");
            assert!(suggestions.iter().any(|s| s == "-h"));
        }
        _ => panic!("unexpected error: {err}"),
    }
}

#[test]
fn suggests_for_unknown_command() {
    let env = ap::Env::new("demo").auto_help(true);
    let sub_start = ap::CmdSpec::<()>::new(Some("start"), None);
    let sub_status = ap::CmdSpec::<()>::new(Some("status"), None);
    let root = ap::CmdSpec::new(None, None).subs([sub_start, sub_status]);
    let err = ap::dispatch(&env, &root, &["stat"], &mut ()).unwrap_err();
    match err {
        ap::Error::UnknownCommand { token, suggestions } => {
            assert_eq!(token, "stat");
            assert!(suggestions.iter().any(|s| s == "start" || s == "status"));
        }
        _ => panic!("unexpected error: {err}"),
    }
}

#[test]
fn long_suggestions_come_from_declared_opts_only() {
    let env = ap::Env::new("demo").auto_help(false); // built-ins disabled
    let root = ap::CmdSpec::new(None, None).opts([
        ap::OptSpec::new("limit", ok).short('l').help("x").optional(),
        ap::OptSpec::new("lines", ok).help("x").optional(),
    ]);
    let mut ctx = Ctx;
    let err = ap::dispatch(&env, &root, &["--limti"], &mut ctx).unwrap_err();
    match err {
        ap::Error::UnknownOption { suggestions, .. } => {
            // should *not* suggest --help, only declared long options
            assert!(suggestions.iter().all(|s| s == "--limit" || s == "--lines"));
        }
        _ => panic!("unexpected error"),
    }
}
