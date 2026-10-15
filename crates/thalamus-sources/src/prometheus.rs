//! Prometheus (PromQL) reads through Connectors' `prometheus` adapter, with `series.query` only.
//!
//! Two extractors:
//!
//! - `promql.value` asks `query.value` as an instant query at the window end; each result series
//!   is one read of its value.
//! - `promql.ratio` asks `query.numerator` and `query.denominator` at the same instant and divides
//!   them series by series, joined on their `dims`. The reading is a fraction, unit `1`; the
//!   denominator, rounded down, is the population `n`, in the unit of the definition's
//!   `denominator`. `0 / 0` is `unknown` with reason [`EMPTY_POPULATION`] and `n = 0`. When both
//!   answers are complete, a series the denominator carries and the numerator lacks reads
//!   numerator 0 (design § 11 D20), provided the numerator answered at least one series and every
//!   numerator series has a denominator twin. A numerator answer with no series, or one with a
//!   series that has no twin (a global numerator over a per-cluster denominator), makes every read
//!   of the pass `unknown`, the latter naming the unmatched numerator series. A numerator series missing from the
//!   denominator is otherwise `unknown` with no population.
//!
//! Result labels named in the definition's `dimensions` become `dims`; every other label is
//! dropped, and a declared key the series does not carry stays unset. Series that fall onto the
//! same `dims` are all `unknown`, naming the collision. A value Prometheus could not give (`NaN`,
//! `+Inf`, `-Inf`, text that is no decimal) is `unknown`, never 0. A result Connectors marks cut
//! (`complete: false`, `series_truncated`, `samples_truncated`) makes every read of the pass
//! `unknown`: Prometheus does not say which side a cut result bounds, so no read is `partial`.
//!
//! The query text is sent as written, so a read at an offset is the same text at an earlier end.
//! `raw` is Prometheus's value text of a `promql.value` read; a ratio has no single producer
//! scalar, so it carries none. No `series.query` answer exposes the source's retention start, so
//! `retention_start` stays unset; `since` and `age_s` of the population are the pipeline's.

use std::collections::BTreeMap;

use serde_json::{Value, json};
use thalamus_model::definition::SignalDefinitionData;
use thalamus_model::observation::{CompleteReading, Reading, RunId, UnreadReading};
use thalamus_model::primitives::{Decimal, Timestamp};
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;

use crate::adapter::{Adapter, Population, Read, unknown, window_ending};
use crate::connectors::{ConnectorsError, ReadClient};

/// The one Connectors operation this adapter reads with: an instant query.
pub const SERIES_QUERY: &str = "series.query";

/// The reason of a ratio read whose numerator and denominator are both 0.
pub const EMPTY_POPULATION: &str = "empty population";

const READS: [&str; 1] = [SERIES_QUERY];

/// The latest evaluation time Connectors admits, in Unix seconds.
const MAX_TIME_UNIX_S: i64 = 9_999_999_999;

/// The Prometheus (PromQL) adapter.
pub fn adapter() -> Option<Box<dyn Adapter>> {
    Some(Box::new(Prometheus))
}

struct Prometheus;

impl Adapter for Prometheus {
    fn read_operations(&self) -> &[&str] {
        &READS
    }

    fn read(
        &self,
        client: &ReadClient<'_>,
        definition: &SignalDefinitionData,
        end: &Timestamp,
        _run_id: &RunId,
    ) -> Result<Vec<Read>, ConnectorsError> {
        let at = match window_ending(&definition.window, end).and_then(|_| instant(end)) {
            Ok(at) => at,
            Err(why) => return Ok(vec![unknown(definition, end, why)]),
        };
        let pass = Pass {
            client,
            definition,
            end,
            at,
        };
        match definition.extractor.0.as_str() {
            "promql.value" => pass.value(),
            "promql.ratio" => pass.ratio(),
            other => Ok(vec![unknown(
                definition,
                end,
                format!(
                    "the prometheus adapter has no extractor {other}; it reads promql.value and promql.ratio"
                ),
            )]),
        }
    }
}

