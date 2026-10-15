//! The definition obligations of the synthesis plan (`generated/thalamus-model/PLAN.md`):
//! `thalamus.definition.ProposeDefinition` and `thalamus.definition.ReviseDefinition`.
//!
//! Each reads and writes only through the generated storage port, and takes the instant it
//! records from the context's command clock: no I/O and no clock of its own. The outcome is
//! selected in the plan's precedence: the input-guarded refusal first, then whether the addressed
//! row exists, then the state it holds, then the accepting branch.

use thalamus_model::behaviour::{SignalDefinitionStorage, TryContext};
use thalamus_model::definition::{
    AnySignalDefinition, BaselineKind, DefinitionExists, DefinitionProposed, DefinitionRevised,
    EvaluationSpec, InstantNeedsFixedBaseline, ProposeDefinition, ProposeDefinitionOutcome,
    RetiredDefinition, ReviseDefinition, ReviseDefinitionOutcome, SignalDefinition,
    SignalDefinitionData, SignalDefinitionState, SignalKind, UnknownDefinition,
};
use thalamus_model::obligation::UnmetObligation;
use thalamus_model::primitives::Timestamp;

const PROPOSE: &str = "thalamus.definition.ProposeDefinition";
const REVISE: &str = "thalamus.definition.ReviseDefinition";

/// `thalamus.definition.ProposeDefinition`: `instant-not-fixed`, then `already-defined`, then
/// `proposed` at revision 1, stamped with the command clock and the input's author.
///
/// # Errors
///
/// [`UnmetObligation`] naming the command clock when the outcome is `proposed` and the context
/// has no instant to stamp it with, and naming the entity invariant the new definition would
/// break, which no declared input can reach.
pub fn propose_definition<P>(
    ports: &mut P,
    input: ProposeDefinition,
) -> Result<ProposeDefinitionOutcome, UnmetObligation>
where
    P: SignalDefinitionStorage + TryContext + ?Sized,
{
    let now = ports.try_command_clock();
    if instant_not_fixed(input.kind, &input.evaluation) {
        return Ok(ProposeDefinitionOutcome::InstantNotFixed {
            error: InstantNeedsFixedBaseline {
                signal_id: input.signal_id,
            },
        });
    }
    if SignalDefinitionStorage::get(ports, &input.signal_id).is_some() {
        return Ok(ProposeDefinitionOutcome::AlreadyDefined {
            error: DefinitionExists {
                signal_id: input.signal_id,
            },
        });
    }
    let revised_at = instant(now, PROPOSE)?;
    let answer = ProposeDefinitionOutcome::Proposed {
        definition_proposed: DefinitionProposed {
            signal_id: input.signal_id.clone(),
            revised_by: input.author.clone(),
        },
    };
    let data = SignalDefinitionData {
        signal_id: input.signal_id,
        title: input.title,
        family: input.family,
        kind: input.kind,
        unit: input.unit,
        dimensions: input.dimensions,
        source: input.source,
        extractor: input.extractor,
        query: input.query,
        window: input.window,
        cadence: input.cadence,
        history: input.history,
        denominator: input.denominator,
        evaluation: input.evaluation,
        objective: input.objective,
        owner: input.owner,
        links: input.links,
        rev: 1,
        revised_at,
        revised_by: input.author,
        mute_reason: None,
        mute_until: None,
    };
    let next = AnySignalDefinition::Proposed(SignalDefinition::new(data)).snapshot();
    if let Some(broken) = next.data.broken_invariant() {
        return Err(UnmetObligation {
            capability: "entity invariant",
            source: broken,
        });
    }
    SignalDefinitionStorage::put(ports, next);
    Ok(answer)
}

/// `thalamus.definition.ReviseDefinition`: `instant-not-fixed`, then `no-such-definition`, then
/// `retired`, then `revised`: the body replaced, `rev` one higher, stamped with the command clock
/// and the input's author. The lifecycle state, and a mute with it, is kept.
///
/// # Errors
///
/// [`UnmetObligation`] naming the command clock when the outcome is `revised` and the context has
/// no instant to stamp it with; naming the command when `rev` has no successor; and naming the
/// entity invariant the revised definition would break, which no declared input can reach.
pub fn revise_definition<P>(
    ports: &mut P,
    input: ReviseDefinition,
) -> Result<ReviseDefinitionOutcome, UnmetObligation>
where
    P: SignalDefinitionStorage + TryContext + ?Sized,
{
    let now = ports.try_command_clock();
    if instant_not_fixed(input.kind, &input.evaluation) {
        return Ok(ReviseDefinitionOutcome::InstantNotFixed {
            error: InstantNeedsFixedBaseline {
                signal_id: input.signal_id,
            },
        });
    }
    let Some(held) = SignalDefinitionStorage::get(ports, &input.signal_id) else {
        return Ok(ReviseDefinitionOutcome::NoSuchDefinition {
            error: UnknownDefinition {
                signal_id: input.signal_id,
            },
        });
    };
    if held.state == SignalDefinitionState::Retired {
        return Ok(ReviseDefinitionOutcome::Retired {
            error: RetiredDefinition {
                signal_id: input.signal_id,
            },
        });
    }
    let previous_rev = held.data.rev;
    let Some(rev) = previous_rev.checked_add(1) else {
        return Err(UnmetObligation {
            capability: "command behaviour",
            source: REVISE,
        });
    };
    let revised_at = instant(now, REVISE)?;
    let answer = ReviseDefinitionOutcome::Revised {
        definition_revised: DefinitionRevised {
            signal_id: input.signal_id.clone(),
            previous_rev,
            revised_by: input.author.clone(),
        },
    };
    let mut next = held;
    next.data = SignalDefinitionData {
        signal_id: input.signal_id,
        title: input.title,
        family: input.family,
        kind: input.kind,
        unit: Some(input.unit),
        dimensions: Some(input.dimensions),
        source: input.source,
        extractor: input.extractor,
        query: input.query,
        window: input.window,
        cadence: input.cadence,
        history: input.history,
        denominator: Some(input.denominator),
        evaluation: input.evaluation,
        objective: input.objective,
        owner: Some(input.owner),
        links: input.links,
        rev,
        revised_at,
        revised_by: input.author,
        mute_reason: next.data.mute_reason,
        mute_until: next.data.mute_until,
    };
    if let Some(broken) = next.data.broken_invariant() {
        return Err(UnmetObligation {
            capability: "entity invariant",
            source: broken,
        });
    }
    SignalDefinitionStorage::put(ports, next);
    Ok(answer)
}

/// The guard of `instant-not-fixed`: `kind == instant and exists b in evaluation.baselines:
/// b != fixed`.
fn instant_not_fixed(kind: SignalKind, evaluation: &EvaluationSpec) -> bool {
    kind == SignalKind::Instant
        && evaluation
            .baselines
            .iter()
            .any(|baseline| *baseline != BaselineKind::Fixed)
}

/// The decision's one instant, which the accepting branch records as `revised_at`: the context's
/// own unavailable answer where it named one, and the missing command clock where it has none.
fn instant(
    now: Result<Option<Timestamp>, UnmetObligation>,
    command: &'static str,
) -> Result<Timestamp, UnmetObligation> {
    now?.ok_or(UnmetObligation {
        capability: "command clock",
        source: command,
    })
}
