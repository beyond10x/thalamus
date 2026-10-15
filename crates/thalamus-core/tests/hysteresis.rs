//! `story:m2-hysteresis-core`: a verdict change settles once enough consecutive evaluations of a
//! series agree (design § 3.2, § 11 D15), and the settled change carries what a transition
//! records (design § 11 D13).

use std::collections::BTreeMap;

use thalamus_core::hysteresis::{Persistence, SeriesHysteresis, SettledChange, replay};
use thalamus_core::registry::{Definition, Registry};
use thalamus_model::definition::SignalId;
use thalamus_model::evaluation::{EvaluationId, SignalEvaluationData, Verdict};
use thalamus_model::observation::{SampleId, SeriesKey};
use thalamus_model::primitives::{Decimal, Timestamp};

const REGISTRY: &str = include_str!("fixtures/instance/dimensions.yaml");
const QUEUE: &str = include_str!("fixtures/instance/definitions/service.queue.depth.yaml");

/// The `index`-th evaluation of one series, one minute after the previous one.
fn evaluation(index: u32, verdict: Verdict) -> SignalEvaluationData {
    let scored = matches!(verdict, Verdict::OK | Verdict::WARN | Verdict::CRIT);
    SignalEvaluationData {
        evaluation_id: EvaluationId(format!("evl_{index}")),
        sample_id: SampleId(format!("smp_{index}")),
        signal: SignalId("service.queue.depth".to_owned()),
        series: SeriesKey("service.queue.depth".to_owned()),
        dims: BTreeMap::new(),
        ts: ts(index),
        definition_rev: 1,
        verdict,
        prev: None,
        score: scored.then(|| Decimal("0.5".to_owned())),
        baseline: None,
        reason: (verdict != Verdict::OK).then(|| "case".to_owned()),
    }
}

fn ts(index: u32) -> Timestamp {
    Timestamp(format!(
        "2026-01-01T{:02}:{:02}:00Z",
        index / 60 % 24,
        index % 60
    ))
}

/// The evaluations of `verdicts`, numbered from 1.
fn series(verdicts: &[Verdict]) -> Vec<SignalEvaluationData> {
    (1..)
        .zip(verdicts)
        .map(|(index, &verdict)| evaluation(index, verdict))
        .collect()
}

/// What each evaluation of `verdicts` settled, in order: `Some(to)` where a change settled.
fn settled_by(persistence: Persistence, verdicts: &[Verdict]) -> Vec<Option<Verdict>> {
    let mut hysteresis = SeriesHysteresis::new(persistence);
    series(verdicts)
        .iter()
        .map(|evaluation| hysteresis.observe(evaluation).map(|change| change.to))
        .collect()
}

/// A hysteresis with default persistence after `verdicts`.
fn after(verdicts: &[Verdict]) -> SeriesHysteresis {
    let mut hysteresis = SeriesHysteresis::new(Persistence::default());
    for evaluation in series(verdicts) {
        let _ = hysteresis.observe(&evaluation);
    }
    hysteresis
}

use Verdict::{CRIT, NA, OK, UNKNOWN, WARN};

#[test]
fn warn_settles_after_two() {
    assert_eq!(
        settled_by(Persistence::default(), &[OK, OK, OK, WARN, WARN, WARN]),
        [None, None, Some(OK), None, Some(WARN), None]
    );
}

#[test]
fn crit_settles_after_one() {
    assert_eq!(
        settled_by(Persistence::default(), &[OK, OK, OK, CRIT, CRIT]),
        [None, None, Some(OK), Some(CRIT), None]
    );
    assert_eq!(settled_by(Persistence::default(), &[CRIT]), [Some(CRIT)]);
}

#[test]
fn warn_clears_after_three_ok() {
    assert_eq!(
        settled_by(Persistence::default(), &[WARN, WARN, OK, OK, OK, OK]),
        [None, Some(WARN), None, None, Some(OK), None]
    );
}

#[test]
fn two_ok_do_not_clear_warn() {
    let verdicts = [WARN, WARN, OK, OK, WARN, OK, OK];
    assert_eq!(
        settled_by(Persistence::default(), &verdicts),
        [None, Some(WARN), None, None, None, None, None]
    );
    assert_eq!(after(&verdicts).settled(), Some(WARN));
}

