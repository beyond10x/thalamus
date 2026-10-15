//! `story:m3-vocabulary-mapping`, the `thalamus-core` half: the rule `producer_mapped` grades a
//! sample by the producer's own value, `sample.text`, through the definition's `evaluation.mapping`
//! at the revision being graded (design § 2.3), so a mapping revision re-grades history (design
//! § 2, D5). A definition naming it without a usable `evaluation.mapping`, or with `warn`/`crit`
//! lines, is refused at load, as `ProposeDefinition` and `ReviseDefinition` refuse it.
//!
//! A mapped verdict's score is its position on the score scale of design § 3.3: 0 at `OK`, 1 at
//! `WARN`, 2 at `CRIT`.

use std::collections::BTreeMap;

use thalamus_core::conformance::Ports;
use thalamus_core::grading::{self, Graded};
use thalamus_core::registry::{Definition, LoadError, Registry};
use thalamus_model::behaviour::Generated;
use thalamus_model::definition::obligations::{
    ProposeDefinitionBehavior, ReviseDefinitionBehavior,
};
use thalamus_model::definition::{
    AdapterId, Author, BaselineKind, ConnectionId, DenominatorSpec, EvaluationSpec, ExtractorId,
    Family, History, Links, MappingMissing, MappingUnused, Owner, ProposeDefinition,
    ProposeDefinitionOutcome, QueryText, ReviseDefinition, ReviseDefinitionOutcome, RuleId,
    SignalDefinitionData, SignalId, SignalKind, Source, Threshold, ThresholdsUnused, Unit,
};
use thalamus_model::dimension::DimensionKey;
use thalamus_model::evaluation::{Baseline, BaselineLabel, Verdict};
use thalamus_model::observation::{
    CompleteReading, Provenance, QueryDigest, Reading, RunId, SampleId, SeriesKey,
    SignalSampleData, TimeWindow, UnreadReading,
};
use thalamus_model::primitives::{Decimal, Duration, Timestamp};
use thalamus_model::server::memory::MemoryPorts;

const RULE: &str = "producer_mapped";
const MAPPING_MISSING: &str = "thalamus.definition.MappingMissing";
const MAPPING_UNUSED: &str = "thalamus.definition.MappingUnused";
const THRESHOLDS_UNUSED: &str = "thalamus.definition.ThresholdsUnused";
const REGISTRY: &str = include_str!("fixtures/instance/dimensions.yaml");
const SIGNAL: &str = "gate.ci.blocking_exit";

/// The colour ladder of design § 2.3, as a definition would write it.
fn colours() -> BTreeMap<String, Verdict> {
    BTreeMap::from([
        ("green".to_owned(), Verdict::OK),
        ("yellow".to_owned(), Verdict::WARN),
        ("red".to_owned(), Verdict::CRIT),
        ("na".to_owned(), Verdict::UNKNOWN),
    ])
}

fn evaluation(mapping: Option<BTreeMap<String, Verdict>>) -> EvaluationSpec {
    EvaluationSpec {
        rule: RuleId(RULE.into()),
        baselines: vec![BaselineKind::Fixed],
        warn: None,
        crit: None,
        min_trend_window: None,
        fixed: Some(Decimal("0".into())),
        rolling_n: None,
        mapping,
    }
}

