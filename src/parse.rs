use crate::matches::{key_for, pos_key_for};
#[cfg(feature = "suggest")]
use crate::suggest::levenshtein;
use crate::util::looks_like_number_token;
use crate::{CmdSpec, Env, Error, GroupMode, Repeat, Result, Source};
use crate::{Matches, Status, Value};
use std::collections::HashMap;
use std::ffi::{OsStr, OsString};

/// Parse command line arguments.
/// # Errors [`Error`]
pub fn parse<'a, Ctx: ?Sized>(
    env: &Env,
    root: &'a CmdSpec<'a, Ctx>,
    argv: &[OsString],
    ctx: &mut Ctx,
) -> Result<Matches> {
    let mut m = Matches::new();
    let mut cursor = ParseCursor::new(root);
    cursor.eager_overlay_here(&mut m);
    let mut i = 0usize;
    while i < argv.len() {
        let tok = &argv[i];
        if !cursor.positional_only {
            if tok == "--" {
                i += 1;
                cursor.positional_only = true;
                continue;
            }
            if let Some(e) = try_handle_builtins(env, &cursor.stack, cursor.current, tok) {
                return Err(e);
            }
            if let Some(sub) = try_select_subcommand(cursor.current, tok) {
                cursor.descend(sub);
                i += 1;
                cursor.eager_overlay_here(&mut m);
                continue;
            }
            if let Some(consumed) =
                try_parse_long(env, cursor.current, &mut m, &cursor.path, &cursor.long_ix, argv, i)?
            {
                i += consumed;
                continue;
            }
            if let Some(consumed) = try_parse_short_or_numeric(
                env,
                cursor.current,
                &mut m,
                &cursor.path,
                &cursor.short_ix,
                argv,
                i,
            )? {
                i += consumed;
                continue;
            }
            if let Some(s) = tok.to_str() {
                if !s.starts_with('-')
                    && !cursor.current.get_subcommands().is_empty()
                    && cursor.current.get_positionals().get(cursor.pos_idx).is_none()
                {
                    return Err(unknown_command_error(env, s, cursor.current));
                }
            }
        }
        // Positional
        if let Some(consumed) = try_push_positional(
            cursor.current,
            &mut m,
            &cursor.path,
            &mut cursor.pos_idx,
            &mut cursor.pos_counts,
            tok,
        ) {
            i += consumed;
            continue;
        }
        return Err(Error::UnexpectedPositional { token: os_dbg(tok) });
    }

    walk_levels(&cursor.stack, |path, cmd| {
        overlay_env_and_defaults(&mut m, path, cmd);
        validate_level(&m, path, cmd)
    })?;
    walk_levels(&cursor.stack, |path, cmd| run_callbacks(&m, path, cmd, ctx))?;
    // Execute **leaf** command handler if any
    if let Some(leaf) = cursor.stack.last() {
        if let Some(h) = leaf.get_handler() {
            h(&m, ctx)?;
        }
    }
    m.set_leaf_path(&cursor.path);
    Ok(m)
}

// Unknown subcommand (with suggestions/aliases)
#[cfg(feature = "suggest")]
fn unknown_command_error<Ctx: ?Sized>(env: &Env, name: &str, cmd: &CmdSpec<'_, Ctx>) -> Error {
    let suggestions = if env.suggest {
        let mut cands: Vec<String> = Vec::new();
        for sc in cmd.get_subcommands() {
            cands.push(sc.get_name().to_string());
            for a in sc.get_aliases() {
                cands.push((*a).to_string());
            }
        }
        cands.sort();
        cands.dedup();
        best_suggestions(name, &cands)
    } else {
        vec![]
    };
    Error::UnknownCommand { token: name.to_string(), suggestions }
}
#[cfg(not(feature = "suggest"))]
fn unknown_command_error<Ctx: ?Sized>(_: &Env, name: &str, _: &CmdSpec<'_, Ctx>) -> Error {
    Error::UnknownCommand { token: name.to_string(), suggestions: vec![] }
}

