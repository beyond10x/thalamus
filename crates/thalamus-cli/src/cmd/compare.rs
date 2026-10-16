//! `thalamus compare <signal> [--dim key=value]... --vs <offset>`: a series against its own past,
//! answered from stored samples only. It reads the store's series range read and the definition's
//! cadence, and constructs no source adapter, so it answers for a `history: store` signal and
//! while the source is unreachable (unlike the `offset_7d` baseline, which re-reads the source).
//!
//! The current sample is the newest stored sample of the series at or before `--at` (default:
//! now). The baseline sample is the stored sample nearest to its `ts` minus the offset, within
//! half the definition's cadence and before the current sample; for `7d` the weekday matches by
//! construction. A series begins at its definition's resolved `denominator.since` (the first stored
//! sample when it declares none); a slot before that predates the series. Both readings, the delta
//! (current minus baseline) and the ratio (current over baseline, 18 fractional digits at most,
//! the last rounded half away from zero) are printed.
//!
//! Exit codes:
//!
//! | code | when |
//! |---|---|
//! | 0 | both sides were read completely and compared |
//! | 3 | `UNKNOWN`: a side is missing, `unknown` or `partial`, or the offset predates the series (the verdict ladder's code, design § 2.3) |
//! | 64 | a usage error: no instance, an offset, a timestamp or a `--dim` that does not parse |
//! | 65 | the instance is invalid, the signal is not defined, or a dim is not admitted |
//! | 74 | the store cannot be opened or read |
//!
//! An unknown or partial side never prints a number: an `unknown` reading carries none, and a
//! `partial` one carries only a bound, which is not the value.

use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::process::ExitCode;

use clap::ValueEnum;
use serde_json::{Map, Value, json};
use thalamus_core::behaviour::observation::Instant;
use thalamus_core::ladder::EXIT_UNKNOWN;
use thalamus_core::series::{admit_dims, series_key};
use thalamus_core::since::resolve_since;
use thalamus_model::dimension::DimensionValue;
use thalamus_model::observation::{Bound, Reading, SeriesKey, SignalSampleData};
use thalamus_model::primitives::Timestamp;
use thalamus_pipeline::instance::LoadedInstance;
use thalamus_store::eventlog::{EventlogStore, Locator, StoreError};
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;

use crate::global::Global;

const EX_USAGE: u8 = 64;
const EX_DATAERR: u8 = 65;
const EX_IOERR: u8 = 74;

/// The reason of a baseline whose slot lies before the first sample of the series (design § 3.1
/// rule 2).
const PREDATES: &str = "baseline predates series";
/// The reason of a baseline slot no stored sample is near enough to.
const NO_SAMPLE_AT_OFFSET: &str = "no stored sample at offset";

/// The fractional digits a ratio that does not terminate keeps.
const RATIO_SCALE: usize = 18;

/// The first instant a range read can name: every sample of a series is at or after it.
const EARLIEST: &str = "0000-01-01T00:00:00Z";
/// An instant after the last RFC 3339 instant in UTC: that instant's local time at the furthest
/// offset, `-23:59`.
const BEYOND_LAST: &str = "9999-12-31T23:59:59.999999999-23:59";

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum Format {
    /// One line per part: series, offset, both readings, the delta and the ratio, or `UNKNOWN`
    /// with its reason.
    Text,
    /// One JSON document.
    Json,
}

#[derive(Debug, clap::Args)]
pub struct Args {
    /// The definition whose series is compared, by id.
    #[arg(value_name = "SIGNAL_ID")]
    signal: String,
    /// A dimension of the series, `key=value`; once per dimension.
    #[arg(long = "dim", value_name = "KEY=VALUE", value_parser = parse_dim)]
    dims: Vec<(String, String)>,
    /// How far back the baseline is: `<n>s`, `<n>m`, `<n>h`, `<n>d` or `<n>w` (`7d`).
    #[arg(long, value_name = "OFFSET", value_parser = parse_offset)]
    vs: Offset,
    /// The latest instant the current sample may have, RFC 3339. Default: now.
    #[arg(long, value_name = "TIMESTAMP")]
    at: Option<String>,
    /// The output format.
    #[arg(long, value_enum, default_value_t = Format::Text)]
    format: Format,
}

