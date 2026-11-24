use crate::util::strip_ansi_len;
use crate::{CmdSpec, Env};
use core::fmt::Write;

#[cfg(feature = "color")]
fn use_color(env: &Env) -> bool {
    use crate::ColorMode;
    use std::io::IsTerminal;

    match env.color {
        ColorMode::Always => true,
        ColorMode::Never => false,
        ColorMode::Auto => std::io::stdout().is_terminal() && std::env::var_os("NO_COLOR").is_none(),
    }
}

#[cfg(not(feature = "color"))]
const fn use_color(_: &Env) -> bool {
    false
}

fn paint(s: &str, prefix: &str, colored: bool) -> String {
    if colored {
        format!("{prefix}{s}\x1b[0m")
    } else {
        s.to_owned()
    }
}

fn paint_title(s: &str, colored: bool) -> String {
    paint(&format!("{s}:"), "\x1b[1m\x1b[4;37m", colored)
}

fn paint_section(s: &str, colored: bool) -> String {
    paint(&format!("{s}:"), "\x1b[4;37m", colored)
}

fn paint_option(s: &str, colored: bool) -> String {
    paint(s, "\x1b[0;94m", colored)
}

fn paint_positional(s: &str, colored: bool) -> String {
    paint(s, "\x1b[0;93m", colored)
}

fn paint_metavar(s: &str, colored: bool) -> String {
    paint(s, "\x1b[0;96m", colored)
}

fn paint_command(s: &str, colored: bool) -> String {
    paint(s, "\x1b[0;95m", colored)
}

fn paint_bin(s: &str, colored: bool) -> String {
    paint(s, "\x1b[0;97m\x1b[1m", colored)
}

#[must_use]
pub fn render_help<Ctx: ?Sized>(env: &Env, cmd: &CmdSpec<'_, Ctx>) -> String {
    render_help_with_path(env, &[cmd.get_name()], cmd)
}

fn print_usage<Ctx: ?Sized>(out_buf: &mut String, path: &[&str], cmd: &CmdSpec<'_, Ctx>, colored: bool) {
    use crate::spec::PosCardinality;

    let mut out = String::new();
    let is_root = path.len() <= 1;
    let _ = writeln!(out, "{}", paint_title("Usage", colored));
    let bin_name = path.first().copied().unwrap_or_else(|| cmd.get_name());
    let _ = write!(out, "  {}", paint_bin(bin_name, colored));
    for command in path.iter().skip(1) {
        let _ = write!(out, " {}", paint_command(command, colored));
    }
    if !cmd.get_opts().is_empty() || is_root {
        let _ = write!(out, " {}", paint_option("[options]", colored));
    }
    if !cmd.get_subcommands().is_empty() {
        let _ = write!(out, " {}", paint_command("<command>", colored));
    }
    for positional in cmd.get_positionals() {
        let name = positional.get_name();
        let (required, multiple) = match positional.get_cardinality() {
            PosCardinality::One { .. } => (positional.is_required(), false),
            PosCardinality::Many => (positional.is_required(), true),
            PosCardinality::Range { min, max } => (min > 0, max > 1),
        };
        let token = if multiple { format!("{name}...") } else { name.to_owned() };
        let label = if required { format!("<{token}>") } else { format!("[{token}]") };
        let _ = write!(out, " {}", paint_positional(&label, colored));
    }
    let _ = writeln!(out_buf, "{out}\n");
}