/// The evaluation time of a read ending at `end`: its whole Unix seconds.
fn instant(end: &Timestamp) -> Result<i64, String> {
    let at = OffsetDateTime::parse(&end.0, &Rfc3339)
        .map_err(|e| format!("window end {:?} is not RFC 3339: {e}", end.0))?
        .unix_timestamp();
    if (0..=MAX_TIME_UNIX_S).contains(&at) {
        Ok(at)
    } else {
        Err(format!(
            "window end {:?} is outside the instants Prometheus can be asked at",
            end.0
        ))
    }
}

/// A series' dimension values, keyed by the definition's dimension keys.
type Dims = BTreeMap<String, String>;

/// The numerator and the denominator series of one `dims`.
type Sides<'a> = (Vec<&'a Series>, Vec<&'a Series>);

/// One series of an answer, its labels already narrowed to the definition's dimensions.
struct Series {
    dims: Dims,
    /// Prometheus's value text, or why the series has no single instant value.
    text: Result<String, String>,
}

/// A `series.query` answer.
struct Answer {
    series: Vec<Series>,
    /// Why the result is cut, when Connectors says it is.
    cut: Option<String>,
}

/// One read pass of one definition at one instant.
struct Pass<'a> {
    client: &'a ReadClient<'a>,
    definition: &'a SignalDefinitionData,
    end: &'a Timestamp,
    at: i64,
}