/// An offset as written, with its length.
#[derive(Debug, Clone)]
struct Offset {
    text: String,
    seconds: i64,
}

fn parse_dim(text: &str) -> Result<(String, String), String> {
    match text.split_once('=') {
        Some((key, value)) if !key.is_empty() => Ok((key.to_owned(), value.to_owned())),
        _ => Err(format!("`{text}` is not key=value")),
    }
}

fn parse_offset(text: &str) -> Result<Offset, String> {
    if !text
        .chars()
        .last()
        .is_some_and(|unit| unit.is_ascii_lowercase())
    {
        return Err(format!("`{text}` is not <n>s, <n>m, <n>h, <n>d or <n>w"));
    }
    match seconds_of(text) {
        Some(seconds) if seconds > 0 => Ok(Offset {
            text: text.to_owned(),
            seconds,
        }),
        _ => Err(format!(
            "`{text}` is not a positive <n>s, <n>m, <n>h, <n>d or <n>w"
        )),
    }
}

/// The length in seconds of a short duration (`<n>s|m|h|d|w`) or of an ISO 8601 duration of
/// weeks, days, hours, minutes and whole seconds, the form a loaded definition stores its
/// cadence in (`PT5M`, `P1W`). `None` for anything else.
fn seconds_of(text: &str) -> Option<i64> {
    let number = |digits: &str| -> Option<i64> {
        (!digits.is_empty() && digits.bytes().all(|byte| byte.is_ascii_digit()))
            .then(|| digits.parse().ok())
            .flatten()
    };
    let scale = |unit: char| match unit {
        's' | 'S' => Some(1),
        'm' | 'M' => Some(60),
        'h' | 'H' => Some(3_600),
        'd' | 'D' => Some(86_400),
        'w' | 'W' => Some(604_800),
        _ => None,
    };
    if let Some(unit) = text.chars().last().filter(char::is_ascii_lowercase) {
        return number(&text[..text.len() - 1])?.checked_mul(scale(unit)?);
    }
    let rest = text.strip_prefix('P')?;
    let (date, time) = match rest.split_once('T') {
        Some((_, "")) => return None,
        Some((date, time)) => (date, Some(time)),
        None => (rest, None),
    };
    let mut total: i64 = 0;
    let mut parts = 0;
    for (part, units) in [(date, "WD"), (time.unwrap_or(""), "HMS")] {
        let mut digits = String::new();
        let mut allowed = units;
        for character in part.chars() {
            if character.is_ascii_digit() {
                digits.push(character);
                continue;
            }
            let at = allowed.find(character)?;
            allowed = &allowed[at + 1..];
            total = total.checked_add(number(&digits)?.checked_mul(scale(character)?)?)?;
            digits.clear();
            parts += 1;
        }
        if !digits.is_empty() {
            return None;
        }
    }
    (parts > 0).then_some(total)
}

