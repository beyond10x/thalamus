//! `story:m3-advisory-gate-mapping`: an advisory gate's result is graded on the one ladder by the
//! rule `fixed_warn_only` (design § 2.3, § 11 D12). The finding count (kind `count_of_cases`, unit
//! `{finding}`) is the value, graded against a declared `fixed` baseline: above it `WARN`, at or
//! below it `OK`, never `CRIT`. A gate that printed no count is an `unknown` reading, which is
//! `UNKNOWN`, never 0. A definition naming the rule with a `crit` line, a `warn` line, or a
//! baseline list that is not `fixed` alone is refused at load, by `ProposeDefinition` and by
//! `ReviseDefinition` alike.
//!
//! Score (design § 3.3): 0 at the baseline, 1 at the first count above it, never above 1.

use std::collections::BTreeMap;

use thalamus_core::conformance::Ports;
use thalamus_core::grading::{self, Graded};
use thalamus_core::registry::{Definition, LoadError, Registry};
use thalamus_model::behaviour::Generated;
use thalamus_model::definition::obligations::{
    ProposeDefinitionBehavior, ReviseDefinitionBehavior,
};
use thalamus_model::definition::{
    AdapterId, AdvisoryNeedsCountOfCases, AdvisoryNeedsFindingUnit, AdvisoryNeedsFixedBaseline,
    Author, BaselineKind, ConnectionId, DenominatorSpec, EvaluationSpec, ExtractorId, Family,
    History, Links, Owner, ProposeDefinition, ProposeDefinitionOutcome, QueryText,
    ReviseDefinition, ReviseDefinitionOutcome, RuleId, SignalDefinitionData, SignalId, SignalKind,
    Source, Threshold, ThresholdsUnused, Unit,
};
use thalamus_model::evaluation::{Baseline, BaselineLabel, Verdict};
use thalamus_model::observation::{
    Bound, CompleteReading, PartialReading, Provenance, QueryDigest, Reading, RunId, SampleId,
    SeriesKey, SignalSampleData, TimeWindow, UnreadReading,
};
use thalamus_model::primitives::{Decimal, Duration, Timestamp};
use thalamus_model::server::memory::MemoryPorts;

const RULE: &str = "fixed_warn_only";
/// The refusal of a `warn` or `crit` line under the rule, shared with `producer_mapped`.
const THRESHOLDS_UNUSED: &str = "thalamus.definition.ThresholdsUnused";
/// The refusal of a baseline list that is not `fixed` alone under the rule.
const NOT_FIXED: &str = "thalamus.definition.AdvisoryNeedsFixedBaseline";
/// The refusal of a signal under the rule whose kind is not `count_of_cases`.
const NOT_COUNT: &str = "thalamus.definition.AdvisoryNeedsCountOfCases";
/// The refusal of a signal under the rule whose unit is not `{finding}`.
const NOT_FINDING_UNIT: &str = "thalamus.definition.AdvisoryNeedsFindingUnit";
/// Units other than `{finding}`, each a valid UCUM unit: another annotation, a near miss in case or
/// number, a dimensionless 1 and a base unit.
const OTHER_UNITS: [&str; 5] = ["{run}", "{Finding}", "{findings}", "1", "By"];
/// Every kind but `count_of_cases`, as a definition file writes it.
const OTHER_KINDS: [(&str, SignalKind); 4] = [
    ("gauge", SignalKind::Gauge),
    ("counter_increase", SignalKind::CounterIncrease),
    ("ratio", SignalKind::Ratio),
    ("instant", SignalKind::Instant),
];
const REGISTRY: &str = include_str!("fixtures/instance/dimensions.yaml");
const SIGNAL: &str = "gate.ci.advisory_findings";
/// The declared baseline: the findings the gate is allowed today.
const BASELINE: &str = "3";

