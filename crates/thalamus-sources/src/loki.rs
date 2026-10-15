//! Loki (LogQL) reads through a Connectors `loki` adapter instance (Connectors v0.45.0,
//! `adapters/loki/README.md`, the Loki profile § 11).
//!
//! | extractor | reads | each read |
//! |---|---|---|
//! | `loki.value` | `logs.query_metric` of `query.value`, instant at the window end | one per returned series, its value |
//! | `loki.ratio` | `logs.query_metric` of `query.numerator` and `query.denominator`, both instant at the window end | one per denominator label set, paired on the full label set; numerator over denominator, unit `1`; when every numerator series has a denominator twin, a label set the complete numerator lacks reads numerator 0 |
//! | `loki.count` | `logs.query_range` of `query.value` over the window, at most [`LINE_LIMIT`] lines | one per series of returned lines, how many; no line at all is 0 for a signal without dimensions and `unknown` for one with |
//!
//! A series' `dims` hold only the labels it carries that the definition declares (L9). A value
//! Loki gives as `NaN`, `+Inf`, `-Inf` or any text that is not a decimal is `unknown` with a
//! reason, never 0 (L5). A count whose answer reached the limit, or that Connectors calls
//! incomplete, is `partial` with bound `lower`. A Connectors refusal is the adapter's error, which
//! the registry turns into one `unknown` read naming Connectors' code and stage.

use std::cmp::Ordering;
use std::collections::BTreeMap;

use serde_json::{Value, json};
use thalamus_model::definition::{DenominatorSpec, SignalDefinitionData};
use thalamus_model::observation::{
    Bound, CompleteReading, PartialReading, Reading, RunId, TimeWindow, UnreadReading,
};
use thalamus_model::primitives::{Decimal, Timestamp};
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;

use crate::adapter::{Adapter, Population, Read, unknown, window_ending};
use crate::connectors::{ConnectorsError, ReadClient};

/// The Connectors operation of a metric query.
pub const QUERY_METRIC: &str = "logs.query_metric";
/// The Connectors operation of a log-line query.
pub const QUERY_RANGE: &str = "logs.query_range";
/// The `limit` of every `logs.query_range`: the largest Connectors admits (Loki profile § 11.3).
pub const LINE_LIMIT: u64 = 1000;
/// Fractional digits of a ratio, the last rounded half away from zero, trailing zeros dropped.
pub const RATIO_SCALE: usize = 18;

const OPERATIONS: [&str; 2] = [QUERY_METRIC, QUERY_RANGE];

/// The Loki (LogQL) adapter.
pub fn adapter() -> Option<Box<dyn Adapter>> {
    Some(Box::new(Loki))
}

/// Reads LogQL through Connectors.
#[derive(Debug, Clone, Copy, Default)]
pub struct Loki;

impl Adapter for Loki {
    fn read_operations(&self) -> &[&str] {
        &OPERATIONS
    }

    fn read(
        &self,
        client: &ReadClient<'_>,
        definition: &SignalDefinitionData,
        end: &Timestamp,
        _run_id: &RunId,
    ) -> Result<Vec<Read>, ConnectorsError> {
        let one = |reason: String| Ok(vec![unknown(definition, end, reason)]);
        let window = match window_ending(&definition.window, end) {
            Ok(window) => window,
            Err(why) => return one(why),
        };
        let at = Reads {
            client,
            definition,
            end,
            window,
        };
        let outcome = match definition.extractor.0.as_str() {
            "loki.value" => at.value(),
            "loki.ratio" => at.ratio(),
            "loki.count" => at.count(),
            other => Ok(Err(format!(
                "no Loki extractor {other}; there are loki.value, loki.ratio and loki.count"
            ))),
        };
        match outcome? {
            Ok(reads) => Ok(reads),
            Err(why) => one(why),
        }
    }
}

/// What a read of one definition at one window end needs.
struct Reads<'a> {
    client: &'a ReadClient<'a>,
    definition: &'a SignalDefinitionData,
    end: &'a Timestamp,
    window: TimeWindow,
}

/// The reads of a pass, or why the whole pass is one `unknown` read; a Connectors failure outside.
type Outcome = Result<Result<Vec<Read>, String>, ConnectorsError>;

/// Each series' full label set and the provider's text of its value, in provider order.
type Series = Vec<(Labels, String)>;

/// A label set, or the dims projected from one.
type Labels = BTreeMap<String, String>;

