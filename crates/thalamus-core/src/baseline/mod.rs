//! Baselines (design § 3.1): which earlier windows a sample's baseline reads, and how their
//! readings fold into the one baseline the sample is graded against.
//!
//! Pure: [`plan`] names the windows to read, the caller reads them, and [`fold`] turns what came
//! back, together with the age of the series, into a recorded [`Baseline`]. A baseline that could
//! not be read (rule 1), or that is older than its series (rule 2, L6), folds to an `unknown`
//! reading, which carries no value, so it can never become 0. A signal of kind `instant` is graded
//! against `fixed` only (rule 3, L8).

pub mod rolling;
pub mod same_slot;

use std::cmp::Ordering;
use std::fmt;

use thalamus_model::definition::{BaselineKind, SignalDefinitionData, SignalKind};
use thalamus_model::evaluation::{Baseline, BaselineLabel};
use thalamus_model::observation::{
    CompleteReading, Reading, SignalSampleData, TimeWindow, UnreadReading,
};
use thalamus_model::primitives::{Decimal, Timestamp};
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;

/// The reason of a baseline whose window starts before its series began (rule 2).
pub const REASON_PREDATES_SERIES: &str = "baseline predates series";

/// The reason of an offset baseline over a series whose age is not known (rule 2, read
/// conservatively).
pub const REASON_SERIES_AGE_UNKNOWN: &str = "series age unknown";

/// The fractional digits a pooled quotient that does not terminate keeps; the last is rounded
/// half away from zero.
pub const POOLED_SCALE: u32 = 18;

const NANOS_PER_SECOND: i128 = 1_000_000_000;

/// How far back an offset baseline reads.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Offset {
    /// The same window one day earlier.
    OneDay,
    /// The same window seven days earlier, so on the same weekday.
    SevenDays,
}

impl Offset {
    /// Both offsets, in the order a combined baseline records them (design § 11 D4).
    pub const ALL: [Offset; 2] = [Offset::OneDay, Offset::SevenDays];

    /// The offset as a recorded baseline names it: `1d`, `7d`.
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Offset::OneDay => "1d",
            Offset::SevenDays => "7d",
        }
    }

    /// Whole days the window is shifted back.
    #[must_use]
    pub fn days(self) -> i64 {
        match self {
            Offset::OneDay => 1,
            Offset::SevenDays => 7,
        }
    }

    /// Seconds the window is shifted back.
    #[must_use]
    pub fn seconds(self) -> i64 {
        self.days() * 86_400
    }
}

/// One earlier window a baseline needs read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WindowRequest {
    /// Which offset this window answers.
    pub offset: Offset,
    /// The sample's window shifted back by the offset.
    pub window: TimeWindow,
}

/// What the read of one requested window returned.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OffsetReading {
    /// Which offset this reading answers.
    pub offset: Offset,
    /// The reading of the shifted window, as the source answered it.
    pub reading: Reading,
    /// The population behind the reading, when the source answered one. A `pooled` baseline
    /// needs it.
    pub n: Option<i64>,
}

/// Why a definition's baseline cannot be planned or folded at all.
///
/// A refusal is about the definition (or a malformed sample), never about a reading: a reading
/// that could not be had folds to an `unknown` baseline instead.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Refusal {
    /// `evaluation.baselines` is empty.
    NoBaseline,
    /// `evaluation.baselines` does not name exactly one baseline: two combiners, or `fixed`
    /// beside another kind.
    Ambiguous {
        /// The kinds the definition declares.
        kinds: Vec<BaselineKind>,
    },
    /// A signal of kind `instant` declared a baseline other than `fixed` (rule 3; the spec's
    /// `thalamus.definition.InstantNeedsFixedBaseline`).
    InstantNeedsFixed {
        /// The first kind other than `fixed` the definition declares.
        kind: BaselineKind,
    },
    /// A signal that is not a ratio declared `pooled`, or two offsets without a combiner, which
    /// read as `pooled` (design § 3.1: `pooled` is the baseline of ratios).
    PooledNeedsRatio {
        /// The signal's kind.
        kind: SignalKind,
    },
    /// A baseline that milestone M1 does not compute (`story:m2-store-baselines`).
    NotInM1 {
        /// The kind requested.
        kind: BaselineKind,
    },
    /// A timestamp of the sample is not RFC 3339, or its shifted window cannot be written as one.
    Timestamp {
        /// Which field of the sample.
        field: &'static str,
        /// Its text.
        value: String,
    },
}

