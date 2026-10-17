//! `thalamus rollup [--day YYYY-MM-DD] [--signal <id>] [--format text|json]`: one row per series
//! per UTC day (design § 4), for every series with a sample on a closed UTC day (default:
//! yesterday), of `--signal` only when it is given. Each row is built by
//! `thalamus_core::rollup` and recorded through the generated `RecordRollup`
//! (`EventlogStore::roll_up_day`).
//!
//! A day that has not closed (today, or later) is refused and nothing is written. A series
//! already rolled up for the day is not rolled up again: a re-run writes nothing and prints the
//! rows that exist, marked `held`.
//!
//! Exit codes, none of them a verdict ladder code (0 to 3, design § 2.3):
//!
//! | code | when |
//! |---|---|
//! | 0 | every series of the day is rolled up, now or before |
//! | 64 | a usage error: no instance, a day that is not `YYYY-MM-DD`, a day that has not closed |
//! | 65 | the instance is invalid, or a series was refused, or `--signal` names a signal no definition has |
//! | 74 | the store cannot be opened or written; the rows recorded before a failure stay recorded |

use std::process::ExitCode;

use clap::ValueEnum;
use serde_json::{Value, json};
use thalamus_core::rollup::{DAY_SECONDS, day_text, parse_day};
use thalamus_model::definition::SignalId;
use thalamus_model::rollup::DailyRollupData;
use thalamus_pipeline::evaluate::verdict_name;
use thalamus_store::rollups::RolledUp;
use time::OffsetDateTime;

use super::eval::{EX_DATAERR, EX_IOERR, EX_USAGE, Opened, quietly};
use crate::global::Global;

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum Format {
    /// One line per row: id, counts, values, worst verdict, and whether it was recorded or held.
    Text,
    /// One JSON document: the rows recorded, the rows held and the refusals.
    Json,
}

#[derive(Debug, clap::Args)]
pub struct Args {
    /// The closed UTC day to roll up, `YYYY-MM-DD`. Default: yesterday.
    #[arg(long, value_name = "YYYY-MM-DD", value_parser = parse_day_arg)]
    day: Option<i64>,
    /// Roll up only this definition's series, by id.
    #[arg(long, value_name = "SIGNAL_ID")]
    signal: Option<String>,
    /// The output format.
    #[arg(long, value_enum, default_value_t = Format::Text)]
    format: Format,
}

fn parse_day_arg(text: &str) -> Result<i64, String> {
    parse_day(text).ok_or_else(|| format!("`{text}` is not a UTC day YYYY-MM-DD"))
}

pub fn run(global: &Global, args: &Args) -> ExitCode {
    let now = OffsetDateTime::now_utc().unix_timestamp();
    let today = now.div_euclid(DAY_SECONDS) * DAY_SECONDS;
    let day = args.day.unwrap_or(today - DAY_SECONDS);
    if day + DAY_SECONDS > now {
        eprintln!(
            "thalamus rollup: the UTC day {} has not closed; nothing was rolled up",
            day_text(day)
        );
        return ExitCode::from(EX_USAGE);
    }
    let Opened {
        mut instance,
        mut store,
        name,
    } = match Opened::open("rollup", global) {
        Ok(opened) => opened,
        Err(code) => return code,
    };
    let signal = args.signal.clone().map(SignalId);
    if let Some(signal) = &signal
        && instance.instance().definition(&signal.0).is_none()
    {
        eprintln!(
            "thalamus rollup: no definition has the signal id {}; nothing was rolled up",
            signal.0
        );
        return ExitCode::from(EX_DATAERR);
    }
    let answer = quietly(|| store.roll_up_day(instance.instance_mut(), day, signal.as_ref()));
    let rolled = match answer {
        Ok(Ok(rolled)) => rolled,
        Ok(Err(error)) => {
            eprintln!("thalamus rollup: store {name} refused: {error}");
            return ExitCode::from(EX_IOERR);
        }
        Err(_) => {
            eprintln!("thalamus rollup: store {name} cannot be read or written");
            return ExitCode::from(EX_IOERR);
        }
    };
    print(args.format, day, &rolled);
    for refusal in &rolled.refused {
        eprintln!("thalamus rollup: {refusal}");
    }
    if rolled.refused.is_empty() {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(EX_DATAERR)
    }
}

fn print(format: Format, day: i64, rolled: &RolledUp) {
    match format {
        Format::Text => {
            for (rows, how) in [(&rolled.recorded, "recorded"), (&rolled.held, "held")] {
                for row in rows {
                    println!("{}", line(row, how));
                }
            }
        }
        Format::Json => {
            let rows = |rows: &[DailyRollupData]| rows.iter().map(row_json).collect::<Vec<_>>();
            println!(
                "{}",
                json!({
                    "day": day_text(day),
                    "recorded": rows(&rolled.recorded),
                    "held": rows(&rolled.held),
                    "refused": rolled.refused,
                })
            );
        }
    }
}

/// One row as a text line; an absent value is `-`, never 0.
fn line(row: &DailyRollupData, how: &str) -> String {
    let value = |value: &Option<thalamus_model::primitives::Decimal>| {
        value
            .as_ref()
            .map_or("-", |value| value.0.as_str())
            .to_owned()
    };
    let counts = &row.verdict_counts;
    format!(
        "{} n={} unknown={} partial={} min={} p50={} max={} last={} worst={} \
         ok={} na={} warn={} crit={} verdict_unknown={} min_score={} max_score={} {how}",
        row.rollup_id.0,
        row.n,
        row.unknown,
        row.partial,
        value(&row.min),
        value(&row.p50),
        value(&row.max),
        value(&row.last),
        row.worst_verdict.map_or("-", verdict_name),
        counts.ok,
        counts.na,
        counts.warn,
        counts.crit,
        counts.unknown,
        value(&row.min_score),
        value(&row.max_score),
    )
}

/// One row as JSON; an absent value is `null`, never 0.
fn row_json(row: &DailyRollupData) -> Value {
    let value = |value: &Option<thalamus_model::primitives::Decimal>| {
        value.as_ref().map(|value| value.0.clone())
    };
    let counts = &row.verdict_counts;
    json!({
        "rollup_id": row.rollup_id.0,
        "signal": row.signal.0,
        "series": row.series.0,
        "day": row.day.0,
        "n": row.n,
        "unknown": row.unknown,
        "partial": row.partial,
        "min": value(&row.min),
        "p50": value(&row.p50),
        "max": value(&row.max),
        "last": value(&row.last),
        "worst_verdict": row.worst_verdict.map(verdict_name),
        "verdict_counts": {
            "ok": counts.ok,
            "na": counts.na,
            "warn": counts.warn,
            "crit": counts.crit,
            "unknown": counts.unknown,
        },
        "min_score": value(&row.min_score),
        "max_score": value(&row.max_score),
    })
}
