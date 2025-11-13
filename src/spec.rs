use crate::Result;
use std::ffi::{OsStr, OsString};

/// Color mode for help rendering.
#[derive(Clone, Copy, Debug)]
pub enum ColorMode {
    Auto,
    Always,
    Never,
}

/// Global environment for a parse/render session.
#[derive(Clone, Copy, Debug)]
pub struct Env {
    /// Wrap columns for help. `0` means no wrapping.
    pub wrap_cols: usize,
    /// Whether to colorize help (honors `NO_COLOR` when `color` feature is enabled).
    pub color: ColorMode,
    /// Whether to compute suggestions on errors (if enabled).
    pub suggest: bool,
    /// Built-ins
    pub auto_help: bool,
    pub version: Option<&'static str>,
    pub author: Option<&'static str>,
}
impl Default for Env {
    fn default() -> Self {
        Self {
            wrap_cols: 80,
            color: ColorMode::Auto,
            suggest: true,
            auto_help: true,
            version: None,
            author: None,
        }
    }
}

/// Whether an option may be repeated.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Repeat {
    Single,
    Many,
}

/// Group rule (applies to a set of options sharing the same group name).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GroupMode {
    Xor,
    ReqOne,
}

/// Provenance of a value in `Matches`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Source {
    Cli,
    Env,
    Default,
}

/// User-pluggable validator for a single value (OsStr-based, cross-platform).
pub type ValueValidator = fn(&OsStr) -> Result<()>;

/// Command-level validator that can inspect the final `Matches`.
pub type CmdValidator = fn(&crate::Matches) -> Result<()>;

/// Command handler (executed for the **leaf** command after callbacks).
pub type CmdHandler<Ctx> = fn(&crate::Matches, &mut Ctx) -> Result<()>;

/// Callback to apply a value/flag into user context.
pub type OnValue<Ctx> = fn(&OsStr, &mut Ctx) -> Result<()>;
pub type OnFlag<Ctx> = fn(&mut Ctx) -> Result<()>;

/// Option (flag or value-bearing).
pub struct OptSpec<'a, Ctx: ?Sized> {
    name: &'a str,
    short: Option<char>,
    long: Option<&'a str>,
    metavar: Option<&'a str>,
    help: Option<&'a str>,
    env: Option<&'a str>,
    default: Option<OsString>,
    group: Option<&'a str>,
    repeat: Repeat,
    takes_value: bool,
    on_value: Option<OnValue<Ctx>>, // value setter
    on_flag: Option<OnFlag<Ctx>>,   // flag setter
    validator: Option<ValueValidator>,
}

impl<'a, Ctx: ?Sized> OptSpec<'a, Ctx> {
    /// Create a **flag** option. Other fields are set via builder methods.
    pub fn flag(name: &'a str, cb: OnFlag<Ctx>) -> Self {
        Self {
            name,
            short: None,
            long: None,
            metavar: None,
            help: None,
            env: None,
            default: None,
            group: None,
            repeat: Repeat::Single,
            takes_value: false,
            on_value: None,
            on_flag: Some(cb),
            validator: None,
        }
    }
    /// Create a **value** option. Other fields are set via builder methods.
    pub const fn value(name: &'a str, cb: OnValue<Ctx>) -> Self {
        Self {
            name,
            short: None,
            long: None,
            metavar: None,
            help: None,
            env: None,
            default: None,
            group: None,
            repeat: Repeat::Single,
            takes_value: true,
            on_value: Some(cb),
            on_flag: None,
            validator: None,
        }
    }
    // --- builders ---
    #[must_use]
    pub const fn short(mut self, s: char) -> Self {
        self.short = Some(s);
        self
    }
    #[must_use]
    pub const fn long(mut self, l: &'a str) -> Self {
        self.long = Some(l);
        self
    }
    #[must_use]
    pub const fn metavar(mut self, mv: &'a str) -> Self {
        self.metavar = Some(mv);
        self
    }
    #[must_use]
    pub const fn help(mut self, h: &'a str) -> Self {
        self.help = Some(h);
        self
    }
    #[must_use]
    pub const fn env(mut self, name: &'a str) -> Self {
        self.env = Some(name);
        self
    }
    #[must_use]
    pub fn default_os(mut self, val: impl Into<OsString>) -> Self {
        self.default = Some(val.into());
        self
    }
    #[must_use]
    pub const fn group(mut self, g: &'a str) -> Self {
        self.group = Some(g);
        self
    }
    #[must_use]
    pub const fn single(mut self) -> Self {
        self.repeat = Repeat::Single;
        self
    }
    #[must_use]
    pub const fn repeatable(mut self) -> Self {
        self.repeat = Repeat::Many;
        self
    }
    #[must_use]
    pub const fn validator(mut self, v: ValueValidator) -> Self {
        self.validator = Some(v);
        self
    }