impl fmt::Display for Refusal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Refusal::NoBaseline => f.write_str("the definition declares no baseline"),
            Refusal::Ambiguous { kinds } => {
                let names: Vec<&str> = kinds.iter().map(|kind| kind_name(*kind)).collect();
                write!(
                    f,
                    "evaluation.baselines [{}] names no single baseline: declare one offset, \
                     one combiner (pooled, higher_of, lower_of) with its offsets, or fixed alone",
                    names.join(", ")
                )
            }
            Refusal::InstantNeedsFixed { kind } => write!(
                f,
                "a signal of kind instant is graded against fixed only; it declares {}",
                kind_name(*kind)
            ),
            Refusal::PooledNeedsRatio { kind } => write!(
                f,
                "pooled is the baseline of a ratio, and two offsets without a combiner read as \
                 pooled; a signal of kind {} declares higher_of or lower_of over its offsets, \
                 or one offset",
                signal_kind_name(*kind)
            ),
            Refusal::NotInM1 { kind } => write!(f, "baseline {} is not in M1", kind_name(*kind)),
            Refusal::Timestamp { field, value } => {
                write!(
                    f,
                    "sample {field} `{value}` is not a usable RFC 3339 timestamp"
                )
            }
        }
    }
}

impl std::error::Error for Refusal {}

/// The spec's name of a baseline kind.
#[must_use]
pub fn kind_name(kind: BaselineKind) -> &'static str {
    match kind {
        BaselineKind::Offset1d => "offset_1d",
        BaselineKind::Offset7d => "offset_7d",
        BaselineKind::Pooled => "pooled",
        BaselineKind::HigherOf => "higher_of",
        BaselineKind::LowerOf => "lower_of",
        BaselineKind::SameSlotMedian4w => "same_slot_median_4w",
        BaselineKind::RollingMedian => "rolling_median",
        BaselineKind::Fixed => "fixed",
    }
}

/// The spec's name of a signal kind.
#[must_use]
pub fn signal_kind_name(kind: SignalKind) -> &'static str {
    match kind {
        SignalKind::Gauge => "gauge",
        SignalKind::CounterIncrease => "counter_increase",
        SignalKind::Ratio => "ratio",
        SignalKind::Instant => "instant",
        SignalKind::CountOfCases => "count_of_cases",
    }
}

/// The windows `fold` needs read for this sample, one per offset, in offset order. `fixed` needs
/// none.
///
/// # Errors
///
/// A [`Refusal`] when the definition names no single M1 baseline (rule 3 included), or when the
/// sample's window is not RFC 3339.
pub fn plan(
    definition: &SignalDefinitionData,
    sample: &SignalSampleData,
) -> Result<Vec<WindowRequest>, Refusal> {
    resolve(definition)?
        .offsets()
        .iter()
        .map(|&offset| {
            Ok(WindowRequest {
                offset,
                window: TimeWindow {
                    start: shift(&sample.window.start, "window.start", offset)?,
                    end: shift(&sample.window.end, "window.end", offset)?,
                },
            })
        })
        .collect()
}

