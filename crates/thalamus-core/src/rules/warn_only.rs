//! The rule `fixed_warn_only`: an advisory gate's finding count (kind `count_of_cases`, unit
//! `{finding}`) against the declared `fixed` baseline (design § 2.3, § 11 D12). A count above the
//! baseline is `WARN`, one at or below it `OK`; the rule has no CRIT line, so it never grades
//! `CRIT`. A gate that printed no count is an `unknown` reading, which `crate::grading` makes
//! `UNKNOWN` before any rule decides, so a missing count is never 0.
//!
//! The verdict and the score are decided exactly, on the decimals, each read without its trailing
//! fractional zeros (`3.000` is `3`), at any magnitude and any number of fractional digits: the
//! digits are held as written, never in a fixed-width integer or a float. The score (design § 3.3)
//! is `(count - baseline) / (floor(baseline) + 1 - baseline)`: 0 at the baseline, and 1 at the
//! first whole count above it, linear in between and below the baseline (negative there), and 1 for
//! every `WARN`: never above 1. Numerator and denominator are exact; only the division rounds, half
//! away from zero, to three fractional digits. A count or baseline that is not a decimal is
//! `UNKNOWN` with a reason, never a number.
//!
//! The rule reads `evaluation.fixed` through the baseline alone. A `crit` line, a `warn` line or a
//! baseline list that is not `fixed` alone is a number it would never read, and a signal whose
//! kind is not `count_of_cases` or whose unit is not `{finding}` is no finding count, so
//! `ProposeDefinition`, `ReviseDefinition` and the definition loader refuse each alike, through
//! `crate::behaviour::definition::input_refusals` ([`thresholds_unused`], [`kind_not_count`],
//! [`unit_not_finding`], [`baseline_not_fixed`]).

use std::cmp::Ordering;

use thalamus_model::definition::{BaselineKind, EvaluationSpec, SignalKind, Unit};
use thalamus_model::evaluation::Verdict;
use thalamus_model::primitives::Decimal;

use super::Decision;

/// The rule's id, as a definition writes it.
pub const ID: &str = "fixed_warn_only";

/// The one unit the rule grades: an advisory gate's findings (design § 11 D12).
pub const UNIT: &str = "{finding}";

/// Whether `evaluation` names this rule and declares a `warn` or `crit` line, which nothing reads:
/// the rule has no CRIT line, and its WARN line is the first count above the fixed baseline.
/// Refused as `thresholds-unused`, as under `producer_mapped`.
#[must_use]
pub fn thresholds_unused(evaluation: &EvaluationSpec) -> bool {
    evaluation.rule.0 == ID && (evaluation.warn.is_some() || evaluation.crit.is_some())
}

/// Whether `evaluation` names this rule for a signal whose `kind` is not `count_of_cases`: the rule
/// grades an advisory gate's finding count (design § 11 D12), and its score steps to the first
/// whole count above the baseline, which no gauge, ratio, increase or instant has. Refused as
/// `advisory-not-count`.
#[must_use]
pub fn kind_not_count(kind: SignalKind, evaluation: &EvaluationSpec) -> bool {
    evaluation.rule.0 == ID && kind != SignalKind::CountOfCases
}

/// Whether `evaluation` names this rule for a signal that declares a unit other than [`UNIT`]: the
/// rule grades an advisory gate's finding count, unit `{finding}` (design § 11 D12). A proposal
/// that declares no unit is not refused here: it cannot be activated before a revision declares
/// one, and every revision does. Refused as `advisory-not-finding-unit`.
#[must_use]
pub fn unit_not_finding(unit: Option<&Unit>, evaluation: &EvaluationSpec) -> bool {
    evaluation.rule.0 == ID && unit.is_some_and(|unit| unit.0 != UNIT)
}

/// Whether `evaluation` names this rule and its `baselines` is not `fixed` alone: a baseline
/// other than `fixed`, or none at all, which ESS 0.60.0 cannot state beside the first
/// (`UNMAPPED:` on `advisory-not-fixed`). Refused as `advisory-not-fixed`.
#[must_use]
pub fn baseline_not_fixed(evaluation: &EvaluationSpec) -> bool {
    evaluation.rule.0 == ID
        && (evaluation.baselines.is_empty()
            || evaluation
                .baselines
                .iter()
                .any(|baseline| *baseline != BaselineKind::Fixed))
}

/// The rule `fixed_warn_only`. It holds nothing: the baseline is the line.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rule;

