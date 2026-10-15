//! Adversary pass 2 on `story:m2-hysteresis-core` (wave 20261015e).
//!
//! Pass 1 found that re-grades, duplicates and input out of `ts` order broke the hysteresis. The
//! coordinator's decisions (samples counted once at their highest revision, `observe` ignoring
//! older input, `set_persistence`, `resume`) are now implemented. This pass attacks those
//! decisions: a re-grade over history (the `Grader` "again over history after a definition
//! revision", `spec/domains/evaluation.yaml`), several samples at one `ts` (which
//! `thalamus.observation.RecordSample` admits), a sample recorded late, and the `Transition`
//! invariants (design § 11 D13, D29) over seeded series that mix all of them.

use std::collections::{BTreeMap, BTreeSet};

use thalamus_core::hysteresis::{Persistence, SeriesHysteresis, SettledChange, replay};
use thalamus_model::definition::SignalId;
use thalamus_model::evaluation::{EvaluationId, SignalEvaluationData, Verdict};
use thalamus_model::observation::{SampleId, SeriesKey};
use thalamus_model::primitives::{Decimal, Timestamp};

use Verdict::{CRIT, NA, OK, UNKNOWN, WARN};

fn ts(index: u32) -> Timestamp {
    Timestamp(format!(
        "2026-01-01T{:02}:{:02}:{:02}Z",
        index / 3600 % 24,
        index / 60 % 60,
        index % 60
    ))
}

/// The evaluation of sample `sample`, read at `ts(at)`, graded at definition revision `rev`.
fn graded_at(sample: u32, at: u32, rev: i64, verdict: Verdict) -> SignalEvaluationData {
    let scored = matches!(verdict, OK | WARN | CRIT);
    SignalEvaluationData {
        evaluation_id: EvaluationId(format!("evl_{sample}_r{rev}")),
        sample_id: SampleId(format!("smp_{sample}")),
        signal: SignalId("service.queue.depth".to_owned()),
        series: SeriesKey("service.queue.depth".to_owned()),
        dims: BTreeMap::new(),
        ts: ts(at),
        definition_rev: rev,
        verdict,
        prev: None,
        score: scored.then(|| Decimal("0.5".to_owned())),
        baseline: None,
        reason: (verdict != OK).then(|| "case".to_owned()),
    }
}

/// The evaluation of sample `sample` at revision `rev`, read at `ts(sample)`.
fn graded(sample: u32, rev: i64, verdict: Verdict) -> SignalEvaluationData {
    graded_at(sample, sample, rev, verdict)
}

fn ids(changes: &[SettledChange]) -> Vec<(Option<Verdict>, Verdict, Vec<String>)> {
    changes
        .iter()
        .map(|change| {
            (
                change.from,
                change.to,
                change.evaluations.iter().map(|id| id.0.clone()).collect(),
            )
        })
        .collect()
}

fn live(persistence: Persistence, evaluations: &[SignalEvaluationData]) -> Vec<SettledChange> {
    let mut hysteresis = SeriesHysteresis::new(persistence);
    evaluations
        .iter()
        .filter_map(|evaluation| hysteresis.observe(evaluation))
        .collect()
}

/// Decision 1 (D5, D14): a sample counts once, at its highest revision. A definition revision
/// re-grades history (the `Grader` runs "again over history after a definition revision"), so the
/// store then holds samples 1 and 2 at revision 1 (WARN, OK) and at revision 2 (WARN, WARN).
/// At their highest revision the two samples are WARN, WARN: with `k = 2` that settles WARN.
///
/// The store keeps ids in minting order (`MemorySignalEvaluationStorage::put` sorts by id;
/// `mint` numbers them in sequence), so a replay reads each `ts` tie in the order graded. The
/// re-grade of sample 2 to WARN arrives after its OK broke the run, and "a re-grade that changes
/// the sample's verdict ... starts a run of its new verdict" forgets sample 1.
#[test]
fn a_regraded_history_settles_what_its_highest_revisions_say() {
    let stored = [
        graded(1, 1, WARN),
        graded(1, 2, WARN),
        graded(2, 1, OK),
        graded(2, 2, WARN),
    ];
    assert_eq!(
        ids(&replay(Persistence::default(), &stored)),
        [(
            None,
            WARN,
            vec!["evl_1_r2".to_owned(), "evl_2_r2".to_owned()]
        )],
        "samples 1 and 2 are both WARN at revision 2"
    );
}

