//! The evaluation obligations of the synthesis plan (`generated/thalamus-model/PLAN.md`): the
//! command `thalamus.evaluation.RecordEvaluation` and the view query
//! `thalamus.evaluation.SeriesVerdicts`.
//!
//! Each reads and writes only through the generated storage ports: no I/O and no clock of its
//! own. The command selects in the plan's precedence for a command with `when_related:`: the
//! related sample missing first, then the input-guarded refusals in declaration order, then the
//! accepting branch.

use thalamus_model::behaviour::{SignalEvaluationStorage, SignalSampleStorage};
use thalamus_model::evaluation::{
    AnySignalEvaluation, EvaluationId, EvaluationRecorded, ReasonRequired, RecordEvaluation,
    RecordEvaluationOutcome, RecordEvaluationResponse, ScoreNotAdmitted, SeriesVerdicts,
    SignalEvaluation, SignalEvaluationData, UnknownSample, Verdict,
};
use thalamus_model::obligation::UnmetObligation;
use thalamus_model::observation::SeriesKey;

use super::observation::{IdSequence, mint};

/// `thalamus.evaluation.RecordEvaluation`: `unknown-sample`, then `reason-missing`, then
/// `score-not-admitted`, then `recorded` under a fresh `evl_` id, with the sample's signal,
/// series, dims and ts copied onto the evaluation and its event.
///
/// # Errors
///
/// [`UnmetObligation`] naming the identity sequence once it is spent, and naming the entity
/// invariant the new evaluation would break, which only a `definition_rev` below 1 reaches: no
/// declared outcome refuses it.
pub fn record_evaluation<P>(
    ports: &mut P,
    input: RecordEvaluation,
) -> Result<RecordEvaluationOutcome, UnmetObligation>
where
    P: SignalSampleStorage + SignalEvaluationStorage + IdSequence + ?Sized,
{
    let Some(sample) = SignalSampleStorage::get(ports, &input.sample_id) else {
        return Ok(RecordEvaluationOutcome::UnknownSample {
            error: UnknownSample {
                sample_id: input.sample_id,
            },
        });
    };
    if input.verdict != Verdict::OK && input.reason.is_none() {
        return Ok(RecordEvaluationOutcome::ReasonMissing {
            error: ReasonRequired {
                verdict: input.verdict,
            },
        });
    }
    if !scored(input.verdict) && input.score.is_some() {
        return Ok(RecordEvaluationOutcome::ScoreNotAdmitted {
            error: ScoreNotAdmitted {
                verdict: input.verdict,
            },
        });
    }
    let evaluation_id = EvaluationId(mint(ports, "evl_", |ports, id| {
        SignalEvaluationStorage::get(ports, &EvaluationId(id.to_owned())).is_some()
    })?);
    let sample = sample.data;
    let data = SignalEvaluationData {
        evaluation_id: evaluation_id.clone(),
        sample_id: input.sample_id,
        signal: sample.signal,
        series: sample.series,
        dims: sample.dims,
        ts: sample.ts,
        definition_rev: input.definition_rev,
        verdict: input.verdict,
        prev: input.prev,
        score: input.score,
        baseline: input.baseline,
        reason: input.reason,
    };
    if let Some(broken) = data.broken_invariant() {
        return Err(UnmetObligation {
            capability: "entity invariant",
            source: broken,
        });
    }
    let answer = RecordEvaluationOutcome::Recorded {
        response: RecordEvaluationResponse {
            evaluation_id: evaluation_id.clone(),
        },
        evaluation_recorded: EvaluationRecorded {
            evaluation_id,
            sample_id: data.sample_id.clone(),
            signal: data.signal.clone(),
            series: data.series.clone(),
            ts: data.ts.clone(),
            verdict: data.verdict,
            prev: data.prev,
        },
    };
    SignalEvaluationStorage::put(
        ports,
        AnySignalEvaluation::Recorded(SignalEvaluation::new(data)).snapshot(),
    );
    Ok(answer)
}

/// `thalamus.evaluation.SeriesVerdicts`: the evaluations whose `series` is `series`, in the
/// store's order. The view declares no order; its current verdict is the row with the newest
/// `ts`, which the reader picks.
///
/// # Errors
///
/// None today. The `Result` is the obligation's contract.
pub fn series_verdicts<P>(
    ports: &P,
    series: &SeriesKey,
) -> Result<Vec<SeriesVerdicts>, UnmetObligation>
where
    P: SignalEvaluationStorage + ?Sized,
{
    Ok(SignalEvaluationStorage::list(ports)
        .into_iter()
        .filter(|evaluation| evaluation.data.series == *series)
        .map(|evaluation| SeriesVerdicts {
            evaluation_id: evaluation.data.evaluation_id,
            signal: evaluation.data.signal,
            ts: evaluation.data.ts,
            definition_rev: evaluation.data.definition_rev,
            verdict: evaluation.data.verdict,
            prev: evaluation.data.prev,
            reason: evaluation.data.reason,
        })
        .collect())
}

/// The verdict's `scored` attribute (`spec/domains/evaluation.yaml`, `thalamus.evaluation.Verdict`):
/// `OK`, `WARN` and `CRIT` carry a score; `N/A` and `UNKNOWN` never do.
fn scored(verdict: Verdict) -> bool {
    match verdict {
        Verdict::OK | Verdict::WARN | Verdict::CRIT => true,
        Verdict::NA | Verdict::UNKNOWN => false,
    }
}
