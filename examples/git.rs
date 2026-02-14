use rust_args_parser as rapp;
use std::ffi::OsStr;
use std::path::PathBuf;

#[derive(Default, Debug)]
struct GitCtx {
    global: Global,
    exec: Option<GitExec>,
}

#[derive(Default, Debug)]
struct Global {
    chdirs: Vec<PathBuf>, // -C <path> (repeatable)
    configs: Vec<String>, // -c <name=val> (repeatable)
    no_pager: bool,       // --no-pager
}

#[derive(Debug)]
enum GitExec {
    Init(Init),
    Add(Add),
    Commit(Commit),
    Status(Status),
    Log(Log),
    Branch(Branch),
    Checkout(Checkout),
}

#[derive(Default, Debug)]
struct Init {
    bare: bool,
    template: Option<PathBuf>,
    dir: Option<PathBuf>,
}
#[derive(Default, Debug)]
struct Add {
    all: bool,
    patch: bool,
    dry_run: bool,
    paths: Vec<PathBuf>,
}
#[derive(Default, Debug)]
struct Commit {
    message: Vec<String>,
    all: bool,
    amend: bool,
}
#[derive(Default, Debug)]
struct Status {
    short: bool,
    branch: bool,
}
#[derive(Default, Debug)]
struct Log {
    oneline: bool,
    limit: Option<usize>,
    grep: Vec<String>,
    paths: Vec<PathBuf>,
}
#[derive(Default, Debug)]
struct Branch {
    list: bool,
    all: bool,
    delete: Option<String>,
}
#[derive(Default, Debug)]
struct Checkout {
    new_branch: Option<String>,
    branch: Option<String>,
}

fn ensure_init(c: &mut GitCtx) -> &mut Init {
    if !matches!(c.exec, Some(GitExec::Init(_))) {
        c.exec = Some(GitExec::Init(Init::default()));
    }
    match c.exec.as_mut().expect("just set") {
        GitExec::Init(x) => x,
        _ => unreachable!(),
    }
}
fn ensure_add(c: &mut GitCtx) -> &mut Add {
    if !matches!(c.exec, Some(GitExec::Add(_))) {
        c.exec = Some(GitExec::Add(Add::default()));
    }
    match c.exec.as_mut().expect("just set") {
        GitExec::Add(x) => x,
        _ => unreachable!(),
    }
}
fn ensure_commit(c: &mut GitCtx) -> &mut Commit {
    if !matches!(c.exec, Some(GitExec::Commit(_))) {
        c.exec = Some(GitExec::Commit(Commit::default()));
    }
    match c.exec.as_mut().expect("just set") {
        GitExec::Commit(x) => x,
        _ => unreachable!(),
    }
}
fn ensure_status(c: &mut GitCtx) -> &mut Status {
    if !matches!(c.exec, Some(GitExec::Status(_))) {
        c.exec = Some(GitExec::Status(Status::default()));
    }
    match c.exec.as_mut().expect("just set") {
        GitExec::Status(x) => x,
        _ => unreachable!(),
    }
}
fn ensure_log(c: &mut GitCtx) -> &mut Log {
    if !matches!(c.exec, Some(GitExec::Log(_))) {
        c.exec = Some(GitExec::Log(Log::default()));
    }
    match c.exec.as_mut().expect("just set") {
        GitExec::Log(x) => x,
        _ => unreachable!(),
    }
}
fn ensure_branch(c: &mut GitCtx) -> &mut Branch {
    if !matches!(c.exec, Some(GitExec::Branch(_))) {
        c.exec = Some(GitExec::Branch(Branch::default()));
    }
    match c.exec.as_mut().expect("just set") {
        GitExec::Branch(x) => x,
        _ => unreachable!(),
    }
}
fn ensure_checkout(c: &mut GitCtx) -> &mut Checkout {
    if !matches!(c.exec, Some(GitExec::Checkout(_))) {
        c.exec = Some(GitExec::Checkout(Checkout::default()));
    }
    match c.exec.as_mut().expect("just set") {
        GitExec::Checkout(x) => x,
        _ => unreachable!(),
    }
}

// ——— Global setters ———
fn push_chdir(v: &OsStr, c: &mut GitCtx) {
    c.global.chdirs.push(v.into());
}
fn push_config(v: &OsStr, c: &mut GitCtx) {
    c.global.configs.push(v.to_string_lossy().into_owned());
}
fn set_no_pager(c: &mut GitCtx) {
    c.global.no_pager = true;
}