/// The baseline `sample` is graded against, from the readings of the windows [`plan`] named.
///
/// `series_age_s` is the age of the series at the sample, in seconds (`denominator.age_s`, or what
/// the caller resolved); `None` when it is not known. A `fixed` baseline is the definition's own
/// number, `evaluation.fixed` (design § 11 D25), and is unknown, never 0, when it declares none.
///
/// The recorded kind and offsets follow design § 11 D4: a combined baseline over both offsets is
/// recorded as, for example, `pooled` over `[1d, 7d]`.
///
/// # Errors
///
/// A [`Refusal`] when the definition names no single M1 baseline (rule 3 included), or when the
/// sample's timestamps are not RFC 3339. An unreadable or too-old baseline is not an error: it is
/// an `Ok` baseline whose reading is `unknown`.
pub fn fold(
    definition: &SignalDefinitionData,
    sample: &SignalSampleData,
    readings: &[OffsetReading],
    series_age_s: Option<i64>,
) -> Result<Baseline, Refusal> {
    let shape = resolve(definition)?;
    let reading = match &shape {
        Shape::Fixed => fixed_reading(definition.evaluation.fixed.as_ref()),
        Shape::Offset(offset) => match series_rule(sample, &[*offset], series_age_s)? {
            Some(reason) => unknown(reason),
            None => match constituents(&[*offset], readings) {
                Err(reason) => unknown(reason),
                Ok(values) => ok(values[0].text.clone()),
            },
        },
        Shape::Combined(combiner, offsets) => match series_rule(sample, offsets, series_age_s)? {
            Some(reason) => unknown(reason),
            None => match constituents(offsets, readings) {
                Err(reason) => unknown(reason),
                Ok(values) => combine(*combiner, &values),
            },
        },
    };
    Ok(Baseline {
        kind: BaselineLabel(shape.label().to_owned()),
        offsets: shape
            .offsets()
            .iter()
            .map(|offset| offset.label().to_owned())
            .collect(),
        reading,
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Combiner {
    Pooled,
    HigherOf,
    LowerOf,
}

/// The one baseline a definition names.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Shape {
    Offset(Offset),
    Combined(Combiner, Vec<Offset>),
    Fixed,
}

impl Shape {
    fn label(&self) -> &'static str {
        match self {
            Shape::Offset(Offset::OneDay) => "offset_1d",
            Shape::Offset(Offset::SevenDays) => "offset_7d",
            Shape::Combined(Combiner::Pooled, _) => "pooled",
            Shape::Combined(Combiner::HigherOf, _) => "higher_of",
            Shape::Combined(Combiner::LowerOf, _) => "lower_of",
            Shape::Fixed => "fixed",
        }
    }

    fn offsets(&self) -> Vec<Offset> {
        match self {
            Shape::Offset(offset) => vec![*offset],
            Shape::Combined(_, offsets) => offsets.clone(),
            Shape::Fixed => Vec::new(),
        }
    }
}

/// Reads `evaluation.baselines` as one baseline.
///
/// One offset is that offset. One combiner is that combiner over the offsets listed beside it, or
/// over both when none is. Two offsets without a combiner are `pooled` over both: the design's
/// own example (§ 2.1, `baselines: [offset_1d, offset_7d]`) is recorded as `pooled` (§ 11 D4).
/// `pooled`, named or read so, is refused for a signal that is not a ratio (§ 3.1).
fn resolve(definition: &SignalDefinitionData) -> Result<Shape, Refusal> {
    let kinds = &definition.evaluation.baselines;
    if definition.kind == SignalKind::Instant
        && let Some(&kind) = kinds.iter().find(|&&kind| kind != BaselineKind::Fixed)
    {
        return Err(Refusal::InstantNeedsFixed { kind });
    }
    if kinds.is_empty() {
        return Err(Refusal::NoBaseline);
    }
    let mut offsets = Vec::new();
    let mut combiners = Vec::new();
    let mut fixed = false;
    for &kind in kinds {
        match kind {
            BaselineKind::Offset1d => offsets.push(Offset::OneDay),
            BaselineKind::Offset7d => offsets.push(Offset::SevenDays),
            BaselineKind::Pooled => combiners.push(Combiner::Pooled),
            BaselineKind::HigherOf => combiners.push(Combiner::HigherOf),
            BaselineKind::LowerOf => combiners.push(Combiner::LowerOf),
            BaselineKind::Fixed => fixed = true,
            // `story:m2-store-baselines` replaces these two arms with `same_slot` and `rolling`.
            BaselineKind::SameSlotMedian4w | BaselineKind::RollingMedian => {
                return Err(Refusal::NotInM1 { kind });
            }
        }
    }
    offsets.sort_unstable();
    offsets.dedup();
    combiners.dedup();
    let ambiguous = || Refusal::Ambiguous {
        kinds: kinds.clone(),
    };
    if fixed {
        return if offsets.is_empty() && combiners.is_empty() {
            Ok(Shape::Fixed)
        } else {
            Err(ambiguous())
        };
    }
    let shape = match (combiners.as_slice(), offsets.as_slice()) {
        ([], [offset]) => Shape::Offset(*offset),
        ([], _) => Shape::Combined(Combiner::Pooled, offsets),
        ([combiner], []) => Shape::Combined(*combiner, Offset::ALL.to_vec()),
        ([combiner], _) => Shape::Combined(*combiner, offsets),
        _ => return Err(ambiguous()),
    };
    // Design § 3.1: `pooled` is the baseline of ratios, whether named or read from two offsets.
    if matches!(shape, Shape::Combined(Combiner::Pooled, _)) && definition.kind != SignalKind::Ratio
    {
        return Err(Refusal::PooledNeedsRatio {
            kind: definition.kind,
        });
    }
    Ok(shape)
}

/// Rule 2: the reason an offset baseline is unknown because of the series age, if it is.
///
/// A baseline window is older than its series when it starts before the series began: the
/// offset plus the span from the window start to the sample exceeds the series age.
fn series_rule(
    sample: &SignalSampleData,
    offsets: &[Offset],
    series_age_s: Option<i64>,
) -> Result<Option<String>, Refusal> {
    let Some(age_s) = series_age_s else {
        return Ok(Some(REASON_SERIES_AGE_UNKNOWN.to_owned()));
    };
    let ts = parse(&sample.ts, "ts")?;
    let start = parse(&sample.window.start, "window.start")?;
    let span = (ts - start).whole_nanoseconds();
    let age = i128::from(age_s) * NANOS_PER_SECOND;
    let predates = offsets
        .iter()
        .any(|offset| i128::from(offset.seconds()) * NANOS_PER_SECOND + span > age);
    Ok(predates.then(|| REASON_PREDATES_SERIES.to_owned()))
}

/// One readable constituent of a baseline.
struct Constituent {
    offset: Offset,
    text: Decimal,
    value: Dec,
    n: Option<i64>,
}

/// Rule 1: the readable reading of every offset, or a reason naming each offset that is not.
fn constituents(
    offsets: &[Offset],
    readings: &[OffsetReading],
) -> Result<Vec<Constituent>, String> {
    let mut found = Vec::new();
    let mut reasons = Vec::new();
    for &offset in offsets {
        let label = offset.label();
        let mut answers = readings.iter().filter(|reading| reading.offset == offset);
        let (Some(answer), None) = (answers.next(), answers.next()) else {
            let read = readings.iter().any(|reading| reading.offset == offset);
            reasons.push(if read {
                format!("offset {label} read more than once")
            } else {
                format!("offset {label} missing: its window was not read")
            });
            continue;
        };
        match &answer.reading {
            Reading::Unknown(UnreadReading { reason }) => {
                reasons.push(format!("offset {label} unread: {reason}"));
            }
            Reading::Partial(_) => {
                reasons.push(format!(
                    "offset {label} read partially: a bound is not a baseline"
                ));
            }
            Reading::Ok(CompleteReading { value }) => match Dec::parse(&value.0) {
                Some(parsed) => found.push(Constituent {
                    offset,
                    text: value.clone(),
                    value: parsed,
                    n: answer.n,
                }),
                None => reasons.push(format!(
                    "offset {label} value `{}` is not a decimal",
                    value.0
                )),
            },
        }
    }
    if reasons.is_empty() {
        Ok(found)
    } else {
        Err(reasons.join("; "))
    }
}

fn combine(combiner: Combiner, values: &[Constituent]) -> Reading {
    match combiner {
        Combiner::HigherOf => pick(values, Ordering::Greater),
        Combiner::LowerOf => pick(values, Ordering::Less),
        Combiner::Pooled => match pooled(values) {
            Ok(value) => ok(value),
            Err(reason) => unknown(reason),
        },
    }
}

/// The constituent that is `wanted` of every other one; the first of equal ones.
fn pick(values: &[Constituent], wanted: Ordering) -> Reading {
    let mut best = &values[0];
    for candidate in &values[1..] {
        if candidate.value.compare(&best.value) == wanted {
            best = candidate;
        }
    }
    ok(best.text.clone())
}

/// The sum of the numerators (value times `n`) over the sum of the denominators, exact at any
/// length of input (the wire grammar of a decimal has none).
fn pooled(values: &[Constituent]) -> Result<Decimal, String> {
    let mut reasons = Vec::new();
    let mut weights = Vec::new();
    for constituent in values {
        let label = constituent.offset.label();
        match constituent.n.map(u64::try_from) {
            None => reasons.push(format!("offset {label} has no denominator to pool")),
            Some(Err(_)) => reasons.push(format!(
                "offset {label} denominator {} is negative",
                constituent.n.unwrap_or_default()
            )),
            Some(Ok(n)) => weights.push((&constituent.value, n)),
        }
    }
    if !reasons.is_empty() {
        return Err(reasons.join("; "));
    }
    let scale = weights
        .iter()
        .map(|(value, _)| value.scale)
        .max()
        .unwrap_or(0);
    let mut numerator = Dec::zero(scale);
    let mut denominator: u128 = 0;
    for (value, n) in weights {
        numerator = numerator.plus(&value.times(n, scale));
        // One constituent per offset, each n below 2^63, so the sum stays below 2^64.
        denominator += u128::from(n);
    }
    if denominator == 0 {
        return Err("pooled baseline has no population: every n is 0".to_owned());
    }
    Ok(Decimal(numerator.divided_by(denominator)))
}

fn fixed_reading(fixed: Option<&Decimal>) -> Reading {
    match fixed {
        None => unknown("fixed baseline declares no number".to_owned()),
        Some(value) if Dec::parse(&value.0).is_none() => {
            unknown(format!("fixed baseline `{}` is not a decimal", value.0))
        }
        Some(value) => ok(value.clone()),
    }
}

fn ok(value: Decimal) -> Reading {
    Reading::Ok(CompleteReading { value })
}

fn unknown(reason: String) -> Reading {
    Reading::Unknown(UnreadReading { reason })
}

fn parse(timestamp: &Timestamp, field: &'static str) -> Result<OffsetDateTime, Refusal> {
    OffsetDateTime::parse(&timestamp.0, &Rfc3339).map_err(|_| Refusal::Timestamp {
        field,
        value: timestamp.0.clone(),
    })
}

/// The timestamp `offset` earlier, in the same UTC offset, so a 7-day shift keeps the weekday.
fn shift(timestamp: &Timestamp, field: &'static str, offset: Offset) -> Result<Timestamp, Refusal> {
    let refusal = || Refusal::Timestamp {
        field,
        value: timestamp.0.clone(),
    };
    parse(timestamp, field)?
        .checked_sub(time::Duration::days(offset.days()))
        .and_then(|shifted| shifted.format(&Rfc3339).ok())
        .map(Timestamp)
        .ok_or_else(refusal)
}

/// An exact decimal of any length: a sign, the decimal digits of its magnitude (most significant
/// first, no leading zero, empty for zero) and how many of them are fractional.
///
/// The wire grammar of a decimal has no length limit, so neither does this: nothing a reading
/// can carry is refused or overflows here.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Dec {
    negative: bool,
    digits: Vec<u8>,
    scale: usize,
}

