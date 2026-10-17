//! `thalamus flaps --since <duration> [--rev current|<n>] [--k <n>] [--k-crit <n>] [--m <n>]
//! [--sweep] [--format text|json]`: how often verdicts flap, measured over stored evaluations so
//! that `k`, `k_crit` and `m` can be fixed per family (design § 3.2).
//!
//! It reads every series' evaluations through the store's series range read, in sample `ts`
//! order, and keeps those at the selected definition revision (default: each definition's current
//! revision). Of the evaluations from `now - since` on (the window), it counts the others, and
//! those of a signal the instance does not define, as `excluded`. Per family it reports:
//!
//! - `series`: the series with a kept evaluation in the window;
//! - `series_days`: observed time, per series the span of its kept evaluations' sample timestamps
//!   in the window (last minus first, plus the median spacing between consecutive timestamps; one
//!   sample is zero), capped at the window's length, over 24 hours, summed: a decimal, to three
//!   fractional digits;
//! - `raw_changes`: the kept evaluations in the window `thalamus.evaluation.VerdictChanges` shows
//!   (`prev` defined and the verdict differs from it), read through the generated query;
//! - `settled`: one run per persistence, each the changes the hysteresis settles inside the
//!   window. Each series' replay is seeded with its newest `max(k, k_crit, m)` kept evaluations
//!   before the window, so a verdict settled before it is the `from` of the first change inside it; a series' first
//!   settled verdict is a start and not a flap. The replay takes the evaluations in sample `ts`
//!   order, so history re-graded at the kept revision after newer samples counts too;
//!
//! and each count per series-day, taken over the rounded series-days (absent when they round to
//! 0, never 0). Without `--k`,
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
use thalamus_core::behaviour::observation::Instant;
use thalamus_core::hysteresis::{Persistence, SeriesHysteresis};
use thalamus_model::behaviour::{Generated, SignalEvaluationStorage};
use thalamus_model::evaluation::obligations::VerdictChangesQuery;
use thalamus_model::evaluation::{
    EvaluationId, SignalEvaluationData, SignalEvaluationSnapshot, SignalEvaluationState,
};
use thalamus_model::observation::SeriesKey;
use thalamus_model::primitives::Timestamp;
use thalamus_store::eventlog::EventlogStore;
use time::format_description::well_known::Rfc3339;
use time::{Duration, OffsetDateTime};

use super::eval::{EX_IOERR, EX_USAGE, ForReading, quietly};
use crate::global::Global;

/// An instant after the last RFC 3339 instant in UTC: the range read's open end.
const BEYOND_LAST: &str = "9999-12-31T23:59:59.999999999-23:59";
/// The first RFC 3339 instant: the range read's start, so the replay can be seeded.
const FIRST: &str = "0000-01-01T00:00:00+23:59";
/// Seconds in a series-day.
const DAY: u64 = 86_400;

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

/// One kept series: its definition's family and persistence, and its kept evaluations before the
/// window and in it, each in sample `ts` order.
struct Series {
    family: String,
    persistence: Persistence,
    before: Vec<SignalEvaluationData>,
    kept: Vec<SignalEvaluationData>,
}

/// What one family measured.
#[derive(Default)]
struct Family {
    series: u64,
    /// Observed seconds: each series' span of kept sample timestamps, capped at the window.
    observed: u64,
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
    let Some(start) = now
        .checked_sub(Duration::seconds(args.since.seconds))
        .and_then(|from| from.format(&Rfc3339).ok())
        .and_then(|from| Instant::parse(&from))
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
                    &Timestamp(FIRST.to_owned()),
                    &Timestamp(BEYOND_LAST.to_owned()),
                );
                let mut kept: Option<Series> = None;
                for evaluation in range.evaluations {
                    // A `ts` that names no instant is in every range, the window's included.
                    let inside = Instant::parse(&evaluation.ts.0).is_none_or(|at| at >= start);
                    let definition = instance.instance().definition(&evaluation.signal.0);
                    let Some(data) = definition.map(|definition| definition.data()) else {
                        excluded += u64::from(inside);
                        continue;
                    };
                    let rev = match args.rev {
                        Rev::Current => data.rev,
                        Rev::Number(rev) => rev,
                    };
                    if evaluation.definition_rev != rev {
                        excluded += u64::from(inside);
                        continue;
                    }
                    let held = kept.get_or_insert_with(|| Series {
                        family: data.family.0.clone(),
                        persistence: Persistence::of(data),
                        before: Vec::new(),
                        kept: Vec::new(),
                    });
                    if inside {
                        held.kept.push(evaluation);
                    } else {
                        held.before.push(evaluation);
                    }
                }
                series.extend(kept.filter(|held| !held.kept.is_empty()));
            }
        }
        (
            excluded,
            measure(series, &runs, args.since.seconds.unsigned_abs()),
        )
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