fn definition() -> SignalDefinitionData {
    SignalDefinitionData {
        signal_id: SignalId(SIGNAL.into()),
        title: "Findings of the advisory lint gate".into(),
        family: Family("gate".into()),
        kind: SignalKind::CountOfCases,
        unit: Some(Unit("{finding}".into())),
        dimensions: Some(Vec::new()),
        source: Source {
            adapter: AdapterId("sql".into()),
            connection: ConnectionId("ci-results".into()),
        },
        extractor: ExtractorId("sql.value".into()),
        query: BTreeMap::new(),
        window: Duration("PT1H".into()),
        cadence: Duration("PT1H".into()),
        history: History::Store,
        denominator: Some(DenominatorSpec {
            unit: Unit("{run}".into()),
            min_n: 0,
            since: "2026-01-01T00:00:00Z".into(),
        }),
        evaluation: EvaluationSpec {
            rule: RuleId(RULE.into()),
            baselines: vec![BaselineKind::Fixed],
            warn: None,
            crit: None,
            min_trend_window: None,
            fixed: Some(Decimal(BASELINE.into())),
            rolling_n: None,
            mapping: None,
        },
        persistence: None,
        objective: None,
        owner: Some(Owner("team:gate".into())),
        links: Links {
            components: Vec::new(),
            issues: Vec::new(),
        },
        rev: 1,
        revised_at: Timestamp("2026-10-01T00:00:00Z".into()),
        revised_by: Author("someone".into()),
        mute_reason: None,
        mute_until: None,
    }
}

fn sample(reading: Reading) -> SignalSampleData {
    SignalSampleData {
        text: None,
        sample_id: SampleId("smp_1".into()),
        signal: SignalId(SIGNAL.into()),
        series: SeriesKey(SIGNAL.into()),
        dims: BTreeMap::new(),
        ts: Timestamp("2026-10-14T22:00:00Z".into()),
        window: TimeWindow {
            start: Timestamp("2026-10-14T21:00:00Z".into()),
            end: Timestamp("2026-10-14T22:00:00Z".into()),
        },
        reading,
        unit: Unit("{finding}".into()),
        denominator: None,
        provenance: Provenance {
            producer: "thalamus-service".into(),
            version: "0.1.0".into(),
            definition_rev: 1,
            adapter: AdapterId("sql".into()),
            connection: ConnectionId("ci-results".into()),
            query_sha256: QueryDigest("00".into()),
            run_id: RunId("run_1".into()),
        },
    }
}

fn fixed(value: &str) -> Baseline {
    Baseline {
        kind: BaselineLabel("fixed".into()),
        offsets: Vec::new(),
        reading: Reading::Ok(CompleteReading {
            value: Decimal(value.into()),
        }),
    }
}

fn count(value: &str) -> Reading {
    Reading::Ok(CompleteReading {
        value: Decimal(value.into()),
    })
}

fn grade(reading: Reading) -> Graded {
    grading::grade(&definition(), &sample(reading), &fixed(BASELINE))
        .expect("a fixed_warn_only definition grades")
}

fn score(text: &str) -> Option<Decimal> {
    Some(Decimal(text.into()))
}

/// A count above the fixed baseline is `WARN`, scored 1, and the reason names the count and the
/// baseline.
#[test]
fn advisory_count_above_baseline_is_warn() {
    for value in ["4", "7"] {
        let graded = grade(count(value));
        assert_eq!(graded.verdict, Verdict::WARN, "count {value}: {graded:?}");
        assert_eq!(graded.score, score("1"), "count {value}: {graded:?}");
        assert!(
            graded.reason.contains(value) && graded.reason.contains(BASELINE),
            "the reason names the count and the baseline: {graded:?}"
        );
    }
}