impl Reads<'_> {
    fn value(&self) -> Outcome {
        let query = match self.query("value") {
            Ok(query) => query,
            Err(why) => return Ok(Err(why)),
        };
        let series = match self.metric(query)? {
            Ok(series) => series,
            Err(why) => return Ok(Err(why)),
        };
        if series.is_empty() {
            return Ok(Err("Loki returned no series".to_string()));
        }
        let projected = series
            .into_iter()
            .map(|(labels, text)| (self.dims(&labels), text));
        let reads = grouped(projected)
            .into_iter()
            .map(|(dims, texts)| match one_of(&dims, texts) {
                Err(why) => self.read(dims, unread(why), None),
                Ok(text) => {
                    let reading = match parse(&text) {
                        Some(value) => ok(Decimal(value.rendered())),
                        None => unread(not_a_decimal("value", &text)),
                    };
                    self.read(dims, reading, Some(text))
                }
            })
            .collect();
        Ok(Ok(reads))
    }

    /// Numerator over denominator, series by series.
    ///
    /// A numerator series and a denominator series are paired on their full label sets, as
    /// LogQL's own `/` pairs them, and only then projected onto the definition's dimensions; two
    /// pairs projected onto the same dims are one `unknown` read naming the collision. When every
    /// numerator series has a denominator series of the same labels, a label set the denominator
    /// answers and the complete numerator answer lacks reads numerator 0: `count_over_time` emits
    /// nothing for a stream without a matching line. A numerator series without that twin, or an
    /// incomplete answer on either side, makes the whole pass one `unknown` read (design § 11
    /// D20).
    fn ratio(&self) -> Outcome {
        if let Some(unit) = &self.definition.unit
            && unit.0 != "1"
        {
            return Ok(Err(format!(
                "a ratio has unit 1; the definition declares unit {}",
                unit.0
            )));
        }
        let (numerator, denominator) = match (self.query("numerator"), self.query("denominator")) {
            (Ok(n), Ok(d)) => (n, d),
            (Err(why), _) | (_, Err(why)) => return Ok(Err(why)),
        };
        let numerator = match self.metric(numerator)? {
            Ok(series) => grouped(series),
            Err(why) => return Ok(Err(format!("numerator: {why}"))),
        };
        let denominator = match self.metric(denominator)? {
            Ok(series) => grouped(series),
            Err(why) => return Ok(Err(format!("denominator: {why}"))),
        };
        // Absence means 0 only when the two queries group alike, which every numerator series
        // having its denominator twin shows (design § 11 D20). Otherwise a missing numerator
        // series may be counted inside a coarser one, and no label set is read.
        let unmatched: Vec<String> = numerator
            .keys()
            .filter(|labels| !denominator.contains_key(*labels))
            .map(shown)
            .collect();
        if !unmatched.is_empty() {
            return Ok(Err(format!(
                "numerator series {} have no denominator series of the same labels; the two \
                 queries do not group alike",
                unmatched.join(", ")
            )));
        }
        if denominator.is_empty() {
            return Ok(Err("Loki returned no series".to_string()));
        }
        let spec = self.definition.denominator.as_ref();
        let mut pairs: BTreeMap<Labels, Vec<(Reading, Option<Population>)>> = BTreeMap::new();
        for (labels, texts) in &denominator {
            let numerator = match numerator.get(labels) {
                Some(texts) => {
                    one_of(labels, texts.clone()).map_err(|why| format!("numerator: {why}"))
                }
                None => Ok("0".to_string()),
            };
            let denominator =
                one_of(labels, texts.clone()).map_err(|why| format!("denominator: {why}"));
            let pair = match (numerator, denominator) {
                (Ok(n), Ok(d)) => ratio_of(spec, &n, &d),
                (Err(why), _) | (_, Err(why)) => (unread(why), None),
            };
            pairs.entry(self.dims(labels)).or_default().push(pair);
        }
        let reads = pairs
            .into_iter()
            .map(|(dims, mut pairs)| {
                if pairs.len() > 1 {
                    let why = collision(pairs.len(), &dims);
                    return self.read(dims, unread(why), None);
                }
                let (reading, population) = pairs.remove(0);
                let mut read = self.read(dims, reading, None);
                read.denominator = population;
                read
            })
            .collect();
        Ok(Ok(reads))
    }

    fn count(&self) -> Outcome {
        let query = match self.query("value") {
            Ok(query) => query,
            Err(why) => return Ok(Err(why)),
        };
        let (start, end) = match (unix_ns(&self.window.start), unix_ns(&self.window.end)) {
            (Ok(start), Ok(end)) => (start, end),
            (Err(why), _) | (_, Err(why)) => return Ok(Err(why)),
        };
        let input = json!({
            "query": query,
            "start_unix_ns": start,
            "end_unix_ns": end,
            "limit": LINE_LIMIT,
        });
        let answer = self
            .client
            .invoke(&self.definition.source, QUERY_RANGE, &input)?;
        let unusable = |what: &str| Ok(Err(format!("{QUERY_RANGE} answered {what}")));
        let Some(lines) = answer["lines"].as_array() else {
            return unusable("no `lines` list");
        };
        let Some(complete) = answer["complete"].as_bool() else {
            return unusable("no `complete` flag");
        };
        let mut counts: BTreeMap<BTreeMap<String, String>, u64> = BTreeMap::new();
        for line in lines {
            let Some(stream) = labels(&line["stream"]) else {
                return unusable("a line whose `stream` is not a map of text");
            };
            *counts.entry(self.dims(&stream)).or_default() += 1;
        }
        if counts.is_empty() {
            // A count of the whole instance is 0. A dimensioned signal has no series to put a 0
            // on, and a 0 at no dims would be a series the definition does not have.
            if self.declares_dimensions() {
                return Ok(Err(format!(
                    "{QUERY_RANGE} answered no series in the window"
                )));
            }
            counts.insert(BTreeMap::new(), 0);
        }
        // A full answer may have left lines out: Connectors keeps an answer at the cap partial
        // (Loki profile § 4.3), and this holds it so even when an answer says otherwise.
        let cut = !complete || lines.len() as u64 >= LINE_LIMIT;
        let reads = counts
            .into_iter()
            .map(|(dims, n)| {
                let value = Decimal(n.to_string());
                let reading = if cut {
                    Reading::Partial(PartialReading {
                        value,
                        bound: Bound::Lower,
                    })
                } else {
                    ok(value)
                };
                self.read(dims, reading, None)
            })
            .collect();
        Ok(Ok(reads))
    }

    /// The definition's query `name`.
    fn query(&self, name: &str) -> Result<&str, String> {
        self.definition
            .query
            .get(name)
            .map(|text| text.0.as_str())
            .ok_or_else(|| {
                format!(
                    "{} reads query.{name}, which the definition does not give",
                    self.definition.extractor.0
                )
            })
    }

    /// The series of an instant metric query at the window end, each as its full label set and
    /// value text, in provider order; `Err` when the answer is cut or not one a read can use.
    fn metric(&self, query: &str) -> Result<Result<Series, String>, ConnectorsError> {
        let time = match unix_ns(&self.window.end) {
            Ok(time) => time,
            Err(why) => return Ok(Err(why)),
        };
        let input = json!({"query": query, "time_unix_ns": time});
        let answer = self
            .client
            .invoke(&self.definition.source, QUERY_METRIC, &input)?;
        Ok(self.series(&answer))
    }

    fn series(&self, answer: &Value) -> Result<Series, String> {
        let unusable = |what: &str| format!("{QUERY_METRIC} answered {what}");
        match answer["complete"].as_bool() {
            Some(true) => {}
            Some(false) => {
                return Err(format!(
                    "{QUERY_METRIC} answered an incomplete result, truncation {}",
                    answer["truncation"]
                ));
            }
            None => return Err(unusable("no `complete` flag")),
        }
        let list = answer["series"]
            .as_array()
            .ok_or_else(|| unusable("no `series` list"))?;
        list.iter()
            .map(|series| {
                let labels = labels(&series["labels"])
                    .ok_or_else(|| unusable("labels that are not text"))?;
                let samples = series["samples"]
                    .as_array()
                    .ok_or_else(|| unusable("a series without `samples`"))?;
                let [sample] = samples.as_slice() else {
                    return Err(unusable(&format!(
                        "{} samples for one series of an instant query",
                        samples.len()
                    )));
                };
                let text = sample["value"]
                    .as_str()
                    .ok_or_else(|| unusable("a sample value that is not text"))?;
                Ok((labels, text.to_string()))
            })
            .collect()
    }

    /// Whether the definition declares at least one dimension.
    fn declares_dimensions(&self) -> bool {
        self.definition
            .dimensions
            .as_ref()
            .is_some_and(|keys| !keys.is_empty())
    }

    /// The labels the definition declares as dimensions; every other label is dropped.
    fn dims(&self, labels: &BTreeMap<String, String>) -> BTreeMap<String, String> {
        let declared = self.definition.dimensions.as_deref().unwrap_or_default();
        labels
            .iter()
            .filter(|(key, _)| declared.iter().any(|d| d.0 == **key))
            .map(|(key, value)| (key.clone(), value.clone()))
            .collect()
    }

    fn read(&self, dims: BTreeMap<String, String>, reading: Reading, raw: Option<String>) -> Read {
        let mut read = unknown(self.definition, self.end, String::new());
        read.window = self.window.clone();
        read.dims = dims;
        read.reading = reading;
        read.raw = raw;
        read
    }
}