impl Pass<'_> {
    /// The text of `query.<name>`, or why the definition has none.
    fn query(&self, name: &str) -> Result<&str, String> {
        self.definition
            .query
            .get(name)
            .map(|text| text.0.as_str())
            .ok_or_else(|| {
                format!(
                    "{} reads query `{name}`, and the definition has none",
                    self.definition.extractor.0
                )
            })
    }

    /// `text` asked as an instant query at the window end; the inner error is an answer no read
    /// can use.
    fn ask(&self, text: &str) -> Result<Result<Answer, String>, ConnectorsError> {
        let input = json!({"query": text, "time_unix_s": self.at});
        let document = self
            .client
            .invoke(&self.definition.source, SERIES_QUERY, &input)?;
        Ok(answer(&document, self.at, &self.dimensions()))
    }

    fn dimensions(&self) -> Vec<&str> {
        self.definition
            .dimensions
            .iter()
            .flatten()
            .map(|key| key.0.as_str())
            .collect()
    }

    /// A read of `dims` with `reading`.
    fn read(
        &self,
        dims: &Dims,
        reading: Result<Decimal, String>,
        raw: Option<String>,
        denominator: Option<Population>,
    ) -> Read {
        let mut read = unknown(self.definition, self.end, String::new());
        read.dims = dims.clone();
        read.reading = match reading {
            Ok(value) => Reading::Ok(CompleteReading { value }),
            Err(reason) => Reading::Unknown(UnreadReading { reason }),
        };
        read.raw = raw;
        read.denominator = denominator;
        read
    }

    /// One `unknown` read with no dims.
    fn nothing(&self, reason: impl Into<String>) -> Vec<Read> {
        vec![unknown(self.definition, self.end, reason)]
    }

    fn value(&self) -> Result<Vec<Read>, ConnectorsError> {
        let text = match self.query("value") {
            Ok(text) => text,
            Err(why) => return Ok(self.nothing(why)),
        };
        let answer = match self.ask(text)? {
            Ok(answer) => answer,
            Err(why) => return Ok(self.nothing(why)),
        };
        if answer.series.is_empty() {
            return Ok(self.nothing(
                answer
                    .cut
                    .unwrap_or_else(|| "the query answered no series".to_string()),
            ));
        }
        let mut counts: BTreeMap<&Dims, usize> = BTreeMap::new();
        for series in &answer.series {
            *counts.entry(&series.dims).or_default() += 1;
        }
        Ok(answer
            .series
            .iter()
            .map(|series| {
                let reading = if let Some(cut) = &answer.cut {
                    Err(cut.clone())
                } else if counts[&series.dims] > 1 {
                    Err(collision(
                        &series.dims,
                        &format!("{} result series", counts[&series.dims]),
                    ))
                } else {
                    series.text.clone().and_then(|text| decimal(&text))
                };
                self.read(&series.dims, reading, series.text.clone().ok(), None)
            })
            .collect())
    }

    fn ratio(&self) -> Result<Vec<Read>, ConnectorsError> {
        if let Some(unit) = &self.definition.unit
            && unit.0 != "1"
        {
            return Ok(self.nothing(format!(
                "a ratio is a fraction with unit 1, and the definition declares unit {}",
                unit.0
            )));
        }
        let (numerator, denominator) = match (self.query("numerator"), self.query("denominator")) {
            (Ok(n), Ok(d)) => (n, d),
            (Err(why), _) | (_, Err(why)) => return Ok(self.nothing(why)),
        };
        let numerator = match self.ask(numerator)? {
            Ok(answer) => answer,
            Err(why) => return Ok(self.nothing(format!("numerator: {why}"))),
        };
        let denominator = match self.ask(denominator)? {
            Ok(answer) => answer,
            Err(why) => return Ok(self.nothing(format!("denominator: {why}"))),
        };
        let cut = [("numerator", &numerator), ("denominator", &denominator)]
            .into_iter()
            .filter_map(|(side, answer)| answer.cut.as_ref().map(|cut| format!("{side}: {cut}")))
            .collect::<Vec<_>>();
        let cut = (!cut.is_empty()).then(|| cut.join("; "));

        let mut joined: BTreeMap<&Dims, Sides<'_>> = BTreeMap::new();
        for series in &numerator.series {
            joined.entry(&series.dims).or_default().0.push(series);
        }
        for series in &denominator.series {
            joined.entry(&series.dims).or_default().1.push(series);
        }
        if joined.is_empty() {
            return Ok(
                self.nothing(cut.unwrap_or_else(|| "neither query answered a series".to_string()))
            );
        }

        // A missing numerator series reads 0 only when every numerator series has a denominator
        // twin (design § 11 D20). Otherwise the two queries are not shaped alike (a global
        // numerator over a per-cluster denominator), and no read of the pass is a count of 0.
        let unmatched: Vec<String> = joined
            .iter()
            .filter(|(_, (over, under))| !over.is_empty() && under.is_empty())
            .map(|(dims, _)| selector(dims))
            .collect();
        let needs_zero = joined
            .values()
            .any(|(over, under)| over.is_empty() && !under.is_empty());
        let unshaped = (needs_zero && !unmatched.is_empty()).then(|| {
            format!(
                "numerator series {} have no denominator series, so a denominator series without \
                 a numerator series is not a count of 0",
                unmatched.join(", ")
            )
        });
        // The zero inference needs at least one numerator series to show the shapes alike: a
        // complete numerator answer with none over a non-empty denominator reads nothing as 0.
        let empty = (numerator.series.is_empty() && !denominator.series.is_empty()).then(|| {
            "the numerator answered no series, so no denominator series is a count of 0".to_string()
        });
        let unread = cut.or(unshaped).or(empty);

        let mut reads = Vec::new();
        for (dims, (over, under)) in joined {
            if let Some(why) = &unread {
                for _ in 0..over.len().max(under.len()) {
                    reads.push(self.read(dims, Err(why.clone()), None, None));
                }
                continue;
            }
            if over.len() > 1 || under.len() > 1 {
                let why = collision(
                    dims,
                    &format!(
                        "{} numerator and {} denominator series",
                        over.len(),
                        under.len()
                    ),
                );
                for _ in 0..over.len().max(under.len()) {
                    reads.push(self.read(dims, Err(why.clone()), None, None));
                }
                continue;
            }
            let (reading, population) = self.divide(dims, over.first(), under.first());
            reads.push(self.read(dims, reading, None, population));
        }
        Ok(reads)
    }

    /// The ratio of one numerator series over one denominator series of the same `dims`, and the
    /// population the denominator counted, when it counted one.
    fn divide(
        &self,
        dims: &Dims,
        over: Option<&&Series>,
        under: Option<&&Series>,
    ) -> (Result<Decimal, String>, Option<Population>) {
        let Some(under) = under else {
            return (
                Err(format!(
                    "the denominator answered no series for {}",
                    selector(dims)
                )),
                None,
            );
        };
        let d = match under.text.clone().and_then(|text| number(&text)) {
            Ok(d) => d,
            Err(why) => return (Err(format!("denominator: {why}")), None),
        };
        if d < 0.0 {
            return (
                Err(format!("denominator: a population of {d} is negative")),
                None,
            );
        }
        // Below 2^63 the floor is an exact integer, so the conversion loses nothing.
        let floor = d.floor();
        if floor >= 9_223_372_036_854_775_808.0 {
            return (
                Err(format!("denominator: a population of {d} exceeds a count")),
                None,
            );
        }
        let population = self.definition.denominator.as_ref().map(|spec| Population {
            n: floor as i64,
            unit: spec.unit.clone(),
        });
        // Both answers are complete here, and when the numerator lacks this series it answered some
        // series, each with a denominator twin (the pass is unknown before the join otherwise). A
        // labelled counter child exists only after its first increment, so it counted 0 (D20).
        let n = match over.map(|over| over.text.clone().and_then(|text| number(&text))) {
            None => 0.0,
            Some(Ok(n)) => n,
            Some(Err(why)) => return (Err(format!("numerator: {why}")), population),
        };
        if d == 0.0 {
            let why = if n == 0.0 {
                EMPTY_POPULATION.to_string()
            } else {
                format!("a numerator of {n} over a denominator of 0")
            };
            return (Err(why), population);
        }
        let ratio = n / d;
        let reading = if ratio.is_finite() {
            Ok(Decimal(render(ratio)))
        } else {
            Err(format!("{n} / {d} is not a finite number"))
        };
        (reading, population)
    }
}