/// The same rule live: the newest sample is re-graded after a revision, from OK to WARN. The
/// sample before it is WARN, so the series at the current revision reads WARN, WARN.
#[test]
fn a_regrade_that_restores_the_runs_verdict_keeps_the_run() {
    let changes = live(
        Persistence::default(),
        &[graded(1, 1, WARN), graded(2, 1, OK), graded(2, 2, WARN)],
    );
    assert_eq!(
        ids(&changes),
        [(
            None,
            WARN,
            vec!["evl_1_r1".to_owned(), "evl_2_r2".to_owned()]
        )]
    );
}

/// Decision 3 drops an evaluation older than the last one taken. A sample recorded late
/// (`RecordSample` sets `ts: input.ts` and refuses no `ts` older than the series' newest,
/// `spec/domains/observation.yaml:258`) is then lost live and counted by `replay`, so a restart
/// that replays stored history disagrees with what the live service recorded: here live settles
/// a WARN over a run that sample 2's OK breaks.
#[test]
fn a_late_sample_counts_the_same_live_and_in_replay() {
    let arrival = [graded(1, 1, WARN), graded(3, 1, WARN), graded(2, 1, OK)];
    assert_eq!(
        ids(&live(Persistence::default(), &arrival)),
        ids(&replay(Persistence::default(), &arrival)),
        "live and replay of the same evaluations disagree"
    );
}

/// Decision 1 says a re-grade at a revision no higher than the one counted is ignored. With two
/// samples at one `ts` the revision check only consults the current run: once sample 1 has left
/// it, its stale revision 1 is counted again and settles a WARN that sample 1's revision 2 (OK)
/// contradicts.
#[test]
fn a_stale_revision_is_ignored_after_its_sample_left_the_run() {
    let mut hysteresis = SeriesHysteresis::new(Persistence::default());
    assert_eq!(hysteresis.observe(&graded_at(1, 1, 2, OK)), None);
    assert_eq!(hysteresis.observe(&graded_at(2, 1, 1, WARN)), None);
    assert_eq!(
        hysteresis.observe(&graded_at(1, 1, 1, WARN)),
        None,
        "revision 1 of sample 1 counted after revision 2 was taken"
    );
}

/// Kills the mutants that drop the id check at one `ts` (hysteresis.rs:176) or keep only the last
/// id seen at it (hysteresis.rs:206): the same evaluation observed again after another sample at
/// the same `ts` broke its run must still be ignored.
#[test]
fn an_evaluation_seen_earlier_at_the_same_ts_is_ignored() {
    let mut hysteresis = SeriesHysteresis::new(Persistence::default());
    assert_eq!(hysteresis.observe(&graded_at(1, 1, 1, WARN)), None);
    assert_eq!(hysteresis.observe(&graded_at(2, 1, 1, OK)), None);
    assert_eq!(hysteresis.observe(&graded_at(1, 1, 1, WARN)), None);
    assert_eq!(
        hysteresis.observe(&graded_at(3, 2, 1, WARN)),
        None,
        "sample 1 was counted twice"
    );
}

