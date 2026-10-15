//! Persistence and hysteresis (design § 3.2, § 11 D15): a verdict change of a series settles only
//! once enough consecutive samples agree, and the settled change carries what a transition
//! records (design § 11 D13).
//!
//! The rules:
//!
//! 1. The settled verdict moves to V when the newest n samples of the series all read V: n is
//!    `k_crit` for `CRIT`, `m` for `OK` and `k` for `WARN`, `UNKNOWN` and `N/A`.
//! 2. Any differing verdict breaks a run. Time between samples does not: the count is of
//!    consecutive samples, not of cadence slots. A sample reads the verdict of its evaluation at
//!    the highest definition revision seen (D5, D14).
//! 3. `N/A` and `UNKNOWN` never clear a settled `WARN` or `CRIT`; only `OK` does.
//! 4. The first settled verdict of a series is a change with no `from`.
//!
//! `prev` on an evaluation stays the raw previous verdict (D6). Pure: the service, a replay of
//! stored history and the flap measurement run this one implementation.

use thalamus_model::definition::SignalDefinitionData;
use thalamus_model::evaluation::{EvaluationId, SignalEvaluationData, Verdict};
use thalamus_model::observation::SampleId;
use thalamus_model::primitives::{Decimal, Timestamp};

use crate::behaviour::observation::Instant;

/// How many consecutive samples must agree before a verdict change settles (D15).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Persistence {
    /// Samples before a change to `WARN`, `UNKNOWN` or `N/A` settles.
    pub k: usize,
    /// Samples before a change to `CRIT` settles.
    pub k_crit: usize,
    /// `OK` samples before a settled verdict clears.
    pub m: usize,
}

impl Persistence {
    /// The design defaults, applied when a definition declares no `persistence` block: `k` 2,
    /// `k_crit` 1, `m` 3.
    pub const DEFAULT: Self = Self {
        k: 2,
        k_crit: 1,
        m: 3,
    };

    /// The persistence of `definition`: its `persistence` block, which declares all three values
    /// (D25), or [`Persistence::DEFAULT`] when it declares none. A value below 1, which the
    /// loader refuses, is read as 1: a change never settles on no evaluation.
    #[must_use]
    pub fn of(definition: &SignalDefinitionData) -> Self {
        let count = |value: i64| usize::try_from(value).map_or(1, |value| value.max(1));
        definition
            .persistence
            .as_ref()
            .map_or(Self::DEFAULT, |declared| Self {
                k: count(declared.k),
                k_crit: count(declared.k_crit),
                m: count(declared.m),
            })
    }

    /// The consecutive evaluations of `verdict` that settle a change to it (rule 1).
    #[must_use]
    pub fn run_for(self, verdict: Verdict) -> usize {
        match verdict {
            Verdict::CRIT => self.k_crit,
            Verdict::OK => self.m,
            Verdict::WARN | Verdict::UNKNOWN | Verdict::NA => self.k,
        }
        .max(1)
    }
}

impl Default for Persistence {
    fn default() -> Self {
        Self::DEFAULT
    }
}

/// One settled verdict change of a series: what a transition records (D13), but the series and
/// the transition id, which the recorder holds.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SettledChange {
    /// The previous settled verdict; absent on the series' first settled verdict.
    pub from: Option<Verdict>,
    /// The verdict the series settled on.
    pub to: Verdict,
    /// The `ts` of the oldest of the agreeing samples.
    pub first_ts: Timestamp,
    /// The `ts` of the settling evaluation: the one that completed the run, a re-grade of an
    /// older sample included (D13).
    pub settled_ts: Timestamp,
    /// The settling evaluation's score; absent when its verdict carries none, never 0.
    pub settling_score: Option<Decimal>,
    /// The settling evaluation's definition revision.
    pub definition_rev: i64,
    /// The agreeing samples' evaluations, one per sample, oldest first.
    pub evaluations: Vec<EvaluationId>,
}

/// One sample of the window, read through its evaluation at the highest revision seen.
#[derive(Debug, Clone)]
struct Sample {
    sample: SampleId,
    /// The sample's `ts`, as its first evaluation gave it.
    ts: Timestamp,
    evaluation: EvaluationId,
    rev: i64,
    verdict: Verdict,
    score: Option<Decimal>,
}

impl Sample {
    fn read(evaluation: &SignalEvaluationData) -> Self {
        Self {
            sample: evaluation.sample_id.clone(),
            ts: evaluation.ts.clone(),
            evaluation: evaluation.evaluation_id.clone(),
            rev: evaluation.definition_rev,
            verdict: evaluation.verdict,
            score: evaluation.score.clone(),
        }
    }
}

/// The hysteresis of one series, fed its evaluations in the order they were minted.
///
/// It keeps the newest `max(k, k_crit, m)` samples of the series (the window), each at the
/// highest `definition_rev` seen (design § 11 D5, D14). The run is the longest suffix of the
/// window that reads one verdict, and a change settles on it. What an evaluation does:
///
/// - **Of a sample in the window** it is a re-grade. At a higher revision it replaces that
///   sample's verdict in place and drops no other sample; at a revision no higher it is ignored.
/// - **Of a new sample** it joins the window as its newest sample, and the oldest leaves when the
///   window is full. A sample that has left is forgotten, and a later evaluation of it is ignored.
/// - **Of a new sample older than the newest one taken** (a sample recorded late) it is dropped.
/// - **With a `ts` that is not an RFC 3339 instant** it is ignored.
///
/// An evaluation that is dropped or ignored returns no change and leaves the state as it was.
/// The window's samples are in `ts` order and the settling sample is one of the agreeing ones,
/// so every change holds the `Transition` invariant `first_ts <= settled_ts`.
#[derive(Debug, Clone)]
pub struct SeriesHysteresis {
    persistence: Persistence,
    settled: Option<Verdict>,
    /// Oldest first.
    window: Vec<Sample>,
    /// The newest `ts` taken, and every sample taken at it, those that left the window included.
    newest: Option<(Instant, Vec<SampleId>)>,
}