/// A count at the baseline is `OK` scored 0; one below it is `OK` scored below 0 (design § 3.3).
/// Both carry a reason.
#[test]
fn advisory_count_at_baseline_is_ok() {
    let at = grade(count(BASELINE));
    assert_eq!(at.verdict, Verdict::OK, "{at:?}");
    assert_eq!(at.score, score("0"), "0 at the baseline: {at:?}");
    assert!(
        at.reason.contains(BASELINE),
        "the reason names the baseline: {at:?}"
    );

    let below = grade(count("1"));
    assert_eq!(below.verdict, Verdict::OK, "{below:?}");
    assert_eq!(
        below.score,
        score("-2"),
        "negative when better than the baseline: {below:?}"
    );
    assert!(!below.reason.is_empty(), "{below:?}");
}

/// The rule has no CRIT line: a count one hundred times the baseline is `WARN`, scored 1, never
/// above; so is a count beyond any CRIT line a threshold rule could declare. A partial read whose
/// lower bound is far above the baseline does not escalate to `CRIT` either.
#[test]
fn advisory_never_grades_crit() {
    for value in ["300", "1000000000000000000000000"] {
        let graded = grade(count(value));
        assert_eq!(graded.verdict, Verdict::WARN, "count {value}: {graded:?}");
        assert_eq!(
            graded.score,
            score("1"),
            "never above 1: count {value}: {graded:?}"
        );
    }
    let cut = Reading::Partial(PartialReading {
        value: Decimal("300".into()),
        bound: Bound::Lower,
    });
    let graded = grade(cut);
    assert_ne!(graded.verdict, Verdict::CRIT, "{graded:?}");
}

/// A gate that printed no count is an `unknown` reading: `UNKNOWN`, with no score and the
/// reading's own reason, never `OK` as a count of 0 would be. An unknown baseline is `UNKNOWN` too.
#[test]
fn advisory_missing_count_is_unknown_not_zero() {
    let unread = Reading::Unknown(UnreadReading {
        reason: "the gate printed no finding count".into(),
    });
    let graded = grade(unread);
    assert_eq!(
        (
            graded.verdict,
            graded.score.as_ref(),
            graded.reason.as_str()
        ),
        (Verdict::UNKNOWN, None, "the gate printed no finding count"),
        "{graded:?}"
    );

    let zero = grade(count("0"));
    assert_eq!(
        zero.verdict,
        Verdict::OK,
        "a printed 0 is a count: {zero:?}"
    );

    let unknown_baseline = Baseline {
        reading: Reading::Unknown(UnreadReading {
            reason: "evaluation.fixed unread".into(),
        }),
        ..fixed(BASELINE)
    };
    let graded = grading::grade(&definition(), &sample(count("9")), &unknown_baseline)
        .expect("a fixed_warn_only definition grades");
    assert_eq!(
        (graded.verdict, graded.score.as_ref()),
        (Verdict::UNKNOWN, None),
        "{graded:?}"
    );
}

// ---------------------------------------------------------------------------------------------
// At load.
// ---------------------------------------------------------------------------------------------

/// A definition file for an advisory gate's finding count, with `extra` appended to its
/// `evaluation` block and `baselines` as its baseline list.
fn definition_file(baselines: &str, extra: &str) -> String {
    definition_file_of_kind("count_of_cases", baselines, extra)
}

/// [`definition_file`] for a signal of `kind`.
fn definition_file_of_kind(kind: &str, baselines: &str, extra: &str) -> String {
    format!(
        "id: {SIGNAL}\n\
         title: Findings of the advisory lint gate\n\
         family: gate\n\
         kind: {kind}\n\
         unit: \"{{finding}}\"\n\
         dimensions: []\n\
         source: {{ adapter: sql, connection: ci-results }}\n\
         extractor: sql.value\n\
         query: {{ value: select findings from gate_runs }}\n\
         window: PT1H\n\
         cadence: PT1H\n\
         history: store\n\
         denominator: {{ unit: \"{{run}}\", min_n: 0, since: 2026-01-01T00:00:00Z }}\n\
         evaluation:\n\
         \x20 rule: {RULE}\n\
         \x20 baselines: [{baselines}]\n\
         \x20 fixed: \"{BASELINE}\"\n\
         {extra}\
         owner: team:gate\n\
         links: {{ components: [], issues: [] }}\n\
         lifecycle: active\n\
         rev: 1\n\
         revised_at: 2026-10-02T00:00:00Z\n\
         revised_by: operator\n"
    )
}

