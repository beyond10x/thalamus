//! `thalamus flaps --since <duration> [--rev current|<n>] [--k <n>] [--k-crit <n>] [--m <n>]
//! [--sweep] [--format text|json]`: how often verdicts flap, measured over stored evaluations so
//! that `k`, `k_crit` and `m` can be fixed per family (design § 3.2).
//!
//! It reads every series' evaluations from `now - since` on through the store's series range
//! read, keeps those at the selected definition revision (default: each definition's current
//! revision) and counts the rest, and those of a signal the instance does not define, as
//! `excluded`. Per family it reports:
//!
//! - `series`: the series with a kept evaluation;
//! - `series_days`: per series, the distinct UTC days its kept evaluations fall on, summed;
//! - `raw_changes`: the kept evaluations `thalamus.evaluation.VerdictChanges` shows (`prev`
//!   defined and the verdict differs from it), read through the generated query;
//! - `settled`: one run per persistence, each the changes `thalamus_core::hysteresis::replay`
//!   settles over each series' kept evaluations in minting (id) order, after the series' first
//!   settled verdict, which is a start and not a flap;
//!
//! and each count per series-day (absent when there are no series-days, never 0). Without `--k`,
//! `--k-crit` and `--m` each definition's own `persistence` applies; each one given overrides that
//! value for every definition. `--sweep` runs `k` 1 to 3 and `m` 1 to 4, `k_crit` from `--k-crit`
//! or each definition.
//!
//! Exit codes: 0 when measured; 64 for a usage error (no instance, a duration that does not
//! parse); 65 for an invalid instance; 74 when the store cannot be opened or read. `flaps` only
//! reads: a store whose SQLite file does not exist reads as empty and is not created.

use std::collections::{BTreeMap, BTreeSet};
use std::process::ExitCode;

use clap::ValueEnum;
use serde_json::{Value, json};
use thalamus_core::hysteresis::{Persistence, replay};
use thalamus_model::behaviour::{Generated, SignalEvaluationStorage};
use thalamus_model::evaluation::obligations::VerdictChangesQuery;
use thalamus_model::evaluation::{
    EvaluationId, SignalEvaluationData, SignalEvaluationSnapshot, SignalEvaluationState,
};
use thalamus_model::observation::SeriesKey;
use thalamus_model::primitives::Timestamp;
use thalamus_store::eventlog::EventlogStore;
use time::format_description::well_known::Rfc3339;
use time::{Duration, OffsetDateTime, UtcOffset};

use super::eval::{EX_IOERR, EX_USAGE, ForReading, quietly};
use crate::global::Global;

/// An instant after the last RFC 3339 instant in UTC: the range read's open end.
const BEYOND_LAST: &str = "9999-12-31T23:59:59.999999999-23:59";

/// The `k` a sweep covers.
const SWEEP_K: std::ops::RangeInclusive<usize> = 1..=3;
/// The `m` a sweep covers.
const SWEEP_M: std::ops::RangeInclusive<usize> = 1..=4;

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum Format {
    /// One line per family and persistence, then the excluded count.
    Text,
    /// One JSON document.
    Json,
}

/// The definition revision whose evaluations are kept.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Rev {
    /// Each definition's current revision.
    Current,
    /// This revision, for every definition.
    Number(i64),
}

#[derive(Debug, clap::Args)]
pub struct Args {
    /// How far back to measure: `<n>s`, `<n>m`, `<n>h`, `<n>d` or `<n>w`.
    #[arg(long, value_name = "DURATION", value_parser = parse_since)]
    since: Since,
    /// The definition revision to measure: `current` (each definition's) or a number.
    #[arg(long, value_name = "current|N", value_parser = parse_rev, default_value = "current")]
    rev: Rev,
    /// Samples before a change to WARN, UNKNOWN or N/A settles, for every definition.
    #[arg(long, value_name = "N", value_parser = clap::value_parser!(u16).range(1..))]
    k: Option<u16>,
    /// Samples before a change to CRIT settles, for every definition.
    #[arg(long = "k-crit", value_name = "N", value_parser = clap::value_parser!(u16).range(1..))]
    k_crit: Option<u16>,
    /// OK samples before a settled verdict clears, for every definition.
    #[arg(long, value_name = "N", value_parser = clap::value_parser!(u16).range(1..))]
    m: Option<u16>,
    /// Measure every k from 1 to 3 against every m from 1 to 4.
    #[arg(long, conflicts_with_all = ["k", "m"])]
    sweep: bool,
    /// The output format.
    #[arg(long, value_enum, default_value_t = Format::Text)]
    format: Format,
}

/// A `--since` as written, with its length.
#[derive(Debug, Clone)]
struct Since {
    text: String,
    seconds: i64,
}