/// Render help with **strict column alignment** based on the *longest* label in the section.
#[allow(clippy::too_many_lines)]
#[must_use]
pub fn render_help_with_path<Ctx: ?Sized>(env: &Env, path: &[&str], cmd: &CmdSpec<'_, Ctx>) -> String {
    let mut out = String::new();
    if let Some(h) = cmd.get_help() {
        let _ = writeln!(out, "{h}\n");
    }

    let colored = use_color(env);
    print_usage(&mut out, path, cmd, colored);
    let mut rows: Vec<(Vec<String>, Option<&str>, String)> = Vec::new();
    let is_root = path.len() <= 1;

    if env.auto_help {
        rows.push((vec!["-h".into(), "--help".into()], None, String::from("Show this help and exit")));
    }

    if is_root {
        if env.version.is_some() {
            rows.push((vec!["-V".into(), "--version".into()], None, String::from("Show version and exit")));
        }
        if env.author.is_some() {
            rows.push((vec!["-A".into(), "--author".into()], None, String::from("Show author and exit")));
        }
    }

    // User‑defined options
    for o in cmd.get_opts() {
        let mut lab = vec![];
        let mut meta: Option<&str> = None;
        if let Some(s) = o.get_short() {
            lab.push(format!("-{s}"));
        }
        if let Some(l) = o.get_long() {
            lab.push(format!("--{l}"));
        }
        if let Some(mv) = o.get_metavar() {
            meta = Some(mv);
        }
        let mut desc: Vec<String> = vec![];
        if let Some(h) = o.get_help() {
            desc.push(h.to_string());
        }
        if let Some(env) = o.get_env() {
            desc.push(format!("Env: {env}"));
        }
        if let Some(d) = o.get_default() {
            desc.push(format!("Default: {d:?}"));
        }
        let desc = desc.join("; ");
        rows.push((lab, meta, desc));
    }

    if !rows.is_empty() {
        let _ = writeln!(out, "{}", paint_section("Options", colored));
        let max_raw =
            rows.iter().map(|(opts, pos, _)| opts.join(", ").len() + pos.map_or(0, |s| s.len() + 1)).max().unwrap_or(0);
        let desc_col = 2 + max_raw + 2; // "  " + label + "  "
        for (lab, pos, desc) in rows {
            let mut painted = lab.into_iter().map(|s| paint_option(&s, colored)).collect::<Vec<String>>().join(", ");
            if let Some(pos) = pos {
                painted.push_str(format!(" {}", paint_metavar(pos, colored)).as_str());
            }
            let raw = strip_ansi_len(&painted);
            let pad = max_raw + (painted.len() - raw);
            let _ = write!(out, "  {painted:pad$}  ");
            wrap_after(&mut out, &desc, desc_col, env.wrap_cols);
        }
    }
    // Arguments
    if !cmd.get_positionals().is_empty() {
        let _ = writeln!(out, "\n{}", paint_section("Arguments", colored));
        let mut prow_labels: Vec<(String, usize, String)> = Vec::new();
        let mut max_raw = 0usize;
        for p in cmd.get_positionals() {
            let lab = paint_positional(p.get_name(), colored);
            let raw = strip_ansi_len(&lab);
            max_raw = max_raw.max(raw);
            prow_labels.push((lab, raw, p.get_help().unwrap_or("").to_string()));
        }
        let desc_col = 2 + max_raw + 2;
        for (lab, raw, desc) in prow_labels {
            let pad = max_raw + (lab.len() - raw);
            let _ = write!(out, "  {lab:pad$}  ");
            wrap_after(&mut out, &desc, desc_col, env.wrap_cols);
        }
    }
    // Commands
    if !cmd.get_subcommands().is_empty() {
        let _ = writeln!(out, "\n{}", paint_section("Commands", colored));
        let mut crow_labels: Vec<(String, usize, String)> = Vec::new();
        let mut max_raw = 0usize;
        for sc in cmd.get_subcommands() {
            let name = sc.get_name();
            let mut lab = vec![paint_command(name, colored)];
            for alias in sc.get_aliases() {
                lab.push(paint_command(alias, colored));
            }
            let lab = lab.join(", ");
            let raw = strip_ansi_len(&lab);
            max_raw = max_raw.max(raw);
            crow_labels.push((lab, raw, sc.get_help().unwrap_or("").to_string()));
        }
        let desc_col = 2 + max_raw + 2;
        for (lab, raw, desc) in crow_labels {
            let pad = max_raw + (lab.len() - raw);
            let _ = write!(out, "  {lab:pad$}  ");
            wrap_after(&mut out, &desc, desc_col, env.wrap_cols);
        }
    }
    out
}

/// Wrap `text` after the already‑printed label. Subsequent lines start at `start_col`.
fn wrap_after(out: &mut String, text: &str, start_col: usize, wrap: usize) {
    if text.is_empty() {
        let _ = writeln!(out);
        return;
    }
    if wrap == 0 {
        let _ = writeln!(out, "{text}");
        return;
    }
    let mut col = start_col;
    let mut first = true;
    for word in text.split_whitespace() {
        let wlen = word.len();
        let add = usize::from(!first);
        if col + add + wlen > wrap && col > start_col {
            let _ = writeln!(out);
            let _ = write!(out, "{}", " ".repeat(start_col));
            col = start_col;
            first = true;
        }
        if !first {
            let _ = write!(out, " ");
            col += 1;
        }
        let _ = write!(out, "{word}");
        col += wlen;
        first = false;
    }
    let _ = writeln!(out);
}
