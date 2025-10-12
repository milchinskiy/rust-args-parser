mod common;
use common::*;

// Required short value missing at end: "-o" → MissingValue("output")
#[test]
fn short_required_value_missing_at_end() {
    let env = env_base();
    let opts = [OptSpec::new("output", cb_output).short('o').required().metavar("FILE")];
    let cmd = CmdSpec::new(None, None).opts(opts);

    let mut u = U::default();
    let mut out = Vec::<u8>::new();
    let err = dispatch_to(&env, &cmd, &["-o"], &mut u, &mut out).unwrap_err();
    match err {
        Error::MissingValue(n) => assert_eq!(n, "output"),
        _ => panic!("expected MissingValue"),
    }
}

// Optional short with a lone dash next: "-j - -v" should consume "-" as no value and keep parsing
#[test]
fn optional_short_consumes_lone_dash_as_none() {
    let env = env_base();
    let opts = [
        OptSpec::new("jobs", cb_jobs).short('j').optional().numeric(),
        OptSpec::new("verbose", cb_verbose).short('v').flag(),
    ];
    let cmd = CmdSpec::new(None, Some(|_,_| Ok(()))).opts(opts);

    let mut u = U::default();
    let mut out = Vec::<u8>::new();
    let r = dispatch_to(&env, &cmd, &["-j", "-", "-v"], &mut u, &mut out);
    assert!(r.is_ok());
    assert_eq!(u.jobs, None);
    assert_eq!(u.verbose, 1);
}

// Short cluster with attached negative value: "-j-3"
#[test]
fn short_attached_negative_value_in_cluster() {
    let env = env_base();
    let opts = [OptSpec::new("jobs", cb_jobs).short('j').optional().numeric()];
    let cmd = CmdSpec::new(None, Some(|_,_| Ok(()))).opts(opts);

    let mut u = U::default();
    let mut out = Vec::<u8>::new();
    let r = dispatch_to(&env, &cmd, &["-j-3"], &mut u, &mut out);
    assert!(r.is_ok());
    assert_eq!(u.jobs.as_deref(), Some("-3"));
}

// Repeated flag counts: "-vvv" → verbose = 3
#[test]
fn verbose_repeated_flag_counts() {
    let env = env_base();
    let opts = [OptSpec::new("verbose", cb_verbose).short('v').flag()];
    let cmd = CmdSpec::new(None, Some(|_,_| Ok(()))).opts(opts);

    let mut u = U::default();
    let mut out = Vec::<u8>::new();
    let r = dispatch_to(&env, &cmd, &["-vvv"], &mut u, &mut out);
    assert!(r.is_ok());
    assert_eq!(u.verbose, 3);
}

// required_one satisfied by default value on one member
#[test]
fn required_one_satisfied_by_default() {
    let env = env_base();
    // group id 7 for illustration
    let opts = [
        OptSpec::new("def", cb_defv).default("D").at_least_one(7),
        OptSpec::new("yy", cb_one_b).at_least_one(7),
    ];
    let cmd = CmdSpec::new(None, Some(|_,_| Ok(()))).opts(opts);

    let mut u = U::default();
    let mut out = Vec::<u8>::new();
    let r = dispatch_to(&env, &cmd, &[], &mut u, &mut out);
    assert!(r.is_ok());
    assert_eq!(u.defv.as_deref(), Some("D"));
}

// Subcommand help exits with 0 and prints Usage for that subcommand
#[test]
fn subcommand_help_exits() {
    let env = env_base();

    let sub_opts = [OptSpec::new("verbose", cb_verbose).short('v').flag()];
    let sub_pos = [PosSpec::new("X").one()];
    let sub = CmdSpec::new(Some("sub"), None).opts(sub_opts).pos(sub_pos);
    let subs = [sub];
    let root = CmdSpec::new(None, None).subs(subs);

    let mut u = U::default();
    let mut out = Vec::<u8>::new();
    match dispatch_to(&env, &root, &["sub", "--help"], &mut u, &mut out) {
        Err(Error::Exit(0)) => {}
        other => panic!("expected Exit(0), got {other:?}"),
    }
    let t = s(&out);
    assert!(t.contains("Usage:"));
}

