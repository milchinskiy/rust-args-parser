mod common;
use common::*;

#[test]
fn flags_and_required_values_long_short() {
    let env = env_base();
    let opts = [
        OptSpec::new("verbose", cb_verbose).short('v').flag().help("increase verbosity"),
        OptSpec::new("output", cb_output).short('o').required().metavar("FILE").help("output file"),
        OptSpec::new("jobs", cb_jobs)
            .short('j')
            .optional()
            .numeric()
            .metavar("N")
            .help("parallel jobs"),
    ];
    let root = CmdSpec::new(None, None).desc("root").opts(opts);

    let mut u = U::default();
    let mut out = Vec::<u8>::new();
    let argv = ["--verbose", "-o", "out.txt", "-j10"];
    let res = dispatch_to(&env, &root, &argv, &mut u, &mut out);
    assert!(res.is_ok());
    assert_eq!(u.verbose, 1);
    assert_eq!(u.output.as_deref(), Some("out.txt"));
    assert_eq!(u.jobs.as_deref(), Some("10"));
    assert!(s(&out).is_empty());
}

#[test]
fn long_required_eq_and_space_and_missing() {
    let env = env_base();
    let opts = [OptSpec::new("output", cb_output)
        .short('o')
        .required()
        .metavar("FILE")
        .help("output file")];
    let root = CmdSpec::new(None, None).opts(opts);

    let mut u1 = U::default();
    let mut out1 = Vec::<u8>::new();
    let res1 = dispatch_to(&env, &root, &["--output=bin"], &mut u1, &mut out1);
    assert!(res1.is_ok());
    assert_eq!(u1.output.as_deref(), Some("bin"));

    let mut u2 = U::default();
    let mut out2 = Vec::<u8>::new();
    let res2 = dispatch_to(&env, &root, &["--output", "bin"], &mut u2, &mut out2);
    assert!(res2.is_ok());
    assert_eq!(u2.output.as_deref(), Some("bin"));

    let mut u3 = U::default();
    let mut out3 = Vec::<u8>::new();
    let res3 = dispatch_to(&env, &root, &["--output="], &mut u3, &mut out3);
    match res3 {
        Err(Error::MissingValue(n)) => assert_eq!(n, "output"),
        _ => panic!("expected MissingValue"),
    }
}

#[test]
fn short_required_attached_and_space() {
    let env = env_base();
    let opts = [OptSpec::new("output", cb_output).short('o').required().metavar("FILE")];
    let root = CmdSpec::new(None, None).opts(opts);

    let mut u1 = U::default();
    let mut out1 = Vec::<u8>::new();
    let res1 = dispatch_to(&env, &root, &["-oFILE"], &mut u1, &mut out1);
    assert!(res1.is_ok());
    assert_eq!(u1.output.as_deref(), Some("FILE"));

    let mut u2 = U::default();
    let mut out2 = Vec::<u8>::new();
    let res2 = dispatch_to(&env, &root, &["-o", "FILE"], &mut u2, &mut out2);
    assert!(res2.is_ok());
    assert_eq!(u2.output.as_deref(), Some("FILE"));
}

#[test]
fn optional_values_detection_and_negative_numbers() {
    let env = env_base();
    let opts = [OptSpec::new("jobs", cb_jobs).short('j').optional().numeric().metavar("N")];
    let root = CmdSpec::new(None, None).opts(opts);

    // --jobs -1 (long optional picks numeric-looking token)
    let mut u1 = U::default();
    let mut out1 = Vec::<u8>::new();
    let res1 = dispatch_to(&env, &root, &["--jobs", "-1"], &mut u1, &mut out1);
    assert!(res1.is_ok());
    assert_eq!(u1.jobs.as_deref(), Some("-1"));

    // -j -12 (short optional accepts dash-number because of .numeric())
    let mut u2 = U::default();
    let mut out2 = Vec::<u8>::new();
    let res2 = dispatch_to(&env, &root, &["-j", "-12"], &mut u2, &mut out2);
    assert!(res2.is_ok());
    assert_eq!(u2.jobs.as_deref(), Some("-12"));

    // -j (no value)
    let mut u3 = U::default();
    let mut out3 = Vec::<u8>::new();
    let res3 = dispatch_to(&env, &root, &["-j"], &mut u3, &mut out3);
    assert!(res3.is_ok());
    assert_eq!(u3.jobs, None);

    // --jobs - (literal "-") not taken as value
    let mut u4 = U::default();
    let mut out4 = Vec::<u8>::new();
    let res4 = dispatch_to(&env, &root, &["--jobs", "-"], &mut u4, &mut out4);
    assert!(res4.is_ok());
    assert_eq!(u4.jobs, None);
}

#[test]
fn clustered_shorts_mixed() {
    let env = env_base();
    let opts = [
        OptSpec::new("verbose", cb_verbose).short('v').flag(),
        OptSpec::new("jobs", cb_jobs).short('j').optional().numeric(),
    ];
    let root = CmdSpec::new(None, None).opts(opts);

    let mut u = U::default();
    let mut out = Vec::<u8>::new();
    let res = dispatch_to(&env, &root, &["-vj10"], &mut u, &mut out);
    assert!(res.is_ok());
    assert_eq!(u.verbose, 1);
    assert_eq!(u.jobs.as_deref(), Some("10"));
}