/// The value texts of each label set, in provider order.
fn grouped(series: impl IntoIterator<Item = (Labels, String)>) -> BTreeMap<Labels, Vec<String>> {
    let mut groups: BTreeMap<_, Vec<String>> = BTreeMap::new();
    for (labels, text) in series {
        groups.entry(labels).or_default().push(text);
    }
    groups
}

/// The one value of `labels`; more than one series on the same labels is no reading.
fn one_of(labels: &Labels, mut texts: Vec<String>) -> Result<String, String> {
    match texts.len() {
        1 => Ok(texts.remove(0)),
        n => Err(collision(n, labels)),
    }
}

fn collision(n: usize, labels: &Labels) -> String {
    format!(
        "{n} series fall onto {}; the definition's dimensions do not tell them apart",
        shown(labels)
    )
}

fn shown(labels: &Labels) -> String {
    serde_json::to_string(labels).unwrap_or_default()
}

/// `numerator / denominator` and the population behind it, when the definition declares one and
/// the denominator is an exact whole count. A zero or negative denominator is no ratio.
fn ratio_of(
    spec: Option<&DenominatorSpec>,
    numerator: &str,
    denominator: &str,
) -> (Reading, Option<Population>) {
    let (Some(n), Some(d)) = (parse(numerator), parse(denominator)) else {
        let why = match parse(numerator) {
            None => not_a_decimal("numerator", numerator),
            Some(_) => not_a_decimal("denominator", denominator),
        };
        return (unread(why), None);
    };
    if d.negative {
        return (
            unread(format!("denominator {denominator} is negative")),
            None,
        );
    }
    let population = spec.zip(d.whole_count()).map(|(spec, n)| Population {
        n,
        unit: spec.unit.clone(),
    });
    if d.digits.is_empty() {
        let why = if n.digits.is_empty() {
            "empty population: numerator and denominator are 0".to_string()
        } else {
            format!("denominator is 0 while the numerator is {numerator}")
        };
        return (unread(why), population);
    }
    (ok(Decimal(n.over(&d, RATIO_SCALE))), population)
}