    // --- getters (get_*; booleans use is_*) ---
    #[must_use]
    pub const fn get_name(&self) -> &str {
        self.name
    }
    #[must_use]
    pub const fn get_short(&self) -> Option<char> {
        self.short
    }
    #[must_use]
    pub const fn get_long(&self) -> Option<&str> {
        self.long
    }
    #[must_use]
    pub const fn get_metavar(&self) -> Option<&str> {
        self.metavar
    }
    #[must_use]
    pub const fn get_help(&self) -> Option<&str> {
        self.help
    }
    #[must_use]
    pub const fn get_env(&self) -> Option<&str> {
        self.env
    }
    #[must_use]
    pub const fn get_default(&self) -> Option<&OsString> {
        self.default.as_ref()
    }
    #[must_use]
    pub const fn get_group(&self) -> Option<&str> {
        self.group
    }
    #[must_use]
    pub const fn is_value(&self) -> bool {
        self.takes_value
    }
    #[must_use]
    pub const fn get_repeat(&self) -> Repeat {
        self.repeat
    }
    #[must_use]
    pub fn get_on_value(&self) -> Option<OnValue<Ctx>> {
        self.on_value
    }
    #[must_use]
    pub fn get_on_flag(&self) -> Option<OnFlag<Ctx>> {
        self.on_flag
    }
    #[must_use]
    pub fn get_validator(&self) -> Option<ValueValidator> {
        self.validator
    }
}

/// Positional cardinality.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PosCardinality {
    One { required: bool },
    Many,
    Range { min: usize, max: usize },
}

/// Positional argument specification.
pub struct PosSpec<'a, Ctx: ?Sized> {
    name: &'a str,
    help: Option<&'a str>,
    card: PosCardinality,
    on_value: OnValue<Ctx>,
    validator: Option<ValueValidator>,
}
impl<'a, Ctx: ?Sized> PosSpec<'a, Ctx> {
    pub const fn new(name: &'a str, cb: OnValue<Ctx>) -> Self {
        Self {
            name,
            help: None,
            card: PosCardinality::One { required: false },
            on_value: cb,
            validator: None,
        }
    }
    // builders
    #[must_use]
    pub const fn help(mut self, h: &'a str) -> Self {
        self.help = Some(h);
        self
    }
    #[must_use]
    pub const fn required(mut self) -> Self {
        self.card = PosCardinality::One { required: true };
        self
    }
    #[must_use]
    pub const fn many(mut self) -> Self {
        self.card = PosCardinality::Many;
        self
    }
    #[must_use]
    pub const fn range(mut self, min: usize, max: usize) -> Self {
        self.card = PosCardinality::Range { min, max };
        self
    }
    #[must_use]
    pub const fn validator(mut self, v: ValueValidator) -> Self {
        self.validator = Some(v);
        self
    }
    // getters
    #[must_use]
    pub const fn get_name(&self) -> &str {
        self.name
    }
    #[must_use]
    pub const fn get_help(&self) -> Option<&str> {
        self.help
    }
    #[must_use]
    pub const fn get_cardinality(&self) -> PosCardinality {
        self.card
    }
    #[must_use]
    pub const fn is_required(&self) -> bool {
        matches!(self.card, PosCardinality::One { required: true })
            || matches!(self.card, PosCardinality::Range { min, .. } if min > 0)
    }
    #[must_use]
    pub const fn is_multiple(&self) -> bool {
        !matches!(self.card, PosCardinality::One { .. })
    }
    #[must_use]
    pub const fn get_on_value(&self) -> OnValue<Ctx> {
        self.on_value
    }
    #[must_use]
    pub const fn get_validator(&self) -> Option<ValueValidator> {
        self.validator
    }
}

