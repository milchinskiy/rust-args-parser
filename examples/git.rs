#![allow(clippy::unnecessary_wraps)]

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

// ——— Global setters ———
fn push_chdir(v: &OsStr, c: &mut GitCtx) -> rapp::Result<()> {
    c.global.chdirs.push(v.into());
    Ok(())
}
fn push_config(v: &OsStr, c: &mut GitCtx) -> rapp::Result<()> {
    let s = v.to_string_lossy();
    if !s.contains('=') {
        return Err(rapp::Error::User("expected name=value for -c"));
    }
    c.global.configs.push(s.into_owned());
    Ok(())
}
fn set_no_pager(c: &mut GitCtx) -> rapp::Result<()> {
    c.global.no_pager = true;
    Ok(())
}

// ——— init ———
fn set_init_bare(c: &mut GitCtx) -> rapp::Result<()> {
    if let Some(GitExec::Init(ref mut x)) = c.exec {
        x.bare = true;
    }
    Ok(())
}
fn set_init_template(v: &OsStr, c: &mut GitCtx) -> rapp::Result<()> {
    if let Some(GitExec::Init(ref mut x)) = c.exec {
        x.template = Some(v.into());
    }
    Ok(())
}
fn set_init_dir(v: &OsStr, c: &mut GitCtx) -> rapp::Result<()> {
    if let Some(GitExec::Init(ref mut x)) = c.exec {
        x.dir = Some(v.into());
    }
    Ok(())
}

// ——— add ———
fn set_add_all(c: &mut GitCtx) -> rapp::Result<()> {
    if let Some(GitExec::Add(ref mut x)) = c.exec {
        x.all = true;
    }
    Ok(())
}
fn set_add_patch(c: &mut GitCtx) -> rapp::Result<()> {
    if let Some(GitExec::Add(ref mut x)) = c.exec {
        x.patch = true;
    }
    Ok(())
}
fn set_add_dry(c: &mut GitCtx) -> rapp::Result<()> {
    if let Some(GitExec::Add(ref mut x)) = c.exec {
        x.dry_run = true;
    }
    Ok(())
}
fn push_add_path(v: &OsStr, c: &mut GitCtx) -> rapp::Result<()> {
    if let Some(GitExec::Add(ref mut x)) = c.exec {
        x.paths.push(v.into());
    }
    Ok(())
}

// ——— commit ———
fn push_commit_msg(v: &OsStr, c: &mut GitCtx) -> rapp::Result<()> {
    if let Some(GitExec::Commit(ref mut x)) = c.exec {
        x.message.push(v.to_string_lossy().into());
    }
    Ok(())
}
fn set_commit_all(c: &mut GitCtx) -> rapp::Result<()> {
    if let Some(GitExec::Commit(ref mut x)) = c.exec {
        x.all = true;
    }
    Ok(())
}
fn set_commit_amend(c: &mut GitCtx) -> rapp::Result<()> {
    if let Some(GitExec::Commit(ref mut x)) = c.exec {
        x.amend = true;
    }
    Ok(())
}

// ——— status ———
fn set_status_short(c: &mut GitCtx) -> rapp::Result<()> {
    if let Some(GitExec::Status(ref mut x)) = c.exec {
        x.short = true;
    }
    Ok(())
}
fn set_status_branch(c: &mut GitCtx) -> rapp::Result<()> {
    if let Some(GitExec::Status(ref mut x)) = c.exec {
        x.branch = true;
    }
    Ok(())
}

// ——— log ———
fn set_log_oneline(c: &mut GitCtx) -> rapp::Result<()> {
    if let Some(GitExec::Log(ref mut x)) = c.exec {
        x.oneline = true;
    }
    Ok(())
}
fn set_log_limit(v: &OsStr, c: &mut GitCtx) -> rapp::Result<()> {
    let n: usize =
        v.to_string_lossy().parse().map_err(|_| rapp::Error::User("-n expects a number"))?;
    if let Some(GitExec::Log(ref mut x)) = c.exec {
        x.limit = Some(n);
    }
    Ok(())
}
fn push_log_grep(v: &OsStr, c: &mut GitCtx) -> rapp::Result<()> {
    if let Some(GitExec::Log(ref mut x)) = c.exec {
        x.grep.push(v.to_string_lossy().into());
    }
    Ok(())
}
fn push_log_path(v: &OsStr, c: &mut GitCtx) -> rapp::Result<()> {
    if let Some(GitExec::Log(ref mut x)) = c.exec {
        x.paths.push(v.into());
    }
    Ok(())
}

// ——— branch ———
fn set_branch_list(c: &mut GitCtx) -> rapp::Result<()> {
    if let Some(GitExec::Branch(ref mut x)) = c.exec {
        x.list = true;
    }
    Ok(())
}
fn set_branch_all(c: &mut GitCtx) -> rapp::Result<()> {
    if let Some(GitExec::Branch(ref mut x)) = c.exec {
        x.all = true;
    }
    Ok(())
}
fn set_branch_delete(v: &OsStr, c: &mut GitCtx) -> rapp::Result<()> {
    if let Some(GitExec::Branch(ref mut x)) = c.exec {
        x.delete = Some(v.to_string_lossy().into());
    }
    Ok(())
}

