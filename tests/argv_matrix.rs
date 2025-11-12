use rust_args_parser::{dispatch, CmdSpec, Env, OptSpec, PosSpec};

#[derive(Default, Debug)]
#[allow(clippy::struct_excessive_bools)]
struct Ctx {
    root_flag: bool,
    v_count: usize,
    sub_flag: bool,
    quiet: bool,
    ran: Vec<&'static str>,
    pos_rec: Vec<String>,
    s_flag: bool,
    q_flag: bool,
}

fn make_spec() -> CmdSpec<'static, Ctx> {
    let subsub = CmdSpec::new(
        Some("subsub"),
        Some(|pos, c: &mut Ctx| {
            c.ran.push("sub/subsub");
            c.pos_rec.extend(pos.iter().map(std::string::ToString::to_string));
            Ok(())
        }),
    )
    .opts(vec![OptSpec::new("s", |_, c: &mut Ctx| {
        c.quiet = true;
        Ok(())
    })
    .short('s')
    .flag()]);

    let sub = CmdSpec::new(
        Some("sub"),
        Some(|pos, c: &mut Ctx| {
            c.ran.push("sub");
            c.pos_rec.extend(pos.iter().map(std::string::ToString::to_string));
            Ok(())
        }),
    )
    .opts(vec![OptSpec::new("sub-flag", |_, c: &mut Ctx| {
        c.sub_flag = true;
        Ok(())
    })
    .flag()])
    .subs(vec![subsub]);

    CmdSpec::new(None, None)
        .opts(vec![
            OptSpec::new("root", |_, c: &mut Ctx| {
                c.root_flag = true;
                Ok(())
            })
            .flag(),
            OptSpec::new("v", |_, c: &mut Ctx| {
                c.v_count += 1;
                Ok(())
            })
            .short('v')
            .flag(),
        ])
        .subs(vec![sub])
}

#[test]
fn root_long_before_sub_descends() {
    let env = Env::new("mybin");
    let mut ctx = Ctx::default();
    let root = make_spec();

    dispatch(&env, &root, &["--root", "sub"], &mut ctx).unwrap();
    assert!(ctx.root_flag);
    assert_eq!(ctx.ran, vec!["sub"]);
}

#[test]
fn root_short_cluster_then_sub() {
    let env = Env::new("mybin");
    let mut ctx = Ctx::default();
    let root = make_spec();

    dispatch(&env, &root, &["-vvv", "sub"], &mut ctx).unwrap();
    assert_eq!(ctx.v_count, 3);
    assert_eq!(ctx.ran, vec!["sub"]);
}

#[test]
fn sub_flag_then_subsub_then_short_ok() {
    let env = Env::new("mybin");
    let mut ctx = Ctx::default();
    let root = make_spec();

    dispatch(&env, &root, &["sub", "--sub-flag", "subsub", "-s"], &mut ctx).unwrap();
    assert!(ctx.sub_flag);
    assert!(ctx.quiet);
    assert_eq!(ctx.ran, vec!["sub/subsub"]);
}

#[test]
fn root_flag_then_full_chain() {
    let env = Env::new("mybin");
    let mut ctx = Ctx::default();
    let root = make_spec();

    dispatch(&env, &root, &["--root", "sub", "--sub-flag", "subsub", "-s"], &mut ctx).unwrap();
    assert!(ctx.root_flag && ctx.sub_flag && ctx.quiet);
    assert_eq!(ctx.ran, vec!["sub/subsub"]);
}

#[test]
fn ddash_at_root_blocks_descent_and_keeps_positional() {
    let env = Env::new("mybin");
    let mut ctx = Ctx::default();

    let sub = CmdSpec::new(
        Some("sub"),
        Some(|_pos, c: &mut Ctx| {
            c.ran.push("sub");
            Ok(())
        }),
    );

    let root = CmdSpec::new(
        None,
        Some(|pos, c: &mut Ctx| {
            c.ran.push("root");
            c.pos_rec.extend(pos.iter().map(std::string::ToString::to_string));
            Ok(())
        }),
    )
    .pos(vec![PosSpec::new("arg").one()])
    .subs(vec![sub]);

    dispatch(&env, &root, &["--", "sub"], &mut ctx).unwrap();
    assert_eq!(ctx.ran, vec!["root"]);
    assert_eq!(ctx.pos_rec, vec!["sub".to_string()]);
}