/// The observed seconds of a series: its kept evaluations' sample timestamps, last minus first
/// plus the median spacing between consecutive ones (zero for one), capped at `window`.
fn observed_seconds(kept: &[SignalEvaluationData], window: u64) -> u64 {
    let mut seconds: Vec<i64> = kept
        .iter()
        .filter_map(|evaluation| Some(Instant::parse(&evaluation.ts.0)?.seconds()))
        .collect();
    seconds.sort_unstable();
    let (Some(first), Some(last)) = (seconds.first(), seconds.last()) else {
        return 0;
    };
    let mut spacings: Vec<i64> = seconds.windows(2).map(|pair| pair[1] - pair[0]).collect();
    spacings.sort_unstable();
    let median = match spacings.len() {
        0 => 0,
        n if n % 2 == 1 => spacings[n / 2],
        n => (spacings[n / 2 - 1] + spacings[n / 2]) / 2,
    };
    u64::try_from(last - first + median)
        .unwrap_or(0)
        .min(window)
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
fn measure(series: Vec<Series>, runs: &[Given], window: u64) -> BTreeMap<String, Family> {
    let mut families: BTreeMap<String, Family> = BTreeMap::new();
    for Series {
        family,
        persistence,
        before,
        kept,
    } in series
    {
        let measured = families.entry(family).or_insert_with(|| Family {
            settled: vec![0; runs.len()],
            ..Family::default()
        });
        measured.series += 1;
        measured.observed += observed_seconds(&kept, window);
        measured.raw_changes += verdict_changes(&kept);
        for (count, given) in measured.settled.iter_mut().zip(runs) {
            *count += settled_inside(given.over(persistence), &before, &kept);
        }
    }
    families
}

/// The changes `persistence` settles over `inside` after replaying the newest `max(k, k_crit, m)`
/// of `before` to seed the series'
/// settled verdict, both in sample `ts` order; the series' first settled verdict is not counted.
fn settled_inside(
    persistence: Persistence,
    before: &[SignalEvaluationData],
    inside: &[SignalEvaluationData],
) -> u64 {
    let mut hysteresis = SeriesHysteresis::new(persistence);
    let seed = persistence.k.max(persistence.k_crit).max(persistence.m);
    for evaluation in &before[before.len().saturating_sub(seed)..] {
        hysteresis.observe(evaluation);
    }
    inside
        .iter()
        .filter_map(|evaluation| hysteresis.observe(evaluation))
        .filter(|change| change.from.is_some())
        .count() as u64
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

/// `observed` seconds as thousandths of a series-day, rounded.
fn thousandths(observed: u64) -> u128 {
    (u128::from(observed) * 1000 + u128::from(DAY) / 2) / u128::from(DAY)
}

/// `observed` seconds as series-days, a decimal to three fractional digits: a whole number when
/// it is one.
fn series_days(observed: u64) -> Value {
    let thousandths = thousandths(observed);
    if thousandths.is_multiple_of(1000) {
        json!(thousandths / 1000)
    } else {
        json!(thousandths as f64 / 1000.0)
    }
}

/// `changes` per series-day of the rounded series-days of `observed` seconds, to three fractional
/// digits; `None` when they round to 0.
fn per_day(changes: u64, observed: u64) -> Option<f64> {
    let thousandths = thousandths(observed);
    (thousandths > 0)
        .then(|| (changes as f64 * 1000.0 / thousandths as f64 * 1000.0).round() / 1000.0)
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
                        series_days(family.observed),
                        family.raw_changes,
                        rate(per_day(family.raw_changes, family.observed)),
                        value(given.k),
                        value(given.k_crit),
                        value(given.m),
                        rate(per_day(*settled, family.observed)),
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
                                "settled_per_series_day": per_day(*settled, family.observed),
                            })
                        })
                        .collect();
                    json!({
                        "family": name,
                        "series": family.series,
                        "series_days": series_days(family.observed),
                        "raw_changes": family.raw_changes,
                        "raw_per_series_day": per_day(family.raw_changes, family.observed),
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
