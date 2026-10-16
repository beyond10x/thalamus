//! `thalamus sample <signal-id>` and `thalamus sample --all --once`: one pass over the pipeline's
//! read → sample half, printing the recorded sample ids.
//!
//! After a pass that recorded without a store failure, every recorded sample whose definition is
//! graded against `fixed` alone (`evaluation.baselines: [fixed]`, so no stored history is needed)
//! is graded through `thalamus_core::grading` against the definition's own number, and each WARN
//! or CRIT is reported as a breach: one line on stdout after the sample ids,
//!
//! ```text
//! breach <VERDICT> <signal> <dims> <value> <unit> <line>
//! ```
//!
//! where `<dims>` is `key=value` joined by `,` (`-` when the series has none) and `<line>` is the
//! grader's statement of the line crossed (`150000000000 <= 1 x 200000000000`), and the same as
//! `breaches` in `--format json`. A floor of free space is `rule: lower_is_worse`, `baselines:
//! [fixed]`, `fixed` the floor, `warn: {x: ...}` and `crit: {x: 1}` (a threshold rule needs both
//! lines); a reading at or under a line is on it, so a reading at the floor is a CRIT breach. No evaluation is recorded (that is `thalamus
//! eval`'s), an unknown reading is never a breach and never OK, and a breach leaves the exit code
//! as it was: a breach is not a failure of the run.
//!
//! Exit codes, none of them a verdict ladder code (0 to 3, design § 2.3):
//!
//! | code | when |
//! |---|---|
//! | 0 | every read was recorded, unknown ones included |
//! | 64 | a usage error: no instance, a window end that is not RFC 3339 |
//! | 65 | the instance is invalid, or a definition or a read was refused |
//! | 74 | the store cannot be opened or written; the samples recorded before a failure are printed first |
//!
//! A store refusal names the store by kind, and a PostgreSQL store by host and database, and says
//! what failed in one fixed sentence per kind: never the locator text, never the store's or the
//! provider's own message, never a panic message.

use std::panic::{AssertUnwindSafe, catch_unwind};
use std::process::ExitCode;
use std::str::FromStr;

use clap::{ArgGroup, ValueEnum};
use eventlog_core::EventLogError;
use serde_json::{Value, json};
use thalamus_core::baseline;
use thalamus_core::behaviour::observation::Instant;
use thalamus_core::grading;
use thalamus_model::behaviour::SignalSampleStorage;
use thalamus_model::definition::BaselineKind;
use thalamus_model::evaluation::Verdict;
use thalamus_model::observation::{Reading, SampleId};
use thalamus_model::primitives::Timestamp;
use thalamus_pipeline::instance::LoadedInstance;
use thalamus_pipeline::sample::{self, Pass, Producer};
use thalamus_sources::connectors::Connectors;
use thalamus_store::eventlog::{
    EventlogStore, LOCATOR_UNKNOWN, LOCATOR_UNUSABLE, Locator, POSTGRES_PASSWORD, POSTGRES_TLS,
    SQLITE_NOT_A_FILE, StoreError,
};
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;
use tokio_postgres::config::Host;

use crate::global::Global;

const EX_USAGE: u8 = 64;
const EX_DATAERR: u8 = 65;
const EX_IOERR: u8 = 74;

/// The producer every sample this command records names.
const PRODUCER: &str = "thalamus-cli";

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum Format {
    /// One line per recorded sample: id, series, quality.
    Text,
    /// One JSON document: the run id, the recorded samples and the refusals.
    Json,
}

#[derive(Debug, clap::Args)]
#[command(group(ArgGroup::new("what").required(true).args(["signal", "all"])))]
pub struct Args {
    /// The definition to sample, by id.
    #[arg(value_name = "SIGNAL_ID")]
    signal: Option<String>,
    /// Sample every Active definition of the instance.
    #[arg(long, requires = "once")]
    all: bool,
    /// One pass, then exit (the service samples on each definition's cadence).
    #[arg(long, requires = "all", conflicts_with = "signal")]
    once: bool,
    /// The end of the window read, RFC 3339. Default: now.
    #[arg(long, value_name = "TIMESTAMP")]
    at: Option<String>,
    /// The output format.
    #[arg(long, value_enum, default_value_t = Format::Text)]
    format: Format,
}

