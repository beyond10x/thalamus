//! The closed list of grading rules: the id a definition names in `evaluation.rule`, and the
//! dispatch from each id to the rule that decides a verdict. A later rule adds one variant and
//! one arm of each `match` here.

pub mod mapped;
pub mod threshold;
pub mod warn_only;

use std::fmt;
use std::str::FromStr;

use thalamus_model::definition::EvaluationSpec;
use thalamus_model::evaluation::Verdict;
use thalamus_model::observation::Bound;
use thalamus_model::primitives::Decimal;

use threshold::{Direction, Events, LineError};

/// A grading rule a definition may name.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RuleId {
    /// A ratio against its pooled baseline; higher is worse, events are value times `n`
    /// (design § 2.1).
    RatioVsPooledBaseline,
    /// A count or gauge against `higher_of` or `fixed`; higher is worse.
    HigherIsWorse,
    /// A count or gauge against `lower_of` or `fixed`; lower is worse.
    LowerIsWorse,
    /// A verdict a producer gave in its own vocabulary, mapped by `evaluation.mapping`
    /// ([`mapped`], design § 2.3).
    ProducerMapped,
    /// An advisory gate's finding count against its `fixed` baseline: `WARN` above it, never
    /// `CRIT` ([`warn_only`], design § 11 D12).
    FixedWarnOnly,
}

impl RuleId {
    /// Every rule, in the order an error lists them.
    pub const ALL: [RuleId; 5] = [
        RuleId::RatioVsPooledBaseline,
        RuleId::HigherIsWorse,
        RuleId::LowerIsWorse,
        RuleId::ProducerMapped,
        RuleId::FixedWarnOnly,
    ];

    /// The id as a definition writes it.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            RuleId::RatioVsPooledBaseline => "ratio_vs_pooled_baseline",
            RuleId::HigherIsWorse => "higher_is_worse",
            RuleId::LowerIsWorse => "lower_is_worse",
            RuleId::ProducerMapped => mapped::ID,
            RuleId::FixedWarnOnly => warn_only::ID,
        }
    }

    /// The rule over the lines of `evaluation`.
    ///
    /// # Errors
    ///
    /// A [`LineError`] when the definition's lines cannot be graded against.
    pub fn prepare(self, evaluation: &EvaluationSpec) -> Result<Prepared, LineError> {
        let (direction, events) = match self {
            RuleId::RatioVsPooledBaseline => (Direction::HigherIsWorse, Events::TimesN),
            RuleId::HigherIsWorse => (Direction::HigherIsWorse, Events::Plain),
            RuleId::LowerIsWorse => (Direction::LowerIsWorse, Events::Plain),
            RuleId::ProducerMapped => return Ok(Prepared::Mapped(mapped::Rule::new(evaluation))),
            // The baseline is the line; the lines and baselines this rule never reads are refused
            // on propose, revise and load (`warn_only::thresholds_unused`, `baseline_not_fixed`).
            RuleId::FixedWarnOnly => return Ok(Prepared::WarnOnly(warn_only::Rule)),
        };
        threshold::Rule::new(self.as_str(), direction, events, evaluation)
            .map(|rule| Prepared::Threshold(Box::new(rule)))
    }
}

impl fmt::Display for RuleId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for RuleId {
    type Err = UnknownRule;

    fn from_str(id: &str) -> Result<Self, Self::Err> {
        RuleId::ALL
            .into_iter()
            .find(|rule| rule.as_str() == id)
            .ok_or_else(|| UnknownRule { id: id.to_owned() })
    }
}

/// `evaluation.rule` names no rule of [`RuleId::ALL`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnknownRule {
    /// The id as the definition wrote it.
    pub id: String,
}

impl fmt::Display for UnknownRule {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let known: Vec<&str> = RuleId::ALL.iter().map(|rule| rule.as_str()).collect();
        write!(
            f,
            "unknown grading rule `{}`; the rules are {}",
            self.id,
            known.join(", ")
        )
    }
}

impl std::error::Error for UnknownRule {}

/// A rule bound to its definition's lines, ready to decide readings.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Prepared {
    /// A [`threshold`] rule.
    Threshold(Box<threshold::Rule>),
    /// The [`mapped`] rule.
    Mapped(mapped::Rule),
    /// The [`warn_only`] rule.
    WarnOnly(warn_only::Rule),
}

impl Prepared {
    /// The verdict of a completely read `value` against `baseline`; `n` is the sample's
    /// population, where the source answered one.
    #[must_use]
    pub fn decide(&self, value: &Decimal, baseline: &Decimal, n: Option<i64>) -> Decision {
        match self {
            Prepared::Threshold(rule) => rule.decide(value, baseline, n),
            // A sample with producer text is graded by `grading::grade` through the mapping; one
            // without is never graded on its number.
            Prepared::Mapped(rule) => rule.without_text(),
            Prepared::WarnOnly(rule) => rule.decide(value, baseline),
        }
    }

    /// The verdict a partial read whose `value` is a `bound` escalates to, if the bound alone
    /// decides it; `None` when it does not.
    #[must_use]
    pub fn escalate(
        &self,
        value: &Decimal,
        bound: Bound,
        baseline: &Decimal,
        n: Option<i64>,
    ) -> Option<Decision> {
        match self {
            Prepared::Threshold(rule) => rule.escalate(value, bound, baseline, n),
            // A bound is no producer value, whatever text the sample records.
            Prepared::Mapped(_) => None,
            // A partial read escalates only to `CRIT` (design § 3.1 rule 4), which this rule
            // never grades.
            Prepared::WarnOnly(_) => None,
        }
    }
}

/// What a rule decided: one verdict, its score where the verdict is scored, and why.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Decision {
    /// The verdict.
    pub verdict: Verdict,
    /// The score (design § 3.3); `None` exactly when the verdict is not scored.
    pub score: Option<Decimal>,
    /// The comparison that decided the verdict.
    pub reason: String,
}
