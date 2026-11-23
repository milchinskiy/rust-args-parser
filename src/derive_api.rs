//! Optional typed interface backed by `CmdSpec` and `Matches`.
//! The core parser remains responsible for validation and event dispatch.

use crate::{CmdSpec, Env, Error, Matches, Result};
use std::ffi::{OsStr, OsString};
use std::path::PathBuf;
use std::str::FromStr;

/// Add named fields to a command and decode their values.
/// `Ctx` is the same callback context accepted by the core parser.
pub trait Args<Ctx: ?Sized = ()>: Sized {
    fn augment(cmd: CmdSpec<'static, Ctx>) -> CmdSpec<'static, Ctx>;
    /// # Errors
    fn from_matches(matches: &Matches, path: &[&str]) -> Result<Self>;
}

/// Declare and decode enum subcommands.
pub trait Subcommand<Ctx: ?Sized = ()>: Sized {
    fn augment_subcommands(cmd: CmdSpec<'static, Ctx>) -> CmdSpec<'static, Ctx>;
    /// # Errors
    fn from_matches(matches: &Matches, path: &[&str]) -> Result<Option<Self>>;
}

/// Parse into a typed structure while dispatching callbacks and the leaf handler.
///
/// Unlike `clap`, the iterator contains CLI arguments only (no executable name).
/// All typed decoding completes before any user callback/handler executes.
pub trait Parser: Sized {
    /// Context supplied to `#[arg(on_...)]` and `#[command(handler...)]` callbacks.
    type Context;

    fn command() -> CmdSpec<'static, Self::Context>;
    /// # Errors
    fn from_matches(matches: &Matches) -> Result<Self>;

    /// Parse and dispatch using a caller-owned context.
    /// # Errors
    fn try_parse_from_with_context_and_env<I, S>(env: &Env, args: I, ctx: &mut Self::Context) -> Result<Self>
    where
        I: IntoIterator<Item = S>,
        S: Into<OsString>,
    {
        let argv: Vec<OsString> = args.into_iter().map(Into::into).collect();
        let spec = Self::command();
        let mut typed = None;
        crate::parse::parse_checked(env, &spec, &argv, ctx, |matches| {
            typed = Some(Self::from_matches(matches)?);
            Ok(())
        })?;
        // parse_checked invokes the decoder once on success.
        typed.ok_or_else(|| Error::Parse("derive decoding was not performed".into()))
    }

    /// Parse arguments and dispatch events into `ctx` with default settings.
    /// # Errors
    fn try_parse_from_with_context<I, S>(args: I, ctx: &mut Self::Context) -> Result<Self>
    where
        I: IntoIterator<Item = S>,
        S: Into<OsString>,
    {
        Self::try_parse_from_with_context_and_env(&Env::default(), args, ctx)
    }

    /// Parse using a default-constructed context.
    /// # Errors
    fn try_parse_from_with_env<I, S>(env: &Env, args: I) -> Result<Self>
    where
        I: IntoIterator<Item = S>,
        S: Into<OsString>,
        Self::Context: Default,
    {
        Self::try_parse_from_with_context_and_env(env, args, &mut Self::Context::default())
    }

    /// Parse arguments with a default context and environment.
    /// # Errors
    fn try_parse_from<I, S>(args: I) -> Result<Self>
    where
        I: IntoIterator<Item = S>,
        S: Into<OsString>,
        Self::Context: Default,
    {
        Self::try_parse_from_with_env(&Env::default(), args)
    }

    /// Read `std::env::args_os().skip(1)` with default context/environment.
    /// # Errors
    fn try_parse() -> Result<Self>
    where
        Self::Context: Default,
    {
        Self::try_parse_from(std::env::args_os().skip(1))
    }
}

/// Strict UTF-8 decoding for `FromStr` values. Paths and `OsString` are byte-safe.
/// # Errors
pub fn parse_value<T>(value: &OsStr, name: &str) -> Result<T>
where
    T: FromStr,
    T::Err: std::fmt::Display,
{
    let s = value.to_str().ok_or_else(|| Error::User(format!("invalid UTF-8 for '{name}'")))?;
    s.parse::<T>().map_err(|e| Error::User(format!("invalid value for '{name}': {e}")))
}

#[must_use]
pub fn os_string(value: &OsStr) -> OsString {
    value.to_os_string()
}

#[must_use]
pub fn path_buf(value: &OsStr) -> PathBuf {
    PathBuf::from(value)
}