fn parse_since(text: &str) -> Result<Since, String> {
    let refused = || format!("`{text}` is not a positive <n>s, <n>m, <n>h, <n>d or <n>w");
    let unit = text.chars().last().ok_or_else(refused)?;
    let scale = match unit {
        's' => 1,
        'm' => 60,
        'h' => 3_600,
        'd' => 86_400,
        'w' => 604_800,
        _ => return Err(refused()),
    };
    let digits = &text[..text.len() - 1];
    if digits.is_empty() || !digits.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(refused());
    }
    match digits
        .parse::<i64>()
        .ok()
        .and_then(|n| n.checked_mul(scale))
    {
        Some(seconds) if seconds > 0 => Ok(Since {
            text: text.to_owned(),
            seconds,
        }),
        _ => Err(refused()),
    }
}

fn parse_rev(text: &str) -> Result<Rev, String> {
    if text == "current" {
        return Ok(Rev::Current);
    }
    match text.parse::<i64>() {
        Ok(rev) if rev >= 1 => Ok(Rev::Number(rev)),
        _ => Err(format!(
            "`{text}` is neither `current` nor a revision of 1 or more"
        )),
    }
}

/// The persistence of one run: a value given for every definition, or `None` where each
/// definition's own applies.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Given {
    k: Option<usize>,
    k_crit: Option<usize>,
    m: Option<usize>,
}

impl Given {
    fn over(self, own: Persistence) -> Persistence {
        Persistence {
            k: self.k.unwrap_or(own.k),
            k_crit: self.k_crit.unwrap_or(own.k_crit),
            m: self.m.unwrap_or(own.m),
        }
    }
}

/// One kept series: its definition's family and persistence, and its kept evaluations.
struct Series {
    family: String,
    persistence: Persistence,
    kept: Vec<SignalEvaluationData>,
}

/// What one family measured.
#[derive(Default)]
struct Family {
    series: u64,
    series_days: u64,
    raw_changes: u64,
    /// One count per run, in the order of the runs.
    settled: Vec<u64>,
}

/// The kept evaluations as the storage port of the generated queries: read here, never written
/// to the store.
struct Kept(Vec<SignalEvaluationSnapshot>);

impl SignalEvaluationStorage for Kept {
    fn get(&self, identity: &EvaluationId) -> Option<SignalEvaluationSnapshot> {
        self.0
            .iter()
            .find(|held| held.data.evaluation_id == *identity)
            .cloned()
    }

    fn put(&mut self, snapshot: SignalEvaluationSnapshot) {
        self.delete(&snapshot.data.evaluation_id);
        self.0.push(snapshot);
    }

    fn delete(&mut self, identity: &EvaluationId) {
        self.0.retain(|held| held.data.evaluation_id != *identity);
    }

    fn list(&self) -> Vec<SignalEvaluationSnapshot> {
        self.0.clone()
    }
}

pub fn run(global: &Global, args: &Args) -> ExitCode {
    let ForReading {
        instance,
        store,
        name,
    } = match ForReading::open("flaps", global) {
        Ok(opened) => opened,
        Err(code) => return code,
    };
    let now = OffsetDateTime::now_utc();
    let Some(from) = now
        .checked_sub(Duration::seconds(args.since.seconds))
        .and_then(|from| from.format(&Rfc3339).ok())
    else {
        eprintln!(
            "thalamus flaps: --since {} reaches before the first instant",
            args.since.text
        );
        return ExitCode::from(EX_USAGE);
    };
    let runs = runs(args);
    let read = quietly(|| {
        let mut excluded = 0_u64;
        let mut series = Vec::new();
        if let Some(store) = &store {
            for key in series_keys(store) {
                let range = store.series_range(
                    &SeriesKey(key),
                    &Timestamp(from.clone()),
                    &Timestamp(BEYOND_LAST.to_owned()),
                );
                let mut kept: Option<Series> = None;
                for evaluation in range.evaluations {
                    let definition = instance.instance().definition(&evaluation.signal.0);
                    let Some(data) = definition.map(|definition| definition.data()) else {
                        excluded += 1;
                        continue;
                    };
                    let rev = match args.rev {
                        Rev::Current => data.rev,
                        Rev::Number(rev) => rev,
                    };
                    if evaluation.definition_rev != rev {
                        excluded += 1;
                        continue;
                    }
                    kept.get_or_insert_with(|| Series {
                        family: data.family.0.clone(),
                        persistence: Persistence::of(data),
                        kept: Vec::new(),
                    })
                    .kept
                    .push(evaluation);
                }
                series.extend(kept);
            }
        }
        (excluded, measure(series, &runs))
    });
    let Ok((excluded, families)) = read else {
        eprintln!("thalamus flaps: store {name} cannot be read");
        return ExitCode::from(EX_IOERR);
    };
    print(args, excluded, &runs, &families);
    ExitCode::SUCCESS
}