fn definition() -> SignalDefinitionData {
    SignalDefinitionData {
        signal_id: SignalId(SIGNAL.into()),
        title: "Status colour of the build board".into(),
        family: Family("gate".into()),
        kind: SignalKind::Gauge,
        unit: Some(Unit("{verdict}".into())),
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
        denominator: None,
        evaluation: evaluation(Some(colours())),
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

fn sample(reading: Reading, text: Option<&str>) -> SignalSampleData {
    SignalSampleData {
        text: text.map(str::to_owned),
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
        unit: Unit("{verdict}".into()),
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

/// A text cell as the `sql` adapter reads it: `unknown`, with the adapter's own reason.
fn text_cell(text: &str) -> Reading {
    Reading::Unknown(UnreadReading {
        reason: format!("value `{text}` in column `value` is not a number"),
    })
}

/// The fixed baseline a mapped signal is graded against: 0, the position of `OK`.
fn fixed_zero() -> Baseline {
    Baseline {
        kind: BaselineLabel("fixed".into()),
        offsets: Vec::new(),
        reading: Reading::Ok(CompleteReading {
            value: Decimal("0".into()),
        }),
    }
}

fn grade_at(definition: &SignalDefinitionData, reading: Reading, text: Option<&str>) -> Graded {
    grading::grade(definition, &sample(reading, text), &fixed_zero())
        .expect("a producer_mapped definition grades")
}

fn grade(reading: Reading, text: Option<&str>) -> Graded {
    grade_at(&definition(), reading, text)
}

/// The sample keeps its own quality (a text cell is `unknown`, an exit code `ok`), and grading maps
/// its text: `green`, `yellow`, `red` grade `OK`, `WARN`, `CRIT`, scored at their position.
#[test]
fn mapped_reading_grades_to_its_verdict_and_score() {
    for (text, verdict, score) in [
        ("green", Verdict::OK, "0"),
        ("yellow", Verdict::WARN, "1"),
        ("red", Verdict::CRIT, "2"),
    ] {
        let graded = grade(text_cell(text), Some(text));
        assert_eq!(graded.verdict, verdict, "`{text}`: {graded:?}");
        assert_eq!(
            graded.score,
            Some(Decimal(score.into())),
            "the score is the verdict's position, so the two cannot disagree: {graded:?}"
        );
        assert!(graded.reason.contains(&format!("`{text}`")), "{graded:?}");
    }

    let mut exits = definition();
    exits.evaluation.mapping = Some(BTreeMap::from([
        ("0".to_owned(), Verdict::OK),
        ("1".to_owned(), Verdict::CRIT),
    ]));
    for (exit, verdict) in [("0", Verdict::OK), ("1", Verdict::CRIT)] {
        let reading = Reading::Ok(CompleteReading {
            value: Decimal(exit.into()),
        });
        let graded = grade_at(&exits, reading, Some(exit));
        assert_eq!(graded.verdict, verdict, "exit {exit}: {graded:?}");
    }
}

/// A producer value the table maps to `UNKNOWN`, and one it does not list, grade `UNKNOWN` with no
/// score, with a reason `mapped.rs` writes: it names the producer value and the revision graded.
#[test]
fn unknown_mapped_reading_grades_unknown_without_score() {
    for text in ["na", "blue"] {
        let graded = grade(text_cell(text), Some(text));
        assert_eq!(graded.verdict, Verdict::UNKNOWN, "{graded:?}");
        assert_eq!(graded.score, None, "UNKNOWN carries no score: {graded:?}");
        assert!(
            graded.reason.contains(&format!("producer value `{text}`"))
                && graded.reason.contains("rev 1"),
            "the reason names the producer value and the revision: {graded:?}"
        );
    }
}

/// Unknown is never OK: a sample with no producer text is `UNKNOWN` under `producer_mapped`,
/// whatever its number, and an unread one keeps its own reason.
#[test]
fn mapped_value_off_the_ladder_is_unknown_never_ok() {
    for value in ["0", "1", "2", "3", "x"] {
        let reading = Reading::Ok(CompleteReading {
            value: Decimal(value.into()),
        });
        let graded = grade(reading, None);
        assert_eq!(
            graded.verdict,
            Verdict::UNKNOWN,
            "value {value}: {graded:?}"
        );
        assert_eq!(graded.score, None, "value {value}: {graded:?}");
    }
    let cut = Reading::Unknown(UnreadReading {
        reason: "the result is cut".into(),
    });
    let graded = grade(cut, None);
    assert_eq!(
        (graded.verdict, graded.reason.as_str()),
        (Verdict::UNKNOWN, "the result is cut")
    );
}

/// Design § 2, D5: a mapping revision re-grades stored history. One stored sample, `red`, graded
/// at revision 1 (`red: CRIT`) and at revision 2 (`red: WARN`, `purple: OK`), takes each
/// revision's verdict, and the reason names the revision graded.
#[test]
fn a_mapping_revision_re_grades_the_same_sample() {
    let rev1 = definition();
    let mut rev2 = definition();
    rev2.rev = 2;
    let table = rev2.evaluation.mapping.as_mut().expect("a table");
    table.insert("red".to_owned(), Verdict::WARN);
    table.insert("purple".to_owned(), Verdict::OK);

    for (text, at_rev1, at_rev2) in [
        ("red", Verdict::CRIT, Verdict::WARN),
        ("purple", Verdict::UNKNOWN, Verdict::OK),
    ] {
        let first = grade_at(&rev1, text_cell(text), Some(text));
        let again = grade_at(&rev2, text_cell(text), Some(text));
        assert_eq!(first.verdict, at_rev1, "`{text}` at rev 1: {first:?}");
        assert_eq!(again.verdict, at_rev2, "`{text}` at rev 2: {again:?}");
        assert!(first.reason.contains("rev 1"), "{first:?}");
        assert!(again.reason.contains("rev 2"), "{again:?}");
    }
}

/// Every other rule ignores `text`: a threshold sample grades the same with or without it.
#[test]
fn other_rules_ignore_the_sample_text() {
    let mut threshold = definition();
    threshold.evaluation.rule = RuleId("higher_is_worse".into());
    threshold.evaluation.mapping = None;
    let line = |abs: &str| Threshold {
        x: None,
        abs: Some(Decimal(abs.into())),
        min_events: None,
    };
    threshold.evaluation.warn = Some(line("1"));
    threshold.evaluation.crit = Some(line("2"));
    for value in ["0", "1", "5"] {
        let reading = || {
            Reading::Ok(CompleteReading {
                value: Decimal(value.into()),
            })
        };
        assert_eq!(
            grade_at(&threshold, reading(), Some("red")),
            grade_at(&threshold, reading(), None),
            "value {value}"
        );
    }
}

// ---------------------------------------------------------------------------------------------
// At load, and in ProposeDefinition and ReviseDefinition.
// ---------------------------------------------------------------------------------------------

/// A definition file for a blocking gate's exit code, with `mapping` as the last line of its
/// `evaluation` block, or no mapping when `mapping` is empty.
fn definition_file(mapping: &str) -> String {
    format!(
        "id: {SIGNAL}\n\
         title: Exit code of the blocking gate\n\
         family: gate\n\
         kind: gauge\n\
         unit: \"{{verdict}}\"\n\
         dimensions: []\n\
         source: {{ adapter: sql, connection: ci-results }}\n\
         extractor: sql.value\n\
         query: {{ value: select exit_code from gate_runs }}\n\
         window: PT1H\n\
         cadence: PT1H\n\
         history: store\n\
         denominator: {{ unit: \"{{run}}\", min_n: 0, since: 2026-01-01T00:00:00Z }}\n\
         evaluation:\n\
         \x20 rule: {RULE}\n\
         \x20 baselines: [fixed]\n\
         \x20 fixed: \"0\"\n\
         {mapping}\
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

fn refused_as(text: &str, rule: &str) {
    let errors = load(text).err().unwrap_or_else(|| panic!("loads:\n{text}"));
    assert!(
        errors.iter().any(|error| error.rule.name() == rule),
        "refused, but not as {rule}: {errors:?}"
    );
}

#[test]
fn producer_mapped_without_mapping_is_refused_at_load() {
    refused_as(&definition_file(""), MAPPING_MISSING);
}

/// The loader reads `evaluation.mapping` into the model: producer value to ladder verdict.
#[test]
fn mapping_is_read_from_the_definition_file() {
    let loaded = load(&definition_file(
        "  mapping: { \"0\": OK, \"1\": CRIT, \"2\": UNKNOWN, yellow: WARN }\n",
    ))
    .unwrap_or_else(|errors| panic!("refused: {errors:?}"));
    assert_eq!(
        loaded.data().evaluation.mapping,
        Some(BTreeMap::from([
            ("0".to_owned(), Verdict::OK),
            ("1".to_owned(), Verdict::CRIT),
            ("2".to_owned(), Verdict::UNKNOWN),
            ("yellow".to_owned(), Verdict::WARN),
        ]))
    );
}

/// A mapping naming anything other than a ladder verdict is refused at load.
#[test]
fn mapping_naming_no_ladder_verdict_is_refused_at_load() {
    for name in ["GREEN", "ok", "NA", "3", "\"\""] {
        let text = definition_file(&format!("  mapping: {{ green: {name} }}\n"));
        assert!(load(&text).is_err(), "`{name}` loads as a verdict");
    }
}

/// A producer value maps onto the ladder at most once.
#[test]
fn mapping_listing_a_producer_value_twice_is_refused_at_load() {
    let text = definition_file("  mapping: { green: OK, green: WARN }\n");
    assert!(load(&text).is_err(), "a value mapped twice loads");
}

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
        denominator: Some(DenominatorSpec {
            unit: Unit("{run}".into()),
            min_n: 0,
            since: "first_sample".into(),
        }),
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
        unit: Unit("{verdict}".into()),
        dimensions: Vec::<DimensionKey>::new(),
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

fn refusal() -> MappingMissing {
    MappingMissing {
        signal_id: SignalId(SIGNAL.into()),
    }
}

/// The loader, `ProposeDefinition` and `ReviseDefinition` refuse the same mappings: none, and one
/// naming `N/A` (`UNMAPPED:` on `mapping-missing` in `spec/domains/definition.yaml`): a producer
/// value is read, so it is never "too little to grade", and a reading carries no `N/A`.
#[test]
fn mapping_onto_n_a_is_refused_like_a_missing_one() {
    refused_as(
        &definition_file("  mapping: { \"0\": OK, \"1\": N/A }\n"),
        MAPPING_MISSING,
    );

    let mut onto_n_a = colours();
    onto_n_a.insert("grey".to_owned(), Verdict::NA);
    for mapping in [None, Some(onto_n_a)] {
        let mut generated = ports();
        assert_eq!(
            ProposeDefinitionBehavior::propose_definition(
                &mut generated,
                propose(evaluation(mapping.clone()))
            )
            .expect("propose answers"),
            ProposeDefinitionOutcome::MappingMissing { error: refusal() },
            "propose, mapping {mapping:?}"
        );
        assert_eq!(
            ReviseDefinitionBehavior::revise_definition(
                &mut generated,
                revise(evaluation(mapping.clone()))
            )
            .expect("revise answers"),
            ReviseDefinitionOutcome::MappingMissing { error: refusal() },
            "revise, mapping {mapping:?}"
        );
    }

    let mut generated = ports();
    assert!(matches!(
        ProposeDefinitionBehavior::propose_definition(
            &mut generated,
            propose(evaluation(Some(colours())))
        )
        .expect("propose answers"),
        ProposeDefinitionOutcome::Proposed { .. }
    ));
}

/// An empty table maps nothing, so every read would grade `UNKNOWN` forever: the loader,
/// `ProposeDefinition` and `ReviseDefinition` refuse it as `mapping-missing` (`UNMAPPED:` on
/// `mapping-missing` in `spec/domains/definition.yaml`). A table whose every value is `UNKNOWN`
/// is an explicit choice and loads and proposes.
#[test]
fn empty_mapping_is_refused_like_a_missing_one() {
    refused_as(&definition_file("  mapping: {}\n"), MAPPING_MISSING);

    let mut generated = ports();
    assert_eq!(
        ProposeDefinitionBehavior::propose_definition(
            &mut generated,
            propose(evaluation(Some(BTreeMap::new())))
        )
        .expect("propose answers"),
        ProposeDefinitionOutcome::MappingMissing { error: refusal() },
    );
    assert_eq!(
        ReviseDefinitionBehavior::revise_definition(
            &mut generated,
            revise(evaluation(Some(BTreeMap::new())))
        )
        .expect("revise answers"),
        ReviseDefinitionOutcome::MappingMissing { error: refusal() },
    );

    load(&definition_file(
        "  mapping: { \"0\": UNKNOWN, \"1\": UNKNOWN }\n",
    ))
    .unwrap_or_else(|errors| panic!("an all-UNKNOWN table is refused: {errors:?}"));
    let unknown_only = BTreeMap::from([("0".to_owned(), Verdict::UNKNOWN)]);
    assert!(matches!(
        ProposeDefinitionBehavior::propose_definition(
            &mut ports(),
            propose(evaluation(Some(unknown_only)))
        )
        .expect("propose answers"),
        ProposeDefinitionOutcome::Proposed { .. }
    ));
}

/// A mapping is the input of `producer_mapped` only. Under any other rule a definition without
/// one loads and proposes as before, and one with a mapping is refused as `mapping-unused`, as
/// `fixed-number-unused` and `rolling-n-unused` refuse a number nothing reads.
#[test]
fn a_mapping_under_any_other_rule_is_refused_as_unused() {
    let threshold_file = |mapping: &str| {
        definition_file(mapping)
            .replace(RULE, "higher_is_worse")
            .replace(
                "  fixed: \"0\"\n",
                "  fixed: \"0\"\n  warn: { abs: \"1\" }\n  crit: { abs: \"2\" }\n",
            )
    };
    load(&threshold_file("")).unwrap_or_else(|errors| panic!("refused: {errors:?}"));
    refused_as(
        &threshold_file("  mapping: { green: OK }\n"),
        MAPPING_UNUSED,
    );

    let unused = MappingUnused {
        signal_id: SignalId(SIGNAL.into()),
    };
    for rule in [
        "ratio_vs_pooled_baseline",
        "higher_is_worse",
        "lower_is_worse",
        "threshold",
    ] {
        let mut without = evaluation(None);
        without.rule = RuleId(rule.into());
        assert!(
            matches!(
                ProposeDefinitionBehavior::propose_definition(&mut ports(), propose(without))
                    .expect("propose answers"),
                ProposeDefinitionOutcome::Proposed { .. }
            ),
            "{rule} without a mapping"
        );

        let mut with = evaluation(Some(colours()));
        with.rule = RuleId(rule.into());
        let mut generated = ports();
        assert_eq!(
            ProposeDefinitionBehavior::propose_definition(&mut generated, propose(with.clone()))
                .expect("propose answers"),
            ProposeDefinitionOutcome::MappingUnused {
                error: unused.clone()
            },
            "{rule} with a mapping"
        );
        assert_eq!(
            ReviseDefinitionBehavior::revise_definition(&mut generated, revise(with))
                .expect("revise answers"),
            ReviseDefinitionOutcome::MappingUnused {
                error: unused.clone()
            },
            "{rule} with a mapping"
        );
    }
}

/// `producer_mapped` grades by `evaluation.mapping` alone, so a `warn` or `crit` line under it is a
/// number nothing reads: the loader, `ProposeDefinition` and `ReviseDefinition` refuse it as
/// `thresholds-unused`, as `mapping-unused` refuses the inverse.
#[test]
fn thresholds_under_producer_mapped_are_refused_as_unused() {
    let line = |abs: &str| Threshold {
        x: None,
        abs: Some(Decimal(abs.into())),
        min_events: None,
    };
    let unused = ThresholdsUnused {
        signal_id: SignalId(SIGNAL.into()),
    };
    for lines in [
        "  warn: { abs: \"1\" }\n",
        "  crit: { abs: \"2\" }\n",
        "  warn: { abs: \"1\" }\n  crit: { abs: \"2\" }\n",
    ] {
        let text = definition_file(&format!("  mapping: {{ green: OK }}\n{lines}"));
        refused_as(&text, THRESHOLDS_UNUSED);
    }
    for (warn, crit) in [
        (Some(line("1")), None),
        (None, Some(line("2"))),
        (Some(line("1")), Some(line("2"))),
    ] {
        let mut with = evaluation(Some(colours()));
        with.warn = warn;
        with.crit = crit;
        let mut generated = ports();
        assert_eq!(
            ProposeDefinitionBehavior::propose_definition(&mut generated, propose(with.clone()))
                .expect("propose answers"),
            ProposeDefinitionOutcome::ThresholdsUnused {
                error: unused.clone()
            },
            "{with:?}"
        );
        assert_eq!(
            ReviseDefinitionBehavior::revise_definition(&mut generated, revise(with.clone()))
                .expect("revise answers"),
            ReviseDefinitionOutcome::ThresholdsUnused {
                error: unused.clone()
            },
            "{with:?}"
        );
    }
}