fn load(text: &str) -> Result<Definition, Vec<LoadError>> {
    let registry = Registry::parse("dimensions.yaml", REGISTRY).expect("the fixture registry");
    Definition::parse("definitions/case.yaml", text, &registry)
}

/// The rule has no CRIT line, and its WARN line is the first count above the fixed baseline, so a
/// declared `crit` or `warn` line is a threshold nothing reads (`thresholds-unused`), and a baseline
/// list that is not `fixed` alone is one the rule cannot grade against (`advisory-not-fixed`): the
/// loader refuses each, and loads the definition without them.
#[test]
fn advisory_crit_line_is_refused_at_load() {
    let plain = definition_file("fixed", "");
    let loaded = load(&plain).unwrap_or_else(|errors| panic!("refused: {errors:?}\n{plain}"));
    assert_eq!(loaded.data().evaluation.rule.0, RULE);

    for (baselines, extra, refusal) in [
        ("fixed", "  crit: { abs: \"10\" }\n", THRESHOLDS_UNUSED),
        ("fixed", "  warn: { abs: \"5\" }\n", THRESHOLDS_UNUSED),
        (
            "fixed",
            "  warn: { abs: \"5\" }\n  crit: { abs: \"10\" }\n",
            THRESHOLDS_UNUSED,
        ),
        ("higher_of", "", NOT_FIXED),
        ("fixed, offset_1d", "", NOT_FIXED),
        ("", "", NOT_FIXED),
    ] {
        let text = definition_file(baselines, extra);
        let errors = load(&text)
            .err()
            .unwrap_or_else(|| panic!("loads:\n{text}"));
        assert!(
            errors.iter().any(|error| error.rule.name() == refusal),
            "refused, but not as {refusal}: {errors:?}\n{text}"
        );
    }
}

/// The rule grades a finding count, and its score steps to the first whole count above the
/// baseline: a signal of any other kind under it is refused at load (`advisory-not-count`, design
/// § 11 D12), with the plain fixed baseline, so nothing else answers for it.
#[test]
fn advisory_non_count_kind_is_refused_at_load() {
    for (kind, _) in OTHER_KINDS {
        let text = definition_file_of_kind(kind, "fixed", "");
        let errors = load(&text)
            .err()
            .unwrap_or_else(|| panic!("loads:\n{text}"));
        assert!(
            errors.iter().any(|error| error.rule.name() == NOT_COUNT),
            "refused, but not as {NOT_COUNT}: {errors:?}\n{text}"
        );
    }
}

/// D12 fixes the unit beside the kind: an advisory gate counts findings, unit `{finding}`, so a
/// signal under the rule with any other unit is refused at load (`advisory-not-finding-unit`),
/// with everything else as the plain definition, so nothing else answers for it.
#[test]
fn advisory_non_finding_unit_is_refused_at_load() {
    let plain = definition_file("fixed", "");
    let finding = "unit: \"{finding}\"\n";
    assert_eq!(plain.matches(finding).count(), 1, "{plain}");
    for unit in OTHER_UNITS {
        let text = plain.replace(finding, &format!("unit: \"{unit}\"\n"));
        let errors = load(&text)
            .err()
            .unwrap_or_else(|| panic!("loads:\n{text}"));
        assert!(
            errors
                .iter()
                .any(|error| error.rule.name() == NOT_FINDING_UNIT),
            "refused, but not as {NOT_FINDING_UNIT}: {errors:?}\n{text}"
        );
    }
}

// ---------------------------------------------------------------------------------------------
// In ProposeDefinition and ReviseDefinition: the same refusals as the loader, through the one
// `input_refusals` (design § 11 D29).
// ---------------------------------------------------------------------------------------------

