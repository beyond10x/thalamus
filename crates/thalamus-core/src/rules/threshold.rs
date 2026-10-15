//! The threshold rules: a value compared with its baseline and with the WARN and CRIT lines of
//! its definition (design § 2.1, § 2.3, § 3.3).
//!
//! - `ratio_vs_pooled_baseline`: higher is worse; the events of a line are the value times the
//!   population `n` (a ratio of failures counts failures).
//! - `higher_is_worse`: a count or gauge graded against `higher_of` or `fixed`; the events are
//!   the value itself.
//! - `lower_is_worse`: a count or gauge graded against `lower_of` or `fixed`, where a drop is the
//!   fault; the events are those at stake, the baseline's (design § 11 D22).
//!
//! A line is crossed when every part it declares holds: the value is at least `x` times the
//! baseline, at least `abs`, and the event count is at least `min_events` (for `lower_is_worse`,
//! at most `x` times the baseline and at most `abs`, with the baseline's events at least
//! `min_events`). The line's position is therefore the
//! stricter of its `x` and `abs` parts. The verdict is decided exactly, on the decimals; the score
//! is placed between the lines and then held inside the band of the verdict, so the two cannot
//! disagree.

use std::borrow::Cow;
use std::cmp::Ordering;
use std::fmt;

use thalamus_model::definition::{EvaluationSpec, Threshold};
use thalamus_model::evaluation::Verdict;
use thalamus_model::observation::Bound;
use thalamus_model::primitives::Decimal;

use super::Decision;

/// The largest score of an `OK`: just under the WARN line.
const OK_CEILING: f64 = 0.999;
/// The largest score of a `WARN`: just under the CRIT line.
const WARN_CEILING: f64 = 1.999;

/// Which way a value gets worse.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    /// Above the baseline is worse: failures, latency, errors.
    HigherIsWorse,
    /// Below the baseline is worse: volume, availability.
    LowerIsWorse,
}

/// What a line's `min_events` counts: the events at stake (design § 11 D22). When higher is
/// worse they are the current value's; when lower is worse, a drop, they are the baseline's.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Events {
    /// The number at stake itself, for a count.
    Plain,
    /// The number at stake times the population `n`, for a ratio.
    TimesN,
}

/// Why a definition's lines cannot be graded against at all.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LineError {
    /// `evaluation.warn` or `evaluation.crit` is absent; a threshold rule needs both.
    Missing {
        /// `warn` or `crit`.
        line: &'static str,
    },
    /// The line declares neither `x` nor `abs`, so it has no position.
    Empty {
        /// `warn` or `crit`.
        line: &'static str,
    },
    /// A part of the line is not a decimal this grader compares exactly.
    NotDecimal {
        /// `warn` or `crit`.
        line: &'static str,
        /// `x` or `abs`.
        part: &'static str,
        /// The text as declared.
        text: String,
    },
}

impl fmt::Display for LineError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LineError::Missing { line } => write!(
                f,
                "evaluation.{line} is absent; a threshold rule needs both warn and crit"
            ),
            LineError::Empty { line } => {
                write!(f, "evaluation.{line} declares neither x nor abs")
            }
            LineError::NotDecimal { line, part, text } => write!(
                f,
                "evaluation.{line}.{part} `{text}` is not a decimal within the grader's range"
            ),
        }
    }
}

impl std::error::Error for LineError {}

/// One line, parsed.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Line {
    x: Option<(Num, String)>,
    abs: Option<(Num, String)>,
    min_events: Option<i64>,
}

impl Line {
    fn parse(line: &'static str, threshold: Option<&Threshold>) -> Result<Line, LineError> {
        let threshold = threshold.ok_or(LineError::Missing { line })?;
        let part = |part: &'static str, text: Option<&Decimal>| {
            text.map(|text| {
                Num::parse(&text.0)
                    .map(|num| (num, text.0.clone()))
                    .ok_or_else(|| LineError::NotDecimal {
                        line,
                        part,
                        text: text.0.clone(),
                    })
            })
            .transpose()
        };
        let parsed = Line {
            x: part("x", threshold.x.as_ref())?,
            abs: part("abs", threshold.abs.as_ref())?,
            min_events: threshold.min_events,
        };
        if parsed.x.is_none() && parsed.abs.is_none() {
            return Err(LineError::Empty { line });
        }
        Ok(parsed)
    }
}

