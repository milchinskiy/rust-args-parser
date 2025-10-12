# rust\_args\_parser

> Tiny, fast, callback-based CLI argument parser for Rust. **No macros. No derive. No global state.**

> ⚙️ **Zero** `unsafe`, 📦 `std`-only. 🧵 Minimal allocations on hot paths. 🎨 Optional colorized help that respects `NO_COLOR`.

This crate is a small, pragmatic alternative to heavy frameworks when you want:

- Simple, **callback-driven** option handling (`fn(Option<&str>, &mut U) -> Result<()>`)
- **Subcommands** with aliases (nested `CmdSpec`)
- **Short clusters** (`-abc`, `-j10`, `-j 10`) and long options `--name[=value]`
- **Optional/required values** with numeric look-ahead (so `-1` / `-.5` / `1e3` aren't mistaken for options)
- **Exclusive / required groups** (`at_most_one(group)`, `at_least_one(group)`)
- **Environment / default** application (`.env("NAME")`, `.default("value")`)
- **Positionals schema** with ranges (`PosSpec::new("FILE").range(1, 10)`)
- **Typo hints** and structured error variants for unknown options/commands (from version 0.3.0)
- Built-ins: `-h/--help`, `-V/--version`, `-A/--author` when enabled in `Env`

It's inspired by the author's C library **[c-args-parser](https://github.com/milchinskiy/c-args-parser)**, re-imagined in safe Rust.

---

## Table of contents

- [Quick start](#quick-start)
- [Examples](#examples)
- [Subcommands](#subcommands)
- [Groups (mutually-exclusive / required one)](#groups-mutually-exclusive--required-one)
- [Environment & defaults](#environment--defaults)
- [Positionals](#positionals)
- [Typo hints](#typo-hints--030)
- [Behavior details](#behavior-details)
- [Color & wrapping](#color--wrapping)
- [Errors](#errors)
- [Performance & safety](#performance--safety)
- [License](#license)

---

## Quick start

Minimal skeleton:

```rust
use rust_args_parser as ap;

#[derive(Default)]
struct App { verbose: bool, n: i32, limit: Option<String> }

fn main() -> ap::Result<()> {
    let mut app = App::default();
    let env = ap::Env::new("demo")
        .version("0.1.0")
        .author("Your Name <you@example.com>")
        .auto_help(true)
        .auto_color();

    let root = ap::CmdSpec::new(None, None)
        .desc("Demo tool")
        .opts([
            // -v, --verbose (flag)
            ap::OptSpec::new("verbose", |_, u: &mut App| { u.verbose = true; Ok(()) })
                .short('v')
                .help("Enable verbose output")
                .flag(),
            // -n N / --n=N (required value, hinted numeric)
            ap::OptSpec::new("n", |v, u: &mut App| {
                u.n = v.unwrap().parse().map_err(|_| ap::Error::User("bad -n"))?;
                Ok(())
            })
                .short('n').metavar("N").help("Required number").numeric().required(),
            // -l[=N] / --limit[=N] (optional value)
            ap::OptSpec::new("limit", |v, u: &mut App| { u.limit = v.map(Into::into); Ok(()) })
                .short('l').metavar("N").help("Optional limit").optional(),
        ])
        .pos([ ap::PosSpec::new("FILE").range(1, 10).desc("Input files") ]);

    // Collect CLI args (skip program name) as &strs
    let argv: Vec<String> = std::env::args().skip(1).collect();
    let args: Vec<&str> = argv.iter().map(String::as_str).collect();

    // Parse and run; prints auto help/version/author to stdout when triggered
    match ap::dispatch(&env, &root, &args, &mut app) {
        Ok(()) => Ok(())
        , Err(ap::Error::Exit(code)) => std::process::exit(code)
        , Err(err) => { eprintln!("{err}"); std::process::exit(2) }
    }
}
```

Run it:

```
$ demo -h
$ demo --n=3 -vv file1 file2
$ demo --limit 10 file
```

> See `examples/basic.rs` in this repo for a full, runnable version.

---

## Examples

Run any example with:

```bash
cargo run --example <name> -- [args...]
```

| Example                      | Teaches                              | Try                                                                    |
| ---------------------------- | ------------------------------------ | ---------------------------------------------------------------------- |
| 01\_minimal                  | one flag + one positional            | `cargo run --example 01_minimal -- -v file.txt`                        |
| 02\_flags\_values            | clusters, required & optional values | `cargo run --example 02_flags_values -- -vv -n10 --limit=5 ./path`     |
| 03\_optional\_numbers        | optional numeric look-ahead          | `cargo run --example 03_optional_numbers -- -t -0.25`                  |
| 04\_groups                   | XOR / REQ\_ONE groups                | `cargo run --example 04_groups -- --mode-a`                            |
| 05\_subcommands              | subcommands & aliases                | `cargo run --example 05_subcommands -- remote add https://example`     |
| 06\_env\_defaults\_and\_help | env/defaults & built-ins             | `THREADS=8 cargo run --example 06_env_defaults_and_help -- -o out.txt` |

The sources live under `examples/`.

---

## Subcommands

Compose subcommands by nesting `CmdSpec`:

```rust
let remote_add = ap::CmdSpec::new(Some("add"), Some(|pos, _u: &mut App| {
    println!("remote add: {}", pos.get(0).unwrap_or(&""));
    Ok(())
}))
.pos([ap::PosSpec::new("URL").one().desc("Remote URL")]);

let remote = ap::CmdSpec::new(Some("remote"), None)
    .aliases(["r"]) // optional
    .subs([remote_add]);

let root = ap::CmdSpec::new(None, None)
    .subs([remote])
    .pos([]);
```

`dispatch` descends through bare tokens until it resolves the final command, then parses options/positionals at that depth.

---

## Groups (mutually-exclusive / required one)

Use group ids to express constraints across options:

```rust
let root = ap::CmdSpec::new(None, None).opts([
    ap::OptSpec::new("color", |_,_| Ok(())).help("Force color").at_most_one(1),
    ap::OptSpec::new("no-color", |_,_| Ok(())).help("Disable color").at_most_one(1),
    ap::OptSpec::new("mode-a", |_,_| Ok(())).help("Mode A").at_least_one(2),
    ap::OptSpec::new("mode-b", |_,_| Ok(())).help("Mode B").at_least_one(2),
]);
```

`at_most_one(g)` enforces **XOR** (≤ 1 present in group `g`), `at_least_one(g)` enforces **REQ\_ONE** (≥ 1 present in group `g`). Defaults and env-applied options count toward these rules.

---

## Environment & defaults

Attach environment variables and string defaults to options. Both are applied **after** parsing and before group/positional checks:

```rust
ap::OptSpec::new("threads", |v, _| Ok(()))
    .metavar("N").numeric()
    .env("THREADS")      // uses value from env if present
    .default("4");       // otherwise falls back to this
```

---

## Positionals

Describe positionals per command with names, descriptions and cardinality:

```rust
ap::PosSpec::new("FILE").one();            // exactly one
ap::PosSpec::new("ITEM").range(2, 5);      // 2..=5
ap::PosSpec::new("PATH").desc("Input path");
```

If no positional schema is declared for a command, any leftover bare token becomes an error.

---

## Typo hints (≥ 0.3.0)

### What changed (breaking)

The two error variants for unknown flags/commands now carry the exact token and a list of suggestions:

```rust
pub enum Error {
    UnknownOption { token: String, suggestions: Vec<String> },
    UnknownCommand { token: String, suggestions: Vec<String> },
    // …other variants unchanged…
}
```

This is a source-compatible change if you only format the error (`{err}`), but a pattern‑matching breaking change if you match on the variants. See migration below.

### Migration guide

Before (≤ 0.2.x):

```rust
match err {
    ap::Error::UnknownOption(tok) => eprintln!("unknown option: {tok}"),
    ap::Error::UnknownCommand(tok) => eprintln!("unknown command: {tok}"),
    _ => {}
}
```

After (≥ 0.3.0):

```rust
match err {
    ap::Error::UnknownOption { token, suggestions } => {
        if suggestions.is_empty() {
            eprintln!("unknown option: {token}");
        } else {
            eprintln!("unknown option: {token}. Did you mean {}?",
            format_alternates(&suggestions));
        }
    }
    ap::Error::UnknownCommand { token, suggestions } => {
        if suggestions.is_empty() {
            eprintln!("unknown command: {token}");
        } else {
            eprintln!("unknown command: {token}. Did you mean {}?",
            format_alternates(&suggestions));
        }
    }
    _ => {}
}


fn format_alternates(items: &[String]) -> String {
    match items.len() {
        0 => String::new(),
        1 => format!("'{}'", items[0]),
        2 => format!("'{}' or '{}'", items[0], items[1]),
        _ => {
            let mut s = String::new();
            for (i, it) in items.iter().enumerate() {
                if i > 0 { s.push_str(if i + 1 == items.len() { ", or " } else { ", " }); }
                s.push('\''); s.push_str(it); s.push('\'');
            }
            s
        }
    }
}
```

> Tip: If you only ever print `err` with `{}` via `Display`, you don't need to change anything—messages now automatically include suggestions when available.

### Where suggestions come from

- **Long/short options**: taken strictly from the current command's `opts`, plus built‑ins (`--help/-h`, `--version/-V`, `--author/-A`) only if enabled in `Env`.
- **Commands**: taken from the current command's `subs` (`names` and `aliases`).
- Suggestions are formatted exactly as users should type them: `--long`, `-x`, `start`.
- Hints are computed only on the error path. Normal parsing paths incur no extra overhead.

### Heuristics (succinct)

- Distance metric: small Levenshtein threshold based on token length (≤1 for short tokens, ≤2 for medium, ≤3 for long).
- Up to 3 closest items are shown; ties are ordered by distance.
- No cross-kind suggestions (a mistyped --long doesn’t suggest commands, and vice versa).

### Examples

```
$ demo --vers
error: unknown option: '--vers'. Did you mean '--version'?

$ demo -H
error: unknown option: '-H'. Did you mean '-h'?

$ demo remot
error: unknown command: remot. Did you mean 'remote'?
```

---

## Behavior details

- **Option forms**
  - Long: `--name`, `--name=value`, `--name value`
  - Short clusters: `-abc`, `-j10`, `-j 10` (no `-j=10`)
- **Optional values** pick up the next token **if** it *looks like a value*. Numeric hints allow `-1`, `-.5`, `1e9` to be consumed as values.
- `--` stops option parsing; remaining tokens are positionals.
- **Built-ins** (when configured in `Env`): `-h|--help`, `-V|--version`, `-A|--author` print to `stdout` and return `Err(Error::Exit(0))` so you can `process::exit(0)` cleanly.

---

## Color & wrapping

Help text is rendered with optional ANSI color. Use `Env::auto_color()` to disable when `NO_COLOR` is set, or `Env::color(false)` to force plain output. Set `wrap_cols` to wrap long descriptions.

---

## Errors

`dispatch` returns `Result<()>`. Common handling:

```rust
match ap::dispatch(&env, &root, &args, &mut app) {
    Ok(()) => {}
    Err(ap::Error::Exit(code)) => std::process::exit(code),
    Err(e) => { eprintln!("{e}"); std::process::exit(2) }
}
```

All errors implement `Display` and `std::error::Error`.

---

## Performance & safety

- **Zero** `unsafe` (`#![forbid(unsafe_code)]`): the parser is implemented entirely in safe Rust.
- No global state; no proc-macros; no derives.
- Minimal allocations: small count vector for options, optional small buffers for help rendering.
- Fast paths for ASCII short clusters and long option parsing.

> For heavy usage, see `benches/` (Criterion®) and measure on your workload. The library aims to avoid surprises and keep hot paths branch-lean.

---

## License

Dual-licensed under **MIT** or **Apache-2.0** at your option.

```
SPDX-License-Identifier: MIT OR Apache-2.0
```

If you contribute, you agree to license your contributions under the same terms.

