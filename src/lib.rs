mod error;
pub use error::*;
mod spec;
pub use spec::*;
mod matches;
pub use matches::*;
mod parse;
pub use parse::*;

#[cfg(feature = "help")]
mod help;

#[cfg(feature = "help")]
pub use help::*;

#[cfg(feature = "suggest")]
mod suggest;

#[cfg(feature = "suggest")]
pub use suggest::*;

pub mod util;
pub use util::*;

#[cfg(feature = "derive")]
pub mod derive_api;
#[cfg(feature = "derive")]
pub use derive_api::{Args, Parser, Subcommand};
#[cfg(feature = "derive")]
pub use rust_args_parser_derive::{Args, Parser, Subcommand};

#[cfg(feature = "derive")]
extern crate self as rust_args_parser;
