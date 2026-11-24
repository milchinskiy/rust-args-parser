# rust-args-parser

A small, callback-driven CLI parser for Rust. Define commands with `CmdSpec`, attach callbacks to arguments, and execute a handler for the selected leaf command. The optional `derive` feature lets you declare the same parser using Rust structs and enums.

[Crate documentation](https://docs.rs/rust-args-parser) · [Changelog](CHANGELOG.md) · MIT / Apache-2.0

## Start here

- [`examples/basic.rs`](examples/basic.rs) — flags, values, validators, required positionals, groups
- [`examples/subcommands.rs`](examples/subcommands.rs) — aliases, nested commands, leaf handlers
- [`examples/try_handlers.rs`](examples/try_handlers.rs) — fallible callbacks and typed errors
- [`examples/env_defaults.rs`](examples/env_defaults.rs) — CLI / environment / default precedence
- [`examples/git.rs`](examples/git.rs) — larger command tree

```sh
cargo run --example basic -- -vv -j4 input.txt
cargo run --example subcommands -- remote add origin
```

### Execution model

`parse(&env, &spec, &argv, &mut context)` accepts arguments **without** the executable name. On success it returns `Matches` and invokes callbacks as follows:

1. Parse the CLI; apply environment variables and defaults for options absent from the CLI (**CLI > ENV > Default**).
2. Validate selected command scopes from root to leaf.
3. Invoke argument callbacks, from root to leaf and in **specification order**, not token order. A repeatable flag calls its callback once per occurrence; repeated values call their callback once per value.
4. Invoke the **selected leaf** command's handler (if registered). Parent command handlers do not run when a child is selected.

Invalid command specifications (duplicate names/switches, reserved built-ins, invalid positional cardinality, or unreachable positionals) are rejected before parsing. An unbounded positional must be last; `range(min, max)` requires `1 <= max` and `min <= max`.

Validation precedes all callbacks. Callback/handler failure stops dispatch; mutations performed by earlier callbacks are not rolled back. `Matches::view()` refers to the selected leaf even inside validators and handlers; `Matches::at(&[])` refers to the root.

`OptSpec::{flag,value}` and `PosSpec::new` accept infallible callbacks. Their `*_try` counterparts accept typed user errors. For commands, use `CmdSpec::{handler,handler_try}` and `CmdSpec::{validator,validator_try}`. See [`try_handlers.rs`](examples/try_handlers.rs) for error handling and downcasting.

### Syntax and scope

- Short clusters such as `-vvj8`, long values such as `--jobs=8`, and next-token values such as `--jobs 8`.
- Negative numbers are accepted as values/positionals; `--` stops option parsing.
- `OsStr`/`OsString` arguments and callbacks preserve non-UTF-8 input.
- Positionals may be required, unbounded (`many`), or bounded (`range(min, max)`); they are consumed greedily.
- Groups: `GroupMode::Xor` (at most one) and `GroupMode::ReqOne` (at least one).
- Subcommands have independent option scopes. Parent options are **not** accepted after selecting a child unless declared there too. Aliases are resolved to canonical command paths.
- Single-valued options keep the last CLI value. For repeatable options, values are collected in order.

## Derive (optional)

```toml
rust-args-parser = { version = "3", features = ["derive"] }
```

Derive uses the same parser and validation rules as `CmdSpec`. It creates the
actual Rust values first, then runs callbacks and the selected command handler
on those values. No separate application context or default-constructed state
is needed. Handlers see final decoded field values.

- [`examples/derive_basic.rs`](examples/derive_basic.rs) — argument events and a root handler
- [`examples/derive_subcommands.rs`](examples/derive_subcommands.rs) — nested commands, aliases and leaf handlers

```sh
cargo run --features derive --example derive_basic -- -vv -j4 input.txt
cargo run --features derive --example derive_subcommands -- remote add origin https://example.com
```

### Handlers

Declare methods on the type being derived:

```rust
#[derive(Parser)]
#[command(handler = Self::run)]
struct Cli {
    #[arg(short = 'v', action = "count", on_flag = Self::on_verbose)]
    verbose: usize,
}
```

- `handler = Self::run` takes `fn(&mut Self)`. It runs only when this command is the selected leaf.
- `on_flag = Self::on_verbose` takes `fn(&mut Self)` and runs once per flag occurrence.
- `on_value = Self::on_value` takes `fn(&mut Self, &OsStr)` and runs once per recorded value.
- The `handler_try`, `on_flag_try`, and `on_value_try` forms return `Result<(), E>`, where `E: Error + Send + Sync + 'static`. Their errors are returned as `Error::UserAny`.
- A subcommand variant's handler receives `&mut` its **enum** (the type with `#[derive(Subcommand)]`), not the parent `Cli`.
- `#[arg(skip)]` initializes a non-CLI field using `Default::default()`; `#[arg(skip = expression)]` initializes it from the supplied expression.

Run `Cli::try_parse()`, `Cli::try_parse_from(args)`, or
`Cli::try_parse_from_with_env(&env, args)`; the argument iterator excludes the
executable name. All typed conversion and validation is finished before event
callbacks run. Argument events run root-to-leaf, options before positionals in
each scope, in specification order rather than token order. A callback failure
stops execution; previous callback mutations are not rolled back.

### Attributes

`Parser` and `Args` derive on named structs; `Subcommand` derives on enums with
unit, named-field, or single-`Args` tuple variants. Generic derived types are
not supported.

- Commands: `name`, `help`, `alias` (variants), `handler`/`handler_try`, `validate`/`validate_try`, `group(name = "...", mode = "xor")` or `mode = "req_one"`.
- Arguments: `short`, `long`, `name`, `value_name`, `help`, `positional`, `required`, `env`, `default_value`, `group`, `action = "count"`, `min`/`max` for bounded positional `Vec<T>`, `value_parser`/`value_parser_try`, event handlers, `skip`.
- Composition: `#[command(flatten)]` for `Args`; `#[command(subcommand)]` for `Subcommand` or `Option<Subcommand>`.

Boolean fields are flags, `Vec<T>` collects repeated values, `Option<T>` is
optional, and other values use strict UTF-8 `FromStr` conversion. `OsString`
and `PathBuf` preserve non-UTF-8 values. Argument help is taken from Rust doc
comments if no explicit `help` is provided.

`Cli::command()` returns a plain `CmdSpec<()>` for inspection, help rendering,
or parsing through the core API. The `Parser` entrypoints run typed handlers and reject ambiguous duplicate option, positional, group, and subcommand names.

## Features

Default features: `help`, `suggest`, `color`. The `derive` feature is off by default.

- `help` — built-in `-h`/`--help` and optional version/author messages (`Error::ExitMsg`).
- `suggest` — spelling suggestions for unknown commands and options.
- `color` — ANSI help coloring, honoring terminal preferences.
- `derive` — procedural derives and typed decoding/event bindings.

## Testing

```sh
cargo test --locked --workspace --all-features --all-targets
cargo test --locked --no-default-features
cargo test --locked --no-default-features --features derive
cargo clippy --workspace --all-features --all-targets -- -D warnings
```