#[test]
fn long_optional_consumes_lone_dash_as_none() {
    let env = env_base();
    let opts = [
        OptSpec::new("jobs", cb_jobs).optional().numeric(),
        OptSpec::new("verbose", cb_verbose).short('v').flag(),
    ];
    let cmd = CmdSpec::new(None, Some(|_,_| Ok(()))).opts(opts);

    let mut u = U::default();
    let mut out = Vec::<u8>::new();
    let r = dispatch_to(&env, &cmd, &["--jobs", "-", "-v"], &mut u, &mut out);
    assert!(r.is_ok());
    assert_eq!(u.jobs, None);
    assert_eq!(u.verbose, 1);
}

#[test]
fn default_suppressed_when_explicit_in_xor_group() {
    let env = env_base();
    let opts = [
        OptSpec::new("def", cb_defv).default("D").at_most_one(3),
        OptSpec::new("x", cb_one_a).flag().at_most_one(3),
    ];
    let cmd = CmdSpec::new(None, Some(|_,_| Ok(()))).opts(opts);

    let mut u = U::default();
    let mut out = Vec::<u8>::new();
    let r = dispatch_to(&env, &cmd, &["--x"], &mut u, &mut out);
    assert!(r.is_ok());
    // default must NOT have been applied because group became occupied by --x
    assert_eq!(u.defv, None);
    assert!(u.one_a);
}

#[test]
fn required_one_satisfied_by_env() {
    let env = env_base();
    let key = "RAP_REQ_ENV";
    std::env::set_var(key, "ENVV");

    let opts = [
        OptSpec::new("envd", cb_envd).env(key).at_least_one(4),
        OptSpec::new("yy", cb_one_b).at_least_one(4),
    ];
    let cmd = CmdSpec::new(None, Some(|_,_| Ok(()))).opts(opts);

    let mut u = U::default();
    let mut out = Vec::<u8>::new();
    let r = dispatch_to(&env, &cmd, &[], &mut u, &mut out);
    assert!(r.is_ok());
    assert_eq!(u.envd.as_deref(), Some("ENVV"));

    std::env::remove_var(key);
}

#[test]
fn long_required_missing_without_value_token() {
    let env = env_base();
    let opts = [OptSpec::new("output", cb_output).required().metavar("FILE")];
    let cmd = CmdSpec::new(None, None).opts(opts);

    let mut u = U::default();
    let mut out = Vec::<u8>::new();
    let err = dispatch_to(&env, &cmd, &["--output"], &mut u, &mut out).unwrap_err();
    match err {
        Error::MissingValue(n) => assert_eq!(n, "output"),
        _ => panic!("expected MissingValue"),
    }
}

#[test]
fn subcommand_unknown_option_errors() {
    let env = env_base();

    let sub_opts = [OptSpec::new("verbose", cb_verbose).short('v').flag()];
    let sub = CmdSpec::new(Some("sub"), None).opts(sub_opts);
    let subs = [sub];
    let root = CmdSpec::new(None, None).subs(subs);

    let mut u = U::default();
    let mut out = Vec::<u8>::new();
    let e = dispatch_to(&env, &root, &["sub", "-Z"], &mut u, &mut out).unwrap_err();
    match e {
        Error::UnknownOption { token: s, .. } => assert_eq!(s, "-Z"),
        _ => panic!("expected UnknownOption"),
    }
}

#[test]
fn short_optional_then_another_short_option() {
    let env = env_base();
    let opts = [
        OptSpec::new("jobs", cb_jobs).short('j').optional().numeric(),
        OptSpec::new("verbose", cb_verbose).short('v').flag(),
    ];
    let cmd = CmdSpec::new(None, Some(|_,_| Ok(()))).opts(opts);

    let mut u = U::default();
    let mut out = Vec::<u8>::new();
    let r = dispatch_to(&env, &cmd, &["-j", "-v"], &mut u, &mut out);
    assert!(r.is_ok());
    assert_eq!(u.jobs, None);
    assert_eq!(u.verbose, 1);
}

// Long optional consumes "+8" as value (plus sign allowed)
#[test]
fn long_optional_plus_number_consumed() {
    let env = env_base();
    let opts = [OptSpec::new("jobs", cb_jobs).optional().numeric()];
    let cmd = CmdSpec::new(None, Some(|_,_| Ok(()))).opts(opts);

    let mut u = U::default();
    let mut out = Vec::<u8>::new();
    let r = dispatch_to(&env, &cmd, &["--jobs", "+8"], &mut u, &mut out);
    assert!(r.is_ok());
    assert_eq!(u.jobs.as_deref(), Some("+8"));
}