/// A threshold rule with its definition's lines.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rule {
    id: &'static str,
    direction: Direction,
    events: Events,
    warn: Line,
    crit: Line,
}

/// Where a line sits for one baseline, and which part put it there.
#[derive(Debug, Clone, Copy)]
struct Position {
    at: Num,
    /// The `x` part decides it, so the score is log-linear up to it; otherwise linear.
    multiplier: bool,
}

/// The first part of a line a value fails, or `None` when it crosses the line.
enum Short {
    Multiplier,
    Absolute,
    Events,
}

/// The grader cannot decide exactly; the result is `UNKNOWN` with this reason.
#[derive(Clone)]
struct Undecidable(String);

/// The event count of a value, as far as it can be had.
#[derive(Debug, Clone, Copy)]
enum Count {
    Known(Num),
    /// The rule counts value times `n`, and the sample carries no `n`.
    NoPopulation,
    /// Value times `n` is beyond the exact range.
    OutOfRange,
}

impl Rule {
    /// The rule `id` going `direction` and counting `events`, over the lines of `evaluation`.
    ///
    /// # Errors
    ///
    /// A [`LineError`] when a line is absent, declares no position, or is not a decimal.
    pub fn new(
        id: &'static str,
        direction: Direction,
        events: Events,
        evaluation: &EvaluationSpec,
    ) -> Result<Rule, LineError> {
        Ok(Rule {
            id,
            direction,
            events,
            warn: Line::parse("warn", evaluation.warn.as_ref())?,
            crit: Line::parse("crit", evaluation.crit.as_ref())?,
        })
    }

    /// The verdict, score and reason of a completely read `value`.
    #[must_use]
    pub fn decide(&self, value: &Decimal, baseline: &Decimal, n: Option<i64>) -> Decision {
        self.try_decide(value, baseline, n)
            .unwrap_or_else(|Undecidable(reason)| unknown(reason))
    }

    /// The verdict of a partial read whose `value` is a `bound`: `CRIT` when the bound is on the
    /// worse side and already crosses the CRIT line, so the true value does too; `None`
    /// otherwise, since a bound can never clear (design § 3.1 rule 4).
    #[must_use]
    pub fn escalate(
        &self,
        value: &Decimal,
        bound: Bound,
        baseline: &Decimal,
        n: Option<i64>,
    ) -> Option<Decision> {
        let worse_side = match self.direction {
            Direction::HigherIsWorse => Bound::Lower,
            Direction::LowerIsWorse => Bound::Upper,
        };
        // The events at stake grow with the value when higher is worse, so a lower bound's count
        // is a lower bound too; when lower is worse they are the baseline's, which no bound moves.
        if bound != worse_side {
            return None;
        }
        let decided = self.try_decide(value, baseline, n).ok()?;
        (decided.verdict == Verdict::CRIT).then(|| Decision {
            reason: format!(
                "partial read, {} bound {}",
                match bound {
                    Bound::Lower => "lower",
                    Bound::Upper => "upper",
                },
                decided.reason
            ),
            ..decided
        })
    }