#[test]
fn unknown_does_not_clear_warn() {
    for settled in [WARN, CRIT] {
        let verdicts = [
            settled, settled, UNKNOWN, UNKNOWN, UNKNOWN, UNKNOWN, UNKNOWN,
        ];
        let changes = settled_by(Persistence::default(), &verdicts);
        assert_eq!(
            changes.iter().flatten().collect::<Vec<_>>(),
            [&settled],
            "UNKNOWN moved a settled {settled:?}: {changes:?}"
        );
        assert_eq!(after(&verdicts).settled(), Some(settled));
    }
}

#[test]
fn na_does_not_clear_warn() {
    for settled in [WARN, CRIT] {
        let verdicts = [settled, settled, NA, NA, NA, NA, NA];
        let changes = settled_by(Persistence::default(), &verdicts);
        assert_eq!(
            changes.iter().flatten().collect::<Vec<_>>(),
            [&settled],
            "N/A moved a settled {settled:?}: {changes:?}"
        );
        assert_eq!(after(&verdicts).settled(), Some(settled));
    }
}

#[test]
fn alternating_ok_warn_never_settles() {
    let alternating: Vec<Verdict> = [OK, WARN].into_iter().cycle().take(20).collect();
    assert_eq!(
        settled_by(Persistence::default(), &alternating),
        vec![None; 20]
    );
    assert_eq!(after(&alternating).settled(), None);

    let mut from_ok = vec![OK, OK, OK];
    from_ok.extend(&alternating);
    let changes = settled_by(Persistence::default(), &from_ok);
    assert_eq!(changes.iter().flatten().collect::<Vec<_>>(), [&OK]);
    assert_eq!(after(&from_ok).settled(), Some(OK));
}

#[test]
fn first_settled_verdict_has_no_from() {
    let mut hysteresis = SeriesHysteresis::new(Persistence::default());
    assert_eq!(hysteresis.settled(), None);
    let evaluations = series(&[WARN, WARN, CRIT]);
    assert_eq!(hysteresis.observe(&evaluations[0]), None);
    let first = hysteresis.observe(&evaluations[1]).expect("WARN settles");
    assert_eq!((first.from, first.to), (None, WARN));
    let second = hysteresis.observe(&evaluations[2]).expect("CRIT settles");
    assert_eq!((second.from, second.to), (Some(WARN), CRIT));
}

/// The settled change `from` → `to` whose agreeing evaluations are those numbered `members`, the
/// last one settling it, as [`evaluation`] builds them.
fn change(from: Option<Verdict>, to: Verdict, members: &[u32]) -> SettledChange {
    let first = *members.first().expect("a change has members");
    let last = *members.last().expect("a change has members");
    SettledChange {
        from,
        to,
        first_ts: ts(first),
        settled_ts: ts(last),
        settling_score: matches!(to, OK | WARN | CRIT).then(|| Decimal("0.5".to_owned())),
        definition_rev: 1,
        evaluations: members
            .iter()
            .map(|index| EvaluationId(format!("evl_{index}")))
            .collect(),
    }
}

fn ids(changes: &[SettledChange]) -> Vec<Vec<String>> {
    changes
        .iter()
        .map(|change| change.evaluations.iter().map(|id| id.0.clone()).collect())
        .collect()
}

#[test]
fn replay_equals_live() {
    let evaluations = series(&[
        OK, OK, OK, WARN, OK, WARN, WARN, UNKNOWN, UNKNOWN, OK, OK, OK, CRIT, NA, NA, WARN, WARN,
        OK, OK, OK, UNKNOWN, UNKNOWN,
    ]);
    let expected = [
        change(None, OK, &[1, 2, 3]),
        change(Some(OK), WARN, &[6, 7]),
        change(Some(WARN), OK, &[10, 11, 12]),
        change(Some(OK), CRIT, &[13]),
        change(Some(CRIT), WARN, &[16, 17]),
        change(Some(WARN), OK, &[18, 19, 20]),
        change(Some(OK), UNKNOWN, &[21, 22]),
    ];

    let mut live = SeriesHysteresis::new(Persistence::default());
    let live_changes: Vec<SettledChange> = evaluations
        .iter()
        .filter_map(|evaluation| live.observe(evaluation))
        .collect();
    assert_eq!(live_changes, expected, "live");

    // Stored history read back in minting order is the sequence live saw.
    assert_eq!(
        replay(Persistence::default(), &evaluations),
        expected,
        "replay"
    );

    // A service restarted after evaluation 13 resumes from the CRIT it recorded there and goes
    // on live with what follows.
    let mut resumed = SeriesHysteresis::resume(Persistence::default(), CRIT);
    let continued: Vec<SettledChange> = evaluations[13..]
        .iter()
        .filter_map(|evaluation| resumed.observe(evaluation))
        .collect();
    assert_eq!(continued, expected[4..], "resumed after 13");
}