// After "--" with no positional schema → UnexpectedArgument for first trailing token
#[test]
fn double_dash_without_positional_schema_is_unexpected() {
    let env = env_base();
    let cmd = CmdSpec::new(None, None).opts(vec![]);

    let mut u = U::default();
    let mut out = Vec::<u8>::new();
    let e = dispatch_to(&env, &cmd, &["--", "-x"], &mut u, &mut out).unwrap_err();
    match e {
        Error::UnexpectedArgument(s) => assert_eq!(s, "-x"),
        _ => panic!("expected UnexpectedArgument"),
    }
}

// Root help includes Commands section when subs exist
#[test]
fn help_includes_commands_when_subs_present() {
    let env = env_base();
    let sub = CmdSpec::new(Some("do"), None).desc("do things");
    let subs = [sub];
    let root = CmdSpec::new(None, None).subs(subs);

    let mut u = U::default();
    let mut out = Vec::<u8>::new();
    match dispatch_to(&env, &root, &["--help"], &mut u, &mut out) {
        Err(Error::Exit(0)) => {}
        other => panic!("{other:?}"),
    }
    let t = s(&out);
    assert!(t.contains("Commands"));
}

// Unknown short inside a cluster: "-vZ" → UnknownOption("-Z")
#[test]
fn unknown_short_inside_cluster() {
    let env = env_base();
    let opts = [OptSpec::new("verbose", cb_verbose).short('v').flag()];
    let cmd = CmdSpec::new(None, None).opts(opts);

    let mut u = U::default();
    let mut out = Vec::<u8>::new();
    let e = dispatch_to(&env, &cmd, &["-vZ"], &mut u, &mut out).unwrap_err();
    match e {
        Error::UnknownOption { token: s, .. } => assert_eq!(s, "-Z"),
        _ => panic!("expected UnknownOption"),
    }
}

// Cluster with flag then required value: "-voFILE"
#[test]
fn cluster_flag_then_required_value() {
    let env = env_base();
    let opts = [
        OptSpec::new("verbose", cb_verbose).short('v').flag(),
        OptSpec::new("output", cb_output).short('o').required().metavar("FILE"),
    ];
    let cmd = CmdSpec::new(None, Some(|_,_| Ok(()))).opts(opts);

    let mut u = U::default();
    let mut out = Vec::<u8>::new();
    let r = dispatch_to(&env, &cmd, &["-voFILE"], &mut u, &mut out);
    assert!(r.is_ok());
    assert_eq!(u.verbose, 1);
    assert_eq!(u.output.as_deref(), Some("FILE"));
}

// Repeated long flag occurrences increment counter
#[test]
fn repeated_long_flags_increment() {
    let env = env_base();
    let opts = [OptSpec::new("verbose", cb_verbose).flag().help("inc")];
    let cmd = CmdSpec::new(None, Some(|_,_| Ok(()))).opts(opts);

    let mut u = U::default();
    let mut out = Vec::<u8>::new();
    let r = dispatch_to(&env, &cmd, &["--verbose", "--verbose"], &mut u, &mut out);
    assert!(r.is_ok());
    assert_eq!(u.verbose, 2);
}

#[test]
fn help_uses_color_and_wraps_and_shows_env_default() {
    // colorized help + narrow wrap + env/default annotations in option help
    let mut env = env_base();
    env = env.color(true).wrap_cols(32);

    let opts = [OptSpec::new("speed", cb_jobs)
        .short('s')
        .optional()
        .numeric()
        .metavar("N")
        .help("very long description that must wrap across multiple lines for coverage")
        .env("RAP_SPEED")
        .default("42")];
    let cmd = CmdSpec::new(None, None).desc("demo").opts(opts);

    let mut u = U::default();
    let mut out = Vec::<u8>::new();
    let _ = dispatch_to(&env, &cmd, &["--help"], &mut u, &mut out);
    let t = s(&out);

    // Colorization adds ANSI escape sequences
    assert!(t.contains("\u{1b}["));
    // Allow wrapping/ANSI: just check both tokens appear somewhere
    assert!(t.contains("RAP_SPEED"));
    assert!(t.contains("default=42"));
    // Be agnostic to exact line breaks in wrapped prose
    assert!(t.contains("very"));
    assert!(t.contains("wrap"));
}