pub fn run(global: &Global, args: &Args) -> ExitCode {
    let Some(dir) = &global.instance else {
        eprintln!("thalamus compare: no instance; give --instance <dir> or set THALAMUS_INSTANCE");
        return ExitCode::from(EX_USAGE);
    };
    let at = match &args.at {
        Some(at) => match nanos(at) {
            Some(nanos) => (at.clone(), nanos),
            None => {
                eprintln!("thalamus compare: --at `{at}` is not an RFC 3339 timestamp");
                return ExitCode::from(EX_USAGE);
            }
        },
        None => {
            let now = OffsetDateTime::now_utc();
            match now
                .replace_nanosecond(0)
                .ok()
                .and_then(|now| now.format(&Rfc3339).ok())
            {
                Some(text) => (text, now.unix_timestamp_nanos()),
                None => {
                    eprintln!("thalamus compare: the clock reads no RFC 3339 instant");
                    return ExitCode::from(EX_IOERR);
                }
            }
        }
    };

    let instance = match LoadedInstance::load(dir) {
        Ok(instance) => instance,
        Err(errors) => {
            for error in &errors {
                eprintln!("thalamus compare: invalid instance: {error}");
            }
            return ExitCode::from(EX_DATAERR);
        }
    };
    let Some(loaded) = instance.instance().definition(&args.signal) else {
        eprintln!(
            "thalamus compare: refused: `{}` names no definition of the instance",
            args.signal
        );
        return ExitCode::from(EX_DATAERR);
    };
    let definition = loaded.data();
    let dims: BTreeMap<String, DimensionValue> = args
        .dims
        .iter()
        .map(|(key, value)| (key.clone(), DimensionValue(value.clone())))
        .collect();
    if let Err(refusal) = admit_dims(
        instance.instance().registry(),
        definition,
        &dims,
        instance.values(),
    ) {
        eprintln!("thalamus compare: refused: {refusal}");
        return ExitCode::from(EX_DATAERR);
    }
    let Some(cadence) = seconds_of(&definition.cadence.0).filter(|seconds| *seconds > 0) else {
        eprintln!(
            "thalamus compare: refused: the cadence `{}` of `{}` has no length in seconds",
            definition.cadence.0, args.signal
        );
        return ExitCode::from(EX_DATAERR);
    };
    let series = series_key(&definition.signal_id, &dims);

    let locator = match global.store_locator(dir).parse::<Locator>() {
        Ok(locator) => locator,
        Err(error) => {
            eprintln!("thalamus compare: store refused: {}", refusal(&error));
            return ExitCode::from(EX_IOERR);
        }
    };
    // A read creates nothing: an instance whose store file does not exist yet holds no samples.
    let stored: Vec<SignalSampleData> = if matches!(&locator, Locator::Sqlite(path) if !path.exists())
    {
        Vec::new()
    } else {
        let store = match EventlogStore::open(&locator, instance.name()) {
            Ok(store) => store,
            Err(error) => {
                eprintln!("thalamus compare: store refused: {}", refusal(&error));
                return ExitCode::from(EX_IOERR);
            }
        };
        // The range end is exclusive: it is the instant after `--at`. The last RFC 3339 instant
        // has no successor in UTC, so the end is spelled with the furthest offset instead, which
        // is later than it and still an instant.
        let to = after(at.1).unwrap_or_else(|| Timestamp(BEYOND_LAST.to_owned()));
        let from = Timestamp(EARLIEST.to_owned());
        match quietly(|| store.series_range(&series, &from, &to)) {
            Ok(range) => range.samples,
            Err(_) => {
                eprintln!("thalamus compare: the store cannot be read");
                return ExitCode::from(EX_IOERR);
            }
        }
    };

    let mut samples: Vec<(i128, SignalSampleData)> = stored
        .into_iter()
        .filter_map(|sample| Some((nanos(&sample.ts.0)?, sample)))
        .filter(|(ts, _)| *ts <= at.1)
        .collect();
    samples.sort_by_key(|(ts, _)| *ts);
    // The series begins at its definition's resolved `denominator.since` (design § 11 D18), the
    // first stored sample when the definition declares none or none resolves.
    let first_stored = samples.first().map(|(_, sample)| sample.ts.clone());
    let began = resolve_since(loaded, &dims, instance.values(), first_stored.clone(), None)
        .or(first_stored)
        .and_then(|since| nanos(&since.0));
    let comparison = compare(&samples, began, args.vs.seconds, cadence, &at.0);
    print(&series, args, &comparison);
    if comparison.unknown.is_some() {
        ExitCode::from(u8::try_from(EXIT_UNKNOWN).unwrap_or(3))
    } else {
        ExitCode::SUCCESS
    }
}

/// What a comparison found: the two samples, and why it is `UNKNOWN` when it is.
struct Comparison<'a> {
    current: Option<&'a SignalSampleData>,
    baseline: Option<&'a SignalSampleData>,
    unknown: Option<String>,
}