fn labels(value: &Value) -> Option<BTreeMap<String, String>> {
    value
        .as_object()?
        .iter()
        .map(|(key, value)| Some((key.clone(), value.as_str()?.to_string())))
        .collect()
}

fn ok(value: Decimal) -> Reading {
    Reading::Ok(CompleteReading { value })
}

fn unread(reason: String) -> Reading {
    Reading::Unknown(UnreadReading { reason })
}

fn not_a_decimal(what: &str, text: &str) -> String {
    format!("Loki {what} {text:?} is not a decimal number")
}

/// `at` in Unix nanoseconds, the canonical decimal Connectors' Loki inputs take.
fn unix_ns(at: &Timestamp) -> Result<String, String> {
    let instant = OffsetDateTime::parse(&at.0, &Rfc3339)
        .map_err(|e| format!("instant {:?} is not RFC 3339: {e}", at.0))?;
    let ns = instant.unix_timestamp_nanos();
    if ns < 0 {
        return Err(format!("instant {:?} is before the Unix epoch", at.0));
    }
    Ok(ns.to_string())
}

/// An exact decimal: a sign, the digits of its magnitude (no leading zero, empty for zero) and how
/// many of them are fractional.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Dec {
    negative: bool,
    digits: Vec<u8>,
    scale: usize,
}