    fn try_decide(
        &self,
        value: &Decimal,
        baseline: &Decimal,
        n: Option<i64>,
    ) -> Result<Decision, Undecidable> {
        let v = Num::parse(&value.0)
            .ok_or_else(|| Undecidable(format!("value `{}` is not a decimal", value.0)))?;
        let b = Num::parse(&baseline.0).ok_or_else(|| {
            Undecidable(format!("baseline value `{}` is not a decimal", baseline.0))
        })?;
        let events = self.events(v, b, n);
        let crit_line = usable(&self.crit, "CRIT", b, baseline)?;
        let crit = self.position(&crit_line, b)?;
        let warn_line = usable(&self.warn, "WARN", b, baseline);
        let warn = match &warn_line {
            Ok(line) => self.position(line, b),
            Err(undecidable) => Err(undecidable.clone()),
        };
        // The CRIT line is read first, and a crossed CRIT line decides, whatever the WARN line
        // could not decide or count (design § 11 D23). The WARN line is needed only below it.
        if self.short(&crit_line, v, b, events)?.is_none() {
            // With no WARN position to place it against, the score is the CRIT mark itself.
            let raw = warn.map_or(2.0, |warn| {
                self.raw_score(v.to_f64(), b.to_f64(), warn, crit)
            });
            return Ok(scored(
                Verdict::CRIT,
                raw,
                self.crossed(&crit_line, value, baseline, events, n),
            ));
        }
        let (warn_line, warn) = (warn_line?, warn?);
        let (verdict, reason) = if let Some(short) = self.short(&warn_line, v, b, events)? {
            (
                Verdict::OK,
                self.below(&warn_line, &short, value, baseline, events, n),
            )
        } else {
            (
                Verdict::WARN,
                self.crossed(&warn_line, value, baseline, events, n),
            )
        };
        let raw = self.raw_score(v.to_f64(), b.to_f64(), warn, crit);
        Ok(scored(verdict, raw, reason))
    }

    /// The events at stake: those of the value `v` when higher is worse, those of the baseline
    /// `b` when lower is worse (design § 11 D22). A ratio's count, the number times `n`, is
    /// rounded to the nearest whole event, half away from zero: a ratio arrives rounded, so 10 of
    /// 30 reads `0.333333333333333333` and counts 10. Only a line declaring `min_events` needs
    /// the count, so one that cannot be had is carried, not raised.
    fn events(&self, v: Num, b: Num, n: Option<i64>) -> Count {
        let at_stake = match self.direction {
            Direction::HigherIsWorse => v,
            Direction::LowerIsWorse => b,
        };
        match (self.events, n) {
            (Events::Plain, _) => Count::Known(at_stake),
            (Events::TimesN, None) => Count::NoPopulation,
            (Events::TimesN, Some(n)) => at_stake
                .times(Num::from(n))
                .and_then(Num::rounded)
                .map_or(Count::OutOfRange, Count::Known),
        }
    }

