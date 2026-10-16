//! `thalamus eval [<signal-id>] [--run <run-id>]`: one pass over the pipeline's grade → evaluate
//! half. Every stored sample with no evaluation at its definition's current revision is graded
//! once (design § 11 D5); the evaluations recorded are printed.
//!
//! Exit codes, the codes of `thalamus sample` and none of them a verdict ladder code (0 to 3,
//! design § 2.3; `thalamus show` exits with the ladder):
//!
//! | code | when |
//! |---|---|
//! | 0 | every sample due was graded and recorded, or none was due |
//! | 64 | a usage error: no instance |
//! | 65 | the instance is invalid, or a definition or a sample was refused |
//! | 74 | the store cannot be opened or written; the evaluations recorded before a failure are printed first |

use std::panic::{AssertUnwindSafe, catch_unwind};
use std::process::ExitCode;
use std::str::FromStr;

use clap::ValueEnum;
use eventlog_core::EventLogError;
use serde_json::{Value, json};
use thalamus_model::observation::RunId;
use thalamus_pipeline::evaluate::{self, Pass, verdict_name};
use thalamus_pipeline::instance::LoadedInstance;
use thalamus_sources::connectors::Connectors;
use thalamus_store::eventlog::{
    EventlogStore, LOCATOR_UNKNOWN, LOCATOR_UNUSABLE, Locator, POSTGRES_PASSWORD, POSTGRES_TLS,
    SQLITE_NOT_A_FILE, StoreError,
};
use tokio_postgres::config::Host;

use crate::global::Global;

pub(crate) const EX_USAGE: u8 = 64;
pub(crate) const EX_DATAERR: u8 = 65;
pub(crate) const EX_IOERR: u8 = 74;

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum Format {
    /// One line per recorded evaluation: id, series, verdict.
    Text,
    /// One JSON document: the recorded evaluations and the refusals.
    Json,
}

#[derive(Debug, clap::Args)]
pub struct Args {
    /// Grade only this definition's samples, by id. Default: every Active or Muted definition.
    #[arg(value_name = "SIGNAL_ID")]
    signal: Option<String>,
    /// Grade only the samples of this sampling pass (`run_…`).
    #[arg(long, value_name = "RUN_ID")]
    run: Option<String>,
    /// The output format.
    #[arg(long, value_enum, default_value_t = Format::Text)]
    format: Format,
}

pub fn run(global: &Global, args: &Args) -> ExitCode {
    let opened = match Opened::open("eval", global) {
        Ok(opened) => opened,
        Err(code) => return code,
    };
    let Opened {
        instance,
        mut store,
        name,
    } = opened;
    let signals = match &args.signal {
        Some(signal) => match evaluate::admit(&instance, signal) {
            Ok(signal) => vec![signal],
            Err(refusal) => {
                eprintln!("thalamus eval: refused: {refusal}");
                return ExitCode::from(EX_DATAERR);
            }
        },
        None => evaluate::gradable(&instance),
    };
    let run = args.run.clone().map(RunId);
    let connectors = Connectors::locate(global.connectors.clone());

    let mut pass = Pass::default();
    let graded = quietly(|| {
        evaluate::evaluate_into(
            &mut pass,
            &instance,
            &mut store,
            &connectors,
            &signals,
            run.as_ref(),
        )
    });
    let failure = match graded {
        Ok(Ok(())) => None,
        Ok(Err(evaluate::PassError::IdsSpent)) => {
            Some("the store's identity sequence of evl_ is spent".to_owned())
        }
        Ok(Err(evaluate::PassError::Store(error))) => Some(refusal(&error)),
        Err(_) => Some("it failed during the pass".to_owned()),
    };
    print(&pass, args.format);
    if let Some(failure) = failure {
        eprintln!(
            "thalamus eval: store {name}: {failure}; the {} evaluation(s) printed were recorded \
             before it failed",
            pass.recorded.len()
        );
        return ExitCode::from(EX_IOERR);
    }
    for refusal in &pass.refused {
        eprintln!("thalamus eval: refused: {refusal}");
    }
    if pass.refused.is_empty() {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(EX_DATAERR)
    }
}

