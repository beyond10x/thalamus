//! `thalamus show [<signal-id>] [--format json|text]`: the current verdict of every series, its
//! newest evaluation (`thalamus.evaluation.SeriesVerdicts`: the row with the newest `ts`), worst
//! first, with the sample's value, the baseline, the score and the reason.
//!
//! Exit codes: the verdict ladder (design § 2.3, `thalamus_core::ladder::pass_exit`): the worst
//! shown verdict's code, `OK` 0, `WARN` 1, `CRIT` 2, `UNKNOWN` 3, and 3 when every shown verdict
//! is `N/A` or nothing is shown. A refusal exits with the codes of `thalamus sample`: 64 without
//! an instance, 65 for an invalid instance or an unknown signal, 74 when the store cannot be
//! opened or read.
//!
//! `show` only reads: when the SQLite file of the store does not exist yet, it answers as an
//! empty store (nothing shown, exit 3) without opening it, so it creates no file.

use std::cmp::Reverse;
use std::process::ExitCode;

use clap::ValueEnum;
use serde_json::{Value, json};
use thalamus_core::ladder;
use thalamus_model::evaluation::{Baseline, SignalEvaluationData};
use thalamus_model::observation::{Bound, Reading};
use thalamus_pipeline::evaluate::verdict_name;

use super::eval::{EX_DATAERR, EX_IOERR, ForReading, quietly};
use crate::global::Global;

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum Format {
    /// One line per series: verdict, series, ts, value, baseline, score, reason.
    Text,
    /// One JSON document: the shown verdicts, worst first.
    Json,
}

#[derive(Debug, clap::Args)]
pub struct Args {
    /// Show only this definition's series, by id.
    #[arg(value_name = "SIGNAL_ID")]
    signal: Option<String>,
    /// The output format.
    #[arg(long, value_enum, default_value_t = Format::Text)]
    format: Format,
}

/// One shown series: its newest evaluation and the reading of the sample it graded.
struct Row {
    evaluation: SignalEvaluationData,
    value: Option<Reading>,
}

pub fn run(global: &Global, args: &Args) -> ExitCode {
    let ForReading {
        instance,
        store,
        name,
    } = match ForReading::open("show", global) {
        Ok(opened) => opened,
        Err(code) => return code,
    };
    if let Some(signal) = &args.signal
        && instance.instance().definition(signal).is_none()
    {
        eprintln!(
            "thalamus show: refused: {signal}: unknown-signal: no definition of the instance has \
             this id"
        );
        return ExitCode::from(EX_DATAERR);
    }
    let read = quietly(|| {
        // A store that does not exist yet holds nothing: it shows as empty and is not created.
        let Some(store) = &store else {
            return Vec::new();
        };
        let mut rows: Vec<Row> = store
            .newest_evaluations()
            .into_iter()
            .filter(|evaluation| {
                args.signal
                    .as_ref()
                    .is_none_or(|signal| evaluation.signal.0 == *signal)
            })
            .map(|evaluation| Row {
                value: store
                    .sample(&evaluation.sample_id)
                    .map(|sample| sample.reading),
                evaluation,
            })
            .collect();
        rows.sort_by_key(|row| {
            (
                Reverse(ladder::rank(row.evaluation.verdict)),
                row.evaluation.series.0.clone(),
            )
        });
        rows
    });
    let Ok(rows) = read else {
        eprintln!("thalamus show: store {name} cannot be read");
        return ExitCode::from(EX_IOERR);
    };
    print(&rows, args.format);
    let code = ladder::pass_exit(rows.iter().map(|row| row.evaluation.verdict));
    ExitCode::from(u8::try_from(code).unwrap_or(EX_IOERR))
}

fn print(rows: &[Row], format: Format) {
    match format {
        Format::Text => {
            for row in rows {
                let evaluation = &row.evaluation;
                let baseline = evaluation.baseline.as_ref().map_or_else(
                    || "none".to_owned(),
                    |baseline| {
                        format!(
                            "{}[{}] {}",
                            baseline.kind.0,
                            baseline.offsets.join(","),
                            reading_text(&baseline.reading)
                        )
                    },
                );
                println!(
                    "{}\t{}\t{}\tvalue {}\tbaseline {}\tscore {}\t{}",
                    verdict_name(evaluation.verdict),
                    evaluation.series.0,
                    evaluation.ts.0,
                    row.value
                        .as_ref()
                        .map_or_else(|| "not stored".to_owned(), reading_text),
                    baseline,
                    evaluation
                        .score
                        .as_ref()
                        .map_or("none", |score| score.0.as_str()),
                    evaluation.reason.as_deref().unwrap_or(""),
                );
            }
        }
        Format::Json => {
            let verdicts: Vec<Value> = rows
                .iter()
                .map(|row| {
                    let evaluation = &row.evaluation;
                    json!({
                        "evaluation_id": evaluation.evaluation_id.0,
                        "sample_id": evaluation.sample_id.0,
                        "signal": evaluation.signal.0,
                        "series": evaluation.series.0,
                        "ts": evaluation.ts.0,
                        "definition_rev": evaluation.definition_rev,
                        "verdict": verdict_name(evaluation.verdict),
                        "prev": evaluation.prev.map(verdict_name),
                        "value": row.value.as_ref().map(reading_json),
                        "baseline": evaluation.baseline.as_ref().map(baseline_json),
                        "score": evaluation.score.as_ref().map(|score| score.0.as_str()),
                        "reason": evaluation.reason,
                    })
                })
                .collect();
            println!("{}", json!({ "verdicts": verdicts }));
        }
    }
}

fn bound_name(bound: Bound) -> &'static str {
    match bound {
        Bound::Lower => "lower",
        Bound::Upper => "upper",
    }
}

/// A reading as one JSON object: an `unknown` one carries its reason and no value.
fn reading_json(reading: &Reading) -> Value {
    match reading {
        Reading::Ok(complete) => json!({"quality": "ok", "value": complete.value.0}),
        Reading::Partial(partial) => json!({
            "quality": "partial", "value": partial.value.0, "bound": bound_name(partial.bound)
        }),
        Reading::Unknown(unread) => json!({"quality": "unknown", "reason": unread.reason}),
    }
}

fn baseline_json(baseline: &Baseline) -> Value {
    json!({
        "kind": baseline.kind.0,
        "offsets": baseline.offsets,
        "reading": reading_json(&baseline.reading),
    })
}

/// A reading as text: its value, a bound as `>= v` or `<= v`, and `unknown` without a value.
fn reading_text(reading: &Reading) -> String {
    match reading {
        Reading::Ok(complete) => complete.value.0.clone(),
        Reading::Partial(partial) => match partial.bound {
            Bound::Lower => format!(">= {}", partial.value.0),
            Bound::Upper => format!("<= {}", partial.value.0),
        },
        Reading::Unknown(_) => "unknown".to_owned(),
    }
}