#[test]
fn ddash_inside_sub_blocks_subsub_descent() {
    let env = Env::new("mybin");
    let mut ctx = Ctx::default();

    let sub_with_pos = CmdSpec::new(
        Some("sub"),
        Some(|pos, c: &mut Ctx| {
            c.ran.push("sub");
            c.pos_rec.extend(pos.iter().map(std::string::ToString::to_string));
            Ok(())
        }),
    )
    .pos(vec![PosSpec::new("x").one()])
    .subs(vec![CmdSpec::new(Some("subsub"), Some(|_, _| Ok(())))]);

    let root = CmdSpec::new(None, None).subs(vec![sub_with_pos]);
    dispatch(&env, &root, &["sub", "--", "subsub"], &mut ctx).unwrap();
    assert_eq!(ctx.ran, vec!["sub"]);
    assert_eq!(ctx.pos_rec, vec!["subsub".to_string()]);
}

#[test]
fn mixed_root_positional_prevents_descent() {
    let env = Env::new("mybin");
    let mut ctx = Ctx::default();

    let root = CmdSpec::new(
        None,
        Some(|pos, c: &mut Ctx| {
            c.ran.push("root");
            c.pos_rec.extend(pos.iter().map(std::string::ToString::to_string));
            Ok(())
        }),
    )
    .pos(vec![PosSpec::new("file").one()])
    .subs(vec![CmdSpec::new(Some("sub"), Some(|_, _| Ok(())))]);
    let res = dispatch(&env, &root, &["FILE", "sub"], &mut ctx);
    assert!(res.is_err());
}

#[test]
fn unknown_root_option_is_error() {
    let env = Env::new("mybin");
    let mut ctx = Ctx::default();
    let root = make_spec();

    let res = dispatch(&env, &root, &["--nope"], &mut ctx);
    assert!(res.is_err());
}

#[test]
fn unknown_sub_option_is_error() {
    let env = Env::new("mybin");
    let mut ctx = Ctx::default();
    let root = make_spec();

    let res = dispatch(&env, &root, &["sub", "--nope"], &mut ctx);
    assert!(res.is_err());
}

fn spec_base() -> CmdSpec<'static, Ctx> {
    let subsub = CmdSpec::new(
        Some("subsub"),
        Some(|pos, c: &mut Ctx| {
            c.ran.push("sub/subsub");
            c.pos_rec.extend(pos.iter().map(std::string::ToString::to_string));
            Ok(())
        }),
    )
    .opts(vec![
        OptSpec::new("s", |_, c: &mut Ctx| {
            c.s_flag = true;
            Ok(())
        })
        .short('s')
        .flag(),
        OptSpec::new("q", |_, c: &mut Ctx| {
            c.q_flag = true;
            Ok(())
        })
        .short('q')
        .flag(),
    ]);

    let sub = CmdSpec::new(
        Some("sub"),
        Some(|pos, c: &mut Ctx| {
            c.ran.push("sub");
            c.pos_rec.extend(pos.iter().map(std::string::ToString::to_string));
            Ok(())
        }),
    )
    .opts(vec![OptSpec::new("sub-flag", |_, c: &mut Ctx| {
        c.sub_flag = true;
        Ok(())
    })
    .flag()])
    .subs(vec![subsub]);

    CmdSpec::new(None, None)
        .opts(vec![
            OptSpec::new("root", |_, c: &mut Ctx| {
                c.root_flag = true;
                Ok(())
            })
            .flag(),
            OptSpec::new("v", |_, c: &mut Ctx| {
                c.v_count += 1;
                Ok(())
            })
            .short('v')
            .flag(),
        ])
        .subs(vec![sub])
}

fn spec_root_with_one_pos() -> CmdSpec<'static, Ctx> {
    let r = CmdSpec::new(
        None,
        Some(|pos, c: &mut Ctx| {
            c.ran.push("root");
            c.pos_rec.extend(pos.iter().map(std::string::ToString::to_string));
            Ok(())
        }),
    )
    .opts(vec![
        OptSpec::new("root", |_, c: &mut Ctx| {
            c.root_flag = true;
            Ok(())
        })
        .flag(),
        OptSpec::new("v", |_, c: &mut Ctx| {
            c.v_count += 1;
            Ok(())
        })
        .short('v')
        .flag(),
    ])
    .pos(vec![PosSpec::new("arg").one()]);
    r
}

fn spec_sub_with_one_pos() -> CmdSpec<'static, Ctx> {
    // sub accepts exactly one positional, keeps subsub as before
    let subsub = CmdSpec::new(
        Some("subsub"),
        Some(|pos, c: &mut Ctx| {
            c.ran.push("sub/subsub");
            c.pos_rec.extend(pos.iter().map(std::string::ToString::to_string));
            Ok(())
        }),
    );

    let sub_with_pos = CmdSpec::new(
        Some("sub"),
        Some(|pos, c: &mut Ctx| {
            c.ran.push("sub");
            c.pos_rec.extend(pos.iter().map(std::string::ToString::to_string));
            Ok(())
        }),
    )
    .pos(vec![PosSpec::new("x").one()])
    .subs(vec![subsub]);

    CmdSpec::new(None, None).subs(vec![sub_with_pos])
}

