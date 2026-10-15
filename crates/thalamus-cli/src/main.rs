//! The `thalamus` command line. A usage error exits 64 (`EX_USAGE`), never a verdict ladder code
//! (0 to 3, design § 2.3) and never clap's default 2.

mod cmd;
mod global;

use std::process::ExitCode;

use clap::{Parser, Subcommand};

const EX_USAGE: u8 = 64;

/// Samples signals, grades each sample into a verdict and shows the verdicts.
#[derive(Debug, Parser)]
#[command(name = "thalamus", version)]
struct Cli {
    #[command(flatten)]
    global: global::Global,
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Read a definition's source and record one sample.
    Sample(cmd::sample::Args),
    /// Grade samples into evaluations.
    Eval(cmd::eval::Args),
    /// Show the current verdicts.
    Show(cmd::show::Args),
    /// Compare a series with its own past.
    Compare(cmd::compare::Args),
    /// Build daily rollups.
    Rollup(cmd::rollup::Args),
    /// Measure how often verdicts flap.
    Flaps(cmd::flaps::Args),
}

fn main() -> ExitCode {
    let cli = match Cli::try_parse() {
        Ok(cli) => cli,
        Err(err) => {
            let _ = err.print();
            return if err.use_stderr() {
                ExitCode::from(EX_USAGE)
            } else {
                ExitCode::SUCCESS
            };
        }
    };
    match &cli.command {
        Command::Sample(args) => cmd::sample::run(&cli.global, args),
        Command::Eval(args) => cmd::eval::run(&cli.global, args),
        Command::Show(args) => cmd::show::run(&cli.global, args),
        Command::Compare(args) => cmd::compare::run(&cli.global, args),
        Command::Rollup(args) => cmd::rollup::run(&cli.global, args),
        Command::Flaps(args) => cmd::flaps::run(&cli.global, args),
    }
}