/// The runs `args` asks for: one, or the sweep's grid.
fn runs(args: &Args) -> Vec<Given> {
    let k_crit = args.k_crit.map(usize::from);
    if args.sweep {
        return SWEEP_K
            .flat_map(|k| {
                SWEEP_M.map(move |m| Given {
                    k: Some(k),
                    k_crit,
                    m: Some(m),
                })
            })
            .collect();
    }
    vec![Given {
        k: args.k.map(usize::from),
        k_crit,
        m: args.m.map(usize::from),
    }]
}

/// Every series key that has a stored evaluation, in key order.
fn series_keys(store: &EventlogStore) -> BTreeSet<String> {
    store
        .current_evaluations()
        .into_iter()
        .map(|evaluation| evaluation.series.0)
        .collect()
}

/// The figures of each family, by family name.
fn measure(series: Vec<Series>, runs: &[Given]) -> BTreeMap<String, Family> {
    let mut families: BTreeMap<String, Family> = BTreeMap::new();
    for Series {
        family,
        persistence,
        mut kept,
    } in series
    {
        let measured = families.entry(family).or_insert_with(|| Family {
            settled: vec![0; runs.len()],
            ..Family::default()
        });
        measured.series += 1;
        measured.series_days += utc_days(&kept);
        measured.raw_changes += verdict_changes(&kept);
        kept.sort_by(|a, b| a.evaluation_id.0.cmp(&b.evaluation_id.0));
        for (count, given) in measured.settled.iter_mut().zip(runs) {
            *count += replay(given.over(persistence), &kept)
                .iter()
                .filter(|change| change.from.is_some())
                .count() as u64;
        }
    }
    families
}

/// The rows of `thalamus.evaluation.VerdictChanges` over `kept`, through the generated query.
fn verdict_changes(kept: &[SignalEvaluationData]) -> u64 {
    let ports = Kept(
        kept.iter()
            .map(|data| SignalEvaluationSnapshot {
                state: SignalEvaluationState::Recorded,
                data: data.clone(),
            })
            .collect(),
    );
    Generated::new(ports)
        .verdict_changes()
        .map_or(0, |rows| rows.len() as u64)
}

/// The distinct UTC days the evaluations' `ts` fall on; a `ts` that names no instant counts none.
fn utc_days(kept: &[SignalEvaluationData]) -> u64 {
    kept.iter()
        .filter_map(|evaluation| OffsetDateTime::parse(&evaluation.ts.0, &Rfc3339).ok())
        .map(|at| at.to_offset(UtcOffset::UTC).date())
        .collect::<BTreeSet<_>>()
        .len() as u64
}

/// `changes` per series-day, to three fractional digits; `None` without series-days.
fn per_day(changes: u64, days: u64) -> Option<f64> {
    (days > 0).then(|| (changes as f64 / days as f64 * 1000.0).round() / 1000.0)
}

fn print(args: &Args, excluded: u64, runs: &[Given], families: &BTreeMap<String, Family>) {
    match args.format {
        Format::Text => {
            let rate =
                |rate: Option<f64>| rate.map_or_else(|| "none".to_owned(), |rate| rate.to_string());
            let value = |value: Option<usize>| {
                value.map_or_else(|| "own".to_owned(), |value| value.to_string())
            };
            for (name, family) in families {
                for (given, settled) in runs.iter().zip(&family.settled) {
                    println!(
                        "{name}\tseries {}\tseries-days {}\traw {} ({}/series-day)\tk {} k_crit \
                         {} m {}\tsettled {settled} ({}/series-day)",
                        family.series,
                        family.series_days,
                        family.raw_changes,
                        rate(per_day(family.raw_changes, family.series_days)),
                        value(given.k),
                        value(given.k_crit),
                        value(given.m),
                        rate(per_day(*settled, family.series_days)),
                    );
                }
            }
            println!("excluded {excluded}");
        }
        Format::Json => {
            let families: Vec<Value> = families
                .iter()
                .map(|(name, family)| {
                    let settled: Vec<Value> = runs
                        .iter()
                        .zip(&family.settled)
                        .map(|(given, settled)| {
                            json!({
                                "k": given.k,
                                "k_crit": given.k_crit,
                                "m": given.m,
                                "settled_changes": settled,
                                "settled_per_series_day": per_day(*settled, family.series_days),
                            })
                        })
                        .collect();
                    json!({
                        "family": name,
                        "series": family.series,
                        "series_days": family.series_days,
                        "raw_changes": family.raw_changes,
                        "raw_per_series_day": per_day(family.raw_changes, family.series_days),
                        "settled": settled,
                    })
                })
                .collect();
            let rev = match args.rev {
                Rev::Current => json!("current"),
                Rev::Number(rev) => json!(rev),
            };
            println!(
                "{}",
                json!({
                    "since": args.since.text,
                    "rev": rev,
                    "excluded": excluded,
                    "families": families,
                })
            );
        }
    }
}