impl SeriesHysteresis {
    /// A series with no evaluations yet, so no settled verdict.
    #[must_use]
    pub fn new(persistence: Persistence) -> Self {
        Self {
            persistence,
            settled: None,
            window: Vec::new(),
            newest: None,
        }
    }

    /// A series resumed from the settled verdict last recorded for it, the `to_verdict` of its
    /// newest transition: the first change after it carries that verdict as `from`.
    ///
    /// Precondition: the caller feeds only evaluations of samples after that transition's
    /// `settled_ts`. Earlier ones would be counted again and could settle a change already
    /// recorded.
    #[must_use]
    pub fn resume(persistence: Persistence, settled: Verdict) -> Self {
        Self {
            settled: Some(settled),
            ..Self::new(persistence)
        }
    }

    /// Applies `persistence` from now on, as after a definition revision that changed it, and
    /// returns the change the current run settles under it, if any. The settled verdict and the
    /// window are kept; a smaller window lets its oldest samples go.
    pub fn set_persistence(&mut self, persistence: Persistence) -> Option<SettledChange> {
        self.persistence = persistence;
        self.trim();
        self.settle(self.window.len().checked_sub(1)?)
    }

    /// The series' settled verdict; `None` until one has settled.
    #[must_use]
    pub fn settled(&self) -> Option<Verdict> {
        self.settled
    }

    /// Takes the series' next evaluation and returns the change it settles, if any.
    pub fn observe(&mut self, evaluation: &SignalEvaluationData) -> Option<SettledChange> {
        let at = Instant::parse(&evaluation.ts.0)?;
        if let Some(index) = self
            .window
            .iter()
            .position(|sample| sample.sample == evaluation.sample_id)
        {
            let sample = &mut self.window[index];
            if evaluation.definition_rev <= sample.rev {
                return None;
            }
            *sample = Sample {
                ts: sample.ts.clone(),
                ..Sample::read(evaluation)
            };
            return self.settle(index);
        }
        match &mut self.newest {
            Some((newest, _)) if at < *newest => return None,
            Some((newest, taken)) if at == *newest => {
                if taken.contains(&evaluation.sample_id) {
                    return None;
                }
                taken.push(evaluation.sample_id.clone());
            }
            newest => *newest = Some((at, vec![evaluation.sample_id.clone()])),
        }
        self.window.push(Sample::read(evaluation));
        self.trim();
        self.settle(self.window.len() - 1)
    }

    /// The window bound: `max(k, k_crit, m)`, at least 1.
    fn trim(&mut self) {
        let bound = [Verdict::WARN, Verdict::CRIT, Verdict::OK]
            .into_iter()
            .map(|verdict| self.persistence.run_for(verdict))
            .max()
            .unwrap_or(1);
        let excess = self.window.len().saturating_sub(bound);
        self.window.drain(..excess);
    }

    /// The change the run, the longest suffix of the window that reads one verdict, settles at
    /// the window's sample `settling`, the one whose evaluation was just taken (the newest one
    /// after `set_persistence`).
    ///
    /// The settling sample is among the agreeing ones: a run that already held `needed` samples
    /// after it had settled, or was blocked, before it arrived. So `first_ts <= settled_ts`.
    fn settle(&mut self, settling: usize) -> Option<SettledChange> {
        let verdict = self.window.last()?.verdict;
        let run = self
            .window
            .iter()
            .rev()
            .take_while(|sample| sample.verdict == verdict)
            .count();
        let needed = self.persistence.run_for(verdict);
        let blocked = matches!(self.settled, Some(Verdict::WARN | Verdict::CRIT))
            && matches!(verdict, Verdict::NA | Verdict::UNKNOWN);
        if blocked || self.settled == Some(verdict) || run < needed {
            return None;
        }
        let agreeing = &self.window[self.window.len() - needed..];
        let settling = &self.window[settling];
        let change = SettledChange {
            from: self.settled,
            to: verdict,
            first_ts: agreeing[0].ts.clone(),
            settled_ts: settling.ts.clone(),
            settling_score: settling.score.clone(),
            definition_rev: settling.rev,
            evaluations: agreeing
                .iter()
                .map(|sample| sample.evaluation.clone())
                .collect(),
        };
        self.settled = Some(verdict);
        Some(change)
    }
}

/// Every change the stored `evaluations` of one series settle, read in the order given: what a
/// [`SeriesHysteresis`] fed them one at a time returns.
///
/// `replay` does not sort. The caller passes the evaluations in minting order, the order the
/// live service took them: minted evaluation ids are `evl_` and 12 digits, so that is id order,
/// the order the store lists them in. Replay then drops the same late samples live dropped and
/// agrees with it.
pub fn replay<'a>(
    persistence: Persistence,
    evaluations: impl IntoIterator<Item = &'a SignalEvaluationData>,
) -> Vec<SettledChange> {
    let mut hysteresis = SeriesHysteresis::new(persistence);
    evaluations
        .into_iter()
        .filter_map(|evaluation| hysteresis.observe(evaluation))
        .collect()
}