/// Compares the newest of `samples` (in `ts` order, none after `--at`; `began` is the instant the
/// series begins at, design § 11 D18) with the sample nearest to
/// its `ts` minus `offset` seconds, within half of `cadence` seconds and before it.
fn compare<'a>(
    samples: &'a [(i128, SignalSampleData)],
    began: Option<i128>,
    offset: i64,
    cadence: i64,
    at: &str,
) -> Comparison<'a> {
    let Some((now, current)) = samples.last() else {
        return Comparison {
            current: None,
            baseline: None,
            unknown: Some(format!("no stored sample at or before {at}")),
        };
    };
    let target = now - i128::from(offset) * 1_000_000_000;
    let tolerance = i128::from(cadence) * 500_000_000;
    let began = began.unwrap_or(*now);
    // A slot before the series' start predates the series, whatever lies near it.
    let baseline = if target < began {
        None
    } else {
        samples
            .iter()
            .filter(|(ts, _)| *ts < *now && (*ts - target).abs() <= tolerance)
            .min_by_key(|(ts, _)| (*ts - target).abs())
            .map(|(_, sample)| sample)
    };
    let unknown = unread("current", &current.reading).or_else(|| match baseline {
        None if target < began => Some(PREDATES.to_owned()),
        None => Some(NO_SAMPLE_AT_OFFSET.to_owned()),
        Some(baseline) => unread("baseline", &baseline.reading),
    });
    Comparison {
        current: Some(current),
        baseline,
        unknown,
    }
}

/// Why a reading cannot be compared, or `None` when it was read completely.
fn unread(side: &str, reading: &Reading) -> Option<String> {
    match reading {
        Reading::Ok(_) => None,
        Reading::Partial(_) => Some(format!("{side} reading partial")),
        Reading::Unknown(unread) => Some(format!("{side} reading unknown: {}", unread.reason)),
    }
}

fn print(series: &SeriesKey, args: &Args, comparison: &Comparison<'_>) {
    let values = match (comparison.current, comparison.baseline, &comparison.unknown) {
        (Some(current), Some(baseline), None) => match (&current.reading, &baseline.reading) {
            (Reading::Ok(current), Reading::Ok(baseline)) => {
                Dec::parse(&current.value.0).zip(Dec::parse(&baseline.value.0))
            }
            _ => None,
        },
        _ => None,
    };
    let delta = values
        .as_ref()
        .map(|(current, baseline)| current.minus(baseline).render());
    let ratio = values
        .as_ref()
        .map(|(current, baseline)| current.over(baseline).map(|ratio| ratio.render()));
    match args.format {
        Format::Text => {
            println!("{:<9}{}", "series", series.0);
            println!("{:<9}{}", "vs", args.vs.text);
            println!("{:<9}{}", "current", side_text(comparison.current));
            println!("{:<9}{}", "baseline", side_text(comparison.baseline));
            if let Some(reason) = &comparison.unknown {
                println!("{:<9}{reason}", "UNKNOWN");
            } else if let (Some(delta), Some(ratio)) = (delta, ratio) {
                println!("{:<9}{delta}", "delta");
                match ratio {
                    Some(ratio) => println!("{:<9}{ratio}", "ratio"),
                    None => println!("{:<9}undefined: the baseline is 0", "ratio"),
                }
            }
        }
        Format::Json => {
            let mut doc = Map::new();
            doc.insert("signal".into(), json!(args.signal));
            doc.insert("series".into(), json!(series.0));
            doc.insert("vs".into(), json!(args.vs.text));
            doc.insert("current".into(), side_json(comparison.current));
            doc.insert("baseline".into(), side_json(comparison.baseline));
            match (&comparison.unknown, delta, ratio) {
                (Some(reason), _, _) => {
                    doc.insert("result".into(), json!("UNKNOWN"));
                    doc.insert("reason".into(), json!(reason));
                }
                (None, Some(delta), Some(ratio)) => {
                    doc.insert("result".into(), json!("compared"));
                    doc.insert("delta".into(), json!(delta));
                    doc.insert("ratio".into(), json!(ratio));
                    if ratio.is_none() {
                        doc.insert("ratio_reason".into(), json!("the baseline is 0"));
                    }
                }
                (None, _, _) => {
                    doc.insert("result".into(), json!("compared"));
                }
            }
            println!("{}", Value::Object(doc));
        }
    }
}