/// D29: `first_ts <= settled_ts` in the order `RecordTransition` compares `Timestamp`s
/// (`timestamp_order`: two instants by instant, any other pair by bytes). That order is not
/// transitive once a `ts` is not an instant (`behaviour/observation.rs:112`), and `observe`
/// checks each evaluation only against the last one taken.
#[test]
fn a_change_holds_first_ts_before_settled_ts_for_any_timestamp_text() {
    let at = |sample: u32, text: &str| SignalEvaluationData {
        ts: Timestamp(text.to_owned()),
        ..graded(sample, 1, OK)
    };
    let changes = live(
        Persistence::default(),
        &[
            at(1, "2026-01-05T10:00:01+02:00"),
            at(2, "2026-01-05T09:00:03Z"),
            at(3, "2026-01-05T09:x"),
        ],
    );
    for change in &changes {
        // first_ts "…T10…" against settled_ts "…T09:x": not both instants, so by bytes.
        assert!(
            change.first_ts.0.as_bytes() <= change.settled_ts.0.as_bytes(),
            "first_ts {:?} after settled_ts {:?}",
            change.first_ts,
            change.settled_ts
        );
    }
}

/// `resume(p, V)` followed by V never re-emits V, for every verdict.
#[test]
fn resume_then_the_same_verdict_settles_nothing() {
    for verdict in [OK, NA, WARN, CRIT, UNKNOWN] {
        let mut hysteresis = SeriesHysteresis::resume(Persistence::default(), verdict);
        for sample in 1..=8 {
            assert_eq!(
                hysteresis.observe(&graded(sample, 1, verdict)),
                None,
                "resumed at {verdict:?}"
            );
        }
        assert_eq!(hysteresis.settled(), Some(verdict));
    }
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

    fn pick(&mut self, bound: usize) -> usize {
        usize::try_from(self.next(u64::try_from(bound).expect("small"))).expect("small")
    }
}