/// The decimal of `text` in the wire grammar of a `Decimal`: an optional `-`, digits without a
/// leading zero, then an optional `.` and digits. `None` for anything else, `NaN` and `±Inf`
/// included.
fn parse(text: &str) -> Option<Dec> {
    let (negative, unsigned) = match text.strip_prefix('-') {
        Some(rest) => (true, rest),
        None => (false, text),
    };
    let (whole, fraction) = unsigned.split_once('.').unwrap_or((unsigned, ""));
    let digits = |part: &str| part.bytes().all(|byte| byte.is_ascii_digit());
    let fraction_ok = !unsigned.contains('.') || !fraction.is_empty();
    if whole.is_empty()
        || !digits(whole)
        || !digits(fraction)
        || !fraction_ok
        || (whole.len() > 1 && whole.starts_with('0'))
    {
        return None;
    }
    let magnitude = whole.bytes().chain(fraction.bytes()).map(|b| b - b'0');
    let digits = trimmed(magnitude.collect());
    Some(Dec {
        negative: negative && !digits.is_empty(),
        digits,
        scale: fraction.len(),
    })
}

impl Dec {
    /// The value as a population count: `None` unless it is a whole number that fits one. A
    /// fraction is no count, and its floor is not taken.
    fn whole_count(&self) -> Option<i64> {
        let split = self.digits.len().saturating_sub(self.scale);
        let (whole, fraction) = self.digits.split_at(split);
        if self.negative || fraction.iter().any(|digit| *digit != 0) {
            return None;
        }
        whole.iter().try_fold(0_i64, |n, digit| {
            n.checked_mul(10)?.checked_add(i64::from(*digit))
        })
    }

    /// The value in the wire grammar of a `Decimal`, one spelling per number: no trailing
    /// fractional zero, never `-0`.
    fn rendered(&self) -> String {
        render(self.negative, self.digits.clone(), self.scale)
    }

    /// `self / divisor` for a non-zero divisor, `scale` fractional digits deep, the last rounded
    /// half away from zero, in the wire grammar of a `Decimal`.
    fn over(&self, divisor: &Dec, scale: usize) -> String {
        // self / divisor = (a / 10^sa) / (b / 10^sb) = a·10^sb / (b·10^sa). One more digit than
        // `scale` decides the rounding.
        let mut dividend = self.digits.clone();
        dividend.resize(dividend.len() + divisor.scale + scale + 1, 0);
        let mut by = divisor.digits.clone();
        by.resize(by.len() + self.scale, 0);
        let mut quotient = Vec::with_capacity(dividend.len());
        let mut remainder: Vec<u8> = Vec::new();
        for digit in dividend {
            remainder.push(digit);
            remainder = trimmed(remainder);
            let mut times = 0;
            while magnitude_order(&remainder, &by) != Ordering::Less {
                remainder = sub(&remainder, &by);
                times += 1;
            }
            quotient.push(times);
        }
        let last = quotient.pop().unwrap_or(0);
        if last >= 5 {
            round_up(&mut quotient);
        }
        render(self.negative != divisor.negative, trimmed(quotient), scale)
    }
}

/// Drops leading zeros; zero is the empty vector.
fn trimmed(mut digits: Vec<u8>) -> Vec<u8> {
    let leading = digits.iter().take_while(|&&digit| digit == 0).count();
    digits.drain(..leading);
    digits
}

/// The order of two magnitudes without leading zeros.
fn magnitude_order(left: &[u8], right: &[u8]) -> Ordering {
    left.len().cmp(&right.len()).then_with(|| left.cmp(right))
}

/// `larger - smaller`, for magnitudes without leading zeros and `larger >= smaller`.
fn sub(larger: &[u8], smaller: &[u8]) -> Vec<u8> {
    let mut difference = Vec::with_capacity(larger.len());
    let mut borrow = 0;
    let mut smaller = smaller.iter().rev();
    for &digit in larger.iter().rev() {
        let take = smaller.next().copied().unwrap_or(0) + borrow;
        if digit >= take {
            difference.push(digit - take);
            borrow = 0;
        } else {
            difference.push(digit + 10 - take);
            borrow = 1;
        }
    }
    difference.reverse();
    trimmed(difference)
}

/// Adds one in the last place of a magnitude.
fn round_up(digits: &mut Vec<u8>) {
    for digit in digits.iter_mut().rev() {
        if *digit == 9 {
            *digit = 0;
        } else {
            *digit += 1;
            return;
        }
    }
    digits.insert(0, 1);
}