fn ports() -> Generated<Ports<MemoryPorts>> {
    Generated::new(Ports::new(
        MemoryPorts::default(),
        Some(Timestamp("2026-01-01T00:00:00Z".into())),
    ))
}

fn propose(evaluation: EvaluationSpec) -> ProposeDefinition {
    let held = definition();
    ProposeDefinition {
        signal_id: held.signal_id,
        title: held.title,
        family: held.family,
        kind: held.kind,
        unit: held.unit,
        dimensions: held.dimensions,
        source: held.source,
        extractor: held.extractor,
        query: BTreeMap::from([("value".to_owned(), QueryText("select 1".into()))]),
        window: held.window,
        cadence: held.cadence,
        history: held.history,
        denominator: held.denominator,
        evaluation,
        persistence: None,
        objective: None,
        owner: held.owner,
        links: held.links,
        author: Author("author".into()),
    }
}

fn revise(evaluation: EvaluationSpec) -> ReviseDefinition {
    let proposed = propose(evaluation);
    ReviseDefinition {
        signal_id: proposed.signal_id,
        title: proposed.title,
        family: proposed.family,
        kind: proposed.kind,
        unit: Unit("{finding}".into()),
        dimensions: Vec::new(),
        source: proposed.source,
        extractor: proposed.extractor,
        query: proposed.query,
        window: proposed.window,
        cadence: proposed.cadence,
        history: proposed.history,
        denominator: proposed.denominator.expect("declared above"),
        evaluation: proposed.evaluation,
        persistence: None,
        objective: None,
        owner: Owner("team:gate".into()),
        links: proposed.links,
        author: Author("reviser".into()),
    }
}

/// `ProposeDefinition` and `ReviseDefinition` refuse what the loader refuses: a `warn` or `crit`
/// line under `fixed_warn_only` as `thresholds-unused`, and a baseline list that is not `fixed`
/// alone as `advisory-not-fixed`, the empty list included (`UNMAPPED:` on `advisory-not-fixed` in
/// `spec/domains/definition.yaml`).
#[test]
fn advisory_lines_and_baselines_are_refused_on_propose_and_revise() {
    let line = |abs: &str| Threshold {
        x: None,
        abs: Some(Decimal(abs.into())),
        min_events: None,
    };
    let signal_id = || SignalId(SIGNAL.into());
    for (warn, crit) in [
        (None, Some(line("10"))),
        (Some(line("5")), None),
        (Some(line("5")), Some(line("10"))),
    ] {
        let mut with = definition().evaluation;
        with.warn = warn;
        with.crit = crit;
        let mut generated = ports();
        assert_eq!(
            ProposeDefinitionBehavior::propose_definition(&mut generated, propose(with.clone()))
                .expect("propose answers"),
            ProposeDefinitionOutcome::ThresholdsUnused {
                error: ThresholdsUnused {
                    signal_id: signal_id()
                }
            },
            "{with:?}"
        );
        assert_eq!(
            ReviseDefinitionBehavior::revise_definition(&mut generated, revise(with.clone()))
                .expect("revise answers"),
            ReviseDefinitionOutcome::ThresholdsUnused {
                error: ThresholdsUnused {
                    signal_id: signal_id()
                }
            },
            "{with:?}"
        );
    }

    // `evaluation.fixed` is dropped where no fixed baseline reads it, so `fixed-number-unused`,
    // which comes first, does not answer instead.
    for (baselines, fixed) in [
        (vec![BaselineKind::HigherOf], None),
        (
            vec![BaselineKind::Fixed, BaselineKind::Offset1d],
            Some(BASELINE),
        ),
        (Vec::new(), None),
    ] {
        let mut with = definition().evaluation;
        with.baselines = baselines;
        with.fixed = fixed.map(|number| Decimal(number.into()));
        let mut generated = ports();
        assert_eq!(
            ProposeDefinitionBehavior::propose_definition(&mut generated, propose(with.clone()))
                .expect("propose answers"),
            ProposeDefinitionOutcome::AdvisoryNotFixed {
                error: AdvisoryNeedsFixedBaseline {
                    signal_id: signal_id()
                }
            },
            "{with:?}"
        );
        assert_eq!(
            ReviseDefinitionBehavior::revise_definition(&mut generated, revise(with.clone()))
                .expect("revise answers"),
            ReviseDefinitionOutcome::AdvisoryNotFixed {
                error: AdvisoryNeedsFixedBaseline {
                    signal_id: signal_id()
                }
            },
            "{with:?}"
        );
    }

    let mut generated = ports();
    assert!(
        matches!(
            ProposeDefinitionBehavior::propose_definition(
                &mut generated,
                propose(definition().evaluation)
            )
            .expect("propose answers"),
            ProposeDefinitionOutcome::Proposed { .. }
        ),
        "the plain advisory definition is proposed"
    );
}