impl Rule {
    /// The verdict, score and reason of the completely read count `value` against the fixed
    /// `baseline`: `WARN` above it, `OK` at or below it, `UNKNOWN` when either is not a decimal.
    #[must_use]
    pub fn decide(&self, value: &Decimal, baseline: &Decimal) -> Decision {
        let (Some(count), Some(line)) = (Exact::parse(&value.0), Exact::parse(&baseline.0)) else {
            return Decision {
                verdict: Verdict::UNKNOWN,
                score: None,
                reason: format!(
                    "count `{}` or fixed baseline `{}` is not a decimal",
                    value.0, baseline.0
                ),
            };
        };
        let aligned = Aligned::of(count, line);
        if aligned.count.compare(&aligned.baseline) == Ordering::Greater {
            return Decision {
                verdict: Verdict::WARN,
                score: Some(Decimal("1".to_owned())),
                reason: format!(
                    "count {} > fixed baseline {}: advisory, WARN at most",
                    value.0, baseline.0
                ),
            };
        }
        Decision {
            verdict: Verdict::OK,
            score: Some(Decimal(aligned.ok_score())),
            reason: format!("count {} <= fixed baseline {}", value.0, baseline.0),
        }
    }
}

/// A non-negative whole number of any size: its decimal digits (0 to 9), most significant first,
/// without leading zeros, so zero has none.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Whole(Vec<u8>);

impl Whole {
    fn new(mut digits: Vec<u8>) -> Whole {
        let zeros = digits.iter().take_while(|digit| **digit == 0).count();
        digits.drain(..zeros);
        Whole(digits)
    }

    fn is_zero(&self) -> bool {
        self.0.is_empty()
    }

    /// `10^exponent`.
    fn power_of_ten(exponent: usize) -> Whole {
        let mut digits = vec![0; exponent + 1];
        digits[0] = 1;
        Whole(digits)
    }

    /// `self * 10^exponent`.
    fn shifted(mut self, exponent: usize) -> Whole {
        if !self.is_zero() {
            self.0.resize(self.0.len() + exponent, 0);
        }
        self
    }

    /// `self mod 10^exponent`: its last `exponent` digits.
    fn low(&self, exponent: usize) -> Whole {
        Whole::new(self.0[self.0.len().saturating_sub(exponent)..].to_vec())
    }

    /// `self * 10 + digit`.
    fn push(&mut self, digit: u8) {
        if !(self.is_zero() && digit == 0) {
            self.0.push(digit);
        }
    }

    fn compare(&self, other: &Whole) -> Ordering {
        // Without leading zeros, the longer is the larger; at one length, the digits decide.
        self.0
            .len()
            .cmp(&other.0.len())
            .then_with(|| self.0.cmp(&other.0))
    }

    fn plus(&self, other: &Whole) -> Whole {
        let mut sum = Vec::with_capacity(self.0.len().max(other.0.len()) + 1);
        let (mut left, mut right) = (self.0.iter().rev(), other.0.iter().rev());
        let mut carry = 0;
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
        Whole::new(sum)
    }

    /// `self - other`, for `other <= self`.
    fn minus(&self, other: &Whole) -> Whole {
        debug_assert!(other.compare(self) != Ordering::Greater);
        let mut difference = Vec::with_capacity(self.0.len());
        let mut right = other.0.iter().rev();
        let mut borrow = 0;
        for a in self.0.iter().rev() {
            let b = right.next().copied().unwrap_or(0) + borrow;
            if *a >= b {
                difference.push(a - b);
                borrow = 0;
            } else {
                difference.push(a + 10 - b);
                borrow = 1;
            }
        }
        difference.reverse();
        Whole::new(difference)
    }

    /// The next decimal digit of a long division: `rest` becomes `rest mod divisor`, and the
    /// quotient, at most 9 while `rest < 10 * divisor`, is returned.
    fn reduce(rest: &mut Whole, divisor: &Whole) -> u8 {
        let mut digit = 0;
        while rest.compare(divisor) != Ordering::Less {
            *rest = rest.minus(divisor);
            digit += 1;
        }
        digit
    }

    /// `(self / divisor, self mod divisor)`, for a divisor that is not zero.
    fn divide(&self, divisor: &Whole) -> (Whole, Whole) {
        let mut quotient = Vec::with_capacity(self.0.len());
        let mut rest = Whole(Vec::new());
        for digit in &self.0 {
            rest.push(*digit);
            quotient.push(Whole::reduce(&mut rest, divisor));
        }
        (Whole::new(quotient), rest)
    }

    fn render(&self) -> String {
        if self.is_zero() {
            return "0".to_owned();
        }
        self.0
            .iter()
            .map(|digit| char::from(b'0' + digit))
            .collect()
    }
}

/// A whole number with a sign; zero is never negative.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Signed {
    negative: bool,
    magnitude: Whole,
}

impl Signed {
    fn new(negative: bool, magnitude: Whole) -> Signed {
        Signed {
            negative: negative && !magnitude.is_zero(),
            magnitude,
        }
    }