pub fn run(global: &Global, args: &Args) -> ExitCode {
    let Some(dir) = &global.instance else {
        eprintln!("thalamus sample: no instance; give --instance <dir> or set THALAMUS_INSTANCE");
        return ExitCode::from(EX_USAGE);
    };
    let end = match &args.at {
        Some(at) if Instant::parse(at).is_some() => Timestamp(at.clone()),
        Some(at) => {
            eprintln!("thalamus sample: --at `{at}` is not an RFC 3339 timestamp");
            return ExitCode::from(EX_USAGE);
        }
        None => match now() {
            Some(now) => now,
            None => {
                eprintln!("thalamus sample: the clock reads no RFC 3339 instant");
                return ExitCode::from(EX_IOERR);
            }
        },
    };

    let mut instance = match LoadedInstance::load(dir) {
        Ok(instance) => instance,
        Err(errors) => {
            for error in &errors {
                eprintln!("thalamus sample: invalid instance: {error}");
            }
            return ExitCode::from(EX_DATAERR);
        }
    };
    let signals = match &args.signal {
        Some(signal) => match sample::admit(&instance, signal) {
            Ok(signal) => vec![signal],
            Err(refusal) => {
                eprintln!("thalamus sample: refused: {refusal}");
                return ExitCode::from(EX_DATAERR);
            }
        },
        None => sample::active(&instance),
    };

    let locator = match global.store_locator(dir).parse::<Locator>() {
        Ok(locator) => locator,
        Err(error) => {
            eprintln!("thalamus sample: store refused: {}", refusal(&error));
            return ExitCode::from(EX_IOERR);
        }
    };
    let store = describe(&locator);
    let mut opened = match EventlogStore::open(&locator, instance.name()) {
        Ok(opened) => opened,
        Err(error) => {
            eprintln!(
                "thalamus sample: store {store} refused: {}",
                refusal(&error)
            );
            return ExitCode::from(EX_IOERR);
        }
    };
    let connectors = Connectors::locate(global.connectors.clone());
    let producer = Producer {
        name: PRODUCER.to_owned(),
        version: env!("CARGO_PKG_VERSION").to_owned(),
    };

    let mut pass = match quietly(|| sample::begin(&mut opened)) {
        Ok(Ok(pass)) => pass,
        Ok(Err(error)) => {
            eprintln!("thalamus sample: store {store}: {error}");
            return ExitCode::from(EX_IOERR);
        }
        Err(_) => {
            eprintln!("thalamus sample: store {store} failed before the pass began");
            return ExitCode::from(EX_IOERR);
        }
    };
    let sampled = quietly(|| {
        sample::sample_into(
            &mut pass,
            &mut instance,
            &mut opened,
            &connectors,
            &signals,
            &end,
            &producer,
        )
    });
    let failure = match sampled {
        Ok(Ok(())) => None,
        Ok(Err(error)) => Some(error.to_string()),
        Err(_) => Some("it failed during the pass".to_owned()),
    };
    if let Some(failure) = failure {
        print(&pass, &[], args.format);
        eprintln!(
            "thalamus sample: store {store}: {failure}; the {} sample(s) printed were recorded \
             under {} before it failed",
            pass.recorded.len(),
            pass.run_id.0
        );
        return ExitCode::from(EX_IOERR);
    }

    let breaches = match quietly(|| breaches(&pass, &instance, &opened)) {
        Ok(breaches) => breaches,
        Err(_) => {
            eprintln!(
                "thalamus sample: store {store}: the recorded samples could not be read back to \
                 report breaches; the samples printed were recorded"
            );
            Vec::new()
        }
    };
    print(&pass, &breaches, args.format);
    for refusal in &pass.refused {
        eprintln!("thalamus sample: refused: {refusal}");
    }
    if pass.refused.is_empty() {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(EX_DATAERR)
    }
}

