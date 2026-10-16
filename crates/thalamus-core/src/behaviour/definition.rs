//! The definition obligations of the synthesis plan (`generated/thalamus-model/PLAN.md`):
//! `thalamus.definition.ProposeDefinition` and `thalamus.definition.ReviseDefinition`.
//!
//! Each reads and writes only through the generated storage port, and takes the instant it
//! records from the context's command clock: no I/O and no clock of its own. The outcome is
//! selected in the plan's precedence: the input-guarded refusal first, then whether the addressed
//! row exists, then the state it holds, then the accepting branch.

use thalamus_model::behaviour::{SignalDefinitionStorage, TryContext};
use thalamus_model::definition::{
    AdvisoryNeedsCountOfCases, AdvisoryNeedsFindingUnit, AdvisoryNeedsFixedBaseline,
    AnySignalDefinition, BaselineKind, DefinitionExists, DefinitionProposed, DefinitionRevised,
    EvaluationSpec, FixedNumberMissing, FixedNumberUnused, HostConnectionNotLocal,
    InstantNeedsFixedBaseline, LocalConnectionNotHost, MappingMissing, MappingUnused, Persistence,
    PersistenceTooSmall, ProposeDefinition, ProposeDefinitionOutcome, RetiredDefinition,
    ReviseDefinition, ReviseDefinitionOutcome, RollingWindowMissing, RollingWindowUnused,
    SignalDefinition, SignalDefinitionData, SignalDefinitionState, SignalKind, Source,
    ThresholdsUnused, Unit, UnknownDefinition,
};
use thalamus_model::obligation::UnmetObligation;
use thalamus_model::primitives::Timestamp;

const PROPOSE: &str = "thalamus.definition.ProposeDefinition";
const REVISE: &str = "thalamus.definition.ReviseDefinition";

/// The adapter id of the local host collector, the one source that reads this machine's own
/// kernel counters and no Connectors operation (design § 11 D46).
pub const HOST_ADAPTER: &str = "host";

/// The connection id reserved for [`HOST_ADAPTER`]: the machine thalamus runs on, and no other.
pub const LOCAL_CONNECTION: &str = "local";

