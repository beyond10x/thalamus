//! Adversary pass 1 on `story:m1-instance-registry`: cases the unit's own suite does not state.
//!
//! Each case drives `thalamus_core::registry` from the specification, the design or a consumer
//! of what it loads, and fails while the loader admits what that source refuses.

use thalamus_core::registry::{Definition, FileValues, Instance, LoadError, Registry};
use thalamus_model::behaviour::Generated;
use thalamus_model::definition::obligations::RetireDefinitionBehavior;
use thalamus_model::definition::{
    Author, BaselineKind, History, QueryText, RetireDefinition, RetireDefinitionOutcome,
    SignalDefinitionState, SignalId, SignalKind,
};
use thalamus_model::dimension::DimensionRule;
use thalamus_model::json::Value as Json;

const REGISTRY_FILE: &str = "dimensions.yaml";
const REGISTRY: &str = include_str!("fixtures/instance/dimensions.yaml");
const RATIO_FILE: &str = "definitions/service.api.request_failure_ratio.yaml";
const RATIO: &str =
    include_str!("fixtures/instance/definitions/service.api.request_failure_ratio.yaml");
const CASE_FILE: &str = "definitions/case.yaml";

fn registry() -> Registry {
    Registry::parse(REGISTRY_FILE, REGISTRY).expect("the fixture registry loads")
}

/// `text` with the top-level entry `key: …` replaced by `line`.
fn with(text: &str, key: &str, line: &str) -> String {
    let prefix = format!("{key}:");
    let mut skipping = false;
    let mut out = String::new();
    for row in text.lines() {
        if skipping && row.starts_with(' ') {
            continue;
        }
        skipping = false;
        if row.starts_with(&prefix) {
            skipping = true;
            out.push_str(line);
        } else {
            out.push_str(row);
        }
        out.push('\n');
    }
    out
}

fn load(text: &str) -> Result<Definition, Vec<LoadError>> {
    Definition::parse(CASE_FILE, text, &registry())
}

fn assert_refused(text: &str, why: &str) {
    if let Ok(definition) = load(text) {
        panic!(
            "{why}: loaded {:?} from\n{text}",
            definition.data().signal_id.0
        );
    }
}

fn registry_text(dimensions: &str) -> String {
    format!("registry_id: example\ntitle: Example\ndimensions:\n{dimensions}")
}

/// Design § 2.4: the registry is closed, one entry per key; the spec's `RegisterDimension`
/// answers `DimensionExists` for a key already registered. A YAML mapping that names a key twice
/// must not load as whichever entry came last.
#[test]
fn adv_registry_key_named_twice_is_refused() {
    let text = registry_text(
        "  component: { registry: file:registries/components.yaml }\n  \
         component: { registry: open }\n",
    );
    match Registry::parse(REGISTRY_FILE, &text) {
        Ok(registry) => panic!(
            "a key named twice loaded as {:?}",
            registry.dimension("component").map(|d| &d.values)
        ),
        Err(errors) => assert!(!errors.is_empty()),
    }
}

/// The query map is hashed into every sample's provenance (design § 2.2); a name given twice must
/// not silently drop one of the two texts.
#[test]
fn adv_query_name_given_twice_is_refused() {
    let text = with(
        RATIO,
        "query",
        "query:\n  numerator: sum(a)\n  denominator: sum(b)\n  numerator: sum(c)",
    );
    assert_refused(&text, "a query name given twice");
}

/// `source.connection` is "an identifier, never a URL" (story Work; the loader's own message).
/// A WHATWG URL parser reads `https:host/path` and `https:/host/path` as
/// `https://host/path`, and `file:/path` is an RFC 8089 file URL.
#[test]
fn adv_connection_url_without_double_slash_is_refused() {
    for connection in [
        "https:metrics.example/api",
        "https:/metrics.example/api",
        "file:/var/run/metrics.sock",
    ] {
        assert_refused(
            &with(
                RATIO,
                "source",
                &format!("source: {{ adapter: prometheus, connection: \"{connection}\" }}"),
            ),
            connection,
        );
    }
}

/// Design § 2.1: the id is `<family>.<layer>.<name>`, and the example's `family: platform`
/// is the first segment of `platform.namespace.request_failure_ratio`.
#[test]
fn adv_signal_id_first_segment_is_the_family() {
    assert_refused(
        &with(RATIO, "family", "family: platform"),
        "id service.api.request_failure_ratio under family platform",
    );
}

/// The adapters read `definition.window` through `thalamus_sources::adapter::window_ending`, which
/// refuses years, months and a length past `i64` seconds. A definition the loader admits with such
/// a window is `Active` and reads unknown on every pass, the outcome the story's baseline
/// addition exists to prevent.
#[test]
fn adv_window_no_adapter_reads_is_refused() {
    for window in ["P1M", "P1Y", "P1Y2M", "99999999999999999999h"] {
        assert_refused(
            &with(RATIO, "window", &format!("window: \"{window}\"")),
            window,
        );
    }
}