/// The reason of the series `which` falling onto the same `dims`.
fn collision(dims: &Dims, which: &str) -> String {
    format!(
        "{which} collide on {}; the labels that tell them apart are not dimensions",
        selector(dims)
    )
}

/// `dims` written as a PromQL selector, `{cluster="alpha"}`.
fn selector(dims: &Dims) -> String {
    let pairs: Vec<String> = dims.iter().map(|(k, v)| format!("{k}={v:?}")).collect();
    format!("{{{}}}", pairs.join(", "))
}

/// The answer in `document`, evaluated at `at`, its labels narrowed to `dimensions`.
fn answer(document: &Value, at: i64, dimensions: &[&str]) -> Result<Answer, String> {
    match document["time_unix_s"].as_i64() {
        Some(evaluated) if evaluated == at => {}
        Some(evaluated) => {
            return Err(format!(
                "prometheus evaluated the query at {evaluated}, not at the {at} asked"
            ));
        }
        None => return Err("the answer names no evaluation time".to_string()),
    }
    match document["result_type"].as_str() {
        Some("vector" | "scalar") => {}
        Some(other) => {
            return Err(format!(
                "a {other} result is not an instant value per series"
            ));
        }
        None => return Err("the answer names no result type".to_string()),
    }
    let complete = document["complete"]
        .as_bool()
        .ok_or_else(|| "the answer does not say whether it is complete".to_string())?;
    let entries = document["series"]
        .as_array()
        .ok_or_else(|| "the answer carries no series list".to_string())?;

    let mut cuts = Vec::new();
    if !complete {
        cuts.push("complete: false".to_string());
    }
    if document["series_truncated"] == Value::Bool(true) {
        cuts.push("series_truncated".to_string());
    }
    let mut series = Vec::with_capacity(entries.len());
    for entry in entries {
        let labels = entry["labels"]
            .as_object()
            .ok_or_else(|| "a series carries no labels".to_string())?;
        if entry["samples_truncated"] == Value::Bool(true)
            && !cuts.iter().any(|c| c == "samples_truncated")
        {
            cuts.push("samples_truncated".to_string());
        }
        let dims = dimensions
            .iter()
            .filter_map(|key| {
                labels
                    .get(*key)
                    .and_then(Value::as_str)
                    .map(|value| ((*key).to_string(), value.to_string()))
            })
            .collect();
        series.push(Series {
            dims,
            text: sample(entry),
        });
    }
    let warnings: Vec<&str> = document["warnings"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .collect();
    if !warnings.is_empty() && cuts.is_empty() {
        cuts.push("warnings".to_string());
    }
    let cut = (!cuts.is_empty()).then(|| {
        let mut why = format!("the result is cut ({})", cuts.join(", "));
        if !warnings.is_empty() {
            let quoted: Vec<String> = warnings.iter().map(|w| format!("{w:?}")).collect();
            why.push_str(&format!("; warnings: {}", quoted.join(", ")));
        }
        why.push_str("; a cut read is unknown");
        why
    });
    Ok(Answer { series, cut })
}

/// The value text of a series' one sample, or why it has none.
fn sample(entry: &Value) -> Result<String, String> {
    let samples = entry["samples"]
        .as_array()
        .ok_or_else(|| "the series carries no samples".to_string())?;
    match samples.as_slice() {
        [one] => one[1]
            .as_str()
            .map(str::to_string)
            .ok_or_else(|| format!("the sample {one} carries no value text")),
        other => Err(format!(
            "the series carries {} samples, an instant read takes one",
            other.len()
        )),
    }
}

/// Prometheus's value `text` as a finite number, or why it is none.
fn number(text: &str) -> Result<f64, String> {
    let n: f64 = text
        .parse()
        .map_err(|_| format!("prometheus value {text:?} is not a decimal"))?;
    if n.is_finite() {
        Ok(n)
    } else {
        Err(format!("prometheus value {text:?} is not a finite number"))
    }
}

/// Prometheus's value `text` as a decimal, or why it is none.
fn decimal(text: &str) -> Result<Decimal, String> {
    number(text).map(|n| Decimal(render(n)))
}

/// A finite number in the wire grammar of a decimal: no exponent, no leading zero, never `-0`.
fn render(n: f64) -> String {
    if n == 0.0 {
        "0".to_string()
    } else {
        format!("{n}")
    }
}

#[cfg(test)]
mod tests {
    //! Answers the Connectors contract rules out (an evaluation time other than the one sent,
    //! `samples_truncated` on an instant vector), so no recording carries them; the adapter still
    //! refuses to read them as this window's values.

    use serde_json::{Value, json};

    use super::answer;

    const AT: i64 = 1_792_018_623;

    fn document(time_unix_s: i64, samples_truncated: bool) -> Value {
        json!({
            "result_type": "vector",
            "series": [{
                "labels": {"cluster": "alpha"},
                "samples": [[time_unix_s, "4"]],
                "samples_truncated": samples_truncated
            }],
            "series_truncated": false,
            "complete": !samples_truncated,
            "warnings": [],
            "infos": [],
            "time_unix_s": time_unix_s
        })
    }

    #[test]
    fn an_answer_evaluated_at_another_instant_is_refused() {
        let Err(why) = answer(&document(1_700_000_000, false), AT, &["cluster"]) else {
            panic!("an answer for another instant was read");
        };
        assert!(
            why.contains("1700000000") && why.contains("1792018623"),
            "{why}"
        );
    }

    #[test]
    fn a_series_with_samples_cut_cuts_the_pass() {
        let Ok(read) = answer(&document(AT, true), AT, &["cluster"]) else {
            panic!("the answer is readable");
        };
        let cut = read.cut.unwrap_or_default();
        assert!(cut.contains("samples_truncated"), "{cut}");
    }
}