fn print(pass: &Pass, format: Format) {
    match format {
        Format::Text => {
            for recorded in &pass.recorded {
                println!(
                    "{}\t{}\t{}",
                    recorded.evaluation_id.0,
                    recorded.series.0,
                    verdict_name(recorded.verdict)
                );
            }
        }
        Format::Json => {
            let evaluations: Vec<Value> = pass
                .recorded
                .iter()
                .map(|recorded| {
                    json!({
                        "evaluation_id": recorded.evaluation_id.0,
                        "sample_id": recorded.sample_id.0,
                        "signal": recorded.signal.0,
                        "series": recorded.series.0,
                        "ts": recorded.ts.0,
                        "definition_rev": recorded.definition_rev,
                        "verdict": verdict_name(recorded.verdict),
                        "prev": recorded.prev.map(verdict_name),
                    })
                })
                .collect();
            let refused: Vec<String> = pass.refused.iter().map(ToString::to_string).collect();
            println!(
                "{}",
                json!({"evaluations": evaluations, "refused": refused})
            );
        }
    }
}

/// An instance and its opened store, and the store as a refusal may name it.
pub(crate) struct Opened {
    pub(crate) instance: LoadedInstance,
    pub(crate) store: EventlogStore,
    pub(crate) name: String,
}

/// An instance and its store as a read command opens it: `None` when the locator names an
/// SQLite file that does not exist yet, which reads as an empty store and is left uncreated.
pub(crate) struct ForReading {
    pub(crate) instance: LoadedInstance,
    pub(crate) store: Option<EventlogStore>,
    pub(crate) name: String,
}

/// An instance and the locator of its store, before the store is opened.
struct Located {
    instance: LoadedInstance,
    locator: Locator,
    name: String,
}

impl Opened {
    /// Loads the instance and opens its store, printing why not under `thalamus <command>`.
    ///
    /// # Errors
    ///
    /// The exit code: 64 without an instance, 65 for an invalid one, 74 for a store that cannot
    /// be opened.
    pub(crate) fn open(command: &str, global: &Global) -> Result<Self, ExitCode> {
        let Located {
            instance,
            locator,
            name,
        } = Located::load(command, global)?;
        let store = open_store(command, &locator, &instance, &name)?;
        Ok(Self {
            instance,
            store,
            name,
        })
    }
}

impl ForReading {
    /// As [`Opened::open`], for a command that only reads: a store whose SQLite file does not
    /// exist is not opened, so the read creates neither the file nor its directory.
    ///
    /// # Errors
    ///
    /// The exit codes of [`Opened::open`].
    pub(crate) fn open(command: &str, global: &Global) -> Result<Self, ExitCode> {
        let Located {
            instance,
            locator,
            name,
        } = Located::load(command, global)?;
        let store = if locator.names_a_missing_sqlite_file() {
            None
        } else {
            Some(open_store(command, &locator, &instance, &name)?)
        };
        Ok(Self {
            instance,
            store,
            name,
        })
    }
}

impl Located {
    /// Loads the instance and parses its store's locator, printing why not under
    /// `thalamus <command>`.
    fn load(command: &str, global: &Global) -> Result<Self, ExitCode> {
        let Some(dir) = &global.instance else {
            eprintln!(
                "thalamus {command}: no instance; give --instance <dir> or set THALAMUS_INSTANCE"
            );
            return Err(ExitCode::from(EX_USAGE));
        };
        let instance = match LoadedInstance::load(dir) {
            Ok(instance) => instance,
            Err(errors) => {
                for error in &errors {
                    eprintln!("thalamus {command}: invalid instance: {error}");
                }
                return Err(ExitCode::from(EX_DATAERR));
            }
        };
        let locator = match global.store_locator(dir).parse::<Locator>() {
            Ok(locator) => locator,
            Err(error) => {
                eprintln!("thalamus {command}: store refused: {}", refusal(&error));
                return Err(ExitCode::from(EX_IOERR));
            }
        };
        let name = describe(&locator);
        Ok(Self {
            instance,
            locator,
            name,
        })
    }
}

