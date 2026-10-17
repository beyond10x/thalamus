//! `thalamus-service run --instance <dir>`: the scheduler of one instance, sampling each `Active`
//! definition on its cadence until the process is stopped. Each settled verdict change is
//! recorded as a transition and written to standard output as one CloudEvents JSON line
//! (`thalamus_service::events`). No HTTP is served yet.

use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, Subcommand};
use thalamus_service::scheduler::{ConnectorsReader, Scheduler, SystemClock};
use thalamus_sources::connectors::Connectors;
use thalamus_store::eventlog::Locator;

/// The store an instance keeps when neither `--store` nor `THALAMUS_STORE` names one, relative to
/// the instance directory.
const DEFAULT_STORE: &str = ".thalamus/store.sqlite";

const EX_USAGE: u8 = 64;
const EX_DATAERR: u8 = 65;

#[derive(Debug, Parser)]
#[command(name = "thalamus-service", version, about)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Sample each Active definition of the instance on its cadence until stopped.
    Run {
        /// The instance directory: `dimensions.yaml`, `definitions/*.yaml` and the registry files.
        #[arg(
            long,
            env = "THALAMUS_INSTANCE",
            hide_env_values = true,
            value_name = "DIR"
        )]
        instance: PathBuf,
        /// The instance's store: `sqlite:<path>` or a loopback `postgres://` URL without a
        /// password. Default: `sqlite:<instance>/.thalamus/store.sqlite`.
        #[arg(
            long,
            env = "THALAMUS_STORE",
            hide_env_values = true,
            value_name = "LOCATOR"
        )]
        store: Option<String>,
        /// The `connectors` binary sources are read through (else `THALAMUS_CONNECTORS`, else
        /// `connectors` on `PATH`).
        #[arg(long, value_name = "BIN")]
        connectors: Option<PathBuf>,
    },
}

fn main() -> ExitCode {
    let Command::Run {
        instance,
        store,
        connectors,
    } = Cli::parse().command;
    let locator =
        store.unwrap_or_else(|| format!("sqlite:{}", instance.join(DEFAULT_STORE).display()));
    let Ok(locator) = locator.parse::<Locator>() else {
        eprintln!("thalamus-service: the store locator names no usable store");
        return ExitCode::from(EX_USAGE);
    };
    let connectors = Connectors::locate(connectors);
    let mut scheduler = match Scheduler::open(
        &instance,
        &locator,
        SystemClock,
        ConnectorsReader(connectors.clone()),
        connectors,
    ) {
        Ok(scheduler) => scheduler,
        Err(error) => {
            eprintln!("thalamus-service: {error}");
            return ExitCode::from(EX_DATAERR);
        }
    };
    scheduler.run(|| false);
    ExitCode::SUCCESS
}