#[test]
fn stop_parsing_with_double_dash() {
    let env = env_base();
    let opts = [OptSpec::new("verbose", cb_verbose).short('v').flag()];
    let pos = [PosSpec::new("ARG").range(0, usize::MAX)];
    let root = CmdSpec::new(None, Some(run_cb)).opts(&opts).pos(pos);

    let mut u = U::default();
    let mut out = Vec::<u8>::new();
    let res = dispatch_to(&env, &root, &["-v", "--", "-x", "--foo"], &mut u, &mut out);
    assert!(res.is_ok());
    assert_eq!(u.verbose, 1);
    assert_eq!(u.received, vec!["-x", "--foo"]);
}

#[test]
fn unknown_options_long_and_short() {
    let env = env_base();
    let opts = [OptSpec::new("verbose", cb_verbose).short('v').flag()];
    let root = CmdSpec::new(None, None).opts(opts);

    let mut u1 = U::default();
    let mut out1 = Vec::<u8>::new();
    let e1 = dispatch_to(&env, &root, &["--nope"], &mut u1, &mut out1).unwrap_err();
    match e1 {
        Error::UnknownOption { token: s, .. } => assert_eq!(s, "--nope"),
        _ => panic!("wrong error"),
    }

    let mut u2 = U::default();
    let mut out2 = Vec::<u8>::new();
    let e2 = dispatch_to(&env, &root, &["-1"], &mut u2, &mut out2).unwrap_err();
    match e2 {
        Error::UnknownOption { token: s, .. } => assert_eq!(s, "-1"),
        _ => panic!("wrong error"),
    }
}

#[test]
fn groups_xor_and_required_one() {
    let env = env_base();
    let opts = [
        OptSpec::new("a", cb_mode_a).at_most_one(1),
        OptSpec::new("b", cb_mode_b).at_most_one(1),
        OptSpec::new("x", cb_one_a).at_least_one(2),
        OptSpec::new("y", cb_one_b).at_least_one(2),
    ];
    let cmd = CmdSpec::new(None, None).opts(&opts);

    let mut u1 = U::default();
    let mut out1 = Vec::<u8>::new();
    let e1 = dispatch_to(&env, &cmd, &["--a", "--b"], &mut u1, &mut out1).unwrap_err();
    match e1 {
        Error::GroupViolation(msg) => assert!(msg.contains("at most one")),
        _ => panic!("expected GroupViolation"),
    }

    let mut u2 = U::default();
    let mut out2 = Vec::<u8>::new();
    let e2 = dispatch_to(&env, &cmd, &[], &mut u2, &mut out2).unwrap_err();
    match e2 {
        Error::GroupViolation(msg) => assert!(msg.contains("required")),
        _ => panic!("expected GroupViolation"),
    }

    let mut u3 = U::default();
    let mut out3 = Vec::<u8>::new();
    let r3 = dispatch_to(&env, &cmd, &["--x"], &mut u3, &mut out3);
    assert!(r3.is_ok());
}

#[test]
fn env_and_default_applied_and_counted_in_groups() {
    let env = env_base();
    let key = "RAP_ENVD";
    std::env::remove_var(key);
    let opts = [
        OptSpec::new("envd", cb_envd).env(key).at_most_one(1),
        OptSpec::new("def", cb_defv).default("D").at_most_one(1),
    ];
    let cmd = CmdSpec::new(None, None).opts(opts);

    let mut u1 = U::default();
    let mut out1 = Vec::<u8>::new();
    let r1 = dispatch_to(&env, &cmd, &[], &mut u1, &mut out1);
    assert!(r1.is_ok());
    assert_eq!(u1.defv.as_deref(), Some("D"));
    assert_eq!(u1.envd, None);

    std::env::set_var(key, "E");
    let mut u2 = U::default();
    let mut out2 = Vec::<u8>::new();
    let r2 = dispatch_to(&env, &cmd, &[], &mut u2, &mut out2);
    assert!(r2.is_ok());
    assert_eq!(u2.envd.as_deref(), Some("E"));

    let mut u3 = U::default();
    let mut out3 = Vec::<u8>::new();
    let e3 = dispatch_to(&env, &cmd, &["--def"], &mut u3, &mut out3).unwrap_err();
    match e3 {
        Error::GroupViolation(msg) => assert!(msg.contains("one of the")),
        _ => panic!("expected GroupViolation"),
    }

    std::env::remove_var(key);
}

#[test]
fn user_error_bubbles_up() {
    let env = env_base();
    let cmd = CmdSpec::new(None, None).opts([OptSpec::new("boom", cb_user_err).flag()]);

    let mut u = U::default();
    let mut out = Vec::<u8>::new();
    match dispatch_to(&env, &cmd, &["--boom"], &mut u, &mut out) {
        Err(Error::Callback(e)) => {
            if let Some(ee) = e.downcast_ref::<AppError>() {
                match ee {
                    AppError::User(msg) => assert_eq!(*msg, "bad"),
                }
            }
        }
        other => panic!("unexpected: {other:?}"),
    }
}