#[test]
fn non_ascii_short_flag_and_cluster() {
    // Ensure fallback lookup for non‑ASCII short works and plays well with clusters
    let mut env = env_base();
    env = env.color(false);

    let opts = [
        OptSpec::new("x", cb_verbose).short('Ж').flag().help("Cyrillic short"),
        OptSpec::new("v", cb_verbose).short('v').flag(),
    ];
    let cmd = CmdSpec::new(None, Some(|_,_| Ok(()))).opts(opts);

    let mut u1 = U::default();
    let mut out1 = Vec::<u8>::new();
    let r1 = dispatch_to(&env, &cmd, &["-Ж"], &mut u1, &mut out1);
    assert!(r1.is_ok());
    assert_eq!(u1.verbose, 1);

    // cluster with ASCII + non‑ASCII: -vЖ
    let mut u2 = U::default();
    let mut out2 = Vec::<u8>::new();
    let r2 = dispatch_to(&env, &cmd, &["-vЖ"], &mut u2, &mut out2);
    assert!(r2.is_ok());
    assert_eq!(u2.verbose, 2);
}

#[test]
fn help_positional_variants_desc_optional_required_minmax() {
    let mut env = env_base();
    env = env.wrap_cols(60);

    let pos = [
        PosSpec::new("OPT").range(0, 1),                       // (optional)
        PosSpec::new("REQ").one(),                             // (required)
        PosSpec::new("FILES").range(2, 4).desc("input files"), // uses desc
        PosSpec::new("LIMITS").range(2, 4),                    // min/max
    ];
    let cmd = CmdSpec::new(None, None).pos(pos);

    let mut u = U::default();
    let mut out = Vec::<u8>::new();
    let _ = dispatch_to(&env, &cmd, &["--help"], &mut u, &mut out);
    let t = s(&out);

    assert!(t.contains("OPT"));
    assert!(t.contains("(optional)"));
    assert!(t.contains("REQ"));
    assert!(t.contains("(required)"));
    assert!(t.contains("FILES"));
    assert!(t.contains("input files"));
    assert!(t.contains("LIMITS"));
    assert!(t.contains("min=2 max=4"));
}

#[test]
fn optional_numeric_accepts_scientific_and_dot_forms() {
    let env = env_base();
    let opts = [OptSpec::new("jobs", cb_jobs).short('j').optional().numeric()];
    let cmd = CmdSpec::new(None, Some(|_,_| Ok(()))).opts(opts);

    // long: 1e-3
    let mut u1 = U::default();
    let mut out1 = Vec::<u8>::new();
    let r1 = dispatch_to(&env, &cmd, &["--jobs", "1e-3"], &mut u1, &mut out1);
    assert!(r1.is_ok());
    assert_eq!(u1.jobs.as_deref(), Some("1e-3"));

    // long: 0.5 (use leading zero to satisfy stricter numeric checks)
    let mut u2 = U::default();
    let mut out2 = Vec::<u8>::new();
    let r2 = dispatch_to(&env, &cmd, &["--jobs", "0.5"], &mut u2, &mut out2);
    assert!(r2.is_ok());
    assert_eq!(u2.jobs.as_deref(), Some("0.5"));

    // short attached: -j5 (some parsers don't accept decimal *attached* to a short option)
    let mut u3 = U::default();
    let mut out3 = Vec::<u8>::new();
    let r3 = dispatch_to(&env, &cmd, &["-j0.5"], &mut u3, &mut out3);
    assert!(r3.is_ok());
    assert_eq!(u3.jobs.as_deref(), Some("0.5"));
}

#[test]
fn help_no_wrap_path() {
    // Exercise write_wrapped with wrap_cols == 0 fast path
    let mut env = env_base();
    env = env.wrap_cols(0);

    let opts = [OptSpec::new("verbose", cb_verbose)
        .short('v')
        .flag()
        .help("toggle verbose output for diagnostics")];
    let cmd = CmdSpec::new(None, None)
        .desc("this description should not wrap even if it is quite long")
        .opts(opts);

    let mut u = U::default();
    let mut out = Vec::<u8>::new();
    let _ = dispatch_to(&env, &cmd, &["--help"], &mut u, &mut out);
    let t = s(&out);
    // Just ensure the long sentence is present in one piece (no wrap enforced by us)
    assert!(t.contains("this description should not wrap"));
}
