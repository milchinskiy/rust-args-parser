//! Check command declarations before parsing, independent of the selected input path.
use crate::spec::PosCardinality;
use crate::{CmdSpec, Env, Error, Result};
use std::collections::HashSet;

/// Validate the complete command tree before parsing or invoking callbacks.
///
/// # Errors
/// Returns `Error::Parse` when any command declaration is ambiguous or invalid.
pub fn validate_spec<Ctx: ?Sized>(env: &Env, cmd: &CmdSpec<'_, Ctx>) -> Result<()> {
    validate_command(env, cmd, true)
}

fn validate_command<Ctx: ?Sized>(env: &Env, cmd: &CmdSpec<'_, Ctx>, root: bool) -> Result<()> {
    validate_options(env, cmd, root)?;
    validate_positionals(cmd)?;
    validate_groups(cmd)?;
    validate_subcommands(env, cmd)
}

fn validate_options<Ctx: ?Sized>(env: &Env, cmd: &CmdSpec<'_, Ctx>, root: bool) -> Result<()> {
    let mut names = HashSet::new();
    let mut shorts = HashSet::new();
    let mut longs = HashSet::new();
    for opt in cmd.get_opts() {
        if !names.insert(opt.get_name()) {
            return Err(Error::Parse(format!("duplicate option '{}' in '{}'", opt.get_name(), cmd.get_name())));
        }
        if opt.get_name().is_empty() {
            return Err(Error::Parse(format!("empty option name in '{}'", cmd.get_name())));
        }
        if !opt.is_value() && opt.get_default().is_some() {
            return Err(Error::Parse(format!(
                "flag '{}' cannot have a value default in '{}'",
                opt.get_name(),
                cmd.get_name()
            )));
        }
        if let Some(short) = opt.get_short() {
            if !shorts.insert(short) {
                return Err(Error::Parse(format!("duplicate short option '-{short}' in '{}'", cmd.get_name())));
            }
            if short == '-'
                || (env.auto_help && short == 'h')
                || (root && env.version.is_some() && short == 'V')
                || (root && env.author.is_some() && short == 'A')
            {
                return Err(Error::Parse(format!("reserved short option '-{short}' in '{}'", cmd.get_name())));
            }
        }
        if let Some(long) = opt.get_long() {
            if !longs.insert(long) {
                return Err(Error::Parse(format!("duplicate long option '--{long}' in '{}'", cmd.get_name())));
            }
            if long.is_empty() || long.contains('=') || long.starts_with('-') {
                return Err(Error::Parse(format!("invalid long option '--{long}' in '{}'", cmd.get_name())));
            }
            if (env.auto_help && long == "help")
                || (root && env.version.is_some() && long == "version")
                || (root && env.author.is_some() && long == "author")
            {
                return Err(Error::Parse(format!("reserved long option '--{long}' in '{}'", cmd.get_name())));
            }
        }
    }

    Ok(())
}

fn validate_positionals<Ctx: ?Sized>(cmd: &CmdSpec<'_, Ctx>) -> Result<()> {
    let mut positionals = HashSet::new();
    for (i, pos) in cmd.get_positionals().iter().enumerate() {
        if !positionals.insert(pos.get_name()) {
            return Err(Error::Parse(format!("duplicate positional '{}' in '{}'", pos.get_name(), cmd.get_name())));
        }
        if pos.get_name().is_empty() {
            return Err(Error::Parse(format!("empty positional name in '{}'", cmd.get_name())));
        }
        match pos.get_cardinality() {
            PosCardinality::Many if i + 1 < cmd.get_positionals().len() => {
                return Err(Error::Parse(format!(
                    "unbounded positional '{}' must be last in '{}'",
                    pos.get_name(),
                    cmd.get_name()
                )));
            }
            PosCardinality::Range { min, max } if max == 0 || min > max => {
                return Err(Error::Parse(format!(
                    "invalid positional range for '{}' in '{}': {min}..={max}",
                    pos.get_name(),
                    cmd.get_name()
                )));
            }
            PosCardinality::One { .. } | PosCardinality::Many | PosCardinality::Range { .. } => {}
        }
    }
    Ok(())
}

fn validate_groups<Ctx: ?Sized>(cmd: &CmdSpec<'_, Ctx>) -> Result<()> {
    let mut groups = HashSet::new();
    for group in cmd.get_groups() {
        if group.name.is_empty() {
            return Err(Error::Parse(format!("empty group name in '{}'", cmd.get_name())));
        }
        if !groups.insert(group.name) {
            return Err(Error::Parse(format!("duplicate group '{}' in '{}'", group.name, cmd.get_name())));
        }
    }
    for opt in cmd.get_opts() {
        if let Some(group) = opt.get_group().filter(|group| !groups.contains(*group)) {
            return Err(Error::Parse(format!(
                "undeclared group '{group}' for option '{}' in '{}'",
                opt.get_name(),
                cmd.get_name()
            )));
        }
    }
    for group in cmd.get_groups() {
        if !cmd.get_opts().iter().any(|opt| opt.get_group() == Some(group.name)) {
            return Err(Error::Parse(format!("group '{}' has no options in '{}'", group.name, cmd.get_name())));
        }
    }

    Ok(())
}

fn validate_subcommands<Ctx: ?Sized>(env: &Env, cmd: &CmdSpec<'_, Ctx>) -> Result<()> {
    let mut subcommands = HashSet::new();
    for sub in cmd.get_subcommands() {
        for name in std::iter::once(sub.get_name()).chain(sub.get_aliases().iter().copied()) {
            if name.is_empty() || name.starts_with('-') {
                return Err(Error::Parse(format!("invalid subcommand name or alias '{name}' in '{}'", cmd.get_name())));
            }
            if !subcommands.insert(name) {
                return Err(Error::Parse(format!(
                    "duplicate subcommand name or alias '{name}' in '{}'",
                    cmd.get_name()
                )));
            }
        }
        validate_command(env, sub, false)?;
    }
    Ok(())
}