    /// What the events at stake are counted from, as a reason names it.
    fn stake(&self) -> &'static str {
        match self.direction {
            Direction::HigherIsWorse => "value",
            Direction::LowerIsWorse => "baseline",
        }
    }

    /// The line's position over baseline `b`: the stricter of `x` times `b` and `abs`.
    fn position(&self, line: &Line, b: Num) -> Result<Position, Undecidable> {
        let multiplier = line
            .x
            .as_ref()
            .map(|(x, text)| {
                x.times(b)
                    .ok_or_else(|| out_of_range(&format!("{text} x {b}")))
            })
            .transpose()?;
        let absolute = line.abs.as_ref().map(|(abs, _)| *abs);
        let by_multiplier = |at| Position {
            at,
            multiplier: true,
        };
        let by_absolute = |at| Position {
            at,
            multiplier: false,
        };
        match (multiplier, absolute) {
            (Some(m), Some(a)) => {
                // `a` is stricter when a value reaching `a` has reached `m` too.
                let a_stricter = !self.reaches(m, a)?;
                Ok(if a_stricter {
                    by_absolute(a)
                } else {
                    by_multiplier(m)
                })
            }
            (Some(m), None) => Ok(by_multiplier(m)),
            (None, Some(a)) => Ok(by_absolute(a)),
            (None, None) => Err(Undecidable("a line declares no position".to_owned())),
        }
    }

    /// Whether `v` reaches `line` in this rule's direction (`at` included).
    fn reaches(&self, v: Num, line: Num) -> Result<bool, Undecidable> {
        let order = v
            .compare(line)
            .ok_or_else(|| out_of_range(&format!("{v} against {line}")))?;
        Ok(match self.direction {
            Direction::HigherIsWorse => order != Ordering::Less,
            Direction::LowerIsWorse => order != Ordering::Greater,
        })
    }

    /// The first part of `line` that `v` fails, or `None` when it crosses the line.
    fn short(
        &self,
        line: &Line,
        v: Num,
        b: Num,
        events: Count,
    ) -> Result<Option<Short>, Undecidable> {
        if let Some((x, text)) = &line.x {
            let at = x
                .times(b)
                .ok_or_else(|| out_of_range(&format!("{text} x {b}")))?;
            if !self.reaches(v, at)? {
                return Ok(Some(Short::Multiplier));
            }
        }
        if let Some((abs, _)) = &line.abs
            && !self.reaches(v, *abs)?
        {
            return Ok(Some(Short::Absolute));
        }
        if let Some(min_events) = line.min_events {
            let events = match events {
                Count::Known(events) => events,
                Count::NoPopulation => {
                    return Err(Undecidable(format!(
                        "rule {} counts events as {} x n and the sample carries no population n",
                        self.id,
                        self.stake()
                    )));
                }
                Count::OutOfRange => {
                    return Err(out_of_range(&format!("{} x n", self.stake())));
                }
            };
            let order = events
                .compare(Num::from(min_events))
                .ok_or_else(|| out_of_range("the event count"))?;
            if order == Ordering::Less {
                return Ok(Some(Short::Events));
            }
        }
        Ok(None)
    }

    /// The comparison of a crossed line: `0.029 >= 2 x 0.013 and >= 0.02, 16 events of 556 >=
    /// min_events 10`.
    fn crossed(
        &self,
        line: &Line,
        value: &Decimal,
        baseline: &Decimal,
        events: Count,
        n: Option<i64>,
    ) -> String {
        let op = match self.direction {
            Direction::HigherIsWorse => ">=",
            Direction::LowerIsWorse => "<=",
        };
        let mut parts = Vec::new();
        if let Some((_, x)) = &line.x {
            parts.push(format!("{op} {x} x {}", baseline.0));
        }
        if let Some((_, abs)) = &line.abs {
            parts.push(format!("{op} {abs}"));
        }
        let mut reason = format!("{} {}", value.0, parts.join(" and "));
        reason.push_str(&self.counted(events, n, line.min_events, ">="));
        reason
    }

    /// The first part of the WARN line `value` falls short of: `under the WARN line: 0.014 < 2 x
    /// 0.013`, `above the WARN line: 80 > 0.5 x 100` when lower is worse, or `too few events for
    /// the WARN line: …` when only `min_events` is not met.
    fn below(
        &self,
        line: &Line,
        short: &Short,
        value: &Decimal,
        baseline: &Decimal,
        events: Count,
        n: Option<i64>,
    ) -> String {
        let (op, side) = match self.direction {
            Direction::HigherIsWorse => ("<", "under the WARN line"),
            Direction::LowerIsWorse => (">", "above the WARN line"),
        };
        let (side, short) = match (short, &line.x, &line.abs) {
            (Short::Multiplier, Some((_, x)), _) => (side, format!(" {op} {x} x {}", baseline.0)),
            (Short::Absolute, _, Some((_, abs))) => (side, format!(" {op} {abs}")),
            _ => (
                "too few events for the WARN line",
                self.counted(events, n, line.min_events, "<"),
            ),
        };
        format!("{side}: {}{short}", value.0)
    }

    /// `, 16 events of 556`, and the `min_events` the line declares, where they apply.
    fn counted(&self, events: Count, n: Option<i64>, min_events: Option<i64>, op: &str) -> String {
        let Count::Known(events) = events else {
            return String::new();
        };
        let mut text = match (self.events, n) {
            (Events::TimesN, Some(n)) => format!(", {events} events of {n}"),
            _ if min_events.is_some() => format!(", {events} events"),
            _ => String::new(),
        };
        if !text.is_empty() && self.direction == Direction::LowerIsWorse {
            text.push_str(" at the baseline");
        }
        if let Some(min_events) = min_events {
            text.push_str(&format!(" {op} min_events {min_events}"));
        }
        text
    }

    /// The score before it is held inside its verdict's band: 0 at the baseline, 1 at the WARN
    /// line, 2 at the CRIT line; log-linear up to a line its `x` part decides, linear otherwise and
    /// on the better side of the baseline. Past the CRIT line of a drop the score is linear too:
    /// log-linear it would grow without bound as the value nears 0, so a value of 0 or below
    /// could not score worse than a small positive one (design § 11 D22).
    fn raw_score(&self, v: f64, b: f64, warn: Position, crit: Position) -> f64 {
        let (w, c) = (warn.at.to_f64(), crit.at.to_f64());
        let worse = |a: f64, than: f64| match self.direction {
            Direction::HigherIsWorse => a > than,
            Direction::LowerIsWorse => a < than,
        };
        if !worse(w, v) {
            if worse(c, w) {
                let past_crit = !worse(c, v);
                let log =
                    crit.multiplier && !(self.direction == Direction::LowerIsWorse && past_crit);
                interpolate(v, (w, 1.0), (c, 2.0), log)
            } else if worse(c, v) {
                1.0
            } else {
                2.0
            }
        } else if worse(w, b) {
            let better_than_baseline = worse(b, v);
            interpolate(
                v,
                (b, 0.0),
                (w, 1.0),
                warn.multiplier && !better_than_baseline,
            )
        } else if worse(c, w) {
            interpolate(v, (w, 1.0), (c, 2.0), false)
        } else {
            0.0
        }
    }
}