/// `ProposeDefinition` and `ReviseDefinition` refuse what the loader refuses: the rule under a
/// kind other than `count_of_cases` is `advisory-not-count`, with nothing else wrong.
#[test]
fn advisory_non_count_kind_is_refused_on_propose_and_revise() {
    let refused = || AdvisoryNeedsCountOfCases {
        signal_id: SignalId(SIGNAL.into()),
    };
    for (_, kind) in OTHER_KINDS {
        let mut generated = ports();
        let mut proposed = propose(definition().evaluation);
        proposed.kind = kind;
        assert_eq!(
            ProposeDefinitionBehavior::propose_definition(&mut generated, proposed)
                .expect("propose answers"),
            ProposeDefinitionOutcome::AdvisoryNotCount { error: refused() },
            "{kind:?}"
        );
        let mut revised = revise(definition().evaluation);
        revised.kind = kind;
        assert_eq!(
            ReviseDefinitionBehavior::revise_definition(&mut generated, revised)
                .expect("revise answers"),
            ReviseDefinitionOutcome::AdvisoryNotCount { error: refused() },
            "{kind:?}"
        );
    }
}

/// `ProposeDefinition` and `ReviseDefinition` refuse what the loader refuses: the rule under a
/// unit other than `{finding}` is `advisory-not-finding-unit`, with nothing else wrong. A proposal
/// that declares no unit is not refused by it: it cannot be activated before a revision declares
/// one, and the revision is refused here.
#[test]
fn advisory_non_finding_unit_is_refused_on_propose_and_revise() {
    let refused = || AdvisoryNeedsFindingUnit {
        signal_id: SignalId(SIGNAL.into()),
    };
    for unit in OTHER_UNITS {
        let mut generated = ports();
        let mut proposed = propose(definition().evaluation);
        proposed.unit = Some(Unit(unit.into()));
        assert_eq!(
            ProposeDefinitionBehavior::propose_definition(&mut generated, proposed)
                .expect("propose answers"),
            ProposeDefinitionOutcome::AdvisoryNotFindingUnit { error: refused() },
            "{unit}"
        );
        let mut revised = revise(definition().evaluation);
        revised.unit = Unit(unit.into());
        assert_eq!(
            ReviseDefinitionBehavior::revise_definition(&mut generated, revised)
                .expect("revise answers"),
            ReviseDefinitionOutcome::AdvisoryNotFindingUnit { error: refused() },
            "{unit}"
        );
    }

    let mut generated = ports();
    let mut unitless = propose(definition().evaluation);
    unitless.unit = None;
    assert!(
        matches!(
            ProposeDefinitionBehavior::propose_definition(&mut generated, unitless)
                .expect("propose answers"),
            ProposeDefinitionOutcome::Proposed { .. }
        ),
        "a proposal without a unit is not refused as advisory-not-finding-unit"
    );
}
