# Changelog

All notable changes to this project will be documented in this file.

This project adheres to [Semantic Versioning](https://semver.org/) once **1.0.0** is released.
Prior to 1.0.0, minor version bumps may include breaking changes.

---

## [3.0.0]

### Core

- Internally scoped option and positional keys cannot collide, including when names contain dots.
- Validate command declarations before parsing, including duplicate labels, reserved built-ins, groups and positional cardinalities.
- Removed unused development dependencies and aligned the lockfile with the Rust 1.74 MSRV.

### Derive

- Argument callbacks and command handlers run directly on the parsed `Parser`, `Args`, or `Subcommand` value.
- Added `#[arg(skip)]` and `#[arg(skip = expression)]` for fields that are not CLI arguments.
- Typed decoding completes before callbacks; callback order and selected-leaf dispatch follow the core parser.
- The derive entrypoints no longer accept a separate application context.
- `rust-args-parser-derive` is version 0.2.0.
- Package manifests declare Rust 1.74.

---

## [2.1.0]

### Added

- Optional `derive` feature for typed `Parser`, `Args`, and `Subcommand` declarations, backed by the existing `CmdSpec` parser.
- Direct event bindings for `on_flag`, `on_value`, `handler` and fallible `_try` variants, with a user-owned mutable context.
- Typed decoding is checked after core validation but before argument callbacks and the selected leaf handler.
- Nested subcommand derives, aliases, flattened arguments, count flags, groups, validators, environment/default values, and non-UTF-8 paths.
- Derive command descriptions use `#[command(help = "...")]`, consistent with `#[arg(help = "...")]`.
- Derive integration tests and callback-driven examples.

### Compatibility

- Derive is disabled by default. Builder API behavior and public signatures remain unchanged.
- Rust requires a small procedural-macro crate as the derive implementation; consumers only depend on `rust-args-parser`.

---

## [2.0.1]

### Fixed

- Environment/default values are applied only when the corresponding CLI option is absent. Repeatable values no longer append environment/default values to CLI values, and a CLI flag no longer inherits an additional environment occurrence.
- The selected leaf command path is set before command validation, callbacks, and the leaf handler. Scoped `Matches::view()` now works inside handlers.
- All selected scopes receive overlays before command validators run, so cross-scope validation sees final values.

No public API changes.

---

## [2.0.0]

### 🚨 Breaking changes

- **User callbacks are infallible by default.** `CmdSpec::handler`, `OptSpec::flag`/`value`, and `PosSpec::new` now take callbacks returning `()`.
- Added fallible variants for user-defined error types: `CmdSpec::handler_try`, `OptSpec::flag_try`/`value_try`, and `PosSpec::new_try`.
- Validators accept any `Result<(), E>` where `E: Display` (mapped to `Error::User(String)`); `validator_try` is available for typed errors (mapped to `Error::UserAny`).
- Specs accept closures for callbacks/validators (not only `fn` pointers).

### Notes

- Parse-level errors remain unchanged (`UnknownOption`, `MissingValue`, etc.).

## [1.0.1]

### Added

- Exposed counting-flag occurrences via `Matches::flag_count()` and `MatchView::flag_count()`.
  This enables common patterns like `-vvv` / repeated `--verbose` to be read directly from parsed matches,
  returning `0` when the flag is absent.

## [1.0.0]

### 🚨 Breaking changes

- **New entrypoint:** `dispatch(...) -> Result<()>` replaced by `parse(&Env, &CmdSpec, &[OsString], &mut Ctx) -> Result<Matches>`.
- **Return type:** parsing now returns a `Matches` tree; use `m.view()` for leaf scope or `m.at(&[])` for root scope.
- **Errors:** `Error::Exit(code)` replaced by `Error::ExitMsg { code, message }` (e.g., for `--help`, `--version`).
- **Env config:** builder-style `Env::new(...).auto_help(...).auto_color()` removed. Use the plain struct fields:

  ```rust
  Env { wrap_cols, color, suggest, auto_help, version, author }
  ```

- **Options API:** unified constructors
  - flags: `OptSpec::flag("name", fn(&mut Ctx)->Result<()>)`
  - values: `OptSpec::value("name", fn(&OsStr, &mut Ctx)->Result<()>)`
    With builders: `.short() .long() .metavar() .help() .env() .default() .group() .repeat() .validator()`
- **Positionals API:** use `.required()`, `.many()`, or `.range(min, max)` (replaces older `.one()`/legacy variants).
- **Args type:** `&[&str]` → `&[OsString]` (non-UTF-8 safe).
- **Features:** removed the unused **`completions`** feature flag.

### ✨ Added

- **Scoped matches API:** `Matches` + `MatchView` for ergonomic, scoped inspection.
- **Provenance:** `is_set_from(name, Source::{Cli,Env,Default})` to assert where a value came from (per-scope).
- **Flag occurrences:** short clusters like `-vvv` now trigger the flag callback **once per occurrence**.
- **Validators everywhere:** option/positional validators run for CLI, ENV and Default values; callbacks don’t fire on validation failure.
- **Groups:** `GroupMode::{Xor, ReqOne}` enforced after overlays, across all sources.
- **Numeric look-ahead:** tokens like `-1`, `-.5`, `+3.14`, `1e3`, `-1.2e-3` are treated as values (not options).
- **End-of-options marker:** `--` reliably forces remaining tokens to be positional (even if they look like options).

### 🔧 Changed

- **Overlay precedence clarified:** **CLI > ENV > Default**; this order is now consistently validated and test-covered.
- **Built-ins:** help/version paths return `ExitMsg { code: 0, message }` for friendly printing.
- **Defaults:** feature set `default = ["help", "suggest", "color"]` (disable with `default-features = false`).

### 🐛 Fixed

- Optional positional greediness after `--` no longer swallows the first token that _looks_ like an option.
- Leading `+` in numeric tokens recognized by `util::looks_like_number_token`.

### 📦 Migration guide (0.4 → 1.0)

**Before (0.4-style):**

```rust
use rust_args_parser as ap;
// Env builder + dispatch + &str argv
let env = ap::Env::new("demo").version("0.1.0").author("A").auto_help(true).auto_color();
let root = ap::CmdSpec::new(None, None)
    .opts([
        ap::OptSpec::new("verbose", |_, u: &mut App| { u.verbose = true; Ok(()) }).flag()
    ]);
let argv: Vec<String> = std::env::args().skip(1).collect();
let args: Vec<&str> = argv.iter().map(String::as_str).collect();
match ap::dispatch(&env, &root, &args, &mut app) {
    Err(ap::Error::Exit(code)) => std::process::exit(code),
    Err(e) => { eprintln!("{e}"); std::process::exit(2) }
    Ok(()) => {}
}
```

**After (1.0-style):**

```rust
use rust_args_parser as ap;
// Plain Env struct + parse + OsString argv + Matches
let env = ap::Env { wrap_cols: 80, color: ap::ColorMode::Auto, suggest: true,
                    auto_help: true, version: Some("0.1.0"), author: Some("A") };

let root = ap::CmdSpec::new("demo")
    .opt(ap::OptSpec::flag("verbose", |u: &mut App| { u.verbose = true; Ok(()) })
        .short('v').long("verbose"));

let argv: Vec<_> = std::env::args_os().skip(1).collect();
match ap::parse(&env, &root, &argv, &mut app) {
    Err(ap::Error::ExitMsg { code, message }) => { if let Some(m) = message { println!("{m}"); } std::process::exit(code) }
    Err(e) => { eprintln!("error: {e}"); std::process::exit(2) }
    Ok(m) => {
        // Scoped inspection
        let leaf = m.view();
        let rootv = m.at(&[]);
        if rootv.is_set_from("verbose", ap::Source::Cli) { /* ... */ }
    }
}
```

**Common rewrites:**

- `Env::new(...).auto_help(...).auto_color()` → struct literal with fields.
- `OptSpec::new("name", handler).flag()` → `OptSpec::flag("name", handler)`.
- Value handlers now take `fn(&OsStr, &mut Ctx) -> Result<()>`.
- Positionals: `.one()` → `.required()`; use `.many()` or `.range(min, max)` as needed.
- Errors: handle `ExitMsg { code, message }` instead of `Exit(code)`.
- Use `std::env::args_os()` and pass `&[OsString]` to preserve non-UTF-8 arguments.

---

## [0.4.x]

### Highlights

- Callback-centric design with `CmdSpec`, `OptSpec`, `PosSpec`.
- Basic subcommands, help output, and suggestion support (feature-gated).
- Early support for groups, ranges for positionals, and environment/default overlays.

---

## Notes

- **MSRV:** Rust 1.74.
- Feature flags can be disabled with `default-features = false`.
- For deterministic help output in tests, set `Env.color = ColorMode::Never`.
