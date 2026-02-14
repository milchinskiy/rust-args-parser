#![allow(dead_code, clippy::missing_errors_doc, clippy::missing_panics_doc)]

use rust_args_parser as ap;
use std::ffi::{OsStr, OsString};

#[must_use]
pub fn argv(a: &[&str]) -> Vec<OsString> {
    a.iter().map(OsString::from).collect()
}

#[must_use]
pub const fn env_basic() -> ap::Env {
    ap::Env {
        wrap_cols: 80,
        color: ap::ColorMode::Never,
        suggest: true,
        auto_help: true,
        version: Some("0.1.0"),
        author: Some("Testy McTestface <t@example.com>"),
    }
}

#[derive(Default, Debug)]
pub struct Ctx {
    pub verbose: u8,
    pub json: bool,
    pub jobs: Option<u32>,
    pub limit: Option<String>,
    pub input: Option<OsString>,
    pub files: Vec<OsString>,
}

/// Helpers
pub fn inc_verbose(c: &mut Ctx) {
    c.verbose = c.verbose.saturating_add(1);
}

pub fn set_json(c: &mut Ctx) {
    c.json = true;
}

pub fn set_jobs(v: &OsStr, c: &mut Ctx) {
    c.jobs = Some(v.to_string_lossy().parse().unwrap());
}

pub fn set_limit(v: &OsStr, c: &mut Ctx) {
    c.limit = Some(v.to_string_lossy().into());
}

pub fn set_input(v: &OsStr, c: &mut Ctx) {
    c.input = Some(v.to_os_string());
}

pub fn push_file(v: &OsStr, c: &mut Ctx) {
    c.files.push(v.to_os_string());
}