fn try_handle_builtins<Ctx: ?Sized>(
    env: &Env,
    stack: &[&CmdSpec<'_, Ctx>],
    current: &CmdSpec<'_, Ctx>,
    tok: &OsString,
) -> Option<Error> {
    let s = tok.to_str()?;
    if env.auto_help && (s == "-h" || s == "--help") {
        #[cfg(feature = "help")]
        {
            let names: Vec<&str> = stack.iter().map(|c| c.get_name()).collect();
            let msg = crate::help::render_help_with_path(env, &names, current);
            return Some(Error::ExitMsg { code: 0, message: Some(msg) });
        }
        #[cfg(not(feature = "help"))]
        {
            let _ = current;
            return Some(Error::ExitMsg { code: 0, message: None });
        }
    }
    if stack.len() == 1 {
        if let Some(ver) = env.version {
            if s == "-V" || s == "--version" {
                return Some(Error::ExitMsg { code: 0, message: Some(ver.to_string()) });
            }
        }
        if let Some(auth) = env.author {
            if s == "-A" || s == "--author" {
                return Some(Error::ExitMsg { code: 0, message: Some(auth.to_string()) });
            }
        }
    }
    None
}

fn try_select_subcommand<'a, Ctx: ?Sized>(
    current: &'a CmdSpec<'a, Ctx>,
    tok: &OsString,
) -> Option<&'a CmdSpec<'a, Ctx>> {
    let s = tok.to_str()?;
    current.find_sub(s)
}

fn try_parse_long<'a, Ctx: ?Sized>(
    env: &Env,
    current: &CmdSpec<'a, Ctx>,
    m: &mut Matches,
    path: &[&str],
    long_ix: &HashMap<&'a str, usize>,
    argv: &[OsString],
    i: usize,
) -> Result<Option<usize>> {
    let Some(s) = argv[i].to_str() else { return Ok(None) };
    if !s.starts_with("--") {
        return Ok(None);
    }
    let body = &s[2..];
    let mut it = body.splitn(2, '=');
    let name = it.next().unwrap();
    let val_inline = it.next();

    let Some(&idx) = long_ix.get(name) else {
        return Err(unknown_long_error(env, name, current, path));
    };
    let opt = &current.get_opts()[idx];
    let key = key_for(path, opt.get_name());

    if opt.is_value() {
        let v = if let Some(v) = val_inline {
            OsString::from(v)
        } else {
            argv.get(i + 1).cloned().ok_or(Error::MissingValue { opt: format!("--{name}") })?
        };
        set_val(m, &key, v, Source::Cli, opt.get_repeat());
        Ok(Some(if val_inline.is_some() { 1 } else { 2 }))
    } else {
        set_flag(m, &key, Source::Cli);
        Ok(Some(1))
    }
}

fn try_parse_short_or_numeric<Ctx: ?Sized>(
    env: &Env,
    current: &CmdSpec<'_, Ctx>,
    m: &mut Matches,
    path: &[&str],
    short_ix: &HashMap<char, usize>,
    argv: &[OsString],
    i: usize,
) -> Result<Option<usize>> {
    let Some(s) = argv[i].to_str() else { return Ok(None) };
    let Some(rest) = s.strip_prefix('-') else { return Ok(None) };
    if rest.is_empty() {
        return Ok(None);
    }

    // Numeric fallback: if first char is not a known short and token looks numeric, treat as positional/value.
    let first = rest.chars().next().unwrap();
    if short_ix.get(&first).is_none() && looks_like_number_token(s) {
        return Ok(None);
    }

    // Cluster walk
    let mut chars = rest.chars().peekable();
    while let Some(c) = chars.next() {
        let Some(&idx) = short_ix.get(&c) else {
            return Err(unknown_short_error(env, c, current, path));
        };
        let opt = &current.get_opts()[idx];
        let key = key_for(path, opt.get_name());
        if opt.is_value() {
            if chars.peek().is_some() {
                let r: String = chars.collect();
                set_val(m, &key, OsString::from(r), Source::Cli, opt.get_repeat());
                return Ok(Some(1));
            }
            let v = argv.get(i + 1).cloned().ok_or(Error::MissingValue { opt: format!("-{c}") })?;
            set_val(m, &key, v, Source::Cli, opt.get_repeat());
            return Ok(Some(2));
        }
        set_flag(m, &key, Source::Cli);
    }
    Ok(Some(1))
}