/// One side as a text line after its label: its `ts` and its value, or what it is instead of a
/// value.
fn side_text(sample: Option<&SignalSampleData>) -> String {
    let Some(sample) = sample else {
        return "-  no sample".to_owned();
    };
    let shown = match &sample.reading {
        Reading::Ok(complete) => complete.value.0.clone(),
        Reading::Partial(_) => "partial (bound withheld)".to_owned(),
        Reading::Unknown(unread) => format!("unknown ({})", unread.reason),
    };
    format!("{}  {shown}", sample.ts.0)
}

/// One side as JSON: `null` when there is none; a `value` only when it was read completely.
fn side_json(sample: Option<&SignalSampleData>) -> Value {
    let Some(sample) = sample else {
        return Value::Null;
    };
    let mut side = Map::new();
    side.insert("sample_id".into(), json!(sample.sample_id.0));
    side.insert("ts".into(), json!(sample.ts.0));
    match &sample.reading {
        Reading::Ok(complete) => {
            side.insert("quality".into(), json!("ok"));
            side.insert("value".into(), json!(complete.value.0));
        }
        Reading::Partial(partial) => {
            side.insert("quality".into(), json!("partial"));
            let bound = match partial.bound {
                Bound::Lower => "lower",
                Bound::Upper => "upper",
            };
            side.insert("bound".into(), json!(bound));
        }
        Reading::Unknown(unread) => {
            side.insert("quality".into(), json!("unknown"));
            side.insert("reason".into(), json!(unread.reason));
        }
    }
    Value::Object(side)
}

/// The nanoseconds from 1970 of the instant `text` names, read by the crate's one instant reader
/// first, so nothing it refuses is placed in time.
fn nanos(text: &str) -> Option<i128> {
    Instant::parse(text)?;
    OffsetDateTime::parse(text, &Rfc3339)
        .ok()
        .map(OffsetDateTime::unix_timestamp_nanos)
}

/// The instant one nanosecond after `nanos`: the exclusive end of a range that holds `nanos`.
/// `None` for the last RFC 3339 instant, which has no successor.
fn after(nanos: i128) -> Option<Timestamp> {
    OffsetDateTime::from_unix_timestamp_nanos(nanos.checked_add(1)?)
        .ok()?
        .format(&Rfc3339)
        .ok()
        .map(Timestamp)
}

/// Runs `work` with a panic hook that prints nothing, then restores the hook it replaced. A store
/// that cannot be read panics with a message that may hold its locator.
fn quietly<T>(work: impl FnOnce() -> T) -> std::thread::Result<T> {
    let previous = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));
    let result = catch_unwind(AssertUnwindSafe(work));
    std::panic::set_hook(previous);
    result
}

/// A store error in one fixed sentence per kind, a locator refusal by its code: never the store's
/// own message, which may hold a path or a URL.
fn refusal(error: &StoreError) -> String {
    match error {
        StoreError::Locator { code, .. } => (*code).to_owned(),
        StoreError::Refused { code } => code.clone(),
        StoreError::Contended { .. } => "another writer kept appending".to_owned(),
        StoreError::Record(_) => "a record in the store fails its checks".to_owned(),
        StoreError::Runtime(_) => "the store's runtime cannot be started".to_owned(),
        StoreError::Eventlog(_) => "the store is unavailable".to_owned(),
    }
}

/// A decimal of the wire grammar (`-?digits(.digits)?`), unbounded: its magnitude as decimal
/// digits, most significant first and without leading zeros (zero is empty), and its scale.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Dec {
    negative: bool,
    digits: Vec<u8>,
    scale: usize,
}