/// Runs `work` with a panic hook that prints nothing, then restores the hook it replaced. A store
/// that fails in the middle of a pass panics with a message that may hold its locator; the
/// caller names the store by [`describe`] instead.
fn quietly<T>(work: impl FnOnce() -> T) -> std::thread::Result<T> {
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

/// The current instant in whole seconds, RFC 3339 in UTC.
fn now() -> Option<Timestamp> {
    let now = OffsetDateTime::now_utc().replace_nanosecond(0).ok()?;
    now.format(&Rfc3339).ok().map(Timestamp)
}

/// A recorded sample graded WARN or CRIT against its definition's fixed number.
struct Breach {
    verdict: &'static str,
    sample_id: SampleId,
    signal: String,
    series: String,
    dims: Vec<(String, String)>,
    value: String,
    unit: String,
    line: String,
}

impl Breach {
    /// The series' dimensions as `key=value` joined by `,`; `-` when it has none.
    fn dims_text(&self) -> String {
        if self.dims.is_empty() {
            return "-".to_owned();
        }
        self.dims
            .iter()
            .map(|(key, value)| format!("{key}={value}"))
            .collect::<Vec<_>>()
            .join(",")
    }
}

/// Every recorded sample of `pass` whose definition is graded against `fixed` alone and that
/// grades WARN or CRIT, in the order recorded. A definition the grader refuses, and a verdict
/// other than WARN or CRIT (`UNKNOWN` included), is no breach. Nothing is written.
fn breaches(pass: &Pass, instance: &LoadedInstance, store: &EventlogStore) -> Vec<Breach> {
    let mut found = Vec::new();
    for recorded in &pass.recorded {
        let Some(definition) = instance.instance().definition(&recorded.signal.0) else {
            continue;
        };
        let definition = definition.data();
        if definition.evaluation.baselines != [BaselineKind::Fixed] {
            continue;
        }
        let Some(sample) = SignalSampleStorage::get(store, &recorded.sample_id) else {
            continue;
        };
        let sample = sample.data;
        let Ok(against) = baseline::fold(definition, &sample, &[], None) else {
            continue;
        };
        let Ok(graded) = grading::grade(definition, &sample, &against) else {
            continue;
        };
        let verdict = match graded.verdict {
            Verdict::WARN => "WARN",
            Verdict::CRIT => "CRIT",
            Verdict::OK | Verdict::NA | Verdict::UNKNOWN => continue,
        };
        let value = match &sample.reading {
            Reading::Ok(read) => read.value.0.clone(),
            Reading::Partial(read) => read.value.0.clone(),
            Reading::Unknown(_) => continue,
        };
        found.push(Breach {
            verdict,
            sample_id: recorded.sample_id.clone(),
            signal: recorded.signal.0.clone(),
            series: recorded.series.0.clone(),
            dims: sample
                .dims
                .iter()
                .map(|(key, value)| (key.clone(), value.0.clone()))
                .collect(),
            value,
            unit: sample.unit.0.clone(),
            line: graded.reason,
        });
    }
    found
}

fn print(pass: &Pass, breaches: &[Breach], format: Format) {
    match format {
        Format::Text => {
            for recorded in &pass.recorded {
                println!(
                    "{}\t{}\t{}",
                    recorded.sample_id.0, recorded.series.0, recorded.quality
                );
            }
            for breach in breaches {
                println!(
                    "breach {} {} {} {} {} {}",
                    breach.verdict,
                    breach.signal,
                    breach.dims_text(),
                    breach.value,
                    breach.unit,
                    breach.line
                );
            }
        }
        Format::Json => {
            let samples: Vec<Value> = pass
                .recorded
                .iter()
                .map(|recorded| {
                    json!({
                        "sample_id": recorded.sample_id.0,
                        "signal": recorded.signal.0,
                        "series": recorded.series.0,
                        "quality": recorded.quality,
                    })
                })
                .collect();
            let refused: Vec<String> = pass.refused.iter().map(ToString::to_string).collect();
            let breaches: Vec<Value> = breaches
                .iter()
                .map(|breach| {
                    let dims: serde_json::Map<String, Value> = breach
                        .dims
                        .iter()
                        .map(|(key, value)| (key.clone(), Value::String(value.clone())))
                        .collect();
                    json!({
                        "verdict": breach.verdict,
                        "sample_id": breach.sample_id.0,
                        "signal": breach.signal,
                        "series": breach.series,
                        "dims": dims,
                        "value": breach.value,
                        "unit": breach.unit,
                        "line": breach.line,
                    })
                })
                .collect();
            println!(
                "{}",
                json!({
                    "run_id": pass.run_id.0,
                    "samples": samples,
                    "refused": refused,
                    "breaches": breaches,
                })
            );
        }
    }
}
