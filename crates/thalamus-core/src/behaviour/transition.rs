//! The transition obligations of the synthesis plan (`generated/thalamus-model/PLAN.md`): the
//! command `thalamus.transition.RecordTransition` and the view query
//! `thalamus.transition.SeriesTransitions`.
//!
//! Each reads and writes only through the generated storage ports: no I/O and no clock of its
//! own. The command selects in the plan's precedence for a command with `when_related:`: the
//! related definition missing, then the accepting branch. Whether k evaluations agreed is the
//! hysteresis' decision (design § 3.2), made before it records; this command records what it is
//! given.

use thalamus_model::behaviour::{SignalDefinitionStorage, TransitionStorage};
use thalamus_model::obligation::UnmetObligation;
use thalamus_model::observation::SeriesKey;
use thalamus_model::transition::{
    AnyTransition, RecordTransition, RecordTransitionOutcome, RecordTransitionResponse,
    SeriesTransitions, Transition, TransitionData, TransitionId, TransitionRecorded, UnknownSignal,
};

use super::observation::{IdSequence, in_time_order, mint};

/// `thalamus.transition.RecordTransition`: `unknown-signal`, then `recorded` under a fresh `trn_`
/// id, which the response returns and the event carries. The evaluations it names are referenced,
/// not read: one that has left raw retention reads as expired (design § 11 D13).
///
/// # Errors
///
/// [`UnmetObligation`] naming the identity sequence once it is spent, and naming the entity
/// invariant the new transition would break, which no declared outcome refuses: a
/// `definition_rev` below 1, a `first_ts` after `settled_ts` (compared as instants), or a
/// `from_verdict` equal to `to_verdict` (design § 11 D29).
pub fn record_transition<P>(
    ports: &mut P,
    input: RecordTransition,
) -> Result<RecordTransitionOutcome, UnmetObligation>
where
    P: SignalDefinitionStorage + TransitionStorage + IdSequence + ?Sized,
{
    if SignalDefinitionStorage::get(ports, &input.signal).is_none() {
        return Ok(RecordTransitionOutcome::UnknownSignal {
            error: UnknownSignal {
                signal: input.signal,
            },
        });
    }
    let transition_id = TransitionId(mint(ports, "trn_", |ports, id| {
        TransitionStorage::get(ports, &TransitionId(id.to_owned())).is_some()
    })?);
    let data = TransitionData {
        transition_id: transition_id.clone(),
        signal: input.signal,
        series: input.series,
        dims: input.dims,
        definition_rev: input.definition_rev,
        from_verdict: input.from_verdict,
        to_verdict: input.to_verdict,
        first_ts: input.first_ts,
        settled_ts: input.settled_ts,
        settling_score: input.settling_score,
        evaluations: input.evaluations,
    };
    if let Some(broken) = data.broken_invariant() {
        return Err(UnmetObligation {
            capability: "entity invariant",
            source: broken,
        });
    }
    let answer = RecordTransitionOutcome::Recorded {
        response: RecordTransitionResponse {
            transition_id: transition_id.clone(),
        },
        transition_recorded: TransitionRecorded {
            transition_id,
            signal: data.signal.clone(),
            series: data.series.clone(),
            from_verdict: data.from_verdict,
            to_verdict: data.to_verdict,
            settled_ts: data.settled_ts.clone(),
        },
    };
    TransitionStorage::put(
        ports,
        AnyTransition::Recorded(Transition::new(data)).snapshot(),
    );
    Ok(answer)
}

/// `thalamus.transition.SeriesTransitions`: the transitions whose `series` is `series`, ordered
/// by `settled_ts` ascending as the view declares (`order_by: [settled_ts asc]`). Two settled at
/// one instant keep the store's order.
///
/// # Errors
///
/// None today. The `Result` is the obligation's contract.
pub fn series_transitions<P>(
    ports: &P,
    series: &SeriesKey,
) -> Result<Vec<SeriesTransitions>, UnmetObligation>
where
    P: TransitionStorage + ?Sized,
{
    let rows: Vec<SeriesTransitions> = TransitionStorage::list(ports)
        .into_iter()
        .filter(|transition| transition.data.series == *series)
        .map(|transition| SeriesTransitions {
            transition_id: transition.data.transition_id,
            signal: transition.data.signal,
            definition_rev: transition.data.definition_rev,
            from_verdict: transition.data.from_verdict,
            to_verdict: transition.data.to_verdict,
            first_ts: transition.data.first_ts,
            settled_ts: transition.data.settled_ts,
            settling_score: transition.data.settling_score,
            evaluations: transition.data.evaluations,
        })
        .collect();
    Ok(in_time_order(rows, |row| row.settled_ts.0.as_str()))
}