    /// `self - other`.
    fn minus(&self, other: &Signed) -> Signed {
        if self.negative != other.negative {
            // `a - (-b)` is `a + b`, and `-a - b` is `-(a + b)`.
            return Signed::new(self.negative, self.magnitude.plus(&other.magnitude));
        }
        // One sign: `±(|a| - |b|)`, the sign turning where `|b|` is the larger.
        if self.magnitude.compare(&other.magnitude) == Ordering::Less {
            Signed::new(!self.negative, other.magnitude.minus(&self.magnitude))
        } else {
            Signed::new(self.negative, self.magnitude.minus(&other.magnitude))
        }
    }

    fn compare(&self, other: &Signed) -> Ordering {
        match (self.negative, other.negative) {
            (false, true) => Ordering::Greater,
            (true, false) => Ordering::Less,
            (false, false) => self.magnitude.compare(&other.magnitude),
            (true, true) => other.magnitude.compare(&self.magnitude),
        }
    }
}

/// An exact decimal, `digits / 10^scale` with its sign, without trailing fractional zeros.
#[derive(Debug, Clone)]
struct Exact {
    value: Signed,
    scale: usize,
}

impl Exact {
    /// The wire grammar of a decimal (an optional `-`, digits, an optional `.` and digits); `None`
    /// for anything else. Trailing fractional zeros are dropped, so `3.000` is `3`.
    fn parse(text: &str) -> Option<Exact> {
        let unsigned = text.strip_prefix('-').unwrap_or(text);
        let (whole, fraction) = unsigned.split_once('.').unwrap_or((unsigned, ""));
        let digits = |part: &str| part.bytes().all(|byte| byte.is_ascii_digit());
        if whole.is_empty()
            || !digits(whole)
            || !digits(fraction)
            || (unsigned.contains('.') && fraction.is_empty())
        {
            return None;
        }
        let fraction = fraction.trim_end_matches('0');
        let magnitude = Whole::new(
            whole
                .bytes()
                .chain(fraction.bytes())
                .map(|byte| byte - b'0')
                .collect(),
        );
        Some(Exact {
            value: Signed::new(text.starts_with('-'), magnitude),
            scale: fraction.len(),
        })
    }

    /// The value as a whole multiple of `10^-scale`, for a `scale` at least its own.
    fn at(self, scale: usize) -> Signed {
        Signed {
            negative: self.value.negative,
            magnitude: self.value.magnitude.shifted(scale - self.scale),
        }
    }
}

/// A count and a baseline as whole multiples of `10^-scale`, the larger of their two scales.
#[derive(Debug, Clone)]
struct Aligned {
    count: Signed,
    baseline: Signed,
    scale: usize,
}

impl Aligned {
    fn of(count: Exact, baseline: Exact) -> Aligned {
        let scale = count.scale.max(baseline.scale);
        Aligned {
            count: count.at(scale),
            baseline: baseline.at(scale),
            scale,
        }
    }

    /// The score of a count at or below the baseline,
    /// `(count - baseline) / (floor(baseline) + 1 - baseline)`, in the wire grammar of a decimal:
    /// at most 0, three fractional digits at most, no trailing fractional zero, never `-0`.
    fn ok_score(&self) -> String {
        // `floor(baseline) + 1 - baseline`, in `10^-scale`: in (0, 1], so in 1..=10^scale. The
        // baseline's fractional part `low` is `baseline - trunc(baseline)` in magnitude; below 0
        // the floor is one further down, so the step is `low` itself there.
        let unit = Whole::power_of_ten(self.scale);
        let low = self.baseline.magnitude.low(self.scale);
        let step = if low.is_zero() {
            unit
        } else if self.baseline.negative {
            low
        } else {
            unit.minus(&low)
        };
        // `baseline - count`, at least 0 at or below the baseline.
        let below = self.baseline.minus(&self.count);
        render_negative(&below.magnitude, &step)
    }
}

/// `-(numerator / denominator)`, rounded half away from zero to three fractional digits, in the
/// wire grammar of a decimal. `denominator` is not zero.
fn render_negative(numerator: &Whole, denominator: &Whole) -> String {
    let (mut whole, mut rest) = numerator.divide(denominator);
    let mut fraction: u32 = 0;
    for _ in 0..3 {
        rest.push(0);
        fraction = fraction * 10 + u32::from(Whole::reduce(&mut rest, denominator));
    }
    if rest.plus(&rest).compare(denominator) != Ordering::Less {
        fraction += 1;
        if fraction == 1000 {
            fraction = 0;
            whole = whole.plus(&Whole::power_of_ten(0));
        }
    }
    if whole.is_zero() && fraction == 0 {
        return "0".to_owned();
    }
    if fraction == 0 {
        return format!("-{}", whole.render());
    }
    let digits = format!("{fraction:03}");
    format!("-{}.{}", whole.render(), digits.trim_end_matches('0'))
}