/// Opens the store `locator` names for `instance`, printing why not under `thalamus <command>`.
fn open_store(
    command: &str,
    locator: &Locator,
    instance: &LoadedInstance,
    name: &str,
) -> Result<EventlogStore, ExitCode> {
    EventlogStore::open(locator, instance.name()).map_err(|error| {
        eprintln!(
            "thalamus {command}: store {name} refused: {}",
            refusal(&error)
        );
        ExitCode::from(EX_IOERR)
    })
}

/// Runs `work` with a panic hook that prints nothing, then restores the hook it replaced. A store
/// that fails in the middle of a pass panics with a message that may hold its locator; the
/// caller names the store by [`describe`] instead.
pub(crate) fn quietly<T>(work: impl FnOnce() -> T) -> std::thread::Result<T> {
    let previous = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));
    let result = catch_unwind(AssertUnwindSafe(work));
    std::panic::set_hook(previous);
    result
}

/// What a refusal may say about a store: its kind, and for PostgreSQL its hosts (a Unix-socket
/// host by its socket directory, a TCP host with its port) and database, as the driver reads the
/// URL. Never the locator text: no user, no password, no query.
fn describe(locator: &Locator) -> String {
    let Locator::Postgres(url) = locator else {
        return "sqlite".to_owned();
    };
    let Ok(config) = tokio_postgres::Config::from_str(url) else {
        return "postgres".to_owned();
    };
    let ports = config.get_ports();
    let hosts: Vec<String> = config
        .get_hosts()
        .iter()
        .enumerate()
        .map(|(index, host)| match host {
            Host::Tcp(name) => match ports.get(index).or_else(|| ports.first()) {
                Some(port) => format!("{name}:{port}"),
                None => name.clone(),
            },
            Host::Unix(directory) => directory.display().to_string(),
        })
        .collect();
    format!(
        "postgres host `{}` database `{}`",
        hosts.join(","),
        config.get_dbname().unwrap_or_default()
    )
}

/// A store error as an operator sees it: one fixed sentence per kind, and a locator refusal by its
/// code. Never the store's or the provider's own message, which may hold a path or a URL.
fn refusal(error: &StoreError) -> String {
    let code = match error {
        StoreError::Locator { code, .. } => code,
        StoreError::Refused { code } => return format!("the store refused the write: {code}"),
        StoreError::Contended { attempts } => {
            return format!(
                "another writer appended first on each of {attempts} attempts; nothing was \
                 written"
            );
        }
        StoreError::Record(_) => return "a record in the store fails its checks".to_owned(),
        StoreError::Runtime(_) => return "the store's runtime cannot be started".to_owned(),
        StoreError::Eventlog(error) => return eventlog(error).to_owned(),
    };
    let why = match *code {
        LOCATOR_UNKNOWN => "the locator is neither `sqlite:<path>` nor a `postgres://` URL",
        SQLITE_NOT_A_FILE => {
            "the SQLite locator names no database file: an in-memory name, a URI or a directory"
        }
        POSTGRES_PASSWORD => {
            "the URL carries a password; thalamus reads no credential, so the database must \
             authenticate the local connection itself (peer or trust)"
        }
        POSTGRES_TLS => {
            "the URL asks for TLS, which is not wired yet; leave `sslmode` out or set it to \
             `disable`"
        }
        LOCATOR_UNUSABLE => "the directory of the store cannot be created",
        _ => "the locator names no store this build opens",
    };
    format!("{code}: {why}")
}

/// An Eventlog failure as one fixed sentence per kind.
fn eventlog(error: &EventLogError) -> &'static str {
    match error {
        EventLogError::Backend(_) | EventLogError::Closed => "the store is unavailable",
        EventLogError::Deadline { .. } => "the store did not answer in time",
        EventLogError::Overloaded => "the store is overloaded",
        EventLogError::Conflict { .. }
        | EventLogError::IdempotencyMismatch { .. }
        | EventLogError::Forked { .. }
        | EventLogError::UnknownCommit => "the store holds a conflicting write",
        EventLogError::Invalid(_) => "the store refused its configuration",
        EventLogError::Unsupported { .. } => "the store lacks a capability thalamus needs",
        _ => "the store failed",
    }
}
