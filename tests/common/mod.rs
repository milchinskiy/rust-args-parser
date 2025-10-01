#![allow(unused_variables, dead_code, clippy::struct_excessive_bools, clippy::unnecessary_wraps)]

#[allow(unused_imports)]
pub use rust_args_parser::{dispatch_to, CmdSpec, Env, Error, OptSpec, PosSpec};

#[derive(Clone, Default, Debug)]
pub struct U {
    pub verbose: usize,
    pub output: Option<String>,
    pub jobs: Option<String>,
    pub mode_a: bool,
    pub mode_b: bool,
    pub one_a: bool,
    pub one_b: bool,
    pub envd: Option<String>,
    pub defv: Option<String>,
    pub ran: bool,
    pub received: Vec<String>,
}

#[derive(Debug)]
pub enum AppError {
    User(&'static str),
}
impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::User(s) => write!(f, "{s}"),
        }
    }
}
impl std::error::Error for AppError {}

// ---- Shared callbacks ----
pub fn cb_verbose(v: Option<&str>, u: &mut U) -> Result<(), Box<dyn std::error::Error>> {
    assert!(v.is_none());
    u.verbose += 1;
    Ok(())
}
pub fn cb_output(v: Option<&str>, u: &mut U) -> Result<(), Box<dyn std::error::Error>> {
    u.output = v.map(std::string::ToString::to_string);
    Ok(())
}
pub fn cb_jobs(v: Option<&str>, u: &mut U) -> Result<(), Box<dyn std::error::Error>> {
    u.jobs = v.map(std::string::ToString::to_string);
    Ok(())
}
pub fn cb_mode_a(v: Option<&str>, u: &mut U) -> Result<(), Box<dyn std::error::Error>> {
    assert!(v.is_none());
    u.mode_a = true;
    Ok(())
}
pub fn cb_mode_b(v: Option<&str>, u: &mut U) -> Result<(), Box<dyn std::error::Error>> {
    assert!(v.is_none());
    u.mode_b = true;
    Ok(())
}
pub fn cb_one_a(v: Option<&str>, u: &mut U) -> Result<(), Box<dyn std::error::Error>> {
    assert!(v.is_none());
    u.one_a = true;
    Ok(())
}
pub fn cb_one_b(v: Option<&str>, u: &mut U) -> Result<(), Box<dyn std::error::Error>> {
    assert!(v.is_none());
    u.one_b = true;
    Ok(())
}
pub fn cb_envd(v: Option<&str>, u: &mut U) -> Result<(), Box<dyn std::error::Error>> {
    u.envd = v.map(std::string::ToString::to_string);
    Ok(())
}
pub fn cb_defv(v: Option<&str>, u: &mut U) -> Result<(), Box<dyn std::error::Error>> {
    u.defv = v.map(std::string::ToString::to_string);
    Ok(())
}
pub fn cb_user_err(_: Option<&str>, _: &mut U) -> Result<(), Box<dyn std::error::Error>> {
    Err(AppError::User("bad").into())
}

pub fn run_cb(args: &[&str], u: &mut U) -> Result<(), Box<dyn std::error::Error>> {
    u.ran = true;
    u.received = args.iter().map(std::string::ToString::to_string).collect();
    Ok(())
}

pub const fn env_base() -> Env<'static> {
    Env::new("prog")
        .version("1.2.3")
        .author("Alice Example")
        .auto_help(true)
        .wrap_cols(80)
        .color(false)
}

pub fn s(out: &[u8]) -> String {
    String::from_utf8(out.to_vec()).unwrap()
}
