//! Adversary pass 1 on `story:m2-hysteresis-core` (wave 20261015e).
//!
//! What the unit's own suite does not say: a sample graded at two definition revisions (design
//! § 11 D5, § 5.1 "re-grades history after a threshold change"), an evaluation observed twice
//! (a restart seeding by replay, then going on live, `story:m2-transitions`), input that is not in
//! `ts` order (`thalamus.evaluation.SeriesVerdicts` declares no order), and the invariants a
//! settled change must satisfy to become a `Transition` (design § 11 D29: `first_ts <=
//! settled_ts`, `from` absent or different from `to`).

use std::collections::BTreeMap;

use thalamus_core::hysteresis::{Persistence, SeriesHysteresis, SettledChange, replay};
use thalamus_core::registry::{Definition, Registry};
use thalamus_model::definition::{Persistence as DeclaredPersistence, SignalId};
use thalamus_model::evaluation::{EvaluationId, SignalEvaluationData, Verdict};
use thalamus_model::observation::{SampleId, SeriesKey};
use thalamus_model::primitives::{Decimal, Timestamp};

use Verdict::{CRIT, NA, OK, UNKNOWN, WARN};

const REGISTRY: &str = include_str!("fixtures/instance/dimensions.yaml");
const QUEUE: &str = include_str!("fixtures/instance/definitions/service.queue.depth.yaml");

fn ts(index: u32) -> Timestamp {
    Timestamp(format!(
        "2026-01-01T{:02}:{:02}:00Z",
        index / 60 % 24,
        index % 60
    ))
}

/// The evaluation of sample `sample` at definition revision `rev`, read at `ts(sample)`.
fn graded(sample: u32, rev: i64, verdict: Verdict) -> SignalEvaluationData {
    let scored = matches!(verdict, OK | WARN | CRIT);
    SignalEvaluationData {
        evaluation_id: EvaluationId(format!("evl_{sample}_r{rev}")),
        sample_id: SampleId(format!("smp_{sample}")),
        signal: SignalId("service.queue.depth".to_owned()),
        series: SeriesKey("service.queue.depth".to_owned()),
        dims: BTreeMap::new(),
        ts: ts(sample),
        definition_rev: rev,
        verdict,
        prev: None,
        score: scored.then(|| Decimal("0.5".to_owned())),
        baseline: None,
        reason: (verdict != OK).then(|| "case".to_owned()),
    }
}

/// D5 and § 5.1: a threshold change re-grades history, and the re-grade is a second evaluation
/// of the same sample, at the same `ts`. Persistence is "k consecutive evaluations agree"
/// (§ 3.2, Prometheus' `for:`); one reading graded twice is one reading, as D14 counts a sample
/// once at its highest revision. With `k = 2` one `WARN` sample must not settle `WARN`.
#[test]
fn one_sample_graded_at_two_revisions_does_not_settle_alone() {
    let evaluations = [graded(1, 1, WARN), graded(1, 2, WARN)];
    let changes = replay(Persistence::default(), &evaluations);
    assert_eq!(
        changes,
        Vec::<SettledChange>::new(),
        "one sample, re-graded, settled a change with k = 2"
    );
}

/// The same evaluation observed twice (a restart that replays stored evaluations and then sees
/// the last one again live) must not count twice: a transition references "the k evaluations
/// that settled it" (D13), not one evaluation k times.
#[test]
fn the_same_evaluation_observed_twice_counts_once() {
    let mut hysteresis = SeriesHysteresis::new(Persistence::default());
    let once = graded(1, 1, WARN);
    assert_eq!(hysteresis.observe(&once), None);
    let again = hysteresis.observe(&once);
    assert_eq!(
        again, None,
        "the same evaluation id counted twice toward k = 2"
    );
}

/// D29: a `Transition` holds `first_ts <= settled_ts`. `SeriesVerdicts` serves a series "in the
/// store's order" and declares no order, and a re-grade of an older sample is appended after newer
/// ones; whatever order the evaluations arrive in, a settled change must be recordable.
#[test]
fn a_settled_change_always_satisfies_the_transition_invariant() {
    let mut hysteresis = SeriesHysteresis::new(Persistence::default());
    assert_eq!(hysteresis.observe(&graded(5, 1, WARN)), None);
    if let Some(change) = hysteresis.observe(&graded(4, 1, WARN)) {
        assert!(
            change.first_ts <= change.settled_ts,
            "first_ts {:?} after settled_ts {:?}: RecordTransition refuses this",
            change.first_ts,
            change.settled_ts
        );
    }
}

/// `Persistence` has public fields, so a caller can build one with 0; `run_for` reads a value
/// below 1 as 1 ("a change never settles on no evaluation"). No test of the unit states it.
#[test]
fn a_persistence_of_zero_settles_on_one_evaluation() {
    let zero = Persistence {
        k: 0,
        k_crit: 0,
        m: 0,
    };
    for verdict in [OK, NA, WARN, CRIT, UNKNOWN] {
        let changes = replay(zero, &[graded(1, 1, verdict)]);
        assert_eq!(
            changes.iter().map(|change| change.to).collect::<Vec<_>>(),
            [verdict],
            "{verdict:?} with persistence 0"
        );
        assert_eq!(
            changes[0].evaluations,
            [EvaluationId("evl_1_r1".to_owned())]
        );
    }
}