// ——— checkout ———
fn set_checkout_new(v: &OsStr, c: &mut GitCtx) -> rapp::Result<()> {
    if let Some(GitExec::Checkout(ref mut x)) = c.exec {
        x.new_branch = Some(v.to_string_lossy().into());
    }
    Ok(())
}
fn set_checkout_branch(v: &OsStr, c: &mut GitCtx) -> rapp::Result<()> {
    if let Some(GitExec::Checkout(ref mut x)) = c.exec {
        x.branch = Some(v.to_string_lossy().into());
    }
    Ok(())
}

#[allow(clippy::too_many_lines)]
fn build_spec<'a>() -> rapp::CmdSpec<'a, GitCtx> {
    use rapp::{CmdSpec, GroupMode, OptSpec, PosSpec};

    // Root (global options)
    let root = CmdSpec::<'a, GitCtx>::new("git")
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
                .help("set config on the command line"),
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
                .opt(
                    OptSpec::flag("bare", set_init_bare)
                        .long("bare")
                        .help("create a bare repository"),
                )
                .opt(OptSpec::value("template", set_init_template).long("template").metavar("DIR"))
                .pos(PosSpec::new("DIR", set_init_dir).help("directory").range(0, 1))
                .handler(|_, c| {
                    c.exec = Some(GitExec::Init(Init::default()));
                    Ok(())
                })
                .validator(|_| Ok(())),
        )
        // --- git add ---
        .subcmd(
            CmdSpec::<'a, GitCtx>::new("add")
                .help("add file contents to the index")
                .opt(
                    OptSpec::flag("all", set_add_all)
                        .short('A')
                        .long("all")
                        .help("stage all tracked and untracked"),
                )
                .opt(
                    OptSpec::flag("patch", set_add_patch)
                        .short('p')
                        .long("patch")
                        .help("interactive hunk selection"),
                )
                .opt(
                    OptSpec::flag("dry_run", set_add_dry)
                        .short('n')
                        .long("dry-run")
                        .help("dry run"),
                )
                .pos(PosSpec::new("PATHSPEC", push_add_path).many().help("paths to add"))
                .handler(|_, c| {
                    c.exec = Some(GitExec::Add(Add::default()));
                    Ok(())
                }),
        )
        // --- git commit ---
        .subcmd(
            CmdSpec::<'a, GitCtx>::new("commit")
                .help("record changes to the repository")
                .opt(
                    OptSpec::value("message", push_commit_msg)
                        .short('m')
                        .long("message")
                        .metavar("MSG")
                        .repeatable(),
                )
                .opt(OptSpec::flag("all", set_commit_all).short('a').long("all"))
                .opt(OptSpec::flag("amend", set_commit_amend).long("amend"))
                .group("mode", GroupMode::Xor) // e.g., you might make --amend xor with others in a real app
                .handler(|_, c| {
                    c.exec = Some(GitExec::Commit(Commit::default()));
                    Ok(())
                })
                .validator(|m| {
                    // Example policy: require at least one -m (like scripting UX)
                    if !m.view().values("message").map_or(false, |v| !v.is_empty()) {
                        return Err(rapp::Error::User(
                            "commit requires at least one -m MSG in this demo",
                        ));
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
                    c.exec = Some(GitExec::Status(Status::default()));
                    Ok(())
                }),
        )
        // --- git log ---
        .subcmd(
            CmdSpec::<'a, GitCtx>::new("log")
                .opt(OptSpec::flag("oneline", set_log_oneline).long("oneline"))
                .opt(OptSpec::value("limit", set_log_limit).short('n').metavar("N"))
                .opt(
                    OptSpec::value("grep", push_log_grep)
                        .long("grep")
                        .metavar("PATTERN")
                        .repeatable(),
                )
                .pos(PosSpec::new("PATH", push_log_path).many())
                .handler(|_, c| {
                    c.exec = Some(GitExec::Log(Log::default()));
                    Ok(())
                }),
        )
        // --- git branch ---
        .subcmd(
            CmdSpec::<'a, GitCtx>::new("branch")
                .opt(OptSpec::flag("list", set_branch_list).long("list"))
                .opt(OptSpec::flag("all", set_branch_all).long("all"))
                .opt(OptSpec::value("delete", set_branch_delete).short('d').metavar("NAME"))
                .group("mode", GroupMode::Xor) // --list/--all/-d are mutually exclusive in this demo
                .handler(|_, c| {
                    c.exec = Some(GitExec::Branch(Branch::default()));
                    Ok(())
                }),
        )
        // --- git checkout ---
        .subcmd(
            CmdSpec::<'a, GitCtx>::new("checkout")
                .opt(OptSpec::value("new", set_checkout_new).short('b').metavar("NAME"))
                .pos(PosSpec::new("BRANCH", set_checkout_branch).range(0, 1))
                .handler(|_, c| {
                    c.exec = Some(GitExec::Checkout(Checkout::default()));
                    Ok(())
                })
                .validator(|m| {
                    let v = m.view();
                    let have_new = v.value("new").is_some();
                    let have_branch = v.value("BRANCH").is_some();
                    if !have_new && !have_branch {
                        return Err(rapp::Error::User("checkout needs -b NAME or BRANCH"));
                    }
                    Ok(())
                }),
        );

    root
}

fn main() {
    let mut env =
        rapp::Env { version: Some("0.1.0"), author: Some("The Demo Team"), ..Default::default() };
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
            // Dispatch according to what the leaf handler set in ctx
            println!("ctx = {ctx:?}");
            // You could also inspect `matches.view()` for leaf-scoped values
            if matches.is_set_from("no_pager", rapp::Source::Cli) {
                eprintln!("--no-pager came from CLI");
            }
        }
    }
}