fn spec_subsub_with_one_pos() -> CmdSpec<'static, Ctx> {
    // subsub accepts exactly one positional; useful to test `--` at that depth
    let subsub_with_pos = CmdSpec::new(
        Some("subsub"),
        Some(|pos, c: &mut Ctx| {
            c.ran.push("sub/subsub");
            c.pos_rec.extend(pos.iter().map(std::string::ToString::to_string));
            Ok(())
        }),
    )
    .pos(vec![PosSpec::new("y").one()])
    .opts(vec![
        OptSpec::new("s", |_, c: &mut Ctx| {
            c.s_flag = true;
            Ok(())
        })
        .short('s')
        .flag(),
        OptSpec::new("q", |_, c: &mut Ctx| {
            c.q_flag = true;
            Ok(())
        })
        .short('q')
        .flag(),
    ]);

    let sub =
        CmdSpec::new(Some("sub"), Some(|_pos, _c: &mut Ctx| Ok(()))).subs(vec![subsub_with_pos]);

    CmdSpec::new(None, None).subs(vec![sub])
}

#[test]
fn complex_chain_with_clusters_and_leaf_flags() {
    let env = Env::new("mybin");
    let mut ctx = Ctx::default();
    let root = spec_base();

    // root flags (cluster), then descend, then leaf flags (cluster)
    dispatch(&env, &root, &["-vvv", "--root", "sub", "subsub", "-sq"], &mut ctx).unwrap();

    assert_eq!(ctx.v_count, 3);
    assert!(ctx.root_flag);
    assert!(ctx.s_flag && ctx.q_flag);
    assert_eq!(ctx.ran, vec!["sub/subsub"]);
}

#[test]
fn ddash_at_root_makes_everything_positional_and_errors_due_to_extra() {
    // root only accepts a single positional, so trailing tokens cause error
    let env = Env::new("mybin");
    let mut ctx = Ctx::default();
    let root = spec_root_with_one_pos();

    let res = dispatch(&env, &root, &["--", "--root", "sub", "subsub", "-sq"], &mut ctx);
    assert!(res.is_err());
}

#[test]
fn ddash_inside_sub_turns_subsub_into_positional() {
    let env = Env::new("mybin");
    let mut ctx = Ctx::default();
    let root = spec_sub_with_one_pos();

    dispatch(&env, &root, &["sub", "--", "subsub"], &mut ctx).unwrap();
    assert_eq!(ctx.ran, vec!["sub"]);
    assert_eq!(ctx.pos_rec, vec!["subsub".to_string()]);
}

#[test]
fn ddash_inside_subsub_treats_flags_as_positional() {
    let env = Env::new("mybin");
    let mut ctx = Ctx::default();
    let root = spec_subsub_with_one_pos();

    dispatch(&env, &root, &["sub", "subsub", "--", "-s"], &mut ctx).unwrap();

    // leaf ran and captured "-s" as positional, not as an option
    assert_eq!(ctx.ran, vec!["sub/subsub"]);
    assert_eq!(ctx.pos_rec, vec!["-s".to_string()]);
    assert!(!ctx.s_flag);
}

#[test]
fn repeated_root_flags_then_deep_leaf() {
    let env = Env::new("mybin");
    let mut ctx = Ctx::default();
    let root = spec_base();

    dispatch(&env, &root, &["-vvvvv", "sub", "subsub"], &mut ctx).unwrap();
    assert_eq!(ctx.v_count, 5);
    assert_eq!(ctx.ran, vec!["sub/subsub"]);
}

#[test]
fn unknown_leaf_flag_before_leaf_token_errors() {
    // Passing a leaf-only flag (-s) while still at `sub` should error
    let env = Env::new("mybin");
    let mut ctx = Ctx::default();
    let root = spec_base();

    let res = dispatch(&env, &root, &["sub", "-s", "subsub"], &mut ctx);
    assert!(res.is_err());
}

#[test]
fn ddash_at_sub_makes_unknown_look_like_positional_ok() {
    // After `--` at sub, an unknown option-looking token must be positional
    let env = Env::new("mybin");
    let mut ctx = Ctx::default();

    let root = {
        let sub = CmdSpec::new(
            Some("sub"),
            Some(|pos, c: &mut Ctx| {
                c.ran.push("sub");
                c.pos_rec.extend(pos.iter().map(std::string::ToString::to_string));
                Ok(())
            }),
        )
        .pos(vec![PosSpec::new("x").one()]);
        CmdSpec::new(None, None).subs(vec![sub])
    };

    dispatch(&env, &root, &["sub", "--", "--no-such"], &mut ctx).unwrap();

    assert_eq!(ctx.ran, vec!["sub"]);
    assert_eq!(ctx.pos_rec, vec!["--no-such".to_string()]);
}
