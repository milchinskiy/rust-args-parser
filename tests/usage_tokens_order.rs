use rust_args_parser as ap;

#[derive(Default)]
struct Ctx;

fn usage_first_line(env: &ap::Env, cmd: &ap::CmdSpec<Ctx>, path: &[&str]) -> String {
    let mut out: Vec<u8> = Vec::new();
    ap::print_help_to(env, cmd, path, &mut out);
    let s = String::from_utf8(out).unwrap();
    s.lines().next().unwrap_or("").trim_end().to_string()
}

#[test]
fn usage_opts_and_subs_on_root() {
    let env = ap::Env::new("demo").color(false).auto_help(true);
    let cmd = ap::CmdSpec::new(None, None)
        .desc("root")
        .opts(vec![
            ap::OptSpec::new("verbose", |_v: Option<&str>, _ctx: &mut Ctx| Ok(()))
                .short('v')
                .flag()
                .help("Verbose"),
        ])
        .subs(vec![
            ap::CmdSpec::new(Some("build"), Some(|_p: &[&str], _c: &mut Ctx| Ok(())))
                .desc("Build"),
        ]);

    let line = usage_first_line(&env, &cmd, &[]);
    assert!(line.contains("Usage: demo [options] <command>"), "{line}");
}

#[test]
fn usage_only_subs_on_root_no_help() {
    let env = ap::Env::new("demo").color(false).auto_help(false);
    let cmd = ap::CmdSpec::new(None, None)
        .desc("root")
        .subs(vec![
            ap::CmdSpec::new(Some("shell"), Some(|_p: &[&str], _c: &mut Ctx| Ok(())))
                .desc("Shell"),
        ]);

    let line = usage_first_line(&env, &cmd, &[]);
    assert!(line.contains("Usage: demo <command>"), "{line}");
    assert!(!line.contains("[options]"), "{line}");
}

#[test]
fn usage_only_opts_on_root() {
    let env = ap::Env::new("demo").color(false).auto_help(true);
    let cmd = ap::CmdSpec::new(None, None)
        .desc("root")
        .opts(vec![
            ap::OptSpec::new("json", |_v: Option<&str>, _c: &mut Ctx| Ok(()))
                .short('j')
                .flag()
                .help("JSON"),
        ]);

    let line = usage_first_line(&env, &cmd, &[]);
    assert!(line.contains("Usage: demo [options]"), "{line}");
    assert!(!line.contains("<command>"), "{line}");
}

#[test]
fn usage_subcommand_has_help_and_subs() {
    // root: `sc` with subcommand `remote` which itself has subcommands (e.g., `add`).
    // `remote` has no explicit user options, but auto-help is enabled → should count as options.
    let env = ap::Env::new("sc").color(false).auto_help(true);

    let remote = ap::CmdSpec::new(Some("remote"), None)
        .desc("Remote management")
        .subs(vec![
            ap::CmdSpec::new(Some("add"), Some(|_p: &[&str], _c: &mut Ctx| Ok(())))
                .desc("Add a remote"),
        ]);

    let line = usage_first_line(&env, &remote, &["remote"]);
    assert!(line.contains("Usage: sc remote [options] <command>"), "{line}");
}