/// A cadence of zero samples without pause and a window of zero holds nothing; neither is a
/// duration a definition can mean.
#[test]
fn adv_zero_window_and_cadence_are_refused() {
    for field in ["window", "cadence"] {
        for zero in ["0h", "PT0S", "P0D"] {
            assert_refused(
                &with(RATIO, field, &format!("{field}: \"{zero}\"")),
                &format!("{field} {zero}"),
            );
        }
    }
}

/// The spec's `RegisterDimension` refuses a parent that is not registered yet
/// (`thalamus.dimension.UnknownParent`), so no sequence of commands builds a key that is its own
/// parent or two keys that are each other's. The file must not load what the commands cannot.
#[test]
fn adv_dimension_parent_cycle_is_refused() {
    for dimensions in [
        "  component: { registry: open, parent: component }\n",
        "  cluster: { registry: open, parent: namespace }\n  \
         namespace: { registry: open, parent: cluster }\n",
    ] {
        if Registry::parse(REGISTRY_FILE, &registry_text(dimensions)).is_ok() {
            panic!("a parent cycle loaded:\n{dimensions}");
        }
    }
}

/// `thalamus.dimension.ValueSource`: "file a registry file in the instance"; design § 5.2: an
/// instance is a directory. A locator that leaves the directory is not a file in the instance.
#[test]
fn adv_file_locator_outside_the_instance_is_refused() {
    for locator in [
        "/etc/passwd",
        "../other-instance/registries/components.yaml",
    ] {
        let text = registry_text(&format!(
            "  component: {{ registry: \"file:{locator}\" }}\n"
        ));
        if Registry::parse(REGISTRY_FILE, &text).is_ok() {
            panic!("file:{locator} loaded");
        }
    }
}

/// `thalamus_model::json::push_value` writes a `Number` as spelled: "a caller building one by hand
/// owns its spelling". The loader builds the objective by hand, so every number it builds must be
/// one the model's JSON reader reads back.
#[test]
fn adv_objective_numbers_are_json_numbers() {
    for number in [".nan", ".inf", "-.inf"] {
        let text = with(
            RATIO,
            "objective",
            &format!("objective: {{ target: {number} }}"),
        );
        let Ok(definition) = load(&text) else {
            continue;
        };
        let objective = definition
            .data()
            .objective
            .as_ref()
            .expect("the objective is kept");
        let mut written = String::new();
        thalamus_model::json::push_value(&mut written, objective);
        assert!(
            thalamus_model::json::parse(&written).is_ok(),
            "objective {number} loaded and is written as {written}, which is not JSON"
        );
    }
}

/// The model's JSON reader refuses a document nested past 64 levels (`thalamus_model::json`,
/// `DEPTH`); the YAML reader stops near 128. An objective between the two loads and is written
/// as JSON the model cannot read back.
#[test]
fn adv_objective_nesting_reads_back() {
    let depth = 100;
    let text = with(
        RATIO,
        "objective",
        &format!("objective: {}1{}", "[".repeat(depth), "]".repeat(depth)),
    );
    let Ok(definition) = load(&text) else {
        return;
    };
    let objective = definition
        .data()
        .objective
        .as_ref()
        .expect("the objective is kept");
    let mut written = String::new();
    thalamus_model::json::push_value(&mut written, objective);
    if let Err(error) = thalamus_model::json::parse(&written) {
        panic!("an objective nested {depth} deep loaded, and its JSON is refused: {error}");
    }
}

/// Pins today's behaviour of a definition command over the file-backed ports (coordinator
/// decision on adversary pass 1, finding 9). The definition file is the one copy, lifecycle
/// included (design § 11 D19), and the generated storage port's `put` cannot fail, so the
/// generated `RetireDefinition` answers `Retired` while the write is a no-op recorded in
/// `refused_writes()`. Never routing a definition command to an `Instance`, and answering it with
/// a D19 refusal, is `story:m4-service-api`'s.
#[test]
fn adv_retire_through_the_file_ports_answers_retired_and_records_the_refused_write() {
    let instance = Instance::load((REGISTRY_FILE, REGISTRY), &[(RATIO_FILE, RATIO)])
        .expect("the instance loads");
    let mut generated = Generated::new(instance);
    let outcome = generated
        .retire_definition(RetireDefinition {
            signal_id: SignalId("service.api.request_failure_ratio".into()),
        })
        .expect("a declared outcome");
    assert!(
        matches!(outcome, RetireDefinitionOutcome::Retired { .. }),
        "{outcome:?}"
    );
    let instance = generated.ports;
    assert_eq!(
        instance
            .definition("service.api.request_failure_ratio")
            .map(Definition::state),
        Some(SignalDefinitionState::Active),
        "the file still says Active"
    );
    let refused = instance.refused_writes();
    assert_eq!(refused.len(), 1, "{refused:#?}");
    assert_eq!(refused[0].entity, "thalamus.definition.SignalDefinition");
    assert_eq!(refused[0].identity, "service.api.request_failure_ratio");
    assert_eq!(refused[0].operation, "put");
    assert!(refused[0].to_string().contains("D19"), "{}", refused[0]);
}