impl Dec {
    /// The pattern the generated wire decoder admits: an optional `-`, digits without a leading
    /// zero, then an optional `.` and digits. `None` for anything else.
    fn parse(text: &str) -> Option<Dec> {
        let (negative, unsigned) = match text.strip_prefix('-') {
            Some(rest) => (true, rest),
            None => (false, text),
        };
        let (whole, fraction) = match unsigned.split_once('.') {
            Some((whole, fraction)) => (whole, Some(fraction)),
            None => (unsigned, None),
        };
        let digits =
            |part: &str| !part.is_empty() && part.bytes().all(|byte| byte.is_ascii_digit());
        let fraction_valid = fraction.is_none_or(digits);
        if !digits(whole) || (whole.len() > 1 && whole.starts_with('0')) || !fraction_valid {
            return None;
        }
        let fraction = fraction.unwrap_or("");
        let magnitude = whole
            .bytes()
            .chain(fraction.bytes())
            .map(|byte| byte - b'0');
        Some(Dec::new(negative, magnitude.collect(), fraction.len()))
    }

    /// Zero, at `scale` fractional digits.
    fn zero(scale: usize) -> Dec {
        Dec::new(false, Vec::new(), scale)
    }

    fn new(negative: bool, digits: Vec<u8>, scale: usize) -> Dec {
        let digits = trimmed(digits);
        Dec {
            negative: negative && !digits.is_empty(),
            digits,
            scale,
        }
    }

