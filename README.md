# rust_args_parser

> Tiny, fast, callback-based CLI argument parser for Rust.

- 📦 Crate: `rust-args-parser`
- 📚 Docs: <https://docs.rs/rust-args-parser>
- 🔧 MSRV: **1.60**
- ⚖️ License: **MIT OR Apache-2.0**

This crate is a pragmatic alternative to heavyweight frameworks when you want:

- **Callbacks**: options/positionals map directly to functions that mutate your context.
- **Subcommands** (nested `CmdSpec`) with aliases.
- **Short clusters** (`-vvj8`) and long forms (`--jobs=8`).
- **Numeric look-ahead** so tokens like `-1`, `-.5`, `+3.14`, `1e3` are treated as values, not options.
- **Groups**: mutually exclusive (`Xor`) / at least one required (`ReqOne`).
- **ENV/Default overlays** with clear precedence (**CLI > ENV > Default**).
- **Readable matches** with **scope** and **provenance**.

---

## Quick start

```rust
use rust_args_parser as ap;
use std::ffi::OsStr;

#[derive(Default, Debug)]
struct Ctx {
    verbose: u8,
    json: bool,
    jobs: Option<u32>,
    input: Option<String>,
}

fn inc_verbose(c: &mut Ctx) -> ap::Result<()> { c.verbose = c.verbose.saturating_add(1); Ok(()) }
fn set_json(c: &mut Ctx) -> ap::Result<()> { c.json = true; Ok(()) }
fn set_jobs(v: &OsStr, c: &mut Ctx) -> ap::Result<()> {
    let n: u32 = v.to_string_lossy().parse().map_err(|_| ap::Error::User("invalid --jobs".into()))?;
    c.jobs = Some(n); Ok(())
}
fn set_input(v: &OsStr, c: &mut Ctx) -> ap::Result<()> { c.input = Some(v.to_string_lossy().into()); Ok(()) }

fn main() -> ap::Result<()> {
    // Global environment for parsing (and help rendering, if enabled)
    let env = ap::Env { wrap_cols: 80, color: ap::ColorMode::Auto, suggest: true, auto_help: true, version: Some("0.1.0"), author: None };

    // Command spec
    let spec = ap::CmdSpec::new("demo")
        .help("Demo tool")
        .opt(ap::OptSpec::flag("verbose", inc_verbose).short('v').long("verbose").help("Enable verbose output"))
        .opt(ap::OptSpec::flag("json", set_json).long("json").help("JSON output"))
        .opt(ap::OptSpec::value("jobs", set_jobs).short('j').long("jobs").metavar("N").help("Worker threads"))
        .pos(ap::PosSpec::new("INPUT", set_input).range(0, 1));

    let mut ctx = Ctx::default();
    let argv: Vec<_> = std::env::args_os().collect();

    match ap::parse(&env, &spec, &argv, &mut ctx) {
        Err(ap::Error::ExitMsg { code, message }) => {
            if let Some(m) = message { println!("{}", m); }
            std::process::exit(code);
        }
        Err(e) => { eprintln!("error: {e}"); std::process::exit(2); }
        Ok(m) => {
            println!("ctx   = {:?}", ctx);            // callbacks applied
            println!("leaf  = {:?}", m.leaf_path());  // selected command path
            Ok(())
        }
    }
}
```

### CLI behavior

- **Short clusters**: `-vvj8` ⇒ `-v -v -j 8` (flag callback fires once per `-v`).
- **Inline/next-arg values**: `-j8` / `-j 8`, `--jobs=8` / `--jobs 8`.
- **Negative numbers**: `-d-3`, `--delta -3` are values (not options).
- **End-of-options**: `--` makes the rest positional, even if they start with `-`.

---

## Subcommands

Subcommands are nested `CmdSpec`s and **scoped**.

```rust
use rust_args_parser as ap; use std::ffi::OsStr;
#[derive(Default)] struct Ctx { remote: Option<String>, branch: Option<String>, files: Vec<String> }
fn set_remote(v: &OsStr, c: &mut Ctx) -> ap::Result<()> { c.remote = Some(v.to_string_lossy().into()); Ok(()) }
fn set_branch(v: &OsStr, c: &mut Ctx) -> ap::Result<()> { c.branch = Some(v.to_string_lossy().into()); Ok(()) }
fn push_file(v: &OsStr, c: &mut Ctx) -> ap::Result<()> { c.files.push(v.to_string_lossy().into()); Ok(()) }

let spec = ap::CmdSpec::new("tool")
    .subcmd(
        ap::CmdSpec::new("repo")
            .alias("r")
            .subcmd(
                ap::CmdSpec::new("push")
                    .pos(ap::PosSpec::new("REMOTE", set_remote).required())
                    .pos(ap::PosSpec::new("BRANCH", set_branch).required())
                    .pos(ap::PosSpec::new("FILE", push_file).many())
            )
    );

let mut ctx = Ctx::default();
let m = ap::parse(&env, &spec, &argv, &mut ctx)?;
assert_eq!(m.leaf_path(), vec!["repo", "push"]);
let v = m.view();
assert_eq!(v.pos_one("BRANCH").unwrap(), OsStr::new("main"));
```