/// `evaluation(sample, …)` graded again at definition revision `rev`, with id `evl_<sample>_r<rev>`.
fn regraded(sample: u32, rev: i64, verdict: Verdict) -> SignalEvaluationData {
    SignalEvaluationData {
        evaluation_id: EvaluationId(format!("evl_{sample}_r{rev}")),
        definition_rev: rev,
        ..evaluation(sample, verdict)
    }
}

/// Decision F1 (D5, D14): persistence counts distinct samples. A re-grade of a sample in the
/// current run at a higher revision replaces that sample's evaluation.
#[test]
fn a_regrade_at_a_higher_rev_replaces_the_sample() {
    let changes = replay(
        Persistence::default(),
        &[
            evaluation(1, WARN),
            regraded(1, 2, WARN),
            regraded(1, 3, WARN),
            evaluation(2, WARN),
        ],
    );
    assert_eq!(ids(&changes), [["evl_1_r3", "evl_2"]]);
    assert_eq!(changes[0].first_ts, ts(1));
    assert_eq!(changes[0].settled_ts, ts(2));
}

/// Decision F1: a re-grade at a revision no higher than the one counted is ignored, whatever its
/// verdict.
#[test]
fn a_regrade_at_a_lower_rev_is_ignored() {
    let mut hysteresis = SeriesHysteresis::new(Persistence::default());
    assert_eq!(hysteresis.observe(&regraded(1, 2, WARN)), None);
    assert_eq!(hysteresis.observe(&regraded(1, 1, OK)), None);
    assert_eq!(hysteresis.observe(&regraded(1, 2, CRIT)), None);
    assert_eq!(hysteresis.settled(), None);
    let settled = hysteresis
        .observe(&evaluation(2, WARN))
        .expect("WARN settles");
    assert_eq!(ids(&[settled]), [["evl_1_r2", "evl_2"]]);
}

/// Pass-2 decision 1: a re-grade replaces its sample's verdict in place and drops no earlier
/// sample. D13 (`adv_hysteresis_p2.rs:352-357`): the evaluation that settles a change gives its
/// `settled_ts`, `definition_rev` and `settling_score`, also when it re-grades an older sample.
#[test]
fn a_regrade_replaces_the_sample_in_place() {
    let mut hysteresis = SeriesHysteresis::new(Persistence::default());
    assert_eq!(hysteresis.observe(&evaluation(1, WARN)), None);
    assert_eq!(hysteresis.observe(&evaluation(2, OK)), None);
    assert_eq!(hysteresis.observe(&evaluation(3, WARN)), None);
    let regrade = SignalEvaluationData {
        score: Some(Decimal("1.7".to_owned())),
        ..regraded(2, 2, WARN)
    };
    let settled = hysteresis
        .observe(&regrade)
        .expect("samples 1, 2 and 3 read WARN, WARN, WARN");
    assert_eq!(ids(std::slice::from_ref(&settled)), [["evl_2_r2", "evl_3"]]);
    assert_eq!((&settled.first_ts, &settled.settled_ts), (&ts(2), &ts(2)));
    assert_eq!(
        settled.definition_rev, 2,
        "the settling re-grade's revision"
    );
    assert_eq!(settled.settling_score, Some(Decimal("1.7".to_owned())));

    // A re-grade away from the run's verdict and back restores it.
    let changes = replay(
        Persistence::default(),
        &[
            evaluation(1, WARN),
            regraded(1, 2, OK),
            regraded(1, 3, WARN),
            evaluation(2, WARN),
        ],
    );
    assert_eq!(ids(&changes), [["evl_1_r3", "evl_2"]]);
}

/// Pass-2 decision 1: a series keeps its newest `max(k, k_crit, m)` samples; one that has left
/// them is forgotten, so a later evaluation of it, at any revision, is ignored, even at the newest
/// `ts`. Here four samples share one `ts`; sample 1 has left the window of 3.
#[test]
fn a_sample_that_left_the_window_is_forgotten() {
    let at_one_ts = |sample: u32, rev: i64, verdict: Verdict| SignalEvaluationData {
        ts: ts(1),
        ..regraded(sample, rev, verdict)
    };
    let mut hysteresis = SeriesHysteresis::new(Persistence::default());
    assert_eq!(hysteresis.observe(&at_one_ts(1, 1, OK)), None);
    assert_eq!(hysteresis.observe(&at_one_ts(2, 1, OK)), None);
    assert!(hysteresis.observe(&at_one_ts(3, 1, OK)).is_some());
    assert_eq!(hysteresis.observe(&at_one_ts(4, 1, WARN)), None);
    assert_eq!(
        hysteresis.observe(&at_one_ts(1, 2, WARN)),
        None,
        "sample 1 counted again after it left the window"
    );
    assert_eq!(hysteresis.settled(), Some(OK));
}

