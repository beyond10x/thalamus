//! The one verdict ladder (design § 2.3, L1): each verdict's exit code, rank and whether it
//! carries a score, the worst of several verdicts, and the exit code of a pass.
//!
//! The attributes are those of `thalamus.evaluation.Verdict` (`spec/domains/evaluation.yaml`);
//! the generated enum carries the names only, so they are read here, once.

use thalamus_model::evaluation::Verdict;

/// The exit code a pass whose worst verdict is `UNKNOWN` ends with, and the exit code of a pass
/// that graded nothing: every verdict `N/A`, or none at all.
pub const EXIT_UNKNOWN: i32 = 3;

/// The verdict's `rank`: the order of an aggregate, worst first, `CRIT > WARN > UNKNOWN > OK =
/// N/A`.
#[must_use]
pub fn rank(verdict: Verdict) -> u8 {
    match verdict {
        Verdict::OK | Verdict::NA => 0,
        Verdict::UNKNOWN => 1,
        Verdict::WARN => 2,
        Verdict::CRIT => 3,
    }
}

/// The verdict's `exit`: the process exit code a verdict maps to.
#[must_use]
pub fn exit(verdict: Verdict) -> i32 {
    match verdict {
        Verdict::OK | Verdict::NA => 0,
        Verdict::WARN => 1,
        Verdict::CRIT => 2,
        Verdict::UNKNOWN => EXIT_UNKNOWN,
    }
}

/// The verdict's `scored`: `OK`, `WARN` and `CRIT` carry a score; `N/A` and `UNKNOWN` never do.
#[must_use]
pub fn scored(verdict: Verdict) -> bool {
    match verdict {
        Verdict::OK | Verdict::WARN | Verdict::CRIT => true,
        Verdict::NA | Verdict::UNKNOWN => false,
    }
}

/// The worst of `verdicts` by [`rank`]; `None` when there are none.
///
/// `OK` and `N/A` share rank 0. Between the two, `N/A` is answered, so the result does not depend
/// on the order the verdicts arrive in (L3) and a set holding an `N/A` never reads as all `OK`.
pub fn worst(verdicts: impl IntoIterator<Item = Verdict>) -> Option<Verdict> {
    verdicts
        .into_iter()
        .max_by_key(|&verdict| (rank(verdict), verdict == Verdict::NA))
}

/// The exit code of a pass: its worst verdict's [`exit`], except that a pass in which every
/// verdict is `N/A`, or which graded nothing, exits [`EXIT_UNKNOWN`]: having read nothing
/// gradable is not a pass.
pub fn pass_exit(verdicts: impl IntoIterator<Item = Verdict>) -> i32 {
    let mut graded = false;
    let mut worst_seen: Option<Verdict> = None;
    for verdict in verdicts {
        graded |= verdict != Verdict::NA;
        worst_seen = worst(worst_seen.into_iter().chain([verdict]));
    }
    match worst_seen {
        Some(verdict) if graded => exit(verdict),
        _ => EXIT_UNKNOWN,
    }
}