/// A magnitude with `fraction` fractional digits in the wire grammar of a `Decimal`: no leading
/// zero, no trailing fractional zero, never `-0`.
fn render(negative: bool, digits: Vec<u8>, fraction: usize) -> String {
    let mut digits = digits;
    if digits.len() <= fraction {
        let mut padded = vec![0; fraction + 1 - digits.len()];
        padded.extend_from_slice(&digits);
        digits = padded;
    }
    let (whole, part) = digits.split_at(digits.len() - fraction);
    let text = |part: &[u8]| -> String { part.iter().map(|d| char::from(b'0' + d)).collect() };
    let whole = text(whole);
    let whole = whole.trim_start_matches('0');
    let whole = if whole.is_empty() { "0" } else { whole };
    let part = text(part);
    let part = part.trim_end_matches('0');
    let sign = if negative && !(whole == "0" && part.is_empty()) {
        "-"
    } else {
        ""
    };
    if part.is_empty() {
        format!("{sign}{whole}")
    } else {
        format!("{sign}{whole}.{part}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use thalamus_model::definition::Unit;

    fn quotient(n: &str, d: &str) -> String {
        parse(n)
            .expect("numerator")
            .over(&parse(d).expect("denominator"), RATIO_SCALE)
    }

    #[test]
    fn quotient_is_exact_to_the_scale_and_rounds_half_away_from_zero() {
        assert_eq!(quotient("15", "556"), "0.02697841726618705");
        assert_eq!(quotient("3", "8"), "0.375");
        assert_eq!(quotient("1", "3"), "0.333333333333333333");
        assert_eq!(quotient("2", "3"), "0.666666666666666667");
        assert_eq!(quotient("-2", "3"), "-0.666666666666666667");
        assert_eq!(quotient("0.5", "0.25"), "2");
        assert_eq!(quotient("7.5", "2.5"), "3");
        assert_eq!(quotient("0", "4"), "0");
        assert_eq!(quotient("1", "0.001"), "1000");
    }

    #[test]
    fn only_the_decimal_grammar_parses() {
        for text in ["0", "42", "7.5", "-3", "0.25"] {
            assert!(parse(text).is_some(), "{text}");
        }
        for text in [
            "NaN", "+Inf", "-Inf", "Inf", "", "-", ".5", "5.", "01", "1e3", "+1", "0x10", " 1",
        ] {
            assert!(parse(text).is_none(), "{text:?}");
        }
    }

    #[test]
    fn a_population_is_an_exact_whole_count() {
        let spec = DenominatorSpec {
            unit: Unit("{line}".into()),
            min_n: 20,
            since: "first_sample".into(),
        };
        let n = |numerator: &str, denominator: &str| {
            ratio_of(Some(&spec), numerator, denominator).1.map(|p| p.n)
        };
        assert_eq!(n("15", "556"), Some(556));
        assert_eq!(n("1", "8.0"), Some(8));
        // A fraction is no count of anything: no population, never its floor.
        assert_eq!(n("1", "8.9"), None);
        assert_eq!(n("0.1", "0.4"), None);
        assert_eq!(n("1", "99999999999999999999"), None);
        assert_eq!(ratio_of(None, "15", "556").1, None);
        // The ratio itself stands without a population.
        assert_eq!(
            ratio_of(Some(&spec), "1", "2.5").0,
            ok(Decimal("0.4".into()))
        );
    }

    #[test]
    fn a_zero_denominator_is_unknown() {
        let spec = DenominatorSpec {
            unit: Unit("{line}".into()),
            min_n: 20,
            since: "first_sample".into(),
        };
        let (reading, population) = ratio_of(Some(&spec), "0", "0");
        assert!(
            matches!(&reading, Reading::Unknown(u) if u.reason.contains("empty population")),
            "{reading:?}"
        );
        assert_eq!(population.map(|p| p.n), Some(0));
        let (reading, _) = ratio_of(Some(&spec), "3", "0");
        assert!(matches!(reading, Reading::Unknown(_)), "{reading:?}");
        let (reading, _) = ratio_of(Some(&spec), "3", "-1");
        assert!(matches!(reading, Reading::Unknown(_)), "{reading:?}");
    }

    #[test]
    fn a_value_is_rendered_once_whatever_its_spelling() {
        for (text, rendered) in [
            ("-0", "0"),
            ("0.0", "0"),
            ("-0.00", "0"),
            ("7.50", "7.5"),
            ("42", "42"),
        ] {
            assert_eq!(
                parse(text).map(|d| d.rendered()).as_deref(),
                Some(rendered),
                "{text}"
            );
        }
    }
}
