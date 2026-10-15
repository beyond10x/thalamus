//! The rule `producer_mapped`: a producer that already speaks another vocabulary (a colour
//! ladder, a detector's or a gate's exit code) is graded by its own value, the sample's `text`,
//! mapped through the definition's `evaluation.mapping` (design § 2.3). The table is read at the
//! revision being graded, so a mapping revision re-grades stored history (design § 2, D5). A
//! mapped verdict is scored at its [`position`] on the score scale of design § 3.3, so the colour
//! and the verdict cannot disagree.

use std::collections::BTreeMap;

use thalamus_model::definition::EvaluationSpec;
use thalamus_model::evaluation::Verdict;
use thalamus_model::primitives::Decimal;

use super::Decision;

/// The rule's id, as a definition writes it.
pub const ID: &str = "producer_mapped";

/// The reason of a sample under this rule that records no producer text.
pub const REASON_NO_TEXT: &str = "the sample records no producer value to map";

/// The verdicts a mapping may name, each with its position: 0 at `OK`, 1 at `WARN`, 2 at `CRIT`;
/// `UNKNOWN` has none.
const POSITIONS: [(Verdict, &str); 3] = [
    (Verdict::OK, "0"),
    (Verdict::WARN, "1"),
    (Verdict::CRIT, "2"),
];

/// The position of a scored `verdict` on the score scale; `None` for `UNKNOWN` and `N/A`, which
/// carry no score.
#[must_use]
pub fn position(verdict: Verdict) -> Option<Decimal> {
    POSITIONS
        .iter()
        .find(|(scored, _)| *scored == verdict)
        .map(|(_, at)| Decimal((*at).to_owned()))
}

/// Whether `evaluation` names this rule and has no mapping onto `OK`, `WARN`, `CRIT` or
/// `UNKNOWN`: none at all, an empty one, which maps nothing, or one naming `N/A`, which a
/// producer value never is (it was read) and a sample cannot carry. A mapping whose every value
/// is `UNKNOWN` is a choice, and is kept. `ProposeDefinition`, `ReviseDefinition` and the
/// definition loader refuse such a definition as `mapping-missing`.
#[must_use]
pub fn mapping_missing(evaluation: &EvaluationSpec) -> bool {
    evaluation.rule.0 == ID
        && evaluation.mapping.as_ref().is_none_or(|mapping| {
            mapping.is_empty() || mapping.values().any(|verdict| *verdict == Verdict::NA)
        })
}

/// Whether `evaluation` declares a mapping under a rule other than this one, which nothing reads.
/// `ProposeDefinition`, `ReviseDefinition` and the definition loader refuse it as
/// `mapping-unused`.
#[must_use]
pub fn mapping_unused(evaluation: &EvaluationSpec) -> bool {
    evaluation.mapping.is_some() && evaluation.rule.0 != ID
}

/// Whether `evaluation` names this rule and declares a `warn` or `crit` line, which nothing reads:
/// this rule grades by the mapping alone. Refused as `thresholds-unused`.
#[must_use]
pub fn thresholds_unused(evaluation: &EvaluationSpec) -> bool {
    evaluation.rule.0 == ID && (evaluation.warn.is_some() || evaluation.crit.is_some())
}

/// The rule `producer_mapped` over its definition's mapping.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rule {
    mapping: BTreeMap<String, Verdict>,
}

impl Rule {
    /// The rule over `evaluation.mapping`; a definition without one maps nothing, and the loader
    /// and `ProposeDefinition` refuse it before it grades.
    #[must_use]
    pub fn new(evaluation: &EvaluationSpec) -> Rule {
        Rule {
            mapping: evaluation.mapping.clone().unwrap_or_default(),
        }
    }

    /// The verdict of the producer value `text`, matched exactly as written, under the mapping of
    /// revision `rev`: an `OK`, `WARN` or `CRIT` scored at its position, and `UNKNOWN`, with no
    /// score, when the mapping says so or does not list `text`. The reason names `text` and `rev`.
    #[must_use]
    pub fn decide_text(&self, text: &str, rev: i64) -> Decision {
        let verdict = self.mapping.get(text).copied();
        match verdict.and_then(|verdict| position(verdict).map(|at| (verdict, at))) {
            Some((verdict, at)) => Decision {
                verdict,
                score: Some(at),
                reason: format!(
                    "producer value `{text}` maps to {} at rev {rev}",
                    name(verdict)
                ),
            },
            None => Decision {
                verdict: Verdict::UNKNOWN,
                score: None,
                reason: match verdict {
                    Some(other) => {
                        format!(
                            "producer value `{text}` maps to {} at rev {rev}",
                            name(other)
                        )
                    }
                    None => {
                        format!("producer value `{text}` is not in evaluation.mapping at rev {rev}")
                    }
                },
            },
        }
    }

    /// The verdict of a sample under this rule that records no producer text, whatever its number:
    /// `UNKNOWN`, [`REASON_NO_TEXT`].
    #[must_use]
    pub fn without_text(&self) -> Decision {
        Decision {
            verdict: Verdict::UNKNOWN,
            score: None,
            reason: REASON_NO_TEXT.to_owned(),
        }
    }
}

fn name(verdict: Verdict) -> &'static str {
    match verdict {
        Verdict::OK => "OK",
        Verdict::NA => "N/A",
        Verdict::WARN => "WARN",
        Verdict::CRIT => "CRIT",
        Verdict::UNKNOWN => "UNKNOWN",
    }
}