/// The line as it can be decided over baseline `b` (design § 11 D23): an `x` part is decidable
/// only over a baseline above 0. Over 0 or below, a line that also declares `abs` is decided by
/// `abs` and `min_events` alone; a line of `x` only cannot be decided.
fn usable<'a>(
    line: &'a Line,
    name: &str,
    b: Num,
    baseline: &Decimal,
) -> Result<Cow<'a, Line>, Undecidable> {
    if line.x.is_none() || b.mantissa > 0 {
        return Ok(Cow::Borrowed(line));
    }
    if line.abs.is_none() {
        return Err(Undecidable(format!(
            "baseline {} is not above 0, so the x part of the {name} line cannot be decided and \
             the line declares no abs",
            baseline.0
        )));
    }
    Ok(Cow::Owned(Line {
        x: None,
        ..line.clone()
    }))
}

/// The score of `v` on the segment from `from` to `to`, extended past either end; log-linear when
/// `log` and every value is positive, linear otherwise.
fn interpolate(v: f64, from: (f64, f64), to: (f64, f64), log: bool) -> f64 {
    let ((lo, lo_score), (hi, hi_score)) = (from, to);
    let fraction = if log && lo > 0.0 && hi > 0.0 && v > 0.0 && lo != hi {
        (v.ln() - lo.ln()) / (hi.ln() - lo.ln())
    } else if hi == lo {
        0.0
    } else {
        (v - lo) / (hi - lo)
    };
    lo_score + (hi_score - lo_score) * fraction
}

/// The score rounded to three fractional digits and held inside the band of `verdict`: `OK`
/// below 1, `WARN` from 1 to under 2, `CRIT` from 2. A line not crossed because its
/// `min_events` is not met leaves the value past the line and the score just under it.
fn banded(verdict: Verdict, raw: f64) -> f64 {
    let rounded = if raw.is_finite() {
        (raw * 1000.0).round() / 1000.0
    } else {
        raw
    };
    match verdict {
        Verdict::WARN => rounded.clamp(1.0, WARN_CEILING),
        Verdict::CRIT => {
            if rounded.is_nan() {
                2.0
            } else {
                rounded.max(2.0)
            }
        }
        _ => {
            if rounded.is_nan() {
                0.0
            } else {
                rounded.min(OK_CEILING)
            }
        }
    }
}

/// A score in the wire grammar of a decimal: no trailing fractional zero, never `-0`. An
/// infinite score (a value past every finite position) is written as the largest finite one.
fn render_score(score: f64) -> String {
    let score = if score.is_infinite() {
        f64::MAX.copysign(score)
    } else {
        score
    };
    let text = format!("{score:.3}");
    let text = text.trim_end_matches('0').trim_end_matches('.');
    if text == "-0" {
        "0".to_owned()
    } else {
        text.to_owned()
    }
}

