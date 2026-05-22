//! Core CLI contracts shared by command handlers and adapters.
//!
//! This crate contains no terminal, network, filesystem, or clipboard access.
//! It defines stable exit codes and deterministic output policy decisions.

mod exit_code;
mod output;

pub use exit_code::CliExitCode;
pub use output::{
    ColorChoice, ColorEnvironment, OutputFacts, OutputFormat, OutputPolicy, RequestedOutput,
    StreamKind,
};
