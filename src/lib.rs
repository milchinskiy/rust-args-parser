#![forbid(unsafe_code)]
#![warn(missing_docs)]

//! SPDX-License-Identifier: MIT or Apache-2.0
//! rust-args-parser — Tiny, fast, callback-based CLI argument parser for Rust inspired by
//! <https://github.com/milchinskiy/c-args-parser>

use std::env;
use std::fmt::{self, Write as _};
use std::io::{self, Write};

/* ================================ Public API ================================= */
type BoxError = Box<dyn std::error::Error>;
/// Library level error type.
pub type Result<T> = std::result::Result<T, Error>;

/// Each option/flag invokes a callback.
pub type OptCallback<Ctx> =
    for<'a> fn(Option<&'a str>, &mut Ctx) -> std::result::Result<(), BoxError>;

/// Command runner for the resolved command (receives final positionals).
pub type RunCallback<Ctx> = fn(&[&str], &mut Ctx) -> std::result::Result<(), BoxError>;

/// Whether the option takes a value or not.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArgKind {
    /// No value required
    None,
    /// Value required
    Required,
    /// Value optional
    Optional,
}

/// Group mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GroupMode {
    /// No group
    None,
    /// Exclusive group
    Xor,
    /// Required one group
    ReqOne,
}
/// Value hint.
#[derive(Clone, Copy)]
pub enum ValueHint {
    /// Any value
    Any,
    /// Number
    Number,
}
/// An option/flag specification.
#[derive(Clone, Copy)]
pub struct OptSpec<'a, Ctx: ?Sized> {
    name: &'a str,            // long name without "--"
    short: Option<char>,      // short form without '-'
    arg: ArgKind,             // whether it takes a value
    metavar: Option<&'a str>, // shown in help for value
    help: &'a str,            // help text
    env: Option<&'a str>,     // environment variable default (name)
    default: Option<&'a str>, // string default
    group_id: u16,            // 0 = none, >0 = group identifier
    group_mode: GroupMode,    // XOR / REQ_ONE semantics
    value_hint: ValueHint,    // value hint
    cb: OptCallback<Ctx>,     // callback on set/apply
}

impl<'a, Ctx: ?Sized> OptSpec<'a, Ctx> {
    /// Create a new option.
    pub const fn new(name: &'a str, cb: OptCallback<Ctx>) -> Self {
        Self {
            name,
            short: None,
            arg: ArgKind::None,
            metavar: None,
            help: "",
            env: None,
            default: None,
            group_id: 0,
            group_mode: GroupMode::None,
            value_hint: ValueHint::Any,
            cb,
        }
    }
    /// Set value hint
    #[must_use]
    pub const fn numeric(mut self) -> Self {
        self.value_hint = ValueHint::Number;
        self
    }
    /// Set short form.
    #[must_use]
    pub const fn short(mut self, short: char) -> Self {
        self.short = Some(short);
        self
    }
    /// Set metavar.
    #[must_use]
    pub const fn metavar(mut self, metavar: &'a str) -> Self {
        self.metavar = Some(metavar);
        self
    }
    /// Set help text.
    #[must_use]
    pub const fn help(mut self, help: &'a str) -> Self {
        self.help = help;
        self
    }
    /// Set argument kind.
    #[must_use]
    pub const fn arg(mut self, arg: ArgKind) -> Self {
        self.arg = arg;
        self
    }
    /// Set optional
    #[must_use]
    pub const fn optional(mut self) -> Self {
        self.arg = ArgKind::Optional;
        self
    }
    /// Set required
    #[must_use]
    pub const fn required(mut self) -> Self {
        self.arg = ArgKind::Required;
        self
    }
    /// Set flag
    #[must_use]
    pub const fn flag(mut self) -> Self {
        self.arg = ArgKind::None;
        self
    }
    /// Set environment variable name.
    #[must_use]
    pub const fn env(mut self, env: &'a str) -> Self {
        self.env = Some(env);
        self
    }
    /// Set default value.
    #[must_use]
    pub const fn default(mut self, val: &'a str) -> Self {
        self.default = Some(val);
        self
    }
    /// Set at most one state and group identifier.
    #[must_use]
    pub const fn at_most_one(mut self, group_id: u16) -> Self {
        self.group_id = group_id;
        self.group_mode = GroupMode::Xor;
        self
    }
    /// Set at least one state and group identifier.
    #[must_use]
    pub const fn at_least_one(mut self, group_id: u16) -> Self {
        self.group_id = group_id;
        self.group_mode = GroupMode::ReqOne;
        self
    }
}

/// Positional argument specification.
#[derive(Clone, Copy)]
pub struct PosSpec<'a> {
    name: &'a str,
    desc: Option<&'a str>,
    min: usize,
    max: usize,
}

impl<'a> PosSpec<'a> {
    /// Create a new positional argument.
    #[must_use]
    pub const fn new(name: &'a str) -> Self {
        Self { name, desc: None, min: 0, max: 0 }
    }
    /// Set description.
    #[must_use]
    pub const fn desc(mut self, desc: &'a str) -> Self {
        self.desc = Some(desc);
        self
    }
    /// Set one required.
    #[must_use]
    pub const fn one(mut self) -> Self {
        self.min = 1;
        self.max = 1;
        self
    }
    /// Set any number.
    #[must_use]
    pub const fn range(mut self, min: usize, max: usize) -> Self {
        self.min = min;
        self.max = max;
        self
    }
}