/// `thalamus.definition.ProposeDefinition`: the input-guarded refusals of [`input_refusal`], then
/// `already-defined`, then `proposed` at revision 1, stamped with the command clock and the
/// input's author.
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
    if let Some(refused) = input_refusal(
        input.kind,
        input.unit.as_ref(),
        &input.source,
        &input.evaluation,
        input.persistence.as_ref(),
    ) {
        let signal_id = input.signal_id;
        return Ok(match refused {
            InputRefusal::HostNotLocal => ProposeDefinitionOutcome::HostNotLocal {
                error: HostConnectionNotLocal { signal_id },
            },
            InputRefusal::LocalNotHost => ProposeDefinitionOutcome::LocalNotHost {
                error: LocalConnectionNotHost { signal_id },
            },
            InputRefusal::InstantNotFixed => ProposeDefinitionOutcome::InstantNotFixed {
                error: InstantNeedsFixedBaseline { signal_id },
            },
            InputRefusal::RollingNMissing => ProposeDefinitionOutcome::RollingNMissing {
                error: RollingWindowMissing { signal_id },
            },
            InputRefusal::RollingNUnused => ProposeDefinitionOutcome::RollingNUnused {
                error: RollingWindowUnused { signal_id },
            },
            InputRefusal::FixedNumberMissing => ProposeDefinitionOutcome::FixedNumberMissing {
                error: FixedNumberMissing { signal_id },
            },
            InputRefusal::FixedNumberUnused => ProposeDefinitionOutcome::FixedNumberUnused {
                error: FixedNumberUnused { signal_id },
            },
            InputRefusal::PersistenceBelowOne => ProposeDefinitionOutcome::PersistenceBelowOne {
                error: PersistenceTooSmall { signal_id },
            },
            InputRefusal::MappingMissing => ProposeDefinitionOutcome::MappingMissing {
                error: MappingMissing { signal_id },
            },
            InputRefusal::MappingUnused => ProposeDefinitionOutcome::MappingUnused {
                error: MappingUnused { signal_id },
            },
            InputRefusal::ThresholdsUnused => ProposeDefinitionOutcome::ThresholdsUnused {
                error: ThresholdsUnused { signal_id },
            },
            InputRefusal::AdvisoryNotCount => ProposeDefinitionOutcome::AdvisoryNotCount {
                error: AdvisoryNeedsCountOfCases { signal_id },
            },
            InputRefusal::AdvisoryNotFindingUnit => {
                ProposeDefinitionOutcome::AdvisoryNotFindingUnit {
                    error: AdvisoryNeedsFindingUnit { signal_id },
                }
            }
            InputRefusal::AdvisoryNotFixed => ProposeDefinitionOutcome::AdvisoryNotFixed {
                error: AdvisoryNeedsFixedBaseline { signal_id },
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
        persistence: input.persistence,
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

/// `thalamus.definition.ReviseDefinition`: the input-guarded refusals of [`input_refusal`], then
/// `no-such-definition`, then `retired`, then `revised`: the body replaced, `rev` one higher,
/// stamped with the command clock and the input's author. The lifecycle state, and a mute with
/// it, is kept.
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
    if let Some(refused) = input_refusal(
        input.kind,
        Some(&input.unit),
        &input.source,
        &input.evaluation,
        input.persistence.as_ref(),
    ) {
        let signal_id = input.signal_id;
        return Ok(match refused {
            InputRefusal::HostNotLocal => ReviseDefinitionOutcome::HostNotLocal {
                error: HostConnectionNotLocal { signal_id },
            },
            InputRefusal::LocalNotHost => ReviseDefinitionOutcome::LocalNotHost {
                error: LocalConnectionNotHost { signal_id },
            },
            InputRefusal::InstantNotFixed => ReviseDefinitionOutcome::InstantNotFixed {
                error: InstantNeedsFixedBaseline { signal_id },
            },
            InputRefusal::RollingNMissing => ReviseDefinitionOutcome::RollingNMissing {
                error: RollingWindowMissing { signal_id },
            },
            InputRefusal::RollingNUnused => ReviseDefinitionOutcome::RollingNUnused {
                error: RollingWindowUnused { signal_id },
            },
            InputRefusal::FixedNumberMissing => ReviseDefinitionOutcome::FixedNumberMissing {
                error: FixedNumberMissing { signal_id },
            },
            InputRefusal::FixedNumberUnused => ReviseDefinitionOutcome::FixedNumberUnused {
                error: FixedNumberUnused { signal_id },
            },
            InputRefusal::PersistenceBelowOne => ReviseDefinitionOutcome::PersistenceBelowOne {
                error: PersistenceTooSmall { signal_id },
            },
            InputRefusal::MappingMissing => ReviseDefinitionOutcome::MappingMissing {
                error: MappingMissing { signal_id },
            },
            InputRefusal::MappingUnused => ReviseDefinitionOutcome::MappingUnused {
                error: MappingUnused { signal_id },
            },
            InputRefusal::ThresholdsUnused => ReviseDefinitionOutcome::ThresholdsUnused {
                error: ThresholdsUnused { signal_id },
            },
            InputRefusal::AdvisoryNotCount => ReviseDefinitionOutcome::AdvisoryNotCount {
                error: AdvisoryNeedsCountOfCases { signal_id },
            },
            InputRefusal::AdvisoryNotFindingUnit => {
                ReviseDefinitionOutcome::AdvisoryNotFindingUnit {
                    error: AdvisoryNeedsFindingUnit { signal_id },
                }
            }
            InputRefusal::AdvisoryNotFixed => ReviseDefinitionOutcome::AdvisoryNotFixed {
                error: AdvisoryNeedsFixedBaseline { signal_id },
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
        persistence: input.persistence,
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

/// An input-guarded refusal `ProposeDefinition` and `ReviseDefinition` share, named after its
/// outcome. The definition loader (`crate::registry`) refuses the same.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputRefusal {
    /// `host-not-local`: a host source names a connection other than [`LOCAL_CONNECTION`].
    HostNotLocal,
    /// `local-not-host`: a source of another adapter names the reserved [`LOCAL_CONNECTION`].
    LocalNotHost,
    /// `instant-not-fixed`.
    InstantNotFixed,
    /// `rolling-n-missing`: also an N below 1 (`UNMAPPED:` in `spec/domains/definition.yaml`).
    RollingNMissing,
    /// `rolling-n-unused`.
    RollingNUnused,
    /// `fixed-number-missing`.
    FixedNumberMissing,
    /// `fixed-number-unused`.
    FixedNumberUnused,
    /// `persistence-below-one`.
    PersistenceBelowOne,
    /// `mapping-missing`: also an empty mapping and one naming `N/A` (`UNMAPPED:` in
    /// `spec/domains/definition.yaml`).
    MappingMissing,
    /// `mapping-unused`.
    MappingUnused,
    /// `thresholds-unused`: under `producer_mapped` or `fixed_warn_only`.
    ThresholdsUnused,
    /// `advisory-not-count`.
    AdvisoryNotCount,
    /// `advisory-not-finding-unit`.
    AdvisoryNotFindingUnit,
    /// `advisory-not-fixed`: also an empty baseline list (`UNMAPPED:` in
    /// `spec/domains/definition.yaml`).
    AdvisoryNotFixed,
}

/// The first input-guarded refusal of `ProposeDefinition` and `ReviseDefinition` whose guard
/// holds, in their declaration order, or `None`. See [`input_refusals`] for the guards.
#[must_use]
pub fn input_refusal(
    kind: SignalKind,
    unit: Option<&Unit>,
    source: &Source,
    evaluation: &EvaluationSpec,
    persistence: Option<&Persistence>,
) -> Option<InputRefusal> {
    input_refusals(kind, unit, source, evaluation, persistence)
        .into_iter()
        .next()
}

/// Every input-guarded refusal of `ProposeDefinition` and `ReviseDefinition` whose guard holds,
/// in their declaration order:
///
/// - `host-not-local`: `source.adapter == "host" and source.connection != "local"`;
/// - `local-not-host`: `source.connection == "local" and source.adapter != "host"`;
/// - `instant-not-fixed`: `kind == instant and exists b in evaluation.baselines: b != fixed`;
/// - `rolling-n-missing`: `exists b: b == rolling_median and not defined(evaluation.rolling_n)`,
///   and, beyond what ESS 0.60.0 can state, `rolling_median` with an N below 1 (design § 11 D25);
/// - `rolling-n-unused`: `defined(evaluation.rolling_n) and forall b: b != rolling_median`;
/// - `fixed-number-missing`: `exists b: b == fixed and not defined(evaluation.fixed)`;
/// - `fixed-number-unused`: `defined(evaluation.fixed) and forall b: b != fixed`;
/// - `persistence-below-one`: `persistence.k < 1 or persistence.k_crit < 1 or persistence.m < 1`;
/// - `mapping-missing`: `evaluation.rule == "producer_mapped" and not defined(evaluation.mapping)`,
///   and, beyond what ESS 0.60.0 can state, an empty mapping and one naming `N/A`
///   ([`crate::rules::mapped::mapping_missing`]);
/// - `mapping-unused`: `defined(evaluation.mapping) and evaluation.rule != "producer_mapped"`;
/// - `thresholds-unused`: `(evaluation.rule == "producer_mapped" or evaluation.rule ==
///   "fixed_warn_only") and (defined(evaluation.warn) or defined(evaluation.crit))`;
/// - `advisory-not-count`: `evaluation.rule == "fixed_warn_only" and kind != count_of_cases`
///   ([`crate::rules::warn_only::kind_not_count`]);
/// - `advisory-not-finding-unit`: `evaluation.rule == "fixed_warn_only" and defined(unit) and
///   unit != "{finding}"` ([`crate::rules::warn_only::unit_not_finding`]); a revision always
///   declares its unit;
/// - `advisory-not-fixed`: `evaluation.rule == "fixed_warn_only" and exists b in
///   evaluation.baselines: b != fixed`, and, beyond what ESS 0.60.0 can state, an empty baseline
///   list ([`crate::rules::warn_only::baseline_not_fixed`]).
#[must_use]
pub fn input_refusals(
    kind: SignalKind,
    unit: Option<&Unit>,
    source: &Source,
    evaluation: &EvaluationSpec,
    persistence: Option<&Persistence>,
) -> Vec<InputRefusal> {
    let names = |wanted: BaselineKind| evaluation.baselines.contains(&wanted);
    let rolling = names(BaselineKind::RollingMedian);
    let fixed = names(BaselineKind::Fixed);
    [
        (
            source.adapter.0 == HOST_ADAPTER && source.connection.0 != LOCAL_CONNECTION,
            InputRefusal::HostNotLocal,
        ),
        (
            source.connection.0 == LOCAL_CONNECTION && source.adapter.0 != HOST_ADAPTER,
            InputRefusal::LocalNotHost,
        ),
        (
            kind == SignalKind::Instant
                && evaluation
                    .baselines
                    .iter()
                    .any(|baseline| *baseline != BaselineKind::Fixed),
            InputRefusal::InstantNotFixed,
        ),
        (
            rolling && evaluation.rolling_n.is_none_or(|n| n < 1),
            InputRefusal::RollingNMissing,
        ),
        (
            evaluation.rolling_n.is_some() && !rolling,
            InputRefusal::RollingNUnused,
        ),
        (
            fixed && evaluation.fixed.is_none(),
            InputRefusal::FixedNumberMissing,
        ),
        (
            evaluation.fixed.is_some() && !fixed,
            InputRefusal::FixedNumberUnused,
        ),
        (
            persistence.is_some_and(|held| held.k < 1 || held.k_crit < 1 || held.m < 1),
            InputRefusal::PersistenceBelowOne,
        ),
        (
            crate::rules::mapped::mapping_missing(evaluation),
            InputRefusal::MappingMissing,
        ),
        (
            crate::rules::mapped::mapping_unused(evaluation),
            InputRefusal::MappingUnused,
        ),
        (
            crate::rules::mapped::thresholds_unused(evaluation)
                || crate::rules::warn_only::thresholds_unused(evaluation),
            InputRefusal::ThresholdsUnused,
        ),
        (
            crate::rules::warn_only::kind_not_count(kind, evaluation),
            InputRefusal::AdvisoryNotCount,
        ),
        (
            crate::rules::warn_only::unit_not_finding(unit, evaluation),
            InputRefusal::AdvisoryNotFindingUnit,
        ),
        (
            crate::rules::warn_only::baseline_not_fixed(evaluation),
            InputRefusal::AdvisoryNotFixed,
        ),
    ]
    .into_iter()
    .filter_map(|(holds, refusal)| holds.then_some(refusal))
    .collect()
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