impl Dec {
    fn parse(text: &str) -> Option<Dec> {
        let (negative, unsigned) = match text.strip_prefix('-') {
            Some(rest) => (true, rest),
            None => (false, text),
        };
        let (whole, fraction) = unsigned.split_once('.').unwrap_or((unsigned, ""));
        let digits = |part: &str| part.bytes().all(|byte| byte.is_ascii_digit());
        if whole.is_empty()
            || !digits(whole)
            || !digits(fraction)
            || (unsigned.contains('.') && fraction.is_empty())
            || (whole.len() > 1 && whole.starts_with('0'))
        {
            return None;
        }
        let magnitude = whole
            .bytes()
            .chain(fraction.bytes())
            .map(|byte| byte - b'0');
        Some(Dec::new(negative, magnitude.collect(), fraction.len()))
    }

    fn new(negative: bool, digits: Vec<u8>, scale: usize) -> Dec {
        let digits = trimmed(digits);
        Dec {
            negative: negative && !digits.is_empty(),
            digits,
            scale,
        }
    }

    /// The magnitude at `scale` fractional digits (`scale` at least the own scale).
    fn at_scale(&self, scale: usize) -> Vec<u8> {
        shifted(&self.digits, scale - self.scale)
    }

    /// `self - other`, exactly.
    fn minus(&self, other: &Dec) -> Dec {
        let scale = self.scale.max(other.scale);
        let (left, right) = (self.at_scale(scale), other.at_scale(scale));
        let other_negative = !other.negative && !other.digits.is_empty();
        if self.negative == other_negative {
            return Dec::new(self.negative, added(&left, &right), scale);
        }
        match compared(&left, &right) {
            Ordering::Less => Dec::new(other_negative, subtracted(&right, &left), scale),
            _ => Dec::new(self.negative, subtracted(&left, &right), scale),
        }
    }

    /// `self / other` at [`RATIO_SCALE`] fractional digits, the last rounded half away from zero;
    /// `None` when `other` is 0.
    fn over(&self, other: &Dec) -> Option<Dec> {
        if other.digits.is_empty() {
            return None;
        }
        let numerator = shifted(&self.digits, other.scale + RATIO_SCALE + 1);
        let denominator = shifted(&other.digits, self.scale);
        let mut quotient = divided(&numerator, &denominator);
        let last = quotient.pop().unwrap_or(0);
        if last >= 5 {
            quotient = added(&trimmed(quotient), &[1]);
        }
        Some(Dec::new(
            self.negative != other.negative,
            quotient,
            RATIO_SCALE,
        ))
    }

    /// The wire form, without trailing fractional zeros.
    fn render(&self) -> String {
        let mut digits: Vec<u8> = self.digits.clone();
        while digits.len() <= self.scale {
            digits.insert(0, 0);
        }
        let point = digits.len() - self.scale;
        let whole: String = digits[..point]
            .iter()
            .map(|d| char::from(b'0' + d))
            .collect();
        let fraction: String = digits[point..]
            .iter()
            .map(|d| char::from(b'0' + d))
            .collect();
        let fraction = fraction.trim_end_matches('0');
        let sign = if self.negative { "-" } else { "" };
        if fraction.is_empty() {
            format!("{sign}{whole}")
        } else {
            format!("{sign}{whole}.{fraction}")
        }
    }
}

fn trimmed(digits: Vec<u8>) -> Vec<u8> {
    let first = digits
        .iter()
        .position(|digit| *digit != 0)
        .unwrap_or(digits.len());
    digits[first..].to_vec()
}

/// `digits` times ten to the `places`.
fn shifted(digits: &[u8], places: usize) -> Vec<u8> {
    if digits.is_empty() {
        return Vec::new();
    }
    let mut out = digits.to_vec();
    out.resize(digits.len() + places, 0);
    out
}

