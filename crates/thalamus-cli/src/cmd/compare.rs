use std::process::ExitCode;

use crate::global::Global;

#[derive(Debug, clap::Args)]
pub struct Args {}

pub fn run(_global: &Global, _args: &Args) -> ExitCode {
    eprintln!("thalamus compare: not implemented");
    ExitCode::from(70)
}