/// Group declaration.
pub struct GroupDecl<'a> {
    pub name: &'a str,
    pub mode: GroupMode,
}

/// Command specification.
pub struct CmdSpec<'a, Ctx: ?Sized> {
    name: &'a str,
    help: Option<&'a str>,
    aliases: Vec<&'a str>,
    opts: Vec<OptSpec<'a, Ctx>>,
    positionals: Vec<PosSpec<'a, Ctx>>,
    subcommands: Vec<CmdSpec<'a, Ctx>>,
    groups: Vec<GroupDecl<'a>>,
    validate_cmd: Option<CmdValidator>,
    handler: Option<CmdHandler<Ctx>>, // leaf command handler
}
impl<'a, Ctx: ?Sized> CmdSpec<'a, Ctx> {
    #[must_use]
    pub fn new(name: &'a str) -> Self {
        Self {
            name,
            help: None,
            aliases: Vec::new(),
            opts: Vec::new(),
            positionals: Vec::new(),
            subcommands: Vec::new(),
            groups: Vec::new(),
            validate_cmd: None,
            handler: None,
        }
    }
    // builders
    #[must_use]
    pub const fn help(mut self, s: &'a str) -> Self {
        self.help = Some(s);
        self
    }
    #[must_use]
    pub fn alias(mut self, a: &'a str) -> Self {
        self.aliases.push(a);
        self
    }
    #[must_use]
    pub fn opt(mut self, o: OptSpec<'a, Ctx>) -> Self {
        self.opts.push(o);
        self
    }
    #[must_use]
    pub fn pos(mut self, p: PosSpec<'a, Ctx>) -> Self {
        self.positionals.push(p);
        self
    }
    #[must_use]
    pub fn subcmd(mut self, c: Self) -> Self {
        self.subcommands.push(c);
        self
    }
    #[must_use]
    pub fn group(mut self, name: &'a str, mode: GroupMode) -> Self {
        self.groups.push(GroupDecl { name, mode });
        self
    }
    /// Set per-command validator (renamed to `validator` for consistency).
    #[must_use]
    pub fn validator(mut self, cb: CmdValidator) -> Self {
        self.validate_cmd = Some(cb);
        self
    }
    /// Set a leaf command handler. Only the **selected leaf** handler is executed.
    #[must_use]
    pub fn handler(mut self, cb: CmdHandler<Ctx>) -> Self {
        self.handler = Some(cb);
        self
    }
    // getters
    #[must_use]
    pub const fn get_name(&self) -> &str {
        self.name
    }
    #[must_use]
    pub const fn get_help(&self) -> Option<&str> {
        self.help
    }
    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn get_aliases(&self) -> &[&'a str] {
        &self.aliases
    }
    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn get_opts(&self) -> &[OptSpec<'a, Ctx>] {
        &self.opts
    }
    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn get_positionals(&self) -> &[PosSpec<'a, Ctx>] {
        &self.positionals
    }
    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn get_subcommands(&self) -> &[Self] {
        &self.subcommands
    }
    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn get_groups(&self) -> &[GroupDecl<'a>] {
        &self.groups
    }
    #[must_use]
    pub fn get_validator(&self) -> Option<CmdValidator> {
        self.validate_cmd
    }
    #[must_use]
    pub fn get_handler(&self) -> Option<CmdHandler<Ctx>> {
        self.handler
    }
    #[must_use]
    pub fn find_sub(&self, needle: &str) -> Option<&Self> {
        self.subcommands.iter().find(|c| c.name == needle || c.aliases.iter().any(|a| *a == needle))
    }
}
