# rust-args-parser

A small, callback-driven CLI parser for Rust. Define commands with `CmdSpec`, attach callbacks to arguments, and execute a handler for the selected leaf command. The optional `derive` feature lets you declare the same parser using Rust structs and enums.

[Crate documentation](https://docs.rs/rust-args-parser) · [Changelog](CHANGELOG.md) · MIT / Apache-2.0

## Start here

The existing builder API is unchanged. It works without macros or additional runtime dependencies.

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

## Derive (optional, 2.1.0)

```toml
rust-args-parser = { version = "2.1", features = ["derive"] }
```

The consumer imports only `rust-args-parser`. The proc-macro implementation is a build-time dependency, not a second API or parser. Rust requires attribute derives to be implemented in a separate `proc-macro` crate.

See the runnable examples:

- [`examples/derive_basic.rs`](examples/derive_basic.rs) — typed fields **and** argument/command callbacks
- [`examples/derive_subcommands.rs`](examples/derive_subcommands.rs) — nested enum commands and leaf handlers

```sh
cargo run --features derive --example derive_basic -- -vv -j4 input.txt
cargo run --features derive --example derive_subcommands -- remote add origin https://example.com
```

### Binding handlers

Use `#[command(context = App)]` to select the same mutable context type passed to the existing parser. Declare function paths on fields and command variants:

```rust
#[arg(short = 'v', action = "count", on_flag = inc_verbose)]
verbose: u8,

#[arg(long, on_value_try = set_port)]
port: Option<u16>,

#[command(handler = run)]
Run,
```

Here `inc_verbose` has signature `fn(&mut App)`, `set_port` has signature `fn(&OsStr, &mut App) -> Result<(), E>`, and `run` has signature `fn(&Matches, &mut App)`. The `_try` functions require `E: std::error::Error + Send + Sync + 'static`. Use `handler_try` for fallible command handlers and `on_flag_try` for fallible flags. For positionals, `on_value` and `on_value_try` use the same signatures as value options.

Parse and dispatch with an existing application context:

```rust
let mut app = App::default();
let cli = Cli::try_parse_from_with_context(std::env::args_os().skip(1), &mut app)?;
```

With `#[command(context = App)]`, annotate **each** derived `Parser`, `Args`, or `Subcommand` type that directly declares handlers or embeds another context-bound type. Types without a custom context may use the default `()`; handler-free types are reusable across contexts. There are no intermediate command structs to initialize: derive generates `CmdSpec` declarations and decodes the selected typed result.

Typed decoding is checked **before** dispatch, so an invalid `FromStr` value, missing required field, or count overflow cannot trigger callbacks. Explicit validators still run first. Callback execution uses the same core engine and ordering as the builder API.

If you do not need handlers, plain `Cli::try_parse()` or `Cli::try_parse_from(args)` works; those methods create a default context. When an application context does not implement `Default`, use `try_parse_from_with_context` instead. `try_parse_from_with_context_and_env(&env, args, &mut app)` accepts custom help/version settings.

### Supported attributes

`#[derive(Parser)]` declares the root struct, `#[derive(Args)]` reusable named-field arguments, and `#[derive(Subcommand)]` enum commands. Subcommand variants may be unit, named-field, or a tuple containing one `Args` type.

- `#[command(name = "...", help = "...", alias = "...")]`; `alias` applies to enum variants.
- `#[command(context = App)]`, `#[command(handler = run)]` / `#[command(handler_try = run_try)]`, and `#[command(validate = check)]` / `#[command(validate_try = check_try)]`. Each pair is mutually exclusive.
- `#[command(group(name = "output", mode = "xor"))]` or `mode = "req_one"`.
- `#[arg(short, long, name = "...", value_name = "...", help = "...")]`; fields without `short`/`long` default to `--kebab-case`.
- `#[arg(positional, required, min = 1, max = 3)]` — `min`/`max` require `Vec<T>` and both must be given.
- `#[arg(action = "count")]` on an integer, `bool` flags, `Option<T>` optional values, and `Vec<T>` repeated values/positionals.
- `#[arg(env = "VAR", default_value = "text", group = "output")]`; defaults are literal strings.
- `#[arg(value_parser = check)]` / `value_parser_try`; `on_flag` / `on_flag_try` for flags, `on_value` / `on_value_try` for values and positionals. Use one form from each pair.
- `#[command(flatten)]` embeds `Args` in the current scope; `#[command(subcommand)]` selects `Subcommand` or `Option<Subcommand>`.

Field values use strict UTF-8 `FromStr` conversion. `OsString` and `PathBuf` retain native OS values. `Matches`, scoped views, source provenance, help/version errors, aliases, and existing validators retain the core parser's behavior.

**Current limitations:** named structs without generics, no flatten collision detection, no inherited global options, and no clap-style value enum support. Flattened `Args` should not install a competing command handler for the same command. The derive feature does not change the manual callback API.

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

See [CHANGELOG.md](CHANGELOG.md) for the 2.0.0 callback error API changes and the 2.0.1 bug fixes.

For a 2.1.0 crates.io release, publish the `derive/` implementation package first (once), then the main package. Cargo requires the optional proc-macro dependency to exist on the registry before the main crate can be published.