/// Command specification.
pub struct CmdSpec<'a, Ctx: ?Sized> {
    name: Option<&'a str>, // None for root
    desc: Option<&'a str>,
    opts: Box<[OptSpec<'a, Ctx>]>,
    subs: Box<[CmdSpec<'a, Ctx>]>,
    pos: Box<[PosSpec<'a>]>,
    aliases: Box<[&'a str]>,
    run: Option<RunCallback<Ctx>>, // called with positionals
}

impl<'a, Ctx: ?Sized> CmdSpec<'a, Ctx> {
    /// Create a new command.
    /// `name` is `None` for root command.
    #[must_use]
    pub fn new(name: Option<&'a str>, run: Option<RunCallback<Ctx>>) -> Self {
        Self {
            name,
            desc: None,
            opts: Vec::new().into_boxed_slice(),
            subs: Vec::new().into_boxed_slice(),
            pos: Vec::new().into_boxed_slice(),
            aliases: Vec::new().into_boxed_slice(),
            run,
        }
    }
    /// Set description.
    #[must_use]
    pub const fn desc(mut self, desc: &'a str) -> Self {
        self.desc = Some(desc);
        self
    }
    /// Set options.
    #[must_use]
    pub fn opts<S>(mut self, s: S) -> Self
    where
        S: Into<Vec<OptSpec<'a, Ctx>>>,
    {
        self.opts = s.into().into_boxed_slice();
        self
    }
    /// Set positionals.
    #[must_use]
    pub fn pos<S>(mut self, s: S) -> Self
    where
        S: Into<Vec<PosSpec<'a>>>,
    {
        self.pos = s.into().into_boxed_slice();
        self
    }
    /// Set subcommands.
    #[must_use]
    pub fn subs<S>(mut self, s: S) -> Self
    where
        S: Into<Vec<Self>>,
    {
        self.subs = s.into().into_boxed_slice();
        self
    }
    /// Set aliases.
    #[must_use]
    pub fn aliases<S>(mut self, s: S) -> Self
    where
        S: Into<Vec<&'a str>>,
    {
        self.aliases = s.into().into_boxed_slice();
        self
    }
}

/// Environment configuration
pub struct Env<'a> {
    name: &'a str,
    version: Option<&'a str>,
    author: Option<&'a str>,
    auto_help: bool,
    wrap_cols: usize,
    color: bool,
}

impl<'a> Env<'a> {
    /// Create a new environment.
    #[must_use]
    pub const fn new(name: &'a str) -> Self {
        Self { name, version: None, author: None, auto_help: false, wrap_cols: 0, color: false }
    }
    /// Set version.
    #[must_use]
    pub const fn version(mut self, version: &'a str) -> Self {
        self.version = Some(version);
        self
    }
    /// Set author.
    #[must_use]
    pub const fn author(mut self, author: &'a str) -> Self {
        self.author = Some(author);
        self
    }
    /// Set auto help.
    #[must_use]
    pub const fn auto_help(mut self, auto_help: bool) -> Self {
        self.auto_help = auto_help;
        self
    }
    /// Set wrap columns.
    #[must_use]
    pub const fn wrap_cols(mut self, wrap_cols: usize) -> Self {
        self.wrap_cols = wrap_cols;
        self
    }
    /// Set color.
    #[must_use]
    pub const fn color(mut self, color: bool) -> Self {
        self.color = color;
        self
    }
    /// Set auto color.
    /// Check for `NO_COLOR` env var.
    #[must_use]
    pub fn auto_color(mut self) -> Self {
        self.color = env::var("NO_COLOR").is_err();
        self
    }
}

/// Parse and dispatch starting from `root` using `argv` (not including program name), writing
/// auto help/version/author output to `out` when triggered.
/// # Errors
/// See [`Error`]
pub fn dispatch_to<Ctx: ?Sized, W: Write>(
    env: &Env<'_>,
    root: &CmdSpec<'_, Ctx>,
    argv: &[&str],
    context: &mut Ctx,
    out: &mut W,
) -> Result<()> {
    let mut idx = 0usize;
    let mut cmd = root;
    let mut chain: Vec<&str> = Vec::new();
    while idx < argv.len() {
        if let Some(next) = find_sub(cmd, argv[idx]) {
            if !cmd.opts.is_empty() {
                let mut tmp = vec![0u8; cmd.opts.len()];
                apply_env_and_defaults(cmd, context, &mut tmp)?;
                check_groups(cmd, &tmp)?;
            }
            chain.push(argv[idx]);
            cmd = next;
            idx += 1;
        } else {
            break;
        }
    }
    // If this command defines subcommands but no positional schema,
    // the next bare token must be a known subcommand; otherwise it's an error.
    if !cmd.subs.is_empty() && cmd.pos.is_empty() && idx < argv.len() {
        let tok = argv[idx];
        if !tok.starts_with('-') && tok != "--" && find_sub(cmd, tok).is_none() {
            return Err(unknown_command_error(cmd, tok));
        }
    }
    // small counter array parallel to opts
    let mut gcounts: Vec<u8> = vec![0; cmd.opts.len()];
    // parse options and collect positionals
    let mut pos: Vec<&str> = Vec::with_capacity(argv.len().saturating_sub(idx));
    let mut stop_opts = false;
    while idx < argv.len() {
        let tok = argv[idx];
        if !stop_opts {
            if tok == "--" {
                stop_opts = true;
                idx += 1;
                continue;
            }
            if tok.starts_with("--") {
                idx += 1;
                parse_long(env, cmd, tok, &mut idx, argv, context, &mut gcounts, out, &chain)?;
                continue;
            }
            if is_short_like(tok) {
                idx += 1;
                parse_short_cluster(
                    env,
                    cmd,
                    tok,
                    &mut idx,
                    argv,
                    context,
                    &mut gcounts,
                    out,
                    &chain,
                )?;
                continue;
            }
            if !tok.starts_with('-') && tok != "--" && cmd.pos.is_empty() {
                if let Some(next) = find_sub(cmd, tok) {
                    apply_env_and_defaults(cmd, context, &mut gcounts)?;
                    check_groups(cmd, &gcounts)?;
                    chain.push(tok);
                    cmd = next;
                    idx += 1;
                    gcounts = vec![0; cmd.opts.len()];
                    pos.clear();
                    continue;
                }
            }
        }
        pos.push(tok);
        idx += 1;
    }
    // When no positional schema is declared, any leftover bare token is unexpected.
    if cmd.pos.is_empty() && !pos.is_empty() {
        return Err(Error::UnexpectedArgument(pos[0].to_string()));
    }
    apply_env_and_defaults(cmd, context, &mut gcounts)?;
    // strict groups: XOR → ≤1, REQ_ONE → ≥1 (env/defaults count)
    check_groups(cmd, &gcounts)?;
    // validate positionals against schema
    validate_positionals(cmd, &pos)?;
    // run command
    if let Some(run) = cmd.run {
        return run(&pos, context).map_err(Error::Callback);
    }
    if env.auto_help {
        print_help_to(env, cmd, &chain, out);
    }
    Err(Error::Exit(1))
}

/// Default dispatch that prints auto help/version/author to **stdout**.
/// # Errors
/// See [`Error`]
pub fn dispatch<Ctx>(
    env: &Env<'_>,
    root: &CmdSpec<'_, Ctx>,
    argv: &[&str],
    context: &mut Ctx,
) -> Result<()> {
    let mut out = io::stdout();
    dispatch_to(env, root, argv, context, &mut out)
}

/* ================================ Suggestions ===================================== */
#[inline]
fn format_alternates(items: &[String]) -> String {
    match items.len() {
        0 => String::new(),
        1 => format!("'{}'", items[0]),
        2 => format!("'{}' or '{}'", items[0], items[1]),
        _ => {
            // 'a', 'b', or 'c'
            let mut s = String::new();
            for (i, it) in items.iter().enumerate() {
                if i > 0 {
                    s.push_str(if i + 1 == items.len() { ", or " } else { ", " });
                }
                s.push('\'');
                s.push_str(it);
                s.push('\'');
            }
            s
        }
    }
}

#[inline]
const fn max_distance_for(len: usize) -> usize {
    match len {
        0..=3 => 1,
        4..=6 => 2,
        _ => 3,
    }
}

// Simple Levenshtein (O(n*m)); inputs are tiny (flags/commands), so it's fine.
fn lev(a: &str, b: &str) -> usize {
    let (na, nb) = (a.len(), b.len());
    if na == 0 {
        return nb;
    }
    if nb == 0 {
        return na;
    }
    let mut prev: Vec<usize> = (0..=nb).collect();
    let mut curr = vec![0; nb + 1];
    for (i, ca) in a.chars().enumerate() {
        curr[0] = i + 1;
        for (j, cb) in b.chars().enumerate() {
            let cost = usize::from(ca != cb);
            curr[j + 1] = (prev[j + 1] + 1).min(curr[j] + 1).min(prev[j] + cost);
        }
        std::mem::swap(&mut prev, &mut curr);
    }
    prev[nb]
}

fn collect_long_candidates<Ctx: ?Sized>(env: &Env<'_>, cmd: &CmdSpec<'_, Ctx>) -> Vec<String> {
    let mut v = Vec::with_capacity(cmd.opts.len() + 3);
    if env.auto_help {
        v.push("help".to_string());
    }
    if cmd.name.is_none() {
        v.push("version".to_string());
        v.push("author".to_string());
    }
    for o in &cmd.opts {
        v.push(o.name.to_string());
    }
    v
}

fn collect_short_candidates<Ctx: ?Sized>(env: &Env<'_>, cmd: &CmdSpec<'_, Ctx>) -> Vec<char> {
    let mut v = Vec::with_capacity(cmd.opts.len() + 3);
    if env.auto_help {
        v.push('h');
    }
    if cmd.name.is_none() {
        v.push('V');
        v.push('A');
    }
    for o in &cmd.opts {
        if let Some(ch) = o.short {
            v.push(ch);
        }
    }
    v
}

fn collect_cmd_candidates<'a, Ctx: ?Sized>(cmd: &'a CmdSpec<'a, Ctx>) -> Vec<&'a str> {
    let mut v = Vec::with_capacity(cmd.subs.len());
    for s in &cmd.subs {
        if let Some(n) = s.name {
            v.push(n);
        }
        for &al in &s.aliases {
            v.push(al);
        }
    }
    v
}

fn suggest_longs<Ctx: ?Sized>(env: &Env<'_>, cmd: &CmdSpec<'_, Ctx>, name: &str) -> Vec<String> {
    let thr = max_distance_for(name.len());
    let mut scored: Vec<(usize, String)> = collect_long_candidates(env, cmd)
        .into_iter()
        .map(|cand| (lev(name, &cand), format!("--{cand}")))
        .collect();
    scored.sort_by_key(|(d, _)| *d);
    scored.into_iter().take_while(|(d, _)| *d <= thr).take(3).map(|(_, s)| s).collect()
}

fn suggest_shorts<Ctx: ?Sized>(env: &Env<'_>, cmd: &CmdSpec<'_, Ctx>, ch: char) -> Vec<String> {
    let mut v: Vec<(usize, String)> = collect_short_candidates(env, cmd)
        .into_iter()
        .map(|c| (usize::from(c != ch), format!("-{c}")))
        .collect();
    v.sort_by_key(|(d, _)| *d);
    v.into_iter().take_while(|(d, _)| *d <= 1).take(3).map(|(_, s)| s).collect()
}

fn suggest_cmds<Ctx: ?Sized>(cmd: &CmdSpec<'_, Ctx>, tok: &str) -> Vec<String> {
    let thr = max_distance_for(tok.len());
    let mut v: Vec<(usize, String)> = collect_cmd_candidates(cmd)
        .into_iter()
        .map(|cand| (lev(tok, cand), cand.to_string()))
        .collect();
    v.sort_by_key(|(d, _)| *d);
    v.into_iter().take_while(|(d, _)| *d <= thr).take(3).map(|(_, s)| s).collect()
}

/* ================================ Errors ===================================== */
#[non_exhaustive]
#[derive(Debug)]
/// Error type
pub enum Error {
    /// Missing value
    MissingValue(String),
    /// Unexpected argument
    UnexpectedArgument(String),
    /// Unknown option
    UnknownOption {
        /// Token
        token: String,
        /// Suggestions
        suggestions: Vec<String>,
    },
    /// Unknown command
    UnknownCommand {
        /// Token
        token: String,
        /// Suggestions
        suggestions: Vec<String>,
    },
    /// Group violation
    GroupViolation(String),
    /// Missing positional
    MissingPositional(String),
    /// Too many positionals
    TooManyPositional(String),
    /// Callback error
    Callback(BoxError),
    /// Exit with code
    Exit(i32),
    /// User error
    User(&'static str),
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownOption { token, suggestions } => {
                write!(f, "unknown option: '{token}'")?;
                if !suggestions.is_empty() {
                    write!(f, ". Did you mean {}?", format_alternates(suggestions))?;
                }
                Ok(())
            }
            Self::MissingValue(n) => write!(f, "missing value for --{n}"),
            Self::UnexpectedArgument(s) => write!(f, "unexpected argument: {s}"),
            Self::UnknownCommand { token, suggestions } => {
                write!(f, "unknown command: {token}")?;
                if !suggestions.is_empty() {
                    write!(f, ". Did you mean {}?", format_alternates(suggestions))?;
                }
                Ok(())
            }
            Self::GroupViolation(s) => write!(f, "{s}"),
            Self::MissingPositional(n) => write!(f, "missing positional: {n}"),
            Self::TooManyPositional(n) => write!(f, "too many values for: {n}"),
            Self::Callback(e) => write!(f, "{e}"),
            Self::Exit(code) => write!(f, "exit {code}"),
            Self::User(s) => write!(f, "{s}"),
        }
    }
}
impl std::error::Error for Error {}

/* ================================ Parsing ==================================== */
fn find_sub<'a, Ctx: ?Sized>(
    cmd: &'a CmdSpec<'a, Ctx>,
    name: &str,
) -> Option<&'a CmdSpec<'a, Ctx>> {
    for c in &cmd.subs {
        if let Some(n) = c.name {
            if n == name {
                return Some(c);
            }
        }
        if c.aliases.contains(&name) {
            return Some(c);
        }
    }
    None
}
fn apply_env_and_defaults<Ctx: ?Sized>(
    cmd: &CmdSpec<'_, Ctx>,
    context: &mut Ctx,
    counts: &mut [u8],
) -> Result<()> {
    if cmd.opts.is_empty() {
        return Ok(());
    }
    if !any_env_or_default(cmd) {
        return Ok(());
    }

    // env first
    for (i, o) in cmd.opts.iter().enumerate() {
        if let Some(key) = o.env {
            if let Ok(val) = std::env::var(key) {
                counts[i] = counts[i].saturating_add(1);
                (o.cb)(Some(val.as_str()), context).map_err(Error::Callback)?;
            }
        }
    }
    // defaults next; skip if anything in the same group is already present
    for (i, o) in cmd.opts.iter().enumerate() {
        if counts[i] != 0 {
            continue;
        }
        let Some(def) = o.default else { continue };
        if o.group_id != 0 {
            let gid = o.group_id;
            let mut taken = false;
            for (j, p) in cmd.opts.iter().enumerate() {
                if p.group_id == gid && counts[j] != 0 {
                    taken = true;
                    break;
                }
            }
            if taken {
                continue;
            }
        }
        counts[i] = counts[i].saturating_add(1);
        (o.cb)(Some(def), context).map_err(Error::Callback)?;
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn parse_long<Ctx: ?Sized, W: std::io::Write>(
    env: &Env<'_>,
    cmd: &CmdSpec<'_, Ctx>,
    tok: &str,
    idx: &mut usize,
    argv: &[&str],
    context: &mut Ctx,
    counts: &mut [u8],
    out: &mut W,
    chain: &[&str],
) -> Result<()> {
    // formats: --name, --name=value, --name value
    let s = &tok[2..];
    let (name, attached) = s
        .as_bytes()
        .iter()
        .position(|&b| b == b'=')
        .map_or((s, None), |eq| (&s[..eq], Some(&s[eq + 1..])));
    // built‑ins
    if env.auto_help && name == "help" {
        print_help_to(env, cmd, chain, out);
        return Err(Error::Exit(0));
    }
    if cmd.name.is_none() {
        if env.version.is_some() && name == "version" {
            print_version_to(env, out);
            return Err(Error::Exit(0));
        }
        if env.author.is_some() && name == "author" {
            print_author_to(env, out);
            return Err(Error::Exit(0));
        }
    }
    let (i, spec) = match cmd.opts.iter().enumerate().find(|(_, o)| o.name == name) {
        Some(x) => x,
        None => return Err(unknown_long_error(env, cmd, name)),
    };
    counts[i] = counts[i].saturating_add(1);
    match spec.arg {
        ArgKind::None => {
            (spec.cb)(None, context).map_err(Error::Callback)?;
        }
        ArgKind::Required => {
            let v = if let Some(a) = attached {
                if a.is_empty() {
                    return Err(Error::MissingValue(spec.name.to_string()));
                }
                a
            } else {
                take_next(idx, argv).ok_or_else(|| Error::MissingValue(spec.name.to_string()))?
            };
            (spec.cb)(Some(v), context).map_err(Error::Callback)?;
        }
        ArgKind::Optional => {
            let v = match (attached, argv.get(*idx).copied()) {
                (Some(a), _) => Some(a),
                (None, Some("-")) => {
                    *idx += 1; // consume standalone "-" but treat as none
                    None
                }
                (None, Some(n))
                    if matches!(spec.value_hint, ValueHint::Number) && is_dash_number(n) =>
                {
                    *idx += 1;
                    Some(n)
                }
                (None, Some(n)) if looks_value_like(n) => {
                    *idx += 1;
                    Some(n)
                }
                _ => None,
            };
            (spec.cb)(v, context).map_err(Error::Callback)?;
        }
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn parse_short_cluster<Ctx: ?Sized, W: std::io::Write>(
    env: &Env<'_>,
    cmd: &CmdSpec<'_, Ctx>,
    tok: &str,
    idx: &mut usize,
    argv: &[&str],
    context: &mut Ctx,
    counts: &mut [u8],
    out: &mut W,
    chain: &[&str],
) -> Result<()> {
    // formats: -abc, -j10, -j 10, -j -12  (no -j=10 by design)
    let short_idx = build_short_idx(cmd);
    let s = &tok[1..];
    let bytes = s.as_bytes();
    let mut i = 0usize;
    while i < bytes.len() {
        // Fast ASCII path for common cases; fall back to UTF‑8 char boundary when needed.
        let (ch, adv) = if bytes[i] < 128 {
            (bytes[i] as char, 1)
        } else {
            let c = s[i..].chars().next().unwrap();
            (c, c.len_utf8())
        };
        i += adv;

        // built‑ins
        if env.auto_help && ch == 'h' {
            print_help_to(env, cmd, chain, out);
            return Err(Error::Exit(0));
        }
        if cmd.name.is_none() {
            if env.version.is_some() && ch == 'V' {
                print_version_to(env, out);
                return Err(Error::Exit(0));
            }
            if env.author.is_some() && ch == 'A' {
                print_author_to(env, out);
                return Err(Error::Exit(0));
            }
        }
        let (oi, spec) = match lookup_short(cmd, &short_idx, ch) {
            Some(x) => x,
            None => return Err(unknown_short_error(env, cmd, ch)),
        };
        counts[oi] = counts[oi].saturating_add(1);
        match spec.arg {
            ArgKind::None => {
                (spec.cb)(None, context).map_err(Error::Callback)?;
            }
            ArgKind::Required => {
                if i < s.len() {
                    let rem = &s[i..];
                    (spec.cb)(Some(rem), context).map_err(Error::Callback)?;
                    return Ok(());
                }
                let v = take_next(idx, argv)
                    .ok_or_else(|| Error::MissingValue(spec.name.to_string()))?;
                (spec.cb)(Some(v), context).map_err(Error::Callback)?;
                return Ok(());
            }
            ArgKind::Optional => {
                if i < s.len() {
                    let rem = &s[i..];
                    (spec.cb)(Some(rem), context).map_err(Error::Callback)?;
                    return Ok(());
                }
                // SPECIAL: if next token is exactly "-", CONSUME it but treat as "no value".
                let v = match argv.get(*idx) {
                    Some(&"-") => {
                        *idx += 1;
                        None
                    }
                    // If hint is Number, allow a directly following numeric like `-j -1.25`.
                    Some(n)
                        if matches!(spec.value_hint, ValueHint::Number) && is_dash_number(n) =>
                    {
                        *idx += 1;
                        Some(n)
                    }
                    // Otherwise consume if it looks like a plausible value (incl. -1, -0.5, 1e3…)
                    Some(n) if looks_value_like(n) => {
                        *idx += 1;
                        Some(n)
                    }
                    _ => None,
                };
                (spec.cb)(v.map(|v| &**v), context).map_err(Error::Callback)?;
                return Ok(());
            }
        }
    }
    Ok(())
}

#[inline]
fn any_env_or_default<Ctx: ?Sized>(cmd: &CmdSpec<'_, Ctx>) -> bool {
    cmd.opts.iter().any(|o| o.env.is_some() || o.default.is_some())
}
#[inline]
fn take_next<'a>(idx: &mut usize, argv: &'a [&'a str]) -> Option<&'a str> {
    let i = *idx;
    if i < argv.len() {
        *idx = i + 1;
        Some(argv[i])
    } else {
        None
    }
}
#[inline]
fn is_short_like(s: &str) -> bool {
    let b = s.as_bytes();
    b.len() >= 2 && b[0] == b'-' && b[1] != b'-'
}
#[inline]
fn is_dash_number(s: &str) -> bool {
    let b = s.as_bytes();
    if b.is_empty() || b[0] != b'-' {
        return false;
    }
    // "-" alone is not a number
    if b.len() == 1 {
        return false;
    }
    is_numeric_like(&b[1..])
}
#[inline]
fn looks_value_like(s: &str) -> bool {
    if !s.starts_with('-') {
        return true;
    }
    if s == "-" {
        return false;
    }
    is_numeric_like(&s.as_bytes()[1..])
}
#[inline]
fn is_numeric_like(b: &[u8]) -> bool {
    // digits, optional dot, optional exponent part
    let mut i = 0;
    let n = b.len();
    // optional leading dot: .5
    if i < n && b[i] == b'.' {
        i += 1;
    }
    // at least one digit
    let mut nd = 0;
    while i < n && (b[i] as char).is_ascii_digit() {
        i += 1;
        nd += 1;
    }
    if nd == 0 {
        return false;
    }
    // optional fractional part .ddd
    if i < n && b[i] == b'.' {
        i += 1;
        while i < n && (b[i] as char).is_ascii_digit() {
            i += 1;
        }
    }
    // optional exponent e[+/-]ddd
    if i < n && (b[i] == b'e' || b[i] == b'E') {
        i += 1;
        if i < n && (b[i] == b'+' || b[i] == b'-') {
            i += 1;
        }
        let mut ed = 0;
        while i < n && (b[i] as char).is_ascii_digit() {
            i += 1;
            ed += 1;
        }
        if ed == 0 {
            return false;
        }
    }
    i == n
}

fn check_groups<Ctx: ?Sized>(cmd: &CmdSpec<'_, Ctx>, counts: &[u8]) -> Result<()> {
    let opts = &cmd.opts;
    let opts_len = opts.len();
    let mut index = 0usize;
    while index < opts_len {
        let id = opts[index].group_id;
        if id != 0 {
            // ensure we only process each id once
            let mut seen = false;
            let mut k = 0usize;
            while k < index {
                if opts[k].group_id == id {
                    seen = true;
                    break;
                }
                k += 1;
            }
            if !seen {
                let mut total = 0u32;
                let mut xor = false;
                let mut req = false;
                let mut j = 0usize;
                while j < opts_len {
                    let o = &opts[j];
                    if o.group_id == id {
                        total += u32::from(counts[j]);
                        match o.group_mode {
                            GroupMode::Xor => xor = true,
                            GroupMode::ReqOne => req = true,
                            GroupMode::None => {}
                        }
                        if xor && total > 1 {
                            return Err(Error::GroupViolation(group_msg(opts, id, true)));
                        }
                    }
                    j += 1;
                }
                if req && total == 0 {
                    return Err(Error::GroupViolation(group_msg(opts, id, false)));
                }
            }
        }
        index += 1;
    }
    Ok(())
}

#[cold]
#[inline(never)]
fn group_msg<Ctx: ?Sized>(opts: &[OptSpec<'_, Ctx>], id: u16, xor: bool) -> String {
    let mut names = String::new();
    for o in opts.iter().filter(|o| o.group_id == id) {
        if !names.is_empty() {
            names.push_str(" | ");
        }
        names.push_str(o.name);
    }
    if xor {
        format!("at most one of the following options may be used: {names}")
    } else {
        format!("one of the following options is required: {names}")
    }
}

fn validate_positionals<Ctx: ?Sized>(cmd: &CmdSpec<'_, Ctx>, pos: &[&str]) -> Result<()> {
    if cmd.pos.is_empty() {
        return Ok(());
    }
    let total = pos.len();
    let mut min_sum: usize = 0;
    let mut max_sum: Option<usize> = Some(0);
    for p in &cmd.pos {
        min_sum = min_sum.saturating_add(p.min);
        if let Some(ms) = max_sum {
            if p.max == usize::MAX {
                max_sum = None;
            } else {
                max_sum = Some(ms.saturating_add(p.max));
            }
        }
    }
    // Not enough arguments: find the first positional whose minimum cannot be met
    if total < min_sum {
        let mut need = 0usize;
        for p in &cmd.pos {
            need = need.saturating_add(p.min);
            if total < need {
                return Err(Error::MissingPositional(p.name.to_string()));
            }
        }
        // Fallback (should be unreachable)
        return Err(Error::MissingPositional(
            cmd.pos.first().map_or("<args>", |p| p.name).to_string(),
        ));
    }
    // Too many arguments (only when all maxima are finite)
    if let Some(ms) = max_sum {
        if total > ms {
            let last = cmd.pos.last().map_or("<args>", |p| p.name);
            return Err(Error::TooManyPositional(last.to_string()));
        }
    }
    Ok(())
}
#[inline]
const fn plain_opt_label_len<Ctx: ?Sized>(o: &OptSpec<'_, Ctx>) -> usize {
    let mut len = if o.short.is_some() { 4 } else { 0 }; // "-x, "
    len += 2 + o.name.len(); // "--" + name
    if let Some(m) = o.metavar {
        len += 1 + m.len();
    }
    len
}
#[inline]
fn make_opt_label<Ctx: ?Sized>(o: &OptSpec<'_, Ctx>) -> String {
    let mut s = String::new();
    if let Some(ch) = o.short {
        s.push('-');
        s.push(ch);
        s.push(',');
        s.push(' ');
    }
    s.push_str("--");
    s.push_str(o.name);
    if let Some(m) = o.metavar {
        s.push(' ');
        s.push_str(m);
    }
    s
}

/* ================================ Help ======================================= */
const C_BOLD: &str = "\u{001b}[1m";
const C_UNDERLINE: &str = "\u{001b}[4m";
const C_BRIGHT_WHITE: &str = "\u{001b}[97m";
const C_CYAN: &str = "\u{001b}[36m";
const C_MAGENTA: &str = "\u{001b}[35m";
const C_YELLOW: &str = "\u{001b}[33m";
const C_RESET: &str = "\u{001b}[0m";
#[inline]
fn colorize(s: &str, color: &str, env: &Env) -> String {
    if !env.color || color.is_empty() {
        s.to_string()
    } else {
        format!("{color}{s}{C_RESET}")
    }
}
#[inline]
fn help_text_for_opt<Ctx: ?Sized>(o: &OptSpec<'_, Ctx>) -> String {
    match (o.env, o.default) {
        (Some(k), Some(d)) => format!("{} (env {k}, default={d})", o.help),
        (Some(k), None) => format!("{} (env {k})", o.help),
        (None, Some(d)) => format!("{} (default={d})", o.help),
        (None, None) => o.help.to_string(),
    }
}
#[inline]
fn print_header(buf: &mut String, text: &str, env: &Env) {
    let _ = writeln!(buf, "\n{}:", colorize(text, &[C_BOLD, C_UNDERLINE].concat(), env).as_str());
}
#[inline]
fn lookup_short<'a, Ctx: ?Sized>(
    cmd: &'a CmdSpec<'a, Ctx>,
    table: &[u16; 128],
    ch: char,
) -> Option<(usize, &'a OptSpec<'a, Ctx>)> {
    let c = ch as u32;
    if c < 128 {
        let i = table[c as usize];
        if i != u16::MAX {
            let idx = i as usize;
            return Some((idx, &cmd.opts[idx]));
        }
        return None;
    }
    cmd.opts.iter().enumerate().find(|(_, o)| o.short == Some(ch))
}
fn build_short_idx<Ctx: ?Sized>(cmd: &CmdSpec<'_, Ctx>) -> [u16; 128] {
    let mut map = [u16::MAX; 128];
    let mut i = 0usize;
    let len = cmd.opts.len();
    while i < len {
        let o = &cmd.opts[i];
        if let Some(ch) = o.short {
            let cu = ch as usize;
            if cu < 128 {
                debug_assert!(u16::try_from(i).is_ok());
                map[cu] = u16::try_from(i).unwrap_or(0); // safe due to debug assert
            }
        }
        i += 1;
    }
    map
}
#[inline]
fn write_wrapped(buf: &mut String, text: &str, indent_cols: usize, wrap_cols: usize) {
    if wrap_cols == 0 {
        let _ = writeln!(buf, "{text}");
        return;
    }
    let mut col = indent_cols;
    let mut first = true;
    for word in text.split_whitespace() {
        let wlen = word.len();
        if first {
            buf.push_str(word);
            col = indent_cols + wlen;
            first = false;
            continue;
        }
        if col + 1 + wlen > wrap_cols {
            buf.push('\n');
            for _ in 0..indent_cols {
                buf.push(' ');
            }
            buf.push_str(word);
            col = indent_cols + wlen;
        } else {
            buf.push(' ');
            buf.push_str(word);
            col += 1 + wlen;
        }
    }
    buf.push('\n');
}

fn write_row(
    buf: &mut String,
    env: &Env,
    color: &str,
    plain_label: &str,
    help: &str,
    label_col: usize,
) {
    let _ = write!(buf, "  {}", colorize(plain_label, color, env));
    let pad = label_col.saturating_sub(plain_label.len());
    for _ in 0..pad {
        buf.push(' ');
    }
    buf.push(' ');
    buf.push(' ');
    let indent = 4 + label_col;
    write_wrapped(buf, help, indent, env.wrap_cols);
}

/// Print help to the provided writer.
#[cold]
#[inline(never)]
pub fn print_help_to<Ctx: ?Sized, W: Write>(
    env: &Env<'_>,
    cmd: &CmdSpec<'_, Ctx>,
    path: &[&str],
    mut out: W,
) {
    let mut buf = String::new();
    let _ = write!(
        buf,
        "Usage: {}",
        colorize(env.name, [C_BOLD, C_BRIGHT_WHITE].concat().as_str(), env)
    );
    for tok in path {
        let _ = write!(buf, " {}", colorize(tok, C_MAGENTA, env));
    }
    let has_subs = !cmd.subs.is_empty();
    let has_opts = !cmd.opts.is_empty()
        || env.auto_help
        || (path.is_empty() && (env.version.is_some() || env.author.is_some()));
    if has_opts && has_subs {
        let _ = write!(buf, " {}", colorize("[options]", C_CYAN, env));
        let _ = write!(buf, " {}", colorize("<command>", C_MAGENTA, env));
    } else if !has_opts && has_subs {
        let _ = write!(buf, " {}", colorize("<command>", C_MAGENTA, env));
    } else if has_opts && !has_subs {
        let _ = write!(buf, " {}", colorize("[options]", C_CYAN, env));
    }
    for p in &cmd.pos {
        if p.min == 0 {
            let _ = write!(buf, " [{}]", colorize(p.name, C_YELLOW, env));
        } else if p.min == 1 && p.max == 1 {
            let _ = write!(buf, " {}", colorize(p.name, C_YELLOW, env));
        } else if p.max > 1 {
            let _ = write!(buf, " {}...", colorize(p.name, C_YELLOW, env));
        }
    }
    let _ = writeln!(buf);
    if let Some(desc) = cmd.desc {
        let _ = writeln!(buf, "\n{desc}");
    }
    if !cmd.opts.is_empty()
        || env.auto_help
        || (cmd.name.is_none() && (env.version.is_some() || env.author.is_some()))
    {
        print_header(&mut buf, "Options", env);
        let mut width = 0usize;
        if env.auto_help {
            width = width.max("-h, --help".len());
        }
        // if cmd is a root command
        if cmd.name.is_none() {
            if env.version.is_some() {
                width = width.max("-V, --version".len());
            }
            if env.author.is_some() {
                width = width.max("-A, --author".len());
            }
        }
        for o in &cmd.opts {
            width = width.max(plain_opt_label_len(o));
        }
        if env.auto_help {
            write_row(&mut buf, env, C_CYAN, "-h, --help", "Show this help and exit", width);
        }
        // if cmd is a root command
        if cmd.name.is_none() {
            if env.version.is_some() {
                write_row(&mut buf, env, C_CYAN, "-V, --version", "Show version and exit", width);
            }
            if env.author.is_some() {
                write_row(&mut buf, env, C_CYAN, "--author", "Show author and exit", width);
            }
        }
        for o in &cmd.opts {
            let label = make_opt_label(o);
            let help = help_text_for_opt(o);
            write_row(&mut buf, env, C_CYAN, &label, &help, width);
        }
    }
    // Commands
    if !cmd.subs.is_empty() {
        print_header(&mut buf, "Commands", env);
        let width = cmd.subs.iter().map(|s| s.name.unwrap_or("<root>").len()).max().unwrap_or(0);
        for s in &cmd.subs {
            let name = s.name.unwrap_or("<root>");
            write_row(&mut buf, env, C_MAGENTA, name, s.desc.unwrap_or(""), width);
        }
    }
    // Positionals
    if !cmd.pos.is_empty() {
        print_header(&mut buf, "Positionals", env);
        let width = cmd.pos.iter().map(|p| p.name.len()).max().unwrap_or(0);
        for p in &cmd.pos {
            let help = help_for_pos(p);
            write_row(&mut buf, env, C_YELLOW, p.name, &help, width);
        }
    }
    let _ = out.write_all(buf.as_bytes());
}
fn help_for_pos(p: &PosSpec) -> String {
    if let Some(d) = p.desc {
        return d.to_string();
    }
    if p.min == 0 {
        return "(optional)".to_string();
    }
    if p.min == 1 && p.max == 1 {
        return "(required)".to_string();
    }
    if p.min == 1 {
        return "(at least one required)".to_string();
    }
    format!("min={} max={}", p.min, p.max)
}
/// Prints the version number
#[cold]
#[inline(never)]
pub fn print_version_to<W: Write>(env: &Env<'_>, mut out: W) {
    if let Some(v) = env.version {
        let _ = writeln!(out, "{v}");
    }
}
/// Prints the author
#[cold]
#[inline(never)]
pub fn print_author_to<W: Write>(env: &Env<'_>, mut out: W) {
    if let Some(a) = env.author {
        let _ = writeln!(out, "{a}");
    }
}
#[cold]
#[inline(never)]
fn unknown_long_error<Ctx: ?Sized>(env: &Env<'_>, cmd: &CmdSpec<'_, Ctx>, name: &str) -> Error {
    let token = {
        let mut s = String::with_capacity(2 + name.len());
        s.push_str("--");
        s.push_str(name);
        s
    };
    let suggestions = suggest_longs(env, cmd, name);
    Error::UnknownOption { token, suggestions }
}

#[cold]
#[inline(never)]
fn unknown_short_error<Ctx: ?Sized>(env: &Env<'_>, cmd: &CmdSpec<'_, Ctx>, ch: char) -> Error {
    let mut token = String::with_capacity(2);
    token.push('-');
    token.push(ch);
    let suggestions = suggest_shorts(env, cmd, ch);
    Error::UnknownOption { token, suggestions }
}

#[cold]
#[inline(never)]
fn unknown_command_error<Ctx: ?Sized>(cmd: &CmdSpec<'_, Ctx>, tok: &str) -> Error {
    Error::UnknownCommand { token: tok.to_string(), suggestions: suggest_cmds(cmd, tok) }
}