fn config_is_name_val(v: &OsStr) -> Result<(), &'static str> {
    let s = v.to_string_lossy();
    if s.contains('=') {
        Ok(())
    } else {
        Err("expected name=value for -c")
    }
}

// ——— init ———
fn set_init_bare(c: &mut GitCtx) {
    ensure_init(c).bare = true;
}
fn set_init_template(v: &OsStr, c: &mut GitCtx) {
    ensure_init(c).template = Some(v.into());
}
fn set_init_dir(v: &OsStr, c: &mut GitCtx) {
    ensure_init(c).dir = Some(v.into());
}

// ——— add ———
fn set_add_all(c: &mut GitCtx) {
    ensure_add(c).all = true;
}
fn set_add_patch(c: &mut GitCtx) {
    ensure_add(c).patch = true;
}
fn set_add_dry(c: &mut GitCtx) {
    ensure_add(c).dry_run = true;
}
fn push_add_path(v: &OsStr, c: &mut GitCtx) {
    ensure_add(c).paths.push(v.into());
}

// ——— commit ———
fn push_commit_msg(v: &OsStr, c: &mut GitCtx) {
    ensure_commit(c).message.push(v.to_string_lossy().into_owned());
}
fn set_commit_all(c: &mut GitCtx) {
    ensure_commit(c).all = true;
}
fn set_commit_amend(c: &mut GitCtx) {
    ensure_commit(c).amend = true;
}

// ——— status ———
fn set_status_short(c: &mut GitCtx) {
    ensure_status(c).short = true;
}
fn set_status_branch(c: &mut GitCtx) {
    ensure_status(c).branch = true;
}

// ——— log ———
fn set_log_oneline(c: &mut GitCtx) {
    ensure_log(c).oneline = true;
}
fn set_log_limit(v: &OsStr, c: &mut GitCtx) {
    ensure_log(c).limit = Some(v.to_string_lossy().parse::<usize>().unwrap());
}
fn push_log_grep(v: &OsStr, c: &mut GitCtx) {
    ensure_log(c).grep.push(v.to_string_lossy().into_owned());
}
fn push_log_path(v: &OsStr, c: &mut GitCtx) {
    ensure_log(c).paths.push(v.into());
}

fn log_limit_is_usize(v: &OsStr) -> Result<(), &'static str> {
    v.to_string_lossy().parse::<usize>().map(|_| ()).map_err(|_| "-n expects a number")
}

// ——— branch ———
fn set_branch_list(c: &mut GitCtx) {
    ensure_branch(c).list = true;
}
fn set_branch_all(c: &mut GitCtx) {
    ensure_branch(c).all = true;
}
fn set_branch_delete(v: &OsStr, c: &mut GitCtx) {
    ensure_branch(c).delete = Some(v.to_string_lossy().into_owned());
}

// ——— checkout ———
fn set_checkout_new(v: &OsStr, c: &mut GitCtx) {
    ensure_checkout(c).new_branch = Some(v.to_string_lossy().into_owned());
}
fn set_checkout_branch(v: &OsStr, c: &mut GitCtx) {
    ensure_checkout(c).branch = Some(v.to_string_lossy().into_owned());
}

