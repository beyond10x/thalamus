//! The assertion obligation of the synthesis plan (`generated/thalamus-model/PLAN.md`):
//! `thalamus.assertion.AskAssertion`.
//!
//! It reads and writes only through the generated storage port: no I/O and no clock of its own.
//! Whether every named signal exists and is `Active` (design § 11 D1) is not stated by the
//! specification and is not decided here.

use thalamus_model::assertion::{
    AnyAssertion, AskAssertion, AskAssertionOutcome, AskAssertionResponse, Assertion,
    AssertionAsked, AssertionData, AssertionId, NoSignals,
};
use thalamus_model::behaviour::AssertionStorage;
use thalamus_model::obligation::UnmetObligation;

use super::observation::{IdSequence, mint};

/// `thalamus.assertion.AskAssertion`: `no-signals`, then `asked`: an open assertion in `Asked`
/// with no result, worst verdict or reason, under a fresh `asr_` id.
///
/// # Errors
///
/// [`UnmetObligation`] naming the identity sequence once it is spent, and naming the entity
/// invariant the new assertion would break, which no declared input can reach.
pub fn ask_assertion<P>(
    ports: &mut P,
    input: AskAssertion,
) -> Result<AskAssertionOutcome, UnmetObligation>
where
    P: AssertionStorage + IdSequence + ?Sized,
{
    if input.signals.is_empty() {
        return Ok(AskAssertionOutcome::NoSignals { error: NoSignals });
    }
    let assertion_id = AssertionId(mint(ports, "asr_", |ports, id| {
        AssertionStorage::get(ports, &AssertionId(id.to_owned())).is_some()
    })?);
    let answer = AskAssertionOutcome::Asked {
        response: AskAssertionResponse {
            assertion_id: assertion_id.clone(),
        },
        assertion_asked: AssertionAsked {
            assertion_id: assertion_id.clone(),
            asker: input.asker.clone(),
        },
    };
    let next = AnyAssertion::Asked(Assertion::new(AssertionData {
        assertion_id,
        signals: input.signals,
        dims: input.dims,
        after: input.after,
        horizon: input.horizon,
        within: input.within,
        asker: input.asker,
        result: None,
        worst: None,
        reason: None,
    }))
    .snapshot();
    if let Some(broken) = next.data.broken_invariant() {
        return Err(UnmetObligation {
            capability: "entity invariant",
            source: broken,
        });
    }
    AssertionStorage::put(ports, next);
    Ok(answer)
}
