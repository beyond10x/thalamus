//! The rule `fixed_warn_only`: an advisory gate's finding count (kind `count_of_cases`, unit
//! `{finding}`) against the declared `fixed` baseline (design § 2.3, § 11 D12). A count above the
//! baseline is `WARN`, one at or below it `OK`; the rule has no CRIT line, so it never grades
//! `CRIT`. A gate that printed no count is an `unknown` reading, which `crate::grading` makes
//! `UNKNOWN` before any rule decides, so a missing count is never 0.
//!
//! The verdict and the score are decided exactly, on the decimals, each read without its trailing
//! fractional zeros (`3.000` is `3`). The score (design § 3.3) is
//! `(count - baseline) / (floor(baseline) + 1 - baseline)`: 0 at the baseline, at any magnitude,
//! and 1 at the first whole count above it, linear in between and below the baseline (negative
//! there), and 1 for every `WARN`: never above 1. Numerator and denominator are exact; only the
//! division rounds, half away from zero, to three fractional digits. A count or baseline beyond
//! the exact range is `UNKNOWN` with a reason, never a number.
//!
//! The rule reads `evaluation.fixed` through the baseline alone. A `crit` line, a `warn` line or a
//! baseline list that is not `fixed` alone is a number it would never read, and a signal whose
//! kind is not `count_of_cases` is no finding count, so `ProposeDefinition`, `ReviseDefinition`
//! and the definition loader refuse each alike, through
//! `crate::behaviour::definition::input_refusals` ([`thresholds_unused`], [`kind_not_count`],
//! [`baseline_not_fixed`]).

use thalamus_model::definition::{BaselineKind, EvaluationSpec, SignalKind};
use thalamus_model::evaluation::Verdict;
use thalamus_model::primitives::Decimal;

use super::Decision;

/// The rule's id, as a definition writes it.
pub const ID: &str = "fixed_warn_only";

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
    /// `baseline`: `WARN` above it, `OK` at or below it, `UNKNOWN` when either is not a decimal
    /// this rule compares and scores exactly.
    #[must_use]
    pub fn decide(&self, value: &Decimal, baseline: &Decimal) -> Decision {
        let (Some(v), Some(b)) = (Exact::parse(&value.0), Exact::parse(&baseline.0)) else {
            return unknown(format!(
                "count `{}` or fixed baseline `{}` is not a decimal within the grader's range",
                value.0, baseline.0
            ));
        };
        let Some(aligned) = Aligned::of(v, b) else {
            return unknown(format!(
                "count {} against fixed baseline {} is beyond the range the grader compares \
                 exactly",
                value.0, baseline.0
            ));
        };
        if aligned.count > aligned.baseline {
            return Decision {
                verdict: Verdict::WARN,
                score: Some(Decimal("1".to_owned())),
                reason: format!(
                    "count {} > fixed baseline {}: advisory, WARN at most",
                    value.0, baseline.0
                ),
            };
        }
        let Some(score) = aligned.ok_score() else {
            return unknown(format!(
                "the score of count {} against fixed baseline {} is beyond the range the grader \
                 computes exactly",
                value.0, baseline.0
            ));
        };
        Decision {
            verdict: Verdict::OK,
            score: Some(Decimal(score)),
            reason: format!("count {} <= fixed baseline {}", value.0, baseline.0),
        }
    }
}

fn unknown(reason: String) -> Decision {
    Decision {
        verdict: Verdict::UNKNOWN,
        score: None,
        reason,
    }
}

/// An exact decimal, `mantissa / 10^scale`, without trailing fractional zeros.
#[derive(Debug, Clone, Copy)]
struct Exact {
    mantissa: i128,
    scale: u32,
}

impl Exact {
    /// The wire grammar of a decimal (an optional `-`, digits, an optional `.` and digits); `None`
    /// for anything else, or for a number too long to hold. Trailing fractional zeros are dropped
    /// before any digit is held, so they never reach the exact range.
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
        let mut mantissa: i128 = 0;
        for byte in whole.bytes().chain(fraction.bytes()) {
            mantissa = mantissa
                .checked_mul(10)?
                .checked_add(i128::from(byte - b'0'))?;
        }
        if text.starts_with('-') {
            mantissa = -mantissa;
        }
        Some(Exact {
            mantissa,
            scale: u32::try_from(fraction.len()).ok()?,
        })
    }
}

/// A count and a baseline as whole multiples of `10^-scale`, the larger of their two scales.
#[derive(Debug, Clone, Copy)]
struct Aligned {
    count: i128,
    baseline: i128,
    scale: u32,
}

impl Aligned {
    /// `None` when aligning the two scales overflows.
    fn of(count: Exact, baseline: Exact) -> Option<Aligned> {
        let scale = count.scale.max(baseline.scale);
        let align = |num: Exact| {
            10_i128
                .checked_pow(scale - num.scale)
                .and_then(|factor| num.mantissa.checked_mul(factor))
        };
        Some(Aligned {
            count: align(count)?,
            baseline: align(baseline)?,
            scale,
        })
    }

    /// The score of a count at or below the baseline,
    /// `(count - baseline) / (floor(baseline) + 1 - baseline)`, in the wire grammar of a decimal:
    /// at most 0, three fractional digits at most, no trailing fractional zero, never `-0`. Both
    /// terms are exact; `None` when either leaves the exact range.
    fn ok_score(self) -> Option<String> {
        let unit = 10_i128.checked_pow(self.scale)?;
        // `floor(baseline) + 1 - baseline`, in `10^-scale`: in (0, 1], so in 1..=unit here.
        let step = unit - self.baseline.rem_euclid(unit);
        let below = self.baseline.checked_sub(self.count)?;
        Some(render_negative(below.unsigned_abs(), step.unsigned_abs()))
    }
}

/// `-(numerator / denominator)`, rounded half away from zero to three fractional digits, in the
/// wire grammar of a decimal. `denominator` is at least 1 and at most `i128::MAX`, so no step
/// below overflows.
fn render_negative(numerator: u128, denominator: u128) -> String {
    let mut whole = numerator / denominator;
    let mut rest = numerator % denominator;
    let mut fraction: u32 = 0;
    for _ in 0..3 {
        // `10 * rest / denominator` and its remainder, by repeated addition: `rest` and `carried`
        // stay below `denominator`, so their sum stays below `2 * i128::MAX < u128::MAX`.
        let (mut digit, mut carried) = (0_u32, 0_u128);
        for _ in 0..10 {
            carried += rest;
            if carried >= denominator {
                carried -= denominator;
                digit += 1;
            }
        }
        fraction = fraction * 10 + digit;
        rest = carried;
    }
    if rest >= denominator - rest {
        fraction += 1;
        if fraction == 1000 {
            fraction = 0;
            whole += 1;
        }
    }
    if whole == 0 && fraction == 0 {
        return "0".to_owned();
    }
    if fraction == 0 {
        return format!("-{whole}");
    }
    let digits = format!("{fraction:03}");
    format!("-{whole}.{}", digits.trim_end_matches('0'))
}