/// Pass-2 decision 4: an evaluation whose `ts` is not an instant is ignored.
#[test]
fn an_evaluation_whose_ts_is_no_instant_is_ignored() {
    let mut hysteresis = SeriesHysteresis::new(Persistence::default());
    assert_eq!(hysteresis.observe(&evaluation(1, WARN)), None);
    let no_instant = SignalEvaluationData {
        ts: Timestamp("2026-01-01T00:x".to_owned()),
        ..evaluation(2, WARN)
    };
    assert_eq!(hysteresis.observe(&no_instant), None);
    assert_eq!(
        hysteresis.observe(&evaluation(3, WARN)),
        Some(change(None, WARN, &[1, 3]))
    );
}

/// Decision F2: an evaluation id already observed is ignored, and a change lists each sample once.
#[test]
fn the_same_evaluation_twice_is_listed_once() {
    let mut hysteresis = SeriesHysteresis::new(Persistence::default());
    let first = evaluation(1, OK);
    let second = evaluation(2, OK);
    assert_eq!(hysteresis.observe(&first), None);
    assert_eq!(hysteresis.observe(&first), None);
    assert_eq!(hysteresis.observe(&second), None);
    assert_eq!(hysteresis.observe(&second), None);
    let settled = hysteresis.observe(&evaluation(3, OK)).expect("OK settles");
    assert_eq!(ids(&[settled]), [["evl_1", "evl_2", "evl_3"]]);
}

/// Decision F3: `observe` returns no change for an evaluation older than the last one it took,
/// and the evaluation leaves the state as it was.
#[test]
fn observe_ignores_an_evaluation_older_than_the_last() {
    let mut hysteresis = SeriesHysteresis::new(Persistence::default());
    assert_eq!(hysteresis.observe(&evaluation(5, WARN)), None);
    assert_eq!(hysteresis.observe(&evaluation(4, WARN)), None);
    assert_eq!(hysteresis.observe(&evaluation(3, CRIT)), None);
    assert_eq!(hysteresis.settled(), None);
    let settled = hysteresis
        .observe(&evaluation(6, WARN))
        .expect("WARN settles");
    assert_eq!(settled, change(None, WARN, &[5, 6]));
}

/// Pass-2 decision 2: `replay` reads its input in minting order, as live took it, so a sample
/// recorded late is dropped by both.
#[test]
fn replay_drops_a_late_sample_as_live_does() {
    let minted = [
        evaluation(5, WARN),
        evaluation(4, WARN),
        evaluation(6, WARN),
    ];
    let mut live = SeriesHysteresis::new(Persistence::default());
    let live_changes: Vec<SettledChange> = minted
        .iter()
        .filter_map(|evaluation| live.observe(evaluation))
        .collect();
    assert_eq!(live_changes, [change(None, WARN, &[5, 6])]);
    assert_eq!(replay(Persistence::default(), &minted), live_changes);
}

/// Decision J1: new persistence applies from the next evaluation and keeps the settled verdict
/// and the current run.
#[test]
fn set_persistence_applies_from_the_next_evaluation() {
    let mut hysteresis = SeriesHysteresis::new(Persistence::default());
    assert_eq!(hysteresis.observe(&evaluation(1, WARN)), None);
    let raised = hysteresis.set_persistence(Persistence {
        k: 3,
        k_crit: 1,
        m: 1,
    });
    assert_eq!(raised, None);
    assert_eq!(hysteresis.observe(&evaluation(2, WARN)), None, "k is now 3");
    assert_eq!(
        hysteresis.observe(&evaluation(3, WARN)),
        Some(change(None, WARN, &[1, 2, 3])),
        "the run kept evaluation 1"
    );
    assert_eq!(
        hysteresis.observe(&evaluation(4, OK)),
        Some(change(Some(WARN), OK, &[4])),
        "m is now 1, and the settled WARN was kept"
    );

    // Pass-2 decision 5: a smaller n settles a run already that long at once, whatever comes
    // next.
    let mut shrunk = SeriesHysteresis::new(Persistence::default());
    assert_eq!(shrunk.observe(&evaluation(1, OK)), None);
    assert_eq!(shrunk.observe(&evaluation(2, OK)), None);
    assert_eq!(
        shrunk.set_persistence(Persistence {
            k: 2,
            k_crit: 1,
            m: 2,
        }),
        Some(change(None, OK, &[1, 2]))
    );
    assert_eq!(shrunk.observe(&evaluation(3, WARN)), None);
    assert_eq!(shrunk.settled(), Some(OK));
}