/// A scored verdict: the raw score rounded and held inside the verdict's band.
fn scored(verdict: Verdict, raw: f64, reason: String) -> Decision {
    Decision {
        verdict,
        score: Some(Decimal(render_score(banded(verdict, raw)))),
        reason,
    }
}

fn unknown(reason: String) -> Decision {
    Decision {
        verdict: Verdict::UNKNOWN,
        score: None,
        reason,
    }
}

fn out_of_range(what: &str) -> Undecidable {
    Undecidable(format!(
        "{what} is beyond the range the grader compares exactly"
    ))
}

/// An exact decimal: `mantissa / 10^scale`, without trailing fractional zeros.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Num {
    mantissa: i128,
    scale: u32,
}

impl Num {
    /// The wire grammar of a decimal (an optional `-`, digits, an optional `.` and digits);
    /// `None` for anything else, or for a number too long to hold.
    fn parse(text: &str) -> Option<Num> {
        let unsigned = text.strip_prefix('-').unwrap_or(text);
        let (whole, fraction) = match unsigned.split_once('.') {
            Some((whole, fraction)) => (whole, fraction),
            None => (unsigned, ""),
        };
        let digits = |part: &str| part.bytes().all(|byte| byte.is_ascii_digit());
        if whole.is_empty()
            || !digits(whole)
            || !digits(fraction)
            || (unsigned.contains('.') && fraction.is_empty())
        {
            return None;
        }
        let mut mantissa: i128 = 0;
        for byte in whole.bytes().chain(fraction.bytes()) {
            mantissa = mantissa
                .checked_mul(10)?
                .checked_add(i128::from(byte - b'0'))?;
        }
        if text.starts_with('-') {
            mantissa = -mantissa;
        }
        Some(Num::normal(mantissa, u32::try_from(fraction.len()).ok()?))
    }

    fn normal(mut mantissa: i128, mut scale: u32) -> Num {
        while scale > 0 && mantissa % 10 == 0 {
            mantissa /= 10;
            scale -= 1;
        }
        Num { mantissa, scale }
    }

    fn times(self, other: Num) -> Option<Num> {
        Some(Num::normal(
            self.mantissa.checked_mul(other.mantissa)?,
            self.scale.checked_add(other.scale)?,
        ))
    }

    /// Numeric order; `None` when aligning the two scales overflows.
    fn compare(self, other: Num) -> Option<Ordering> {
        let scale = self.scale.max(other.scale);
        let align = |num: Num| {
            10_i128
                .checked_pow(scale - num.scale)
                .and_then(|factor| num.mantissa.checked_mul(factor))
        };
        Some(align(self)?.cmp(&align(other)?))
    }

    /// To the nearest whole number, half away from zero; `None` past the exact range.
    fn rounded(self) -> Option<Num> {
        let factor = 10_i128.checked_pow(self.scale)?;
        let (whole, rest) = (self.mantissa / factor, self.mantissa % factor);
        let whole = if rest.unsigned_abs() * 2 >= factor.unsigned_abs() {
            whole + self.mantissa.signum()
        } else {
            whole
        };
        Some(Num::normal(whole, 0))
    }

    fn to_f64(self) -> f64 {
        self.to_string().parse().unwrap_or(f64::NAN)
    }
}

impl From<i64> for Num {
    fn from(value: i64) -> Num {
        Num::normal(i128::from(value), 0)
    }
}

impl fmt::Display for Num {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let digits = self.mantissa.unsigned_abs().to_string();
        let scale = self.scale as usize;
        let sign = if self.mantissa < 0 { "-" } else { "" };
        if scale == 0 {
            return write!(f, "{sign}{digits}");
        }
        let padded = format!("{digits:0>width$}", width = scale + 1);
        let (whole, fraction) = padded.split_at(padded.len() - scale);
        write!(f, "{sign}{whole}.{fraction}")
    }
}