fn compared(left: &[u8], right: &[u8]) -> Ordering {
    left.len().cmp(&right.len()).then_with(|| left.cmp(right))
}

fn added(left: &[u8], right: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(left.len().max(right.len()) + 1);
    let mut carry = 0;
    let (mut l, mut r) = (left.iter().rev(), right.iter().rev());
    loop {
        let (a, b) = (l.next(), r.next());
        if a.is_none() && b.is_none() {
            break;
        }
        let sum = a.copied().unwrap_or(0) + b.copied().unwrap_or(0) + carry;
        out.push(sum % 10);
        carry = sum / 10;
    }
    if carry > 0 {
        out.push(carry);
    }
    out.reverse();
    trimmed(out)
}

/// `left - right`, `left` at least `right`.
fn subtracted(left: &[u8], right: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(left.len());
    let mut borrow = 0;
    let mut r = right.iter().rev();
    for a in left.iter().rev() {
        let b = r.next().copied().unwrap_or(0) + borrow;
        if *a >= b {
            out.push(a - b);
            borrow = 0;
        } else {
            out.push(a + 10 - b);
            borrow = 1;
        }
    }
    out.reverse();
    trimmed(out)
}

/// `numerator / denominator`, truncated; `denominator` is not 0.
fn divided(numerator: &[u8], denominator: &[u8]) -> Vec<u8> {
    let mut quotient = Vec::with_capacity(numerator.len());
    let mut remainder: Vec<u8> = Vec::new();
    for digit in numerator {
        remainder.push(*digit);
        remainder = trimmed(remainder);
        let mut count = 0;
        while compared(&remainder, denominator) != Ordering::Less {
            remainder = subtracted(&remainder, denominator);
            count += 1;
        }
        quotient.push(count);
    }
    trimmed(quotient)
}

#[cfg(test)]
mod tests {
    use super::{Dec, seconds_of};

    fn dec(text: &str) -> Dec {
        Dec::parse(text).expect("a wire decimal")
    }

    #[test]
    fn delta_and_ratio_are_exact_decimals() {
        let cases = [
            ("42", "40", "2", Some("1.05")),
            ("0.5", "0.25", "0.25", Some("2")),
            ("0.01", "0.03", "-0.02", Some("0.333333333333333333")),
            ("2", "3", "-1", Some("0.666666666666666667")),
            ("-1.5", "0.5", "-2", Some("-3")),
            ("1", "-0.5", "1.5", Some("-2")),
            ("0", "0", "0", None),
            (
                "123456789012345678901234567890",
                "1",
                "123456789012345678901234567889",
                Some("123456789012345678901234567890"),
            ),
        ];
        for (current, baseline, delta, ratio) in cases {
            let (current_dec, baseline_dec) = (dec(current), dec(baseline));
            assert_eq!(
                current_dec.minus(&baseline_dec).render(),
                delta,
                "{current} - {baseline}"
            );
            assert_eq!(
                current_dec.over(&baseline_dec).map(|ratio| ratio.render()),
                ratio.map(str::to_owned),
                "{current} / {baseline}"
            );
        }
    }

    #[test]
    fn only_wire_decimals_parse() {
        for text in ["", "-", "01", "1.", ".5", "1e3", "+1", "1.2.3"] {
            assert_eq!(Dec::parse(text), None, "{text}");
        }
    }

    #[test]
    fn durations_read_short_and_iso_forms() {
        for (text, seconds) in [
            ("7d", Some(604_800)),
            ("1h", Some(3_600)),
            ("2w", Some(1_209_600)),
            ("PT5M", Some(300)),
            ("PT1H", Some(3_600)),
            ("P1W", Some(604_800)),
            ("P1DT6H", Some(108_000)),
            ("PT1H0M1S", Some(3_601)),
            ("P", None),
            ("PT", None),
            ("PT5", None),
            ("PM5", None),
            ("7x", None),
            ("d", None),
            ("P1Y", None),
        ] {
            assert_eq!(seconds_of(text), seconds, "{text}");
        }
    }
}
