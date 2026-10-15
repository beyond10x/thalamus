//! Adversary pass 2, `story:m3-vocabulary-mapping`, the `thalamus-core` half: every YAML form of
//! "no mapping" against D44, loader parity with `ProposeDefinition` under another rule, the key
//! text a table keeps, and a `partial` reading under `producer_mapped`.

use std::collections::BTreeMap;

use thalamus_core::grading::{self, REASON_PARTIAL};
use thalamus_core::registry::{Definition, LoadError, Registry};
use thalamus_model::evaluation::{Baseline, BaselineLabel, Verdict};
use thalamus_model::observation::{
    Bound, CompleteReading, PartialReading, Provenance, QueryDigest, Reading, RunId, SampleId,
    SeriesKey, SignalSampleData, TimeWindow,
};
use thalamus_model::primitives::{Decimal, Timestamp};

const REGISTRY: &str = include_str!("fixtures/instance/dimensions.yaml");
const MAPPING_MISSING: &str = "thalamus.definition.MappingMissing";
const MAPPING_UNUSED: &str = "thalamus.definition.MappingUnused";

/// A blocking gate's definition file under `rule`, `extra` the last line(s) of its `evaluation`
/// block. A threshold rule carries `warn` and `crit` lines; `producer_mapped` carries none, which
/// `thresholds-unused` would refuse (coordinator decision on pass-2 finding 5).
fn definition_file(rule: &str, extra: &str) -> String {
    let lines = if rule == "producer_mapped" {
        ""
    } else {
        "  warn: { abs: \"1\" }\n  crit: { abs: \"2\" }\n"
    };
    format!(
        "id: gate.ci.blocking_exit\n\
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
         \x20 rule: {rule}\n\
         \x20 baselines: [fixed]\n\
         \x20 fixed: \"0\"\n\
         {lines}\
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

fn rules_of(text: &str) -> Vec<&'static str> {
    match load(text) {
        Ok(_) => Vec::new(),
        Err(errors) => errors.iter().map(|error| error.rule.name()).collect(),
    }
}

/// D44: "`mapping-missing` refuses ... no mapping (a bare `mapping:` or YAML null included, as
/// JSON null on the wire)". Every spelling YAML 1.2 gives null is refused as `mapping-missing`,
/// never as a parse error.
#[test]
fn adv_every_yaml_null_mapping_is_refused_as_mapping_missing() {
    for null in ["", " ~", " null", " Null", " NULL"] {
        let text = definition_file("producer_mapped", &format!("  mapping:{null}\n"));
        let rules = rules_of(&text);
        assert!(
            rules.contains(&MAPPING_MISSING),
            "`mapping:{null}` is refused as {rules:?}, not {MAPPING_MISSING}"
        );
    }
}

/// Parity with `ProposeDefinition`, where JSON null under another rule is "not defined": a null
/// mapping under `higher_is_worse` loads as no mapping, an empty one is `mapping-unused`, and one
/// onto `N/A` is `mapping-unused` and not `mapping-missing`, which names `producer_mapped`.
#[test]
fn adv_null_mapping_under_another_rule_is_absent_and_empty_one_unused() {
    for null in ["", " ~", " null"] {
        let text = definition_file("higher_is_worse", &format!("  mapping:{null}\n"));
        let loaded = load(&text).unwrap_or_else(|errors| panic!("`mapping:{null}`: {errors:?}"));
        assert_eq!(loaded.data().evaluation.mapping, None, "`mapping:{null}`");
    }
    for table in ["{}", "{ \"1\": N/A }"] {
        let text = definition_file("higher_is_worse", &format!("  mapping: {table}\n"));
        let rules = rules_of(&text);
        assert_eq!(rules, vec![MAPPING_UNUSED], "`mapping: {table}`");
    }
}

/// D44: a producer value is "matched exactly as the source printed it", and the loader keeps a
/// key's text. A plain YAML key that YAML 1.2 would type as a float, a hex integer, a boolean or
/// a zero-padded integer is still the text written, so `1.0` never silently matches a producer's
/// `1`.
#[test]
fn adv_plain_yaml_keys_keep_the_text_written() {
    let loaded = load(&definition_file(
        "producer_mapped",
        "  mapping: { 1.0: OK, 0x1: WARN, true: CRIT, 007: UNKNOWN, 1e3: OK }\n",
    ))
    .unwrap_or_else(|errors| panic!("refused: {errors:?}"));
    assert_eq!(
        loaded.data().evaluation.mapping,
        Some(BTreeMap::from([
            ("1.0".to_owned(), Verdict::OK),
            ("0x1".to_owned(), Verdict::WARN),
            ("true".to_owned(), Verdict::CRIT),
            ("007".to_owned(), Verdict::UNKNOWN),
            ("1e3".to_owned(), Verdict::OK),
        ]))
    );
}

fn sample(reading: Reading) -> SignalSampleData {
    SignalSampleData {
        text: None,
        sample_id: SampleId("smp_1".into()),
        signal: thalamus_model::definition::SignalId("gate.ci.blocking_exit".into()),
        series: SeriesKey("gate.ci.blocking_exit".into()),
        dims: BTreeMap::new(),
        ts: Timestamp("2026-10-14T22:00:00Z".into()),
        window: TimeWindow {
            start: Timestamp("2026-10-14T21:00:00Z".into()),
            end: Timestamp("2026-10-14T22:00:00Z".into()),
        },
        reading,
        unit: thalamus_model::definition::Unit("{verdict}".into()),
        denominator: None,
        provenance: Provenance {
            producer: "thalamus-service".into(),
            version: "0.1.0".into(),
            definition_rev: 1,
            adapter: thalamus_model::definition::AdapterId("sql".into()),
            connection: thalamus_model::definition::ConnectionId("ci-results".into()),
            query_sha256: QueryDigest("00".into()),
            run_id: RunId("run_1".into()),
        },
    }
}

/// Grading step 4 (`grading.rs`): a `partial` reading may escalate but never clear. Under
/// `producer_mapped` a partial reading is no producer value (`map_reads` keeps it as the adapter
/// made it), so a bound that happens to equal a position, `0` (OK) or `2` (CRIT), is never that
/// verdict: it is `UNKNOWN`, `partial read`, with no score. Holds the `Prepared::Mapped` arm of
/// `Prepared::escalate` at `None`; nothing else in the suite grades a partial mapped reading.
#[test]
fn adv_partial_reading_under_producer_mapped_is_unknown_never_a_position() {
    let loaded = load(&definition_file(
        "producer_mapped",
        "  mapping: { \"0\": OK, \"1\": WARN, \"2\": CRIT }\n",
    ))
    .unwrap_or_else(|errors| panic!("refused: {errors:?}"));
    let fixed = Baseline {
        kind: BaselineLabel("fixed".into()),
        offsets: Vec::new(),
        reading: Reading::Ok(CompleteReading {
            value: Decimal("0".into()),
        }),
    };
    for value in ["0", "1", "2"] {
        for bound in [Bound::Lower, Bound::Upper] {
            let reading = Reading::Partial(PartialReading {
                value: Decimal(value.into()),
                bound,
            });
            let graded = grading::grade(loaded.data(), &sample(reading), &fixed)
                .expect("a producer_mapped definition grades");
            assert_eq!(
                (graded.verdict, graded.score.clone(), graded.reason.as_str()),
                (Verdict::UNKNOWN, None, REASON_PARTIAL),
                "partial {value} {bound:?}: {graded:?}"
            );
        }
    }
}