/// Decision J2: a hysteresis resumed from a recorded settled verdict carries it as `from`.
#[test]
fn resume_carries_the_recorded_settled_verdict() {
    let resumed = |verdicts: &[Verdict]| {
        let mut hysteresis = SeriesHysteresis::resume(Persistence::default(), WARN);
        assert_eq!(hysteresis.settled(), Some(WARN));
        series(verdicts)
            .iter()
            .filter_map(|evaluation| hysteresis.observe(evaluation))
            .collect::<Vec<_>>()
    };
    assert_eq!(resumed(&[WARN, WARN, WARN]), []);
    assert_eq!(resumed(&[UNKNOWN, UNKNOWN, NA, NA]), []);
    assert_eq!(resumed(&[OK, OK, OK]), [change(Some(WARN), OK, &[1, 2, 3])]);
}

#[test]
fn persistence_block_overrides_defaults() {
    let registry = Registry::parse("dimensions.yaml", REGISTRY).expect("the fixture registry");
    let plain = Definition::parse("definitions/service.queue.depth.yaml", QUEUE, &registry)
        .expect("the fixture definition loads");
    assert_eq!(Persistence::of(plain.data()), Persistence::default());
    assert_eq!(
        Persistence::default(),
        Persistence {
            k: 2,
            k_crit: 1,
            m: 3
        }
    );

    let with_block = QUEUE.replace(
        "owner: team:service\n",
        "persistence: { k: 3, k_crit: 2, m: 4 }\nowner: team:service\n",
    );
    assert_ne!(with_block, QUEUE);
    let declared = Definition::parse(
        "definitions/service.queue.depth.yaml",
        &with_block,
        &registry,
    )
    .expect("a definition with a persistence block loads");
    let persistence = Persistence::of(declared.data());
    assert_eq!(
        persistence,
        Persistence {
            k: 3,
            k_crit: 2,
            m: 4
        }
    );

    assert_eq!(
        settled_by(persistence, &[WARN, WARN, WARN, WARN]),
        [None, None, Some(WARN), None],
        "WARN settles after three"
    );
    assert_eq!(
        settled_by(persistence, &[CRIT, CRIT, CRIT]),
        [None, Some(CRIT), None],
        "CRIT settles after two"
    );
    assert_eq!(
        settled_by(persistence, &[WARN, WARN, WARN, OK, OK, OK, OK, OK]),
        [None, None, Some(WARN), None, None, None, Some(OK), None],
        "WARN clears after four"
    );
}

#[test]
fn settled_change_carries_d13_fields() {
    let mut evaluations = series(&[OK, OK, OK, WARN, WARN]);
    evaluations[4].definition_rev = 2;
    evaluations[4].score = Some(Decimal("1.3".to_owned()));
    let changes = replay(Persistence::default(), &evaluations);
    assert_eq!(
        changes.last(),
        Some(&SettledChange {
            from: Some(OK),
            to: WARN,
            first_ts: ts(4),
            settled_ts: ts(5),
            settling_score: Some(Decimal("1.3".to_owned())),
            definition_rev: 2,
            evaluations: vec![
                EvaluationId("evl_4".to_owned()),
                EvaluationId("evl_5".to_owned())
            ],
        })
    );

    // A clearing change names the `m` OK evaluations that cleared it, none of the WARN before.
    let cleared = replay(
        Persistence::default(),
        &series(&[WARN, WARN, WARN, OK, OK, OK]),
    );
    let clearing = cleared.last().expect("WARN clears");
    assert_eq!((clearing.from, clearing.to), (Some(WARN), OK));
    assert_eq!((&clearing.first_ts, &clearing.settled_ts), (&ts(4), &ts(6)));
    assert_eq!(
        clearing.evaluations,
        ["evl_4", "evl_5", "evl_6"].map(|id| EvaluationId(id.to_owned()))
    );
    assert_eq!(clearing.settling_score, Some(Decimal("0.5".to_owned())));

    // An UNKNOWN settles with no score, never 0.
    let unknown = replay(Persistence::default(), &series(&[UNKNOWN, UNKNOWN]));
    assert_eq!(unknown.len(), 1);
    assert_eq!(unknown[0].settling_score, None);
}