fn try_push_positional<Ctx: ?Sized>(
    current: &CmdSpec<'_, Ctx>,
    m: &mut Matches,
    path: &[&str],
    pos_idx: &mut usize,
    pos_counts: &mut [usize],
    tok: &OsString,
) -> Option<usize> {
    let pos = current.get_positionals().get(*pos_idx)?;
    let key = pos_key_for(path, pos.get_name());
    push_pos(m, &key, tok.clone());
    pos_counts[*pos_idx] += 1;
    // advance if capacity reached
    match pos.get_cardinality() {
        crate::spec::PosCardinality::One { .. } => {
            *pos_idx += 1;
        }
        crate::spec::PosCardinality::Many => { /* stay */ }
        crate::spec::PosCardinality::Range { min: _, max } => {
            if pos_counts[*pos_idx] >= max {
                *pos_idx += 1;
            }
        }
    }
    Some(1)
}

fn rebuild_indexes<'a, Ctx: ?Sized>(
    cmd: &'a CmdSpec<'_, Ctx>,
    long: &mut HashMap<&'a str, usize>,
    short: &mut HashMap<char, usize>,
) {
    long.clear();
    short.clear();
    for (i, o) in cmd.get_opts().iter().enumerate() {
        if let Some(l) = o.get_long() {
            long.insert(l, i);
        }
        if let Some(s) = o.get_short() {
            short.insert(s, i);
        }
    }
}

fn eager_overlay<Ctx: ?Sized>(m: &mut Matches, path: &[&str], cmd: &CmdSpec<'_, Ctx>, src: Source) {
    for o in cmd.get_opts() {
        let k = key_for(path, o.get_name());
        if !m.status.contains_key(&k) {
            match src {
                Source::Env => {
                    if let Some(var) = o.get_env() {
                        if let Some(v) = std::env::var_os(var) {
                            if o.is_value() {
                                set_val(m, &k, v, Source::Env, o.get_repeat());
                            } else {
                                set_flag(m, &k, Source::Env);
                            }
                        }
                    }
                }
                Source::Default => {
                    if let Some(d) = o.get_default() {
                        set_val(m, &k, d.clone(), Source::Default, o.get_repeat());
                    }
                }
                Source::Cli => {}
            }
        }
    }
}

fn set_flag(m: &mut Matches, key: &str, src: Source) {
    *m.flag_counts.entry(key.to_string()).or_insert(0) += 1;
    m.values.insert(key.to_string(), Value::Flag);
    m.status.insert(key.to_string(), Status::Set(src));
}

fn set_val(m: &mut Matches, key: &str, val: OsString, src: Source, rep: Repeat) {
    match rep {
        Repeat::Single => {
            m.values.insert(key.to_string(), Value::One(val));
        }
        Repeat::Many => {
            m.values
                .entry(key.to_string())
                .and_modify(|v| {
                    if let Value::Many(vs) = v {
                        vs.push(val.clone());
                    }
                })
                .or_insert_with(|| Value::Many(vec![val]));
        }
    }
    m.status.insert(key.to_string(), Status::Set(src));
}
fn push_pos(m: &mut Matches, key: &str, val: OsString) {
    use crate::Value::{Flag, Many, One};
    match m.values.get_mut(key) {
        Some(Many(vs)) => vs.push(val),
        Some(One(_) | Flag) => {
            let old = m.values.remove(key).unwrap();
            if let One(s) = old {
                m.values.insert(key.to_string(), Many(vec![s, val]));
            }
        }
        None => {
            m.values.insert(key.to_string(), One(val));
        }
    }
    m.status.insert(key.to_string(), Status::Set(Source::Cli));
}

fn os_dbg(s: &OsStr) -> String {
    s.to_string_lossy().into_owned()
}

#[cfg(feature = "suggest")]
fn unknown_long_error<Ctx: ?Sized>(
    env: &Env,
    name: &str,
    cmd: &CmdSpec<'_, Ctx>,
    path: &[&str],
) -> Error {
    let suggestions = if env.suggest {
        let mut cands: Vec<String> = cmd
            .get_opts()
            .iter()
            .filter_map(|o| o.get_long().map(std::string::ToString::to_string))
            .collect();
        if path.is_empty() {
            if env.author.is_some() {
                cands.push("author".to_string());
            }
            if env.version.is_some() {
                cands.push("version".to_string());
            }
        }
        cands.push("help".to_string());
        cands.sort();
        best_suggestions(name, &cands).into_iter().map(|s| format!("--{s}")).collect()
    } else {
        vec![]
    };
    Error::UnknownOption { token: format!("--{name}"), suggestions }
}
#[cfg(not(feature = "suggest"))]
fn unknown_long_error<Ctx: ?Sized>(_: &Env, name: &str, _: &CmdSpec<'_, Ctx>, _: &[&str]) -> Error {
    Error::UnknownOption { token: format!("--{}", name), suggestions: vec![] }
}