> Root options are **not** accepted after you descend into a subcommand unless re-declared at that level.

---

## Options, positionals, groups, validators

### Options

- **Flag**: `OptSpec::flag("name", on_flag)`
- **Value**: `OptSpec::value("name", on_value)`
- Builders: `.short('j')`, `.long("jobs")`, `.metavar("N")`, `.help("…")`, `.env("VAR")`, `.default(OsString)`, `.group("name")`, `.repeat(Repeat::Many)`, `.validator(fn)`

### Positionals

- `PosSpec::new("NAME", on_value)` then choose one:
  - `.required()`
  - `.many()` (0..∞)
  - `.range(min, max)`
- Also `.help("…")`, `.validator(fn)`.

### Groups

- `GroupMode::Xor` — options in the same group are mutually exclusive.
- `GroupMode::ReqOne` — require at least one option from the group.

```rust
let spec = ap::CmdSpec::new("fmt")
    .opt(ap::OptSpec::flag("json", |_| Ok(())).long("json").group("fmt"))
    .opt(ap::OptSpec::flag("yaml", |_| Ok(())).long("yaml").group("fmt"))
    .group("fmt", ap::GroupMode::Xor);
```

### Validators

Validators run on **CLI, ENV, and Default** values. If a validator fails, the callback for that option/positional is not invoked.

---

## Overlays & provenance

- **Precedence**: **CLI > ENV > Default**.
- Bind ENV via `.env("NAME")`, defaults via `.default(…)`.
- Check where a value came from with `matches.is_set_from(name, Source::{Cli,Env,Default})`.
- `Matches` is **scoped**: use `m.view()` for the leaf command or `m.at(&[])` for root.

---

## Built-ins & features

Feature flags (enabled by default unless you disable `default-features`):

- `help` — built-in `-h/--help` and `--version` returning `Error::ExitMsg { code: 0, message }`.
- `color` — colorized help output (honors `NO_COLOR`), with `ColorMode::{Auto,Always,Never}`.
- `suggest` — suggestions for unknown options/commands.

---

## Matches & views

`Matches` collects everything the parser saw. `MatchView` gives you a scoped, read-only accessor.

```rust
let m: ap::Matches = ap::parse(&env, &spec, &argv, &mut ctx)?;
let leaf = m.view();          // leaf scope
let root = m.at(&[]);         // root scope

leaf.is_set("verbose");
root.is_set_from("limit", ap::Source::Env);
leaf.value("jobs");          // first value
leaf.values("file");         // all values for an option
leaf.pos_one("INPUT");       // single positional by name
leaf.pos_all("FILE");        // all positionals with that name
```

> Flags are stored as presence (`Value::Flag`). The parser also counts flag **occurrences** internally so `-vvv` calls the flag callback three times.

---

## Errors

Top-level error type: `ap::Error`.

- `Error::User(String)` / `Error::UserAny(Box<dyn Error + Send + Sync>)`
- `Error::Parse(String)`
- `Error::ExitMsg { code, message }`
- Structured diagnostics:
  - `UnknownOption { token, suggestions }`
  - `UnknownCommand { token, suggestions }`
  - `MissingValue { opt }`
  - `UnexpectedPositional { token }`

Typical handling:

```rust
match ap::parse(&env, &spec, &argv, &mut ctx) {
    Err(ap::Error::ExitMsg { code, message }) => { if let Some(m) = message { println!("{}", m); } std::process::exit(code) }
    Err(e) => { eprintln!("error: {e}"); std::process::exit(2) }
    Ok(m) => { /* use ctx and/or m */ }
}
```

---

## Utilities (`ap::util`)

- `looks_like_number_token(&str) -> bool` — `-1`, `+3.14`, `-.5`, `1e3`, `-1.2e-3`.
- `strip_ansi_len(&str) -> usize` — visible length, ignoring minimal ANSI sequences used in help.

---

## Examples

See `examples/`:

- `basic.rs` — flags, values, callbacks, errors
- `subcommands.rs` — nested commands, leaf scoping
- `env_defaults.rs` — ENV/default precedence
- `git.rs` — realistic multi-command layout

Run:

```bash
cargo run --example basic -- --help
```

---

## Testing

A comprehensive test suite covers options/positionals, subcommands, groups, overlays, validators, suggestions, help, utils, and an end-to-end **golden** test.

```bash
cargo test --features "help suggest color"
# or core only
cargo test
```

---

## License

Dual-licensed under **MIT** or **Apache-2.0** at your option.
