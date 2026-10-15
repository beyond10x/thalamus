//! Adversary pass 1, `story:m3-vocabulary-mapping`, the `thalamus-core` half: the loader's reading
//! of `evaluation.mapping` against `ProposeDefinition`'s, and YAML key forms.

use std::collections::BTreeMap;

use thalamus_core::registry::{Definition, LoadError, Registry};
use thalamus_model::evaluation::Verdict;

const REGISTRY: &str = include_str!("fixtures/instance/dimensions.yaml");
const MAPPING_MISSING: &str = "thalamus.definition.MappingMissing";

/// A blocking gate's definition file, `mapping` the last line(s) of its `evaluation` block.
fn definition_file(mapping: &str) -> String {
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
         \x20 rule: producer_mapped\n\
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

/// `mapping:` with nothing after it is YAML null, the same "not defined" that `"mapping": null`
/// is on `ProposeDefinition`'s JSON wire, which `ProposeDefinition` refuses as `mapping-missing`
/// (`producer_mapped.rs::mapping_onto_n_a_is_refused_like_a_missing_one`, mapping `None`). The
/// loader must refuse it the same way. Beside it, `fixed:` with nothing after it loads as no
/// `fixed` at all, so in one `evaluation` block a bare key means "absent" for every optional
/// field but this one.
#[test]
fn adv_bare_mapping_key_is_refused_as_mapping_missing_like_json_null() {
    let text = definition_file("  mapping:\n");
    match load(&text) {
        Ok(loaded) => panic!(
            "`mapping:` (YAML null) loads, with mapping {:?}, so every read of this blocking \
             gate grades UNKNOWN forever; ProposeDefinition refuses the same definition as \
             mapping-missing",
            loaded.data().evaluation.mapping
        ),
        Err(errors) => assert!(
            errors
                .iter()
                .any(|error| error.rule.name() == MAPPING_MISSING),
            "refused, but not as {MAPPING_MISSING}: {errors:?}"
        ),
    }
}

/// YAML 1.2 reads an unquoted `1` as an integer; the loader keeps the key's text, so `1: CRIT`
/// and `"1": CRIT` are one producer value, and giving both is a duplicate.
#[test]
fn adv_unquoted_integer_keys_load_as_their_text() {
    let loaded = load(&definition_file(
        "  mapping: { 0: OK, 1: CRIT, 2: UNKNOWN }\n",
    ))
    .unwrap_or_else(|errors| panic!("refused: {errors:?}"));
    assert_eq!(
        loaded.data().evaluation.mapping,
        Some(BTreeMap::from([
            ("0".to_owned(), Verdict::OK),
            ("1".to_owned(), Verdict::CRIT),
            ("2".to_owned(), Verdict::UNKNOWN),
        ]))
    );
    let both = definition_file("  mapping: { 1: CRIT, \"1\": OK }\n");
    assert!(load(&both).is_err(), "`1` and `\"1\"` both load");
}