#[cfg(feature = "suggest")]
fn unknown_short_error<Ctx: ?Sized>(
    env: &Env,
    c: char,
    cmd: &CmdSpec<'_, Ctx>,
    path: &[&str],
) -> Error {
    let suggestions = if env.suggest {
        let mut cands: Vec<String> =
            cmd.get_opts().iter().filter_map(|o| o.get_short().map(|s| s.to_string())).collect();
        if path.is_empty() {
            if env.author.is_some() {
                cands.push("A".into());
            }
            if env.version.is_some() {
                cands.push("V".into());
            }
        }
        cands.push("h".into());
        cands.sort();
        best_suggestions(&c.to_string(), &cands).into_iter().map(|s| format!("-{s}")).collect()
    } else {
        vec![]
    };
    Error::UnknownOption { token: format!("-{c}"), suggestions }
}
#[cfg(not(feature = "suggest"))]
fn unknown_short_error<Ctx: ?Sized>(_: &Env, c: char, _: &CmdSpec<'_, Ctx>, _: &[&str]) -> Error {
    Error::UnknownOption { token: format!("-{}", c), suggestions: vec![] }
}

#[cfg(feature = "suggest")]
fn best_suggestions(needle: &str, hay: &[String]) -> Vec<String> {
    let mut scored: Vec<(usize, String)> =
        hay.iter().map(|h| (levenshtein(needle, h), h.clone())).collect();
    scored.sort_by_key(|(d, _)| *d);
    scored.into_iter().filter(|(d, _)| *d <= 2).take(3).map(|(_, s)| s).collect()
}

