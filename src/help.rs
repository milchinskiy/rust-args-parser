use crate::util::strip_ansi_len;
use crate::{CmdSpec, Env};
use core::fmt::Write;

#[cfg(feature = "color")]
mod ansi {
    pub const RESET: &str = "\x1b[0m";
    pub const BOLD: &str = "\x1b[1m";
    pub const TITLE: &str = "\x1b[4;37m"; // section titles
    pub const OPT_LABEL: &str = "\x1b[0;94m"; // option labels
    pub const POS_LABEL: &str = "\x1b[0;93m"; // positional labels
    pub const METAVAR: &str = "\x1b[0;96m"; // metavars
    pub const COMMAND: &str = "\x1b[0;95m"; // command names
    pub const BRIGHT_WHITE: &str = "\x1b[0;97m";
}

#[cfg(feature = "color")]
fn paint_title(s: &str) -> String {
    format!("{}{}{}{}:", ansi::BOLD, ansi::TITLE, s, ansi::RESET)
}
#[cfg(not(feature = "color"))]
fn paint_title(s: &str) -> String {
    s.to_string()
}

#[cfg(feature = "color")]
fn paint_section(s: &str) -> String {
    format!("{}{}{}:", ansi::TITLE, s, ansi::RESET)
}
#[cfg(not(feature = "color"))]
fn paint_section(s: &str) -> String {
    s.to_string()
}

#[cfg(feature = "color")]
fn paint_option(s: &str) -> String {
    format!("{}{}{}", ansi::OPT_LABEL, s, ansi::RESET)
}
#[cfg(not(feature = "color"))]
fn paint_option(s: &str) -> String {
    s.to_string()
}

#[cfg(feature = "color")]
fn paint_positional(s: &str) -> String {
    format!("{}{}{}", ansi::POS_LABEL, s, ansi::RESET)
}
#[cfg(not(feature = "color"))]
fn paint_positional(s: &str) -> String {
    s.to_string()
}

#[cfg(feature = "color")]
fn paint_metavar(s: &str) -> String {
    format!("{}{}{}", ansi::METAVAR, s, ansi::RESET)
}
#[cfg(not(feature = "color"))]
fn paint_metavar(s: &str) -> String {
    s.to_string()
}

#[cfg(feature = "color")]
fn paint_command(s: &str) -> String {
    format!("{}{}{}", ansi::COMMAND, s, ansi::RESET)
}
#[cfg(not(feature = "color"))]
fn paint_command(s: &str) -> String {
    s.to_string()
}

#[must_use]
pub fn render_help<Ctx: ?Sized>(env: &Env, cmd: &CmdSpec<'_, Ctx>) -> String {
    render_help_with_path(env, &[], cmd)
}

fn print_usage<Ctx: ?Sized>(out_buf: &mut String, path: &[&str], cmd: &CmdSpec<'_, Ctx>) {
    use crate::spec::PosCardinality;
    let mut out = String::new();
    let is_root = path.len() <= 1;
    let _ = writeln!(out, "{}", paint_title("Usage"));
    let bin_name = (*path.first().unwrap_or(&"")).to_string();
    #[cfg(feature = "color")]
    let _ = write!(out, "  {}{}{}{}", ansi::BRIGHT_WHITE, ansi::BOLD, bin_name, ansi::RESET);
    #[cfg(not(feature = "color"))]
    let _ = write!(out, "  {}", bin_name);
    for command in path.iter().skip(1) {
        let _ = write!(out, " {}", paint_command(command));
    }
    if !cmd.get_opts().is_empty() || is_root {
        #[cfg(feature = "color")]
        let _ = write!(out, " {}[options]{}", ansi::OPT_LABEL, ansi::RESET);
        #[cfg(not(feature = "color"))]
        let _ = write!(out, " [options]");
    }
    if !cmd.get_subcommands().is_empty() {
        #[cfg(feature = "color")]
        let _ = write!(out, " {}<command>{}", ansi::COMMAND, ansi::RESET);
        #[cfg(not(feature = "color"))]
        let _ = write!(out, " <command>");
    }
    for p in cmd.get_positionals() {
        let name = p.get_name();
        let (req, ellip) = match p.get_cardinality() {
            PosCardinality::One { .. } => (p.is_required(), false),
            PosCardinality::Many => (p.is_required(), true),
            PosCardinality::Range { min, max } => (min > 0, max > 1),
        };
        let token = if ellip { format!("{name}...") } else { name.to_string() };
        if req {
            #[cfg(feature = "color")]
            let _ = write!(out, " {}<{token}>{}", ansi::POS_LABEL, ansi::RESET);
            #[cfg(not(feature = "color"))]
            let _ = write!(out, " <{token}>");
        } else {
            #[cfg(feature = "color")]
            let _ = write!(out, " {}[{token}]{}", ansi::POS_LABEL, ansi::RESET);
            #[cfg(not(feature = "color"))]
            let _ = write!(out, " [{token}]");
        }
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

    print_usage(&mut out, path, cmd);
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
        let _ = writeln!(out, "{}", paint_section("Options"));
        let max_raw =
            rows.iter().map(|(opts, pos, _)| opts.join(", ").len() + pos.map_or(0, |s| s.len() + 1)).max().unwrap_or(0);
        let desc_col = 2 + max_raw + 2; // "  " + label + "  "
        for (lab, pos, desc) in rows {
            let mut painted = lab.into_iter().map(|s| paint_option(&s)).collect::<Vec<String>>().join(", ");
            if let Some(pos) = pos {
                painted.push_str(format!(" {}", paint_metavar(pos)).as_str());
            }
            let raw = strip_ansi_len(&painted);
            let pad = max_raw + (painted.len() - raw);
            let _ = write!(out, "  {painted:pad$}  ");
            wrap_after(&mut out, &desc, desc_col, env.wrap_cols);
        }
    }
    // Arguments
    if !cmd.get_positionals().is_empty() {
        let _ = writeln!(out, "\n{}", paint_section("Arguments"));
        let mut prow_labels: Vec<(String, usize, String)> = Vec::new();
        let mut max_raw = 0usize;
        for p in cmd.get_positionals() {
            let lab = paint_positional(p.get_name());
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
        let _ = writeln!(out, "\n{}", paint_section("Commands"));
        let mut crow_labels: Vec<(String, usize, String)> = Vec::new();
        let mut max_raw = 0usize;
        for sc in cmd.get_subcommands() {
            let name = sc.get_name();
            let mut lab = vec![paint_command(name)];
            for alias in sc.get_aliases() {
                lab.push(paint_command(alias));
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