    /// The magnitude's digits at `scale` fractional digits, which is at least `self.scale`.
    fn aligned(&self, scale: usize) -> Vec<u8> {
        let mut digits = self.digits.clone();
        digits.resize(digits.len() + (scale - self.scale), 0);
        trimmed(digits)
    }

    /// Numeric order, whatever the two scales.
    fn compare(&self, other: &Dec) -> Ordering {
        let scale = self.scale.max(other.scale);
        match (self.negative, other.negative) {
            (false, true) => Ordering::Greater,
            (true, false) => Ordering::Less,
            (negative, _) => {
                let order = magnitude_order(&self.aligned(scale), &other.aligned(scale));
                if negative { order.reverse() } else { order }
            }
        }
    }

    /// This value times `n`, at `scale` fractional digits (at least `self.scale`).
    fn times(&self, n: u64, scale: usize) -> Dec {
        let mut product = Vec::with_capacity(self.digits.len() + scale + 20);
        let mut carry: u128 = 0;
        for digit in self.aligned(scale).iter().rev() {
            let step = u128::from(*digit) * u128::from(n) + carry;
            product.push(digit_of(step % 10));
            carry = step / 10;
        }
        while carry > 0 {
            product.push(digit_of(carry % 10));
            carry /= 10;
        }
        product.reverse();
        Dec::new(self.negative, product, scale)
    }

