//! Grading: one sample against one recorded baseline, under one definition, yields exactly one
//! verdict of the ladder, its score and its reason (design § 2.3, § 3.1 rule 4, § 3.3).
//!
//! In this order:
//!
//! 1. a sample whose `denominator.n` was read and is below `min_n` is `N/A`, with no score;
//! 2. under `producer_mapped`, a sample that records the producer's text and is not `partial` is
//!    graded by that text through the mapping at the definition's revision once its baseline is
//!    read (step 3), whatever the reading's own quality (`crate::rules::mapped`); otherwise a
//!    reading of quality `unknown` is `UNKNOWN`, with the reading's reason;
//! 3. a baseline whose reading is not `ok` is `UNKNOWN`, with the baseline's reason;
//! 4. a `partial` reading may escalate but never clear: `CRIT` when its bound is already over the
//!    CRIT line, `UNKNOWN` with reason [`REASON_PARTIAL`] otherwise;
//! 5. otherwise the definition's rule decides.
//!
//! An unknown reading is never `OK` (L3, L5), and every result carries a reason, so none trips
//! `RecordEvaluation`'s `reason-missing` or `score-not-admitted`.

use std::fmt;

use thalamus_model::definition::SignalDefinitionData;
use thalamus_model::evaluation::{Baseline, Verdict};
use thalamus_model::observation::{CompleteReading, Reading, SignalSampleData, UnreadReading};
use thalamus_model::primitives::Decimal;

use crate::rules::threshold::LineError;
use crate::rules::{Decision, Prepared, RuleId, UnknownRule};

/// The reason of a partial reading that does not escalate (design § 3.1 rule 4).
pub const REASON_PARTIAL: &str = "partial read";

/// One graded sample.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Graded {
    /// The verdict.
    pub verdict: Verdict,
    /// The score; `None` exactly when the verdict is `N/A` or `UNKNOWN`.
    pub score: Option<Decimal>,
    /// Why; present on every verdict, `OK` included.
    pub reason: String,
    /// The baseline the sample was graded against, as it was handed in.
    pub baseline: Baseline,
}

/// Why a definition cannot grade any sample. A reading that cannot be graded is not a refusal:
/// it is `UNKNOWN`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Refusal {
    /// `evaluation.rule` names no rule.
    UnknownRule(UnknownRule),
    /// The rule's lines cannot be graded against.
    Lines(LineError),
}

impl fmt::Display for Refusal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Refusal::UnknownRule(unknown) => unknown.fmt(f),
            Refusal::Lines(lines) => lines.fmt(f),
        }
    }
}

impl std::error::Error for Refusal {}

/// Grades `sample` against `baseline` under `definition`.
///
/// # Errors
///
/// A [`Refusal`] when the definition names no known rule or its lines are unusable, whatever the
/// reading.
pub fn grade(
    definition: &SignalDefinitionData,
    sample: &SignalSampleData,
    baseline: &Baseline,
) -> Result<Graded, Refusal> {
    let rule: RuleId = definition
        .evaluation
        .rule
        .0
        .parse()
        .map_err(Refusal::UnknownRule)?;
    let prepared = rule
        .prepare(&definition.evaluation)
        .map_err(Refusal::Lines)?;
    let Decision {
        verdict,
        score,
        reason,
    } = decide(definition, sample, baseline, &prepared);
    Ok(Graded {
        verdict,
        score,
        reason,
        baseline: baseline.clone(),
    })
}

fn decide(
    definition: &SignalDefinitionData,
    sample: &SignalSampleData,
    baseline: &Baseline,
    prepared: &Prepared,
) -> Decision {
    let n = sample.denominator.as_ref().map(|denominator| denominator.n);
    if let (Some(n), Some(spec)) = (n, &definition.denominator)
        && n < spec.min_n
    {
        return unscored(
            Verdict::NA,
            format!("n {n} below min_n {}: too little to grade", spec.min_n),
        );
    }
    if let Prepared::Mapped(rule) = prepared
        && let Some(text) = sample.text.as_deref()
        && !matches!(sample.reading, Reading::Partial(_))
    {
        return match baseline_value(baseline) {
            Ok(_) => rule.decide_text(text, definition.rev),
            Err(unread) => unread,
        };
    }
    let read = match &sample.reading {
        Reading::Unknown(UnreadReading { reason }) => {
            let reason = if reason.is_empty() {
                "sample unknown".to_owned()
            } else {
                reason.clone()
            };
            return unscored(Verdict::UNKNOWN, reason);
        }
        Reading::Ok(CompleteReading { value }) => Ok(value),
        Reading::Partial(partial) => Err(partial),
    };
    let base = match baseline_value(baseline) {
        Ok(base) => base,
        Err(unread) => return unread,
    };
    match read {
        Ok(value) => prepared.decide(value, base, n),
        Err(partial) => prepared
            .escalate(&partial.value, partial.bound, base, n)
            .unwrap_or_else(|| unscored(Verdict::UNKNOWN, REASON_PARTIAL.to_owned())),
    }
}

/// The baseline's value; `UNKNOWN` with the baseline's reason when its reading is not `ok`
/// (step 3).
fn baseline_value(baseline: &Baseline) -> Result<&Decimal, Decision> {
    match &baseline.reading {
        Reading::Ok(CompleteReading { value }) => Ok(value),
        Reading::Unknown(UnreadReading { reason }) => Err(unscored(
            Verdict::UNKNOWN,
            format!("baseline unknown: {reason}"),
        )),
        Reading::Partial(_) => Err(unscored(
            Verdict::UNKNOWN,
            "baseline read partially: a bound is not a baseline".to_owned(),
        )),
    }
}

fn unscored(verdict: Verdict, reason: String) -> Decision {
    Decision {
        verdict,
        score: None,
        reason,
    }
}
