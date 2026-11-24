//! Typed command declarations and event dispatch.
//!
//! Parsing and validation are performed by the core parser. Events run on the
//! fully constructed typed command, in command-specification order.

use crate::{CmdSpec, Env, Matches, Result};
use std::ffi::{OsStr, OsString};
use std::path::PathBuf;
use std::str::FromStr;

/// Reusable fields in a command.
pub trait Args: Sized {
    /// Attach these arguments to the command specification.
    fn augment(cmd: CmdSpec<'static, ()>) -> CmdSpec<'static, ()>;

    /// Decode fields from validated matches.
    ///
    /// # Errors
    /// Returns an error if a required field is absent or a value cannot be decoded.
    fn from_matches(matches: &Matches, path: &[&str]) -> Result<Self>;

    #[doc(hidden)]
    ///
    /// # Errors
    /// Returns an error from an option callback.
    fn dispatch_options(&mut self, matches: &Matches, path: &[&str]) -> Result<()>;
    #[doc(hidden)]
    ///
    /// # Errors
    /// Returns an error from a positional callback.
    fn dispatch_positionals(&mut self, matches: &Matches, path: &[&str]) -> Result<()>;
    #[doc(hidden)]
    ///
    /// # Errors
    /// Returns an error from a selected subcommand callback.
    fn dispatch_children(&mut self, matches: &Matches, path: &[&str]) -> Result<()>;
    #[doc(hidden)]
    ///
    /// # Errors
    /// Returns an error from a command handler.
    fn dispatch_handler(&mut self, matches: &Matches, path: &[&str]) -> Result<()>;

    #[doc(hidden)]
    ///
    /// # Errors
    /// Returns an error from an argument or subcommand callback.
    fn dispatch_events(&mut self, matches: &Matches, path: &[&str]) -> Result<()> {
        self.dispatch_options(matches, path)?;
        self.dispatch_positionals(matches, path)?;
        self.dispatch_children(matches, path)
    }
}

/// Typed subcommand enum.
pub trait Subcommand: Sized {
    /// Attach variants as subcommands.
    fn augment_subcommands(cmd: CmdSpec<'static, ()>) -> CmdSpec<'static, ()>;

    /// Decode the selected variant, if any.
    ///
    /// # Errors
    /// Returns an error if fields in the selected variant cannot be decoded.
    fn from_matches(matches: &Matches, path: &[&str]) -> Result<Option<Self>>;

    #[doc(hidden)]
    ///
    /// # Errors
    /// Returns an error from an argument or subcommand callback.
    fn dispatch_events(&mut self, matches: &Matches, path: &[&str]) -> Result<()>;
    #[doc(hidden)]
    ///
    /// # Errors
    /// Returns an error from a command handler.
    fn dispatch_handler(&mut self, matches: &Matches, path: &[&str]) -> Result<()>;
}

/// Root typed command. The supplied arguments exclude the executable name.
///
/// All validation and typed decoding finish before any event handler runs.
/// Handlers modify the returned value. Earlier mutations are not rolled back
/// if a later handler fails.
pub trait Parser: Sized {
    /// Build the specification for the core parser.
    fn command() -> CmdSpec<'static, ()>;

    /// Decode the root command from validated matches.
    ///
    /// # Errors
    /// Returns an error if fields cannot be decoded.
    fn from_matches(matches: &Matches) -> Result<Self>;

    #[doc(hidden)]
    ///
    /// # Errors
    /// Returns an error from an argument or command handler.
    fn dispatch(&mut self, matches: &Matches) -> Result<()>;

    /// Parse and dispatch using custom parser settings.
    ///
    /// # Errors
    /// Returns an error from parsing, validation, typed decoding, or a handler.
    fn try_parse_from_with_env<I, S>(env: &Env, args: I) -> Result<Self>
    where
        I: IntoIterator<Item = S>,
        S: Into<OsString>,
    {
        let argv: Vec<OsString> = args.into_iter().map(Into::into).collect();
        let spec = Self::command();
        let matches = crate::parse(env, &spec, &argv, &mut ())?;
        let mut typed = Self::from_matches(&matches)?;
        typed.dispatch(&matches)?;
        Ok(typed)
    }

    /// Parse CLI arguments and dispatch handlers.
    ///
    /// # Errors
    /// Returns an error from parsing, validation, typed decoding, or a handler.
    fn try_parse_from<I, S>(args: I) -> Result<Self>
    where
        I: IntoIterator<Item = S>,
        S: Into<OsString>,
    {
        Self::try_parse_from_with_env(&Env::default(), args)
    }

    /// Parse `std::env::args_os().skip(1)` and dispatch handlers.
    ///
    /// # Errors
    /// Returns an error from parsing, validation, typed decoding, or a handler.
    fn try_parse() -> Result<Self> {
        Self::try_parse_from(std::env::args_os().skip(1))
    }
}

/// Decode a value using strict UTF-8 and `FromStr`.
/// `OsString` and `PathBuf` are decoded without UTF-8 conversion.
///
/// # Errors
/// Returns an error for non-UTF-8 text or a failed `FromStr` conversion.
pub fn parse_value<T>(value: &OsStr, name: &str) -> Result<T>
where
    T: FromStr,
    T::Err: std::fmt::Display,
{
    let s = value.to_str().ok_or_else(|| crate::Error::User(format!("invalid UTF-8 for '{name}'")))?;
    s.parse::<T>().map_err(|e| crate::Error::User(format!("invalid value for '{name}': {e}")))
}

#[must_use]
pub fn os_string(value: &OsStr) -> OsString {
    value.to_os_string()
}

#[must_use]
pub fn path_buf(value: &OsStr) -> PathBuf {
    PathBuf::from(value)
}