// Rules the unit's suite states no case for: each mutant that disables one (adversary pass 1,
// mutants m01 to m07) leaves `tests/registry.rs` green. These cases pass on the tree and fail on
// the mutant.

fn assert_registry_refused(text: &str, why: &str) {
    if Registry::parse(REGISTRY_FILE, text).is_ok() {
        panic!("{why}: loaded\n{text}");
    }
}

fn assert_values_refused(text: &str, why: &str) {
    if FileValues::parse("registries/case.yaml", text).is_ok() {
        panic!("{why}: loaded\n{text}");
    }
}

#[test]
fn adv_character_sets_are_held() {
    assert_refused(&with(RATIO, "family", "family: Service"), "family Service");
    assert_refused(
        &with(
            RATIO,
            "source",
            "source: { adapter: Prom-QL, connection: metrics-main }",
        ),
        "adapter Prom-QL",
    );
    assert_registry_refused(
        "registry_id: Example Instance\ntitle: t\ndimensions:\n  suite: { registry: open }\n",
        "registry_id with a space",
    );
    assert_registry_refused(
        &registry_text("  Suite: { registry: open }\n"),
        "dimension key Suite",
    );
}

#[test]
fn adv_timestamps_are_held() {
    assert_refused(
        &with(RATIO, "revised_at", "revised_at: yesterday"),
        "revised_at yesterday",
    );
    assert_values_refused(
        "- value: checkout\n  created: 2025-02-30T00:00:00Z\n",
        "created on 30 February",
    );
}

#[test]
fn adv_decimals_are_held() {
    assert_refused(
        &with(
            RATIO,
            "evaluation",
            "evaluation: { rule: ratio_vs_pooled_baseline, baselines: [pooled, offset_1d, \
             offset_7d], warn: { x: 2e1 } }",
        ),
        "warn.x 2e1",
    );
}

#[test]
fn adv_unknown_parent_is_refused() {
    assert_registry_refused(
        &registry_text("  namespace: { registry: open, parent: cluster }\n"),
        "parent cluster is not registered",
    );
}

#[test]
fn adv_value_listed_twice_is_refused() {
    assert_values_refused("- checkout\n- value: checkout\n", "checkout listed twice");
}

#[test]
fn adv_rev_below_one_is_refused() {
    assert_refused(&with(RATIO, "rev", "rev: 0"), "rev 0");
}

#[test]
fn adv_dimension_declared_twice_is_refused() {
    assert_refused(
        &with(
            RATIO,
            "dimensions",
            "dimensions: [component, environment, component]",
        ),
        "component declared twice",
    );
}

// Field mappings the unit's suite does not read back: each mutant that maps one field to another
// value (mutants m11 to m21) leaves `tests/registry.rs` green.

#[test]
fn adv_every_field_loads_as_written() {
    let instance = Instance::load((REGISTRY_FILE, REGISTRY), &[(RATIO_FILE, RATIO)])
        .expect("the instance loads");
    let endpoint = instance
        .registry()
        .dimension("endpoint")
        .expect("registered");
    assert_eq!(endpoint.rule, DimensionRule::SourceOnly);
    let component = instance
        .registry()
        .dimension("component")
        .expect("registered");
    assert_eq!(component.otel_attribute.as_deref(), Some("service.name"));
    assert_eq!(component.rule, DimensionRule::Derived);

    let data = instance
        .definition("service.api.request_failure_ratio")
        .expect("loaded")
        .data()
        .clone();
    assert_eq!(data.kind, SignalKind::Ratio);
    assert_eq!(data.history, History::Source);
    assert_eq!(data.denominator.as_ref().map(|d| d.min_n), Some(20));
    assert_eq!(
        data.evaluation.warn.as_ref().and_then(|w| w.min_events),
        Some(10)
    );
    assert_eq!(
        data.query.get("numerator"),
        Some(&QueryText(
            "sum by (component, environment) (increase(requests_failed_total[6h]))".into()
        ))
    );
    assert_eq!(data.links.components, ["api"]);
    assert_eq!(data.revised_by, Author("operator".into()));

    let counted = load(&with(
        &with(
            &with(
                &with(RATIO, "kind", "kind: count_of_cases"),
                "unit",
                "unit: \"{case}\"",
            ),
            "evaluation",
            "evaluation: { rule: threshold, baselines: [lower_of, offset_1d, offset_7d] }",
        ),
        "objective",
        "objective: { budgeting: true }",
    ))
    .unwrap_or_else(|errors| panic!("{errors:#?}"));
    assert_eq!(counted.data().kind, SignalKind::CountOfCases);
    assert_eq!(
        counted.data().evaluation.baselines,
        [
            BaselineKind::LowerOf,
            BaselineKind::Offset1d,
            BaselineKind::Offset7d
        ]
    );
    assert_eq!(
        counted.data().objective,
        Some(Json::Object(vec![("budgeting".into(), Json::Bool(true))]))
    );
}