    /// The sum of two values of the same scale.
    fn plus(&self, other: &Dec) -> Dec {
        debug_assert_eq!(self.scale, other.scale);
        if self.negative == other.negative {
            return Dec::new(self.negative, add(&self.digits, &other.digits), self.scale);
        }
        match magnitude_order(&self.digits, &other.digits) {
            Ordering::Less => {
                Dec::new(other.negative, sub(&other.digits, &self.digits), self.scale)
            }
            _ => Dec::new(self.negative, sub(&self.digits, &other.digits), self.scale),
        }
    }

    /// This value over a positive `divisor`, as a decimal string, at least [`POOLED_SCALE`]
    /// fractional digits deep, the last rounded half away from zero, trailing zeros dropped.
    fn divided_by(&self, divisor: u128) -> String {
        let fraction = self.scale.max(POOLED_SCALE as usize);
        let extra = fraction - self.scale;
        // The remainder stays below `divisor` (under 2^64), so `remainder * 10 + 9` fits a u128.
        let mut remainder: u128 = 0;
        let mut quotient = Vec::with_capacity(self.digits.len() + extra);
        let step = |remainder: &mut u128, digit: u8| {
            let current = *remainder * 10 + u128::from(digit);
            *remainder = current % divisor;
            digit_of(current / divisor)
        };
        for digit in &self.digits {
            quotient.push(step(&mut remainder, *digit));
        }
        for _ in 0..extra {
            quotient.push(step(&mut remainder, 0));
        }
        if step(&mut remainder, 0) >= 5 {
            round_up(&mut quotient);
        }
        render(self.negative, trimmed(quotient), fraction)
    }
}

/// A digit value below 10 as a `u8`.
fn digit_of(value: u128) -> u8 {
    u8::try_from(value % 10).unwrap_or_default()
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

/// The sum of two magnitudes.
fn add(left: &[u8], right: &[u8]) -> Vec<u8> {
    let mut sum = Vec::with_capacity(left.len().max(right.len()) + 1);
    let mut carry = 0;
    let (mut left, mut right) = (left.iter().rev(), right.iter().rev());
    loop {
        let (a, b) = (left.next(), right.next());
        if a.is_none() && b.is_none() {
            break;
        }
        let total = a.copied().unwrap_or(0) + b.copied().unwrap_or(0) + carry;
        sum.push(total % 10);
        carry = total / 10;
    }
    if carry > 0 {
        sum.push(carry);
    }
    sum.reverse();
    sum
}

/// `larger - smaller`, for magnitudes with `larger >= smaller`.
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
    difference
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

/// A magnitude with `fraction` fractional digits written in the wire grammar: no leading zero,
/// no trailing fractional zero, and never `-0`.
fn render(negative: bool, digits: Vec<u8>, fraction: usize) -> String {
    let mut digits = digits;
    if digits.len() <= fraction {
        let mut padded = vec![0; fraction + 1 - digits.len()];
        padded.extend_from_slice(&digits);
        digits = padded;
    }
    let (whole, part) = digits.split_at(digits.len() - fraction);
    let text =
        |part: &[u8]| -> String { part.iter().map(|digit| char::from(b'0' + digit)).collect() };
    let whole = text(whole);
    let whole = whole.trim_start_matches('0');
    let whole = if whole.is_empty() { "0" } else { whole };
    let part = text(part);
    let part = part.trim_end_matches('0');
    let zero = whole == "0" && part.is_empty();
    let sign = if negative && !zero { "-" } else { "" };
    if part.is_empty() {
        format!("{sign}{whole}")
    } else {
        format!("{sign}{whole}.{part}")
    }
}