/// Over 3000 seeded series mixing new samples, samples sharing a `ts`, samples recorded late,
/// re-grades at higher and lower revisions, duplicates and `set_persistence`, every settled change
/// holds the `Transition` invariants (D29) and D13's shape. Each violated invariant is counted,
/// and the first instance of each is printed.
#[test]
#[allow(clippy::too_many_lines)]
fn settled_changes_hold_the_transition_invariants_on_mixed_series() {
    let ladder = [OK, NA, WARN, CRIT, UNKNOWN];
    let mut random = Lcg(202_610_152);
    let mut violations: BTreeMap<&'static str, (usize, String)> = BTreeMap::new();
    let mut changes_seen = 0usize;
    let mut violate = |name: &'static str, detail: String| {
        violations
            .entry(name)
            .and_modify(|(count, _)| *count += 1)
            .or_insert((1, detail));
    };
    for case in 0..3000 {
        let mut persistence = Persistence {
            k: random.pick(4) + 1,
            k_crit: random.pick(4) + 1,
            m: random.pick(4) + 1,
        };
        let mut hysteresis = SeriesHysteresis::new(persistence);
        // sample -> (ts index, revisions graded so far)
        let mut samples: Vec<(u32, BTreeSet<i64>)> = Vec::new();
        let mut handed: Vec<SignalEvaluationData> = Vec::new();
        // sample -> highest revision handed to observe so far
        let mut highest: BTreeMap<String, i64> = BTreeMap::new();
        let mut cursor = 1u32;
        let steps = random.next(40) + 1;
        let mut log = Vec::new();
        for _ in 0..steps {
            let roll = random.next(100);
            let verdict = ladder[random.pick(5)];
            let evaluation = if roll < 5 {
                persistence = Persistence {
                    k: random.pick(4) + 1,
                    k_crit: random.pick(4) + 1,
                    m: random.pick(4) + 1,
                };
                hysteresis.set_persistence(persistence);
                log.push(format!("set {persistence:?}"));
                continue;
            } else if roll < 20 && !samples.is_empty() {
                // a re-grade at a revision this sample has not been graded at, higher or lower
                let index = random.pick(samples.len());
                let free: Vec<i64> = (1..=5)
                    .filter(|rev| !samples[index].1.contains(rev))
                    .collect();
                if free.is_empty() {
                    continue;
                }
                let rev = free[random.pick(free.len())];
                samples[index].1.insert(rev);
                let sample = u32::try_from(index).expect("small") + 1;
                graded_at(sample, samples[index].0, rev, verdict)
            } else if roll < 30 && !handed.is_empty() {
                handed[random.pick(handed.len())].clone()
            } else {
                // a new sample: usually later, sometimes at the same ts, sometimes late
                let shift = random.next(100);
                let at = if shift < 15 {
                    cursor
                } else if shift < 25 {
                    cursor
                        .saturating_sub(u32::try_from(random.next(3)).expect("small") + 1)
                        .max(1)
                } else {
                    cursor += 1;
                    cursor
                };
                samples.push((at, BTreeSet::from([1])));
                let sample = u32::try_from(samples.len()).expect("small");
                graded_at(sample, at, 1, verdict)
            };
            log.push(format!(
                "{} {} r{} {:?}",
                evaluation.evaluation_id.0,
                evaluation.ts.0,
                evaluation.definition_rev,
                evaluation.verdict
            ));
            let earlier_highest = highest.get(&evaluation.sample_id.0).copied();
            handed.push(evaluation.clone());
            let entry = highest.entry(evaluation.sample_id.0.clone()).or_insert(0);
            *entry = (*entry).max(evaluation.definition_rev);
            let Some(change) = hysteresis.observe(&evaluation) else {
                continue;
            };
            changes_seen += 1;
            let detail = || format!("case {case}: {change:?}\n  after {log:?}");
            let by_id = |id: &EvaluationId| {
                handed
                    .iter()
                    .find(|handed| handed.evaluation_id == *id)
                    .expect("a listed evaluation was handed in")
            };
            if change.from == Some(change.to) {
                violate("D29 from != to", detail());
            }
            if change.first_ts.0 > change.settled_ts.0 {
                violate("D29 first_ts <= settled_ts", detail());
            }
            let listed: Vec<&SignalEvaluationData> = change.evaluations.iter().map(by_id).collect();
            if listed.is_empty() {
                violate("D13 at least one evaluation", detail());
                continue;
            }
            let distinct: BTreeSet<&str> = listed.iter().map(|e| e.sample_id.0.as_str()).collect();
            if distinct.len() != listed.len() {
                violate("D13 one evaluation per sample", detail());
            }
            if listed.iter().any(|e| e.verdict != change.to) {
                violate("D13 every listed evaluation has verdict to", detail());
            }
            if listed.windows(2).any(|pair| pair[0].ts.0 > pair[1].ts.0) {
                violate("D13 listed oldest first", detail());
            }
            if change.first_ts != listed[0].ts {
                violate("D13 first_ts is the first listed ts", detail());
            }
            if change.settled_ts != evaluation.ts
                || change.definition_rev != evaluation.definition_rev
                || change.settling_score != evaluation.score
            {
                violate("D13 settled fields copy the settling evaluation", detail());
            }
            if !change.evaluations.contains(&evaluation.evaluation_id) {
                violate("D13 the settling evaluation is listed", detail());
            }
            if listed.len() != persistence.run_for(change.to) {
                violate("rule 1 exactly n evaluations", detail());
            }
            // decision 1: a revision no higher than one already taken for the sample is ignored
            if earlier_highest.is_some_and(|rev| evaluation.definition_rev < rev) {
                violate("decision 1 a stale revision settles nothing", detail());
            }
            if listed.iter().any(|e| {
                handed
                    .iter()
                    .position(|h| h.evaluation_id == e.evaluation_id)
                    .is_some_and(|first| {
                        handed[..first].iter().any(|h| {
                            h.sample_id == e.sample_id && h.definition_rev > e.definition_rev
                        })
                    })
            }) {
                violate(
                    "decision 1 no listed evaluation is a stale revision",
                    detail(),
                );
            }
        }
    }
    assert!(
        changes_seen > 3000,
        "the generator settles too little: {changes_seen}"
    );
    assert!(
        violations.is_empty(),
        "{} invariants violated over {changes_seen} changes:\n{}",
        violations.len(),
        violations
            .iter()
            .map(|(name, (count, first))| format!("- {name}: {count}x, first {first}"))
            .collect::<Vec<_>>()
            .join("\n")
    );
}