/// `Persistence::of` reads a declared value below 1 as 1 (its doc comment). The unit's suite
/// loads only valid blocks, so the clamp is untested.
#[test]
fn persistence_of_reads_values_below_one_as_one() {
    let registry = Registry::parse("dimensions.yaml", REGISTRY).expect("the fixture registry");
    let definition = Definition::parse("definitions/service.queue.depth.yaml", QUEUE, &registry)
        .expect("the fixture definition loads");
    let mut data = definition.data().clone();
    data.persistence = Some(DeclaredPersistence {
        k: 0,
        k_crit: -1,
        m: i64::MIN,
    });
    assert_eq!(
        Persistence::of(&data),
        Persistence {
            k: 1,
            k_crit: 1,
            m: 1
        }
    );
}

/// `CRIT` clears only on `m` `OK`, never on `k`: with `k = 2` and `m = 4`, three `OK` do not
/// clear a `CRIT`, the fourth does. A `CRIT` steps down to `WARN` after `k` `WARN`.
#[test]
fn crit_clears_after_m_ok_and_steps_down_after_k_warn() {
    let persistence = Persistence {
        k: 2,
        k_crit: 1,
        m: 4,
    };
    let evaluations: Vec<SignalEvaluationData> = [CRIT, OK, OK, OK, OK, CRIT, WARN, WARN]
        .iter()
        .zip(1..)
        .map(|(&verdict, sample)| graded(sample, 1, verdict))
        .collect();
    let changes: Vec<(Option<Verdict>, Verdict, Timestamp)> = replay(persistence, &evaluations)
        .into_iter()
        .map(|change| (change.from, change.to, change.settled_ts))
        .collect();
    assert_eq!(
        changes,
        [
            (None, CRIT, ts(1)),
            (Some(CRIT), OK, ts(5)),
            (Some(OK), CRIT, ts(6)),
            (Some(CRIT), WARN, ts(8)),
        ]
    );
}

/// A small deterministic generator (fixed seed), so the property below is reproducible.
struct Lcg(u64);

impl Lcg {
    fn next(&mut self, bound: u64) -> u64 {
        self.0 = self
            .0
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        (self.0 >> 33) % bound
    }
}

/// An oracle written from the story's four rules, not from the implementation: at each
/// evaluation, the settled verdict moves to V when the last n verdicts are all V, V is not
/// already settled, and V is not `N/A` or `UNKNOWN` over a settled `WARN` or `CRIT`.
fn oracle(
    persistence: Persistence,
    verdicts: &[Verdict],
) -> Vec<(usize, Option<Verdict>, Verdict)> {
    let mut settled = None;
    let mut out = Vec::new();
    for (index, &verdict) in verdicts.iter().enumerate() {
        let needed = match verdict {
            CRIT => persistence.k_crit,
            OK => persistence.m,
            WARN | UNKNOWN | NA => persistence.k,
        }
        .max(1);
        if index + 1 < needed {
            continue;
        }
        let agree = verdicts[index + 1 - needed..=index]
            .iter()
            .all(|&other| other == verdict);
        let blocked = matches!(settled, Some(WARN | CRIT)) && matches!(verdict, NA | UNKNOWN);
        if agree && settled != Some(verdict) && !blocked {
            out.push((index, settled, verdict));
            settled = Some(verdict);
        }
    }
    out
}

/// Over 2000 seeded series: the implementation agrees with the oracle, and every settled change
/// satisfies the `Transition` invariants (D29) and D13's shape: `from != to`, `first_ts <=
/// settled_ts`, exactly n evaluations, the last of which settled it.
#[test]
fn settled_changes_match_the_rules_on_seeded_series() {
    let ladder = [OK, NA, WARN, CRIT, UNKNOWN];
    let mut random = Lcg(20_261_015);
    for case in 0..2000 {
        let persistence = Persistence {
            k: usize::try_from(random.next(4)).expect("small") + 1,
            k_crit: usize::try_from(random.next(4)).expect("small") + 1,
            m: usize::try_from(random.next(4)).expect("small") + 1,
        };
        let length = random.next(40) + 1;
        let verdicts: Vec<Verdict> = (0..length)
            .map(|_| ladder[usize::try_from(random.next(5)).expect("small")])
            .collect();
        let evaluations: Vec<SignalEvaluationData> = verdicts
            .iter()
            .zip(1..)
            .map(|(&verdict, sample)| graded(sample, 1, verdict))
            .collect();

        let mut hysteresis = SeriesHysteresis::new(persistence);
        let mut got = Vec::new();
        for (index, evaluation) in evaluations.iter().enumerate() {
            if let Some(change) = hysteresis.observe(evaluation) {
                let needed = persistence.run_for(change.to);
                assert_ne!(change.from, Some(change.to), "case {case}: from == to");
                assert!(change.first_ts <= change.settled_ts, "case {case}");
                assert_eq!(change.settled_ts, evaluation.ts, "case {case}");
                assert_eq!(change.evaluations.len(), needed, "case {case}");
                assert_eq!(
                    change.evaluations,
                    evaluations[index + 1 - needed..=index]
                        .iter()
                        .map(|evaluation| evaluation.evaluation_id.clone())
                        .collect::<Vec<_>>(),
                    "case {case}"
                );
                assert_eq!(change.first_ts, evaluations[index + 1 - needed].ts);
                got.push((index, change.from, change.to));
            }
        }
        assert_eq!(
            got,
            oracle(persistence, &verdicts),
            "case {case}: {persistence:?} {verdicts:?}"
        );
        assert_eq!(
            hysteresis.settled(),
            got.last().map(|&(_, _, to)| to),
            "case {case}"
        );
    }
}