#[allow(clippy::too_many_lines)]
fn build_spec<'a>() -> rapp::CmdSpec<'a, GitCtx> {
    use rapp::{CmdSpec, GroupMode, OptSpec, PosSpec};

    // Root (global options)
    CmdSpec::<'a, GitCtx>::new("git")
        .help("tiny, fast, callback-based git-like demo")
        .opt(
            OptSpec::value("chdir", push_chdir)
                .short('C')
                .metavar("DIR")
                .repeatable()
                .help("run as if git was started in DIR"),
        )
        .opt(
            OptSpec::value("config", push_config)
                .short('c')
                .metavar("NAME=VAL")
                .repeatable()
                .help("set config on the command line")
                .validator(config_is_name_val),
        )
        .opt(
            OptSpec::flag("no_pager", set_no_pager)
                .long("no-pager")
                .help("do not pipe output into a pager"),
        )
        // --- git init ---
        .subcmd(
            CmdSpec::<'a, GitCtx>::new("init")
                .help("create an empty Git repository")
                .opt(OptSpec::flag("bare", set_init_bare).long("bare").help("create a bare repository"))
                .opt(OptSpec::value("template", set_init_template).long("template").metavar("DIR"))
                .pos(PosSpec::new("DIR", set_init_dir).help("directory").range(0, 1))
                .handler(|_, c| {
                    // If no options/positionals touched exec during callbacks, ensure it exists.
                    ensure_init(c);
                }),
        )
        // --- git add ---
        .subcmd(
            CmdSpec::<'a, GitCtx>::new("add")
                .help("add file contents to the index")
                .opt(OptSpec::flag("all", set_add_all).short('A').long("all").help("stage all tracked and untracked"))
                .opt(OptSpec::flag("patch", set_add_patch).short('p').long("patch").help("interactive hunk selection"))
                .opt(OptSpec::flag("dry_run", set_add_dry).short('n').long("dry-run").help("dry run"))
                .pos(PosSpec::new("PATHSPEC", push_add_path).many().help("paths to add"))
                .handler(|_, c| {
                    ensure_add(c);
                }),
        )
        // --- git commit ---
        .subcmd(
            CmdSpec::<'a, GitCtx>::new("commit")
                .help("record changes to the repository")
                .opt(OptSpec::value("message", push_commit_msg).short('m').long("message").metavar("MSG").repeatable())
                .opt(OptSpec::flag("all", set_commit_all).short('a').long("all"))
                .opt(OptSpec::flag("amend", set_commit_amend).long("amend"))
                .group("mode", GroupMode::Xor)
                .handler(|_, c| {
                    ensure_commit(c);
                })
                .validator(|m| {
                    // Example policy: require at least one -m (like scripting UX)
                    if !m.view().values("message").map_or(false, |v| !v.is_empty()) {
                        return Err("commit requires at least one -m MSG in this demo");
                    }
                    Ok(())
                }),
        )
        // --- git status ---
        .subcmd(
            CmdSpec::<'a, GitCtx>::new("status")
                .opt(OptSpec::flag("short", set_status_short).short('s').long("short"))
                .opt(OptSpec::flag("branch", set_status_branch).long("branch"))
                .handler(|_, c| {
                    ensure_status(c);
                }),
        )
        // --- git log ---
        .subcmd(
            CmdSpec::<'a, GitCtx>::new("log")
                .opt(OptSpec::flag("oneline", set_log_oneline).long("oneline"))
                .opt(OptSpec::value("limit", set_log_limit).short('n').metavar("N").validator(log_limit_is_usize))
                .opt(OptSpec::value("grep", push_log_grep).long("grep").metavar("PATTERN").repeatable())
                .pos(PosSpec::new("PATH", push_log_path).many())
                .handler(|_, c| {
                    ensure_log(c);
                }),
        )
        // --- git branch ---
        .subcmd(
            CmdSpec::<'a, GitCtx>::new("branch")
                .opt(OptSpec::flag("list", set_branch_list).long("list"))
                .opt(OptSpec::flag("all", set_branch_all).long("all"))
                .opt(OptSpec::value("delete", set_branch_delete).short('d').metavar("NAME"))
                .group("mode", GroupMode::Xor)
                .handler(|_, c| {
                    ensure_branch(c);
                }),
        )
        // --- git checkout ---
        .subcmd(
            CmdSpec::<'a, GitCtx>::new("checkout")
                .opt(OptSpec::value("new", set_checkout_new).short('b').metavar("NAME"))
                .pos(PosSpec::new("BRANCH", set_checkout_branch).range(0, 1))
                .handler(|_, c| {
                    ensure_checkout(c);
                })
                .validator(|m| {
                    let v = m.view();
                    let have_new = v.value("new").is_some();
                    let have_branch = v.value("BRANCH").is_some();
                    if !have_new && !have_branch {
                        return Err("checkout needs -b NAME or BRANCH");
                    }
                    Ok(())
                }),
        )
}

fn main() {
    let mut env = rapp::Env { version: Some("2.0.0"), author: Some("The Demo Team"), ..Default::default() };
    env.wrap_cols = 100;

    let argv: Vec<std::ffi::OsString> = std::env::args_os().skip(1).collect();
    let mut ctx = GitCtx::default();
    let root = build_spec();

    match rapp::parse(&env, &root, &argv, &mut ctx) {
        Err(rapp::Error::ExitMsg { code, message }) => {
            if let Some(msg) = message {
                println!("{msg}");
            }
            std::process::exit(code);
        }
        Err(e) => {
            eprintln!("error: {e}");
            std::process::exit(2);
        }
        Ok(matches) => {
            // Dispatch according to what callbacks/leaf handler set in ctx
            println!("ctx = {ctx:?}");
            // You could also inspect `matches.view()` for leaf-scoped values
            if matches.is_set_from("no_pager", rapp::Source::Cli) {
                eprintln!("--no-pager came from CLI");
            }
        }
    }
}