/// Walk stack from root→leaf, yielding the *scoped* path (without root) and the cmd.
fn walk_levels<'a, Ctx, F>(stack: &[&'a CmdSpec<'a, Ctx>], mut f: F) -> Result<()>
where
    Ctx: ?Sized,
    F: FnMut(&[&'a str], &'a CmdSpec<'a, Ctx>) -> Result<()>,
{
    let mut path: Vec<&'a str> = Vec::with_capacity(stack.len().saturating_sub(1));
    for (idx, cmd) in stack.iter().enumerate() {
        if idx > 0 {
            path.push(cmd.get_name());
        }
        f(&path, cmd)?;
    }
    Ok(())
}

fn overlay_env_and_defaults<Ctx: ?Sized>(m: &mut Matches, path: &[&str], cmd: &CmdSpec<'_, Ctx>) {
    eager_overlay(m, path, cmd, crate::Source::Env);
    eager_overlay(m, path, cmd, crate::Source::Default);
}

fn validate_level<'a, Ctx: ?Sized>(
    m: &Matches,
    path: &[&'a str],
    cmd: &CmdSpec<'a, Ctx>,
) -> Result<()> {
    use crate::spec::PosCardinality;
    use crate::Value;

    // Positionals: required + Range{min} check
    for p in cmd.get_positionals() {
        let k = pos_key_for(path, p.get_name());
        if p.get_cardinality() == (PosCardinality::One { required: true })
            && !m.values.contains_key(&k)
        {
            return Err(Error::User("missing required positional".into()));
        }
        if let PosCardinality::Range { min, .. } = p.get_cardinality() {
            let count = match m.values.get(&k) {
                Some(Value::One(_)) => 1,
                Some(Value::Many(vs)) => vs.len(),
                _ => 0,
            };
            if count < min {
                return Err(Error::User("positional count below minimum".into()));
            }
        }
    }

    // Groups: Xor/ReqOne like in your code
    for g in cmd.get_groups() {
        let mut hits = 0u32;
        for o in cmd.get_opts() {
            if o.get_group() == Some(g.name) && m.status.contains_key(&key_for(path, o.get_name()))
            {
                hits += 1;
            }
        }
        match g.mode {
            GroupMode::Xor if hits > 1 => {
                return Err(Error::User(format!(
                    "options in group '{}' are mutually exclusive",
                    g.name
                )))
            }
            GroupMode::ReqOne if hits == 0 => {
                return Err(Error::User(format!(
                    "one of the options in group '{}' is required",
                    g.name
                )))
            }
            _ => {}
        }
    }

    // Option validators
    for o in cmd.get_opts() {
        if let Some(vf) = o.get_validator() {
            match m.values.get(&key_for(path, o.get_name())) {
                Some(Value::One(v)) => vf(v.as_os_str())?,
                Some(Value::Many(vs)) => {
                    for v in vs {
                        vf(v.as_os_str())?;
                    }
                }
                _ => {}
            }
        }
    }

    // Positional validators
    for p in cmd.get_positionals() {
        if let Some(vf) = p.get_validator() {
            match m.values.get(&pos_key_for(path, p.get_name())) {
                Some(Value::One(v)) => vf(v.as_os_str())?,
                Some(Value::Many(vs)) => {
                    for v in vs {
                        vf(v.as_os_str())?;
                    }
                }
                _ => {}
            }
        }
    }

    // Command-level validator
    if let Some(cv) = cmd.get_validator() {
        cv(m)?;
    }

    Ok(())
}

fn run_callbacks<'a, Ctx: ?Sized>(
    m: &Matches,
    path: &[&'a str],
    cmd: &CmdSpec<'a, Ctx>,
    ctx: &mut Ctx,
) -> Result<()> {
    use crate::Value;

    // options
    for o in cmd.get_opts() {
        let k = key_for(path, o.get_name());
        match m.values.get(&k) {
            Some(Value::Flag) => {
                if let Some(cb) = o.get_on_flag() {
                    let n = *m.flag_counts.get(&k).unwrap_or(&1);
                    for _ in 0..n {
                        cb(ctx)?;
                    }
                }
            }
            Some(Value::One(v)) => {
                if let Some(cb) = o.get_on_value() {
                    cb(v.as_os_str(), ctx)?;
                }
            }
            Some(Value::Many(vs)) => {
                if let Some(cb) = o.get_on_value() {
                    for v in vs {
                        cb(v.as_os_str(), ctx)?;
                    }
                }
            }
            None => {}
        }
    }

    // positionals
    for p in cmd.get_positionals() {
        let k = pos_key_for(path, p.get_name());
        match m.values.get(&k) {
            Some(Value::One(v)) => (p.get_on_value())(v.as_os_str(), ctx)?,
            Some(Value::Many(vs)) => {
                for v in vs {
                    (p.get_on_value())(v.as_os_str(), ctx)?;
                }
            }
            _ => {}
        }
    }

    Ok(())
}

struct ParseCursor<'a, Ctx: ?Sized> {
    path: Vec<&'a str>,
    stack: Vec<&'a CmdSpec<'a, Ctx>>,
    current: &'a CmdSpec<'a, Ctx>,
    long_ix: HashMap<&'a str, usize>,
    short_ix: HashMap<char, usize>,
    positional_only: bool,
    pos_idx: usize,
    pos_counts: Vec<usize>,
}

impl<'a, Ctx: ?Sized> ParseCursor<'a, Ctx> {
    fn new(root: &'a CmdSpec<'a, Ctx>) -> Self {
        let mut cur = Self {
            path: Vec::new(),
            stack: vec![root],
            current: root,
            long_ix: HashMap::new(),
            short_ix: HashMap::new(),
            positional_only: false,
            pos_idx: 0,
            pos_counts: vec![0; root.get_positionals().len()],
        };
        rebuild_indexes(cur.current, &mut cur.long_ix, &mut cur.short_ix);
        cur
    }
    fn rebuild_indexes(&mut self) {
        rebuild_indexes(self.current, &mut self.long_ix, &mut self.short_ix);
    }
    fn descend(&mut self, sub: &'a CmdSpec<'a, Ctx>) {
        self.stack.push(sub);
        self.path.push(sub.get_name());
        self.current = sub;
        self.positional_only = false;
        self.pos_idx = 0;
        self.pos_counts = vec![0; self.current.get_positionals().len()];
        self.rebuild_indexes();
    }
    fn eager_overlay_here(&self, m: &mut Matches) {
        eager_overlay(m, &self.path, self.current, Source::Env);
        eager_overlay(m, &self.path, self.current, Source::Default);
    }
}
