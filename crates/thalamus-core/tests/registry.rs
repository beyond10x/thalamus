//! `story:m1-instance-registry`: an instance's definition files and dimension registry load and
//! validate, a sample is admitted only for an `Active` definition with `dims` the registry knows,
//! and `denominator.since` resolves from one of the three origins design § 11 D18 admits.
//!
//! The instance under `tests/fixtures/instance` uses neutral names only. Every refusal case is a
//! copy of one fixture definition with one field changed, so the field is what is refused.

use std::collections::BTreeMap;

use thalamus_core::behaviour::observation::{IdSequence, record_sample};
use thalamus_core::registry::{Definition, FileValues, Instance, LoadError, Registry, Rule};
use thalamus_core::series::{DimsRefusal, admit_dims, series_key};
use thalamus_core::since::{ADMITTED_FORMS, Since, SinceRefusal, resolve_since};
use thalamus_model::behaviour::{
    DimensionRegistryStorage, DimensionStorage, Generated, SignalDefinitionStorage,
    SignalSampleStorage,
};
use thalamus_model::definition::obligations::RetireDefinitionBehavior;
use thalamus_model::definition::{
    AdapterId, ConnectionId, RetireDefinition, SignalDefinitionState, SignalId, Unit,
};
use thalamus_model::dimension::{DimensionKey, DimensionValue, RegistryId};
use thalamus_model::observation::{
    CompleteReading, Provenance, QueryDigest, Reading, RecordSample, RecordSampleOutcome, RunId,
    SampleId, SeriesKey, SignalSampleSnapshot, TimeWindow,
};
use thalamus_model::primitives::{Decimal, Timestamp};

const REGISTRY_FILE: &str = "dimensions.yaml";
const REGISTRY: &str = include_str!("fixtures/instance/dimensions.yaml");
const COMPONENTS: &str = include_str!("fixtures/instance/registries/components.yaml");
const ENVIRONMENTS: &str = include_str!("fixtures/instance/registries/environments.yaml");

const RATIO_FILE: &str = "definitions/service.api.request_failure_ratio.yaml";
const RATIO: &str =
    include_str!("fixtures/instance/definitions/service.api.request_failure_ratio.yaml");
const MUTED_FILE: &str = "definitions/service.queue.depth.yaml";
const MUTED: &str = include_str!("fixtures/instance/definitions/service.queue.depth.yaml");
const PROPOSED_FILE: &str = "definitions/service.batch.duration.yaml";
const PROPOSED: &str = include_str!("fixtures/instance/definitions/service.batch.duration.yaml");
const RETIRED_FILE: &str = "definitions/test.run.pass_ratio.yaml";
const RETIRED: &str = include_str!("fixtures/instance/definitions/test.run.pass_ratio.yaml");

/// The file a refusal case is reported under.
const CASE_FILE: &str = "definitions/case.yaml";

fn registry() -> Registry {
    Registry::parse(REGISTRY_FILE, REGISTRY).expect("the fixture registry loads")
}

fn instance() -> Instance {
    Instance::load(
        (REGISTRY_FILE, REGISTRY),
        &[
            (RATIO_FILE, RATIO),
            (MUTED_FILE, MUTED),
            (PROPOSED_FILE, PROPOSED),
            (RETIRED_FILE, RETIRED),
        ],
    )
    .expect("the fixture instance loads")
}

/// The parsed `file:` registries of the fixture, by dimension key.
fn file_values() -> BTreeMap<String, FileValues> {
    BTreeMap::from([
        (
            "component".to_owned(),
            FileValues::parse("registries/components.yaml", COMPONENTS)
                .expect("the components registry loads"),
        ),
        (
            "environment".to_owned(),
            FileValues::parse("registries/environments.yaml", ENVIRONMENTS)
                .expect("the environments registry loads"),
        ),
    ])
}

/// `text` with the top-level entry `key: …` (and its indented continuation lines) replaced by
/// `line`, or removed when `line` is empty; appended when `text` has no such entry.
fn with(text: &str, key: &str, line: &str) -> String {
    let prefix = format!("{key}:");
    let mut found = false;
    let mut skipping = false;
    let mut out = String::new();
    for row in text.lines() {
        if skipping && row.starts_with(' ') {
            continue;
        }
        skipping = false;
        if row.starts_with(&prefix) {
            found = true;
            skipping = true;
            if !line.is_empty() {
                out.push_str(line);
                out.push('\n');
            }
        } else {
            out.push_str(row);
            out.push('\n');
        }
    }
    if !found && !line.is_empty() {
        out.push_str(line);
        out.push('\n');
    }
    out
}

fn load(text: &str) -> Result<Definition, Vec<LoadError>> {
    Definition::parse(CASE_FILE, text, &registry())
}

/// The refusal of `text`, which must name the case file and carry a rule `matches` accepts.
fn refused(text: &str, matches: impl Fn(&Rule) -> bool) -> Vec<LoadError> {
    let errors = match load(text) {
        Ok(definition) => panic!(
            "expected a refusal, loaded {:?}",
            definition.data().signal_id
        ),
        Err(errors) => errors,
    };
    assert!(
        errors.iter().any(|error| matches(&error.rule)),
        "no expected rule among {errors:#?}"
    );
    for error in &errors {
        assert_eq!(error.file, CASE_FILE);
        let shown = error.to_string();
        assert!(shown.contains(CASE_FILE), "{shown}");
        assert!(shown.contains(error.rule.name()), "{shown}");
    }
    errors
}

fn shown(errors: &[LoadError]) -> String {
    errors
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join("\n")
}

fn dims(pairs: &[(&str, &str)]) -> BTreeMap<String, DimensionValue> {
    pairs
        .iter()
        .map(|(key, value)| ((*key).to_owned(), DimensionValue((*value).to_owned())))
        .collect()
}

fn ts(text: &str) -> Timestamp {
    Timestamp(text.to_owned())
}

/// The fixture instance as the definition storage port, beside an in-memory sample store.
struct Ports {
    instance: Instance,
    samples: BTreeMap<String, SignalSampleSnapshot>,
    next: u64,
}

impl SignalDefinitionStorage for Ports {
    fn get(
        &self,
        identity: &SignalId,
    ) -> Option<thalamus_model::definition::SignalDefinitionSnapshot> {
        SignalDefinitionStorage::get(&self.instance, identity)
    }

    fn put(&mut self, snapshot: thalamus_model::definition::SignalDefinitionSnapshot) {
        SignalDefinitionStorage::put(&mut self.instance, snapshot);
    }

    fn delete(&mut self, identity: &SignalId) {
        SignalDefinitionStorage::delete(&mut self.instance, identity);
    }

    fn list(&self) -> Vec<thalamus_model::definition::SignalDefinitionSnapshot> {
        SignalDefinitionStorage::list(&self.instance)
    }
}

impl SignalSampleStorage for Ports {
    fn get(&self, identity: &SampleId) -> Option<SignalSampleSnapshot> {
        self.samples.get(&identity.0).cloned()
    }

    fn put(&mut self, snapshot: SignalSampleSnapshot) {
        self.samples
            .insert(snapshot.data.sample_id.0.clone(), snapshot);
    }

    fn delete(&mut self, identity: &SampleId) {
        self.samples.remove(&identity.0);
    }

    fn list(&self) -> Vec<SignalSampleSnapshot> {
        self.samples.values().cloned().collect()
    }
}

impl IdSequence for Ports {
    fn next_number(&mut self, _prefix: &'static str) -> Option<u64> {
        self.next += 1;
        Some(self.next)
    }
}

fn record(ports: &mut Ports, signal: &str, pairs: &[(&str, &str)]) -> RecordSampleOutcome {
    let signal = SignalId(signal.to_owned());
    let dims = dims(pairs);
    let input = RecordSample {
        series: series_key(&signal, &dims),
        signal,
        dims,
        ts: ts("2026-10-14T22:57:03Z"),
        window: TimeWindow {
            start: ts("2026-10-14T16:57:03Z"),
            end: ts("2026-10-14T22:57:03Z"),
        },
        reading: Reading::Ok(CompleteReading {
            value: Decimal("0.029".into()),
        }),
        unit: Unit("1".into()),
        denominator: None,
        provenance: Provenance {
            producer: "thalamus-test".into(),
            version: "0.1.0".into(),
            definition_rev: 1,
            adapter: AdapterId("prometheus".into()),
            connection: ConnectionId("metrics-main".into()),
            query_sha256: QueryDigest("00".into()),
            run_id: RunId("run_1".into()),
        },
    };
    record_sample(ports, input).expect("every input selects a declared outcome")
}

fn ports() -> Ports {
    Ports {
        instance: instance(),
        samples: BTreeMap::new(),
        next: 0,
    }
}

#[test]
fn sample_of_non_active_definition_is_refused() {
    let mut ports = ports();
    for (signal, state) in [
        ("service.queue.depth", SignalDefinitionState::Muted),
        ("service.batch.duration", SignalDefinitionState::Proposed),
        ("test.run.pass_ratio", SignalDefinitionState::Retired),
    ] {
        let held = SignalDefinitionStorage::get(&ports.instance, &SignalId(signal.into()))
            .expect("the fixture definition is served");
        assert_eq!(held.state, state, "{signal}");
        match record(&mut ports, signal, &[]) {
            RecordSampleOutcome::NotActive { error } => assert_eq!(error.signal.0, signal),
            other => panic!("{signal}: expected SignalNotActive, got {other:?}"),
        }
    }
    assert!(ports.samples.is_empty(), "nothing was recorded");

    let active = record(
        &mut ports,
        "service.api.request_failure_ratio",
        &[("component", "checkout"), ("environment", "production")],
    );
    assert!(
        matches!(active, RecordSampleOutcome::Recorded { .. }),
        "{active:?}"
    );
    assert_eq!(ports.samples.len(), 1);
}

#[test]
fn sample_of_unknown_signal_is_refused() {
    let mut ports = ports();
    match record(&mut ports, "service.api.not_defined", &[]) {
        RecordSampleOutcome::UnknownSignal { error } => {
            assert_eq!(error.signal.0, "service.api.not_defined");
        }
        other => panic!("expected UnknownSignal, got {other:?}"),
    }
    assert!(ports.samples.is_empty());
}

#[test]
fn unregistered_dims_key_is_refused() {
    let instance = instance();
    let definition = instance
        .definition("service.api.request_failure_ratio")
        .expect("loaded");
    let values = file_values();
    assert_eq!(
        admit_dims(
            instance.registry(),
            definition.data(),
            &dims(&[("component", "checkout"), ("environment", "staging")]),
            &values,
        ),
        Ok(())
    );
    let refusal = admit_dims(
        instance.registry(),
        definition.data(),
        &dims(&[("component", "checkout"), ("tenant", "a")]),
        &values,
    )
    .expect_err("tenant is not registered");
    assert_eq!(
        refusal,
        DimsRefusal::UnregisteredKey {
            key: "tenant".into()
        }
    );
    assert!(refusal.to_string().contains("tenant"));

    // Registered, but not declared by this definition.
    let refusal = admit_dims(
        instance.registry(),
        definition.data(),
        &dims(&[("component", "checkout"), ("suite", "smoke")]),
        &values,
    )
    .expect_err("suite is not declared by the definition");
    assert_eq!(
        refusal,
        DimsRefusal::UndeclaredKey {
            key: "suite".into()
        }
    );

    // A definition file naming an unregistered key does not load.
    let errors = refused(
        &with(RATIO, "dimensions", "dimensions: [component, tenant]"),
        |rule| matches!(rule, Rule::UnregisteredDimension { key } if key == "tenant"),
    );
    assert!(shown(&errors).contains("tenant"));
}

#[test]
fn unlisted_dims_value_is_refused() {
    let instance = instance();
    let definition = instance
        .definition("service.api.request_failure_ratio")
        .expect("loaded");
    let values = file_values();
    let refusal = admit_dims(
        instance.registry(),
        definition.data(),
        &dims(&[("component", "inventory"), ("environment", "staging")]),
        &values,
    )
    .expect_err("inventory is not listed in the components registry");
    assert_eq!(
        refusal,
        DimsRefusal::UnlistedValue {
            key: "component".into(),
            value: "inventory".into(),
            locator: "registries/components.yaml".into(),
        }
    );
    assert!(refusal.to_string().contains("registries/components.yaml"));

    // A plain entry and an entry without `created` are both listed.
    for value in ["billing", "ledger"] {
        assert_eq!(
            admit_dims(
                instance.registry(),
                definition.data(),
                &dims(&[("component", value)]),
                &values,
            ),
            Ok(()),
            "{value}"
        );
    }

    // A file registry whose contents were not passed in admits nothing.
    let refusal = admit_dims(
        instance.registry(),
        definition.data(),
        &dims(&[("environment", "staging")]),
        &BTreeMap::new(),
    )
    .expect_err("the environments file was not passed");
    assert!(
        matches!(refusal, DimsRefusal::ValuesNotGiven { .. }),
        "{refusal:?}"
    );

    // An open registry admits any value.
    let open = Definition::parse(
        CASE_FILE,
        &with(
            &with(RATIO, "dimensions", "dimensions: [suite]"),
            "denominator",
            "denominator: { unit: \"{request}\", min_n: 20, since: first_sample }",
        ),
        instance.registry(),
    )
    .expect("a definition over an open key loads");
    assert_eq!(
        admit_dims(
            instance.registry(),
            open.data(),
            &dims(&[("suite", "anything at all")]),
            &values,
        ),
        Ok(())
    );
}

#[test]
fn active_definition_without_dimensions_is_invalid() {
    let errors = refused(
        &with(RATIO, "dimensions", ""),
        |rule| matches!(rule, Rule::Incomplete { missing, .. } if missing == &["dimensions"]),
    );
    let text = shown(&errors);
    assert!(
        text.contains("thalamus.definition.IncompleteDefinition"),
        "{text}"
    );
    assert!(text.contains("dimensions"), "{text}");

    for field in ["unit", "denominator", "owner"] {
        refused(
            &with(RATIO, field, ""),
            |rule| matches!(rule, Rule::Incomplete { missing, .. } if missing == &[field]),
        );
    }

    // Muted is held to the same rule; Proposed is not.
    refused(&with(MUTED, "dimensions", ""), |rule| {
        matches!(rule, Rule::Incomplete { .. })
    });
    let proposed = load(PROPOSED).expect("a Proposed definition may leave dimensions undeclared");
    assert_eq!(proposed.data().dimensions, None);
}

#[test]
fn empty_dimensions_is_zero_dimensions() {
    let instance = instance();
    let definition = instance.definition("service.queue.depth").expect("loaded");
    assert_eq!(definition.data().dimensions, Some(Vec::new()));

    let active = load(&with(
        &with(
            &with(MUTED, "lifecycle", "lifecycle: active"),
            "mute_reason",
            "",
        ),
        "mute_until",
        "",
    ))
    .expect("an Active definition with an empty dimensions list loads");
    assert_eq!(active.state(), SignalDefinitionState::Active);
    assert_eq!(active.data().dimensions, Some(Vec::new()));

    let values = file_values();
    assert_eq!(
        admit_dims(instance.registry(), active.data(), &dims(&[]), &values),
        Ok(())
    );
    assert_eq!(
        admit_dims(
            instance.registry(),
            active.data(),
            &dims(&[("component", "checkout")]),
            &values,
        ),
        Err(DimsRefusal::UndeclaredKey {
            key: "component".into()
        })
    );
    assert_eq!(
        series_key(&active.data().signal_id, &dims(&[])),
        SeriesKey("service.queue.depth".into())
    );
}

#[test]
fn signal_id_needs_three_segments() {
    for id in [
        "service.api",
        "service.api.request.failures",
        "service..failures",
        ".api.failures",
        "service.api.",
        "Service.api.failures",
        "service.api.request-failures",
        "service",
    ] {
        let errors = refused(
            &with(RATIO, "id", &format!("id: \"{id}\"")),
            |rule| matches!(rule, Rule::SignalIdGrammar { id: refused } if refused == id),
        );
        assert!(shown(&errors).contains("<family>.<layer>.<name>"), "{id}");
    }
    load(&with(RATIO, "id", "id: service.api_2.failure_ratio_5m"))
        .expect("three lower-case segments load");
}

#[test]
fn ratio_unit_must_be_1() {
    for unit in ["%", "{request}", "s"] {
        refused(
            &with(RATIO, "unit", &format!("unit: \"{unit}\"")),
            |rule| matches!(rule, Rule::RatioUnit { unit: refused } if refused == unit),
        );
    }
    // A unit other than 1 is fine on a signal that is not a ratio.
    let gauge = with(
        &with(&with(RATIO, "kind", "kind: gauge"), "unit", "unit: s"),
        "evaluation",
        "evaluation: { rule: threshold, baselines: [offset_1d] }",
    );
    load(&gauge).expect("a gauge in seconds loads");
}

#[test]
fn non_ucum_unit_is_invalid() {
    let gauge = with(
        &with(RATIO, "kind", "kind: gauge"),
        "evaluation",
        "evaluation: { rule: threshold, baselines: [offset_1d] }",
    );
    for unit in ["requests", "m s", "1/", "{request", "", "\u{b5}s"] {
        refused(
            &with(&gauge, "unit", &format!("unit: \"{unit}\"")),
            |rule| matches!(rule, Rule::NotUcum { field: "unit", unit: refused, .. } if refused == unit),
        );
    }
    refused(
        &with(
            RATIO,
            "denominator",
            "denominator: { unit: \"requests\", min_n: 20, since: first_sample }",
        ),
        |rule| {
            matches!(
                rule,
                Rule::NotUcum {
                    field: "denominator.unit",
                    ..
                }
            )
        },
    );
    // Nesting past the parser's bound is refused, not overflowed.
    let deep = format!("{}s{}", "(".repeat(200), ")".repeat(200));
    refused(
        &with(&gauge, "unit", &format!("unit: \"{deep}\"")),
        |rule| matches!(rule, Rule::NotUcum { field: "unit", .. }),
    );
}

#[test]
fn connection_url_is_refused() {
    for connection in [
        "https://metrics.example/api",
        "postgres://reader@db/main",
        "//metrics/api",
        "/var/run/metrics.sock",
        "metrics main",
        "HTTP:metrics.example",
        "Https:metrics.example",
        "FILE:metrics.sock",
        "metrics:/api",
        "ns/metrics:/api",
        "ftp:metrics.example",
        "WS:metrics.example",
        "wss:metrics.example",
    ] {
        let errors = refused(
            &with(
                RATIO,
                "source",
                &format!("source: {{ adapter: prometheus, connection: \"{connection}\" }}"),
            ),
            |rule| matches!(rule, Rule::ConnectionNotIdentifier { connection: refused } if refused == connection),
        );
        assert!(shown(&errors).contains("connection id"), "{connection}");
    }
    for connection in ["metrics-main", "team:metrics.main", "ns/metrics_main"] {
        load(&with(
            RATIO,
            "source",
            &format!("source: {{ adapter: prometheus, connection: \"{connection}\" }}"),
        ))
        .unwrap_or_else(|errors| panic!("{connection}: {}", shown(&errors)));
    }
}

#[test]
fn connectors_registry_is_refused_naming_the_missing_inventory_operation() {
    let text = "registry_id: example\ntitle: Example\ndimensions:\n  \
                cluster: { registry: connectors:inventory/clusters }\n  \
                suite: { registry: open }\n";
    let errors =
        Registry::parse(REGISTRY_FILE, text).expect_err("a connectors registry is refused");
    assert!(
        errors.iter().any(|error| matches!(
            &error.rule,
            Rule::ConnectorsRegistry { key, locator }
                if key == "cluster" && locator == "inventory/clusters"
        )),
        "{errors:#?}"
    );
    let text = shown(&errors);
    assert!(text.contains(REGISTRY_FILE), "{text}");
    assert!(text.contains("inventory/clusters"), "{text}");
    assert!(text.contains("inventory operation"), "{text}");

    let errors = Instance::load(
        (REGISTRY_FILE, with_connectors().as_str()),
        &[(RATIO_FILE, RATIO)],
    )
    .expect_err("the instance does not load");
    assert!(
        errors
            .iter()
            .any(|error| matches!(error.rule, Rule::ConnectorsRegistry { .. })),
        "{errors:#?}"
    );

    for value in ["inventory:clusters", "file:", "connectors:"] {
        let text = format!(
            "registry_id: example\ntitle: Example\ndimensions:\n  cluster: {{ registry: \"{value}\" }}\n"
        );
        Registry::parse(REGISTRY_FILE, &text)
            .expect_err("only file:<locator> and open registries load");
    }
}

fn with_connectors() -> String {
    format!("{REGISTRY}  cluster: {{ registry: connectors:inventory/clusters }}\n")
}

#[test]
fn since_admits_first_sample_fixed_timestamp_and_file_dimension_created() {
    let with_since = |since: &str| {
        with(
            RATIO,
            "denominator",
            &format!("denominator: {{ unit: \"{{request}}\", min_n: 20, since: \"{since}\" }}"),
        )
    };
    let since = |text: &str| {
        load(&with_since(text))
            .unwrap_or_else(|errors| panic!("{text}: {}", shown(&errors)))
            .since()
            .cloned()
    };
    assert_eq!(since("first_sample"), Some(Since::FirstSample));
    assert_eq!(
        since("2025-03-02T00:00:00Z"),
        Some(Since::Fixed(ts("2025-03-02T00:00:00Z")))
    );
    assert_eq!(
        since("component.created"),
        Some(Since::DimensionCreated {
            key: "component".into()
        })
    );
    // The design's spelling names the definition's one dimension with a file: registry.
    let one_file = with(
        &with_since("dimension.created"),
        "dimensions",
        "dimensions: [component, endpoint]",
    );
    assert_eq!(
        load(&one_file)
            .unwrap_or_else(|errors| panic!("{}", shown(&errors)))
            .since(),
        Some(&Since::DimensionCreated {
            key: "component".into()
        })
    );
    // Two file: dimensions leave the shorthand ambiguous.
    refused(&with_since("dimension.created"), |rule| {
        matches!(
            rule,
            Rule::Since {
                refusal: SinceRefusal::Ambiguous { .. },
                ..
            }
        )
    });
    // A since naming a dimension the definition does not declare is refused.
    refused(
        &with_since("suite.created"),
        |rule| matches!(rule, Rule::Since { refusal: SinceRefusal::NotDeclared { key }, .. } if key == "suite"),
    );
}

#[test]
fn since_dimension_created_on_open_registry_is_refused() {
    let open = with(
        &with(RATIO, "dimensions", "dimensions: [suite]"),
        "denominator",
        "denominator: { unit: \"{request}\", min_n: 20, since: suite.created }",
    );
    let errors = refused(&open, |rule| {
        matches!(
            rule,
            Rule::Since { refusal: SinceRefusal::NotFile { key, .. }, .. } if key == "suite"
        )
    });
    let text = shown(&errors);
    assert!(text.contains("open"), "{text}");
    assert!(text.contains("file:"), "{text}");

    let shorthand = with(
        &open,
        "denominator",
        "denominator: { unit: \"{request}\", min_n: 20, since: dimension.created }",
    );
    refused(&shorthand, |rule| matches!(rule, Rule::Since { .. }));
}

#[test]
fn since_other_value_is_refused_naming_admitted_forms() {
    for value in [
        "source.retention_start",
        "yesterday",
        "2026-13-01T00:00:00Z",
        "2026-10-01",
        "",
        "first sample",
        ".created",
        "component.environment.created",
    ] {
        let errors = refused(
            &with(
                RATIO,
                "denominator",
                &format!("denominator: {{ unit: \"{{request}}\", min_n: 20, since: \"{value}\" }}"),
            ),
            |rule| matches!(rule, Rule::Since { .. }),
        );
        let text = shown(&errors);
        assert!(text.contains(ADMITTED_FORMS), "{value}: {text}");
    }
    assert!(ADMITTED_FORMS.contains("first_sample"));
    assert!(ADMITTED_FORMS.contains("RFC 3339"));
    assert!(ADMITTED_FORMS.contains(".created"));
}

fn definition_with_since(since: &str) -> Definition {
    load(&with(
        RATIO,
        "denominator",
        &format!("denominator: {{ unit: \"{{request}}\", min_n: 20, since: \"{since}\" }}"),
    ))
    .unwrap_or_else(|errors| panic!("{since}: {}", shown(&errors)))
}

#[test]
fn resolve_since_first_sample_is_first_stored_ts() {
    let definition = definition_with_since("first_sample");
    let checkout = dims(&[("component", "checkout")]);
    assert_eq!(
        resolve_since(
            &definition,
            &checkout,
            &file_values(),
            Some(ts("2026-09-01T00:00:00Z")),
            None,
        ),
        Some(ts("2026-09-01T00:00:00Z"))
    );
    assert_eq!(
        resolve_since(&definition, &checkout, &file_values(), None, None),
        None,
        "no stored sample yet: the origin is unknown"
    );
    assert_eq!(
        resolve_since(
            &definition,
            &checkout,
            &file_values(),
            None,
            Some(ts("2026-01-01T00:00:00Z")),
        ),
        None,
        "a retention start is no origin by itself"
    );
}

#[test]
fn resolve_since_fixed_timestamp() {
    let definition = definition_with_since("2025-03-02T00:00:00Z");
    assert_eq!(
        resolve_since(
            &definition,
            &dims(&[("component", "checkout")]),
            &file_values(),
            Some(ts("2026-09-01T00:00:00Z")),
            None,
        ),
        Some(ts("2025-03-02T00:00:00Z"))
    );
}

#[test]
fn resolve_since_dimension_created_reads_file_entry() {
    let instance = instance();
    let definition = instance
        .definition("service.api.request_failure_ratio")
        .expect("loaded");
    let values = file_values();
    let at = |component: &str| {
        resolve_since(
            definition,
            &dims(&[("component", component), ("environment", "staging")]),
            &values,
            Some(ts("2026-10-12T00:00:00Z")),
            None,
        )
    };
    assert_eq!(at("checkout"), Some(ts("2025-03-02T00:00:00Z")));
    assert_eq!(at("search"), Some(ts("2026-10-10T12:00:00Z")));
    assert_eq!(
        at("billing"),
        None,
        "an entry without created resolves to none"
    );
    assert_eq!(at("ledger"), None, "a plain entry resolves to none");
    assert_eq!(at("inventory"), None, "an unlisted value resolves to none");
    assert_eq!(
        resolve_since(
            definition,
            &dims(&[("environment", "staging")]),
            &values,
            None,
            None,
        ),
        None,
        "a sample without the named dimension resolves to none"
    );
}

#[test]
fn resolve_since_never_precedes_retention_start() {
    let retention = Some(ts("2026-01-01T00:00:00Z"));
    let checkout = dims(&[("component", "checkout")]);
    let values = file_values();
    let fixed = definition_with_since("2025-03-02T00:00:00Z");
    assert_eq!(
        resolve_since(&fixed, &checkout, &values, None, retention.clone()),
        Some(ts("2026-01-01T00:00:00Z"))
    );
    let first = definition_with_since("first_sample");
    assert_eq!(
        resolve_since(
            &first,
            &checkout,
            &values,
            Some(ts("2025-06-01T00:00:00Z")),
            retention.clone(),
        ),
        Some(ts("2026-01-01T00:00:00Z"))
    );
    assert_eq!(
        resolve_since(
            &first,
            &checkout,
            &values,
            Some(ts("2026-02-01T00:00:00Z")),
            retention.clone(),
        ),
        Some(ts("2026-02-01T00:00:00Z")),
        "an origin after the retention start is kept"
    );
    let created = definition_with_since("component.created");
    assert_eq!(
        resolve_since(&created, &checkout, &values, None, retention.clone()),
        Some(ts("2026-01-01T00:00:00Z"))
    );
    // An instant written with an offset compares by the instant it names.
    assert_eq!(
        resolve_since(
            &fixed,
            &checkout,
            &values,
            None,
            Some(ts("2025-03-02T01:00:00+02:00")),
        ),
        Some(ts("2025-03-02T00:00:00Z"))
    );
    assert_eq!(
        resolve_since(&fixed, &checkout, &values, None, Some(ts("not a time"))),
        None,
        "a retention start that names no instant leaves the origin unknown"
    );
}

#[test]
fn definition_ports_refuse_writes() {
    let ratio = SignalId("service.api.request_failure_ratio".into());
    let mut generated = Generated::new(instance());
    let outcome = generated
        .retire_definition(RetireDefinition {
            signal_id: ratio.clone(),
        })
        .expect("a declared outcome");
    let instance = generated.ports;
    // Whatever the command answered, the file is the one copy: nothing changed.
    let held = SignalDefinitionStorage::get(&instance, &ratio).expect("served");
    assert_eq!(held.state, SignalDefinitionState::Active, "{outcome:?}");

    let mut instance = instance;
    let before = SignalDefinitionStorage::list(&instance);
    let snapshot = before[0].clone();
    SignalDefinitionStorage::put(&mut instance, snapshot.clone());
    SignalDefinitionStorage::delete(&mut instance, &snapshot.data.signal_id);
    assert_eq!(SignalDefinitionStorage::list(&instance), before);

    let dimension = DimensionStorage::get(&instance, &DimensionKey("component".into()))
        .expect("component is registered");
    DimensionStorage::delete(&mut instance, &DimensionKey("component".into()));
    DimensionStorage::put(&mut instance, dimension.clone());
    assert_eq!(
        DimensionStorage::get(&instance, &DimensionKey("component".into())),
        Some(dimension)
    );
    let registry = DimensionRegistryStorage::get(&instance, &RegistryId("example".into()))
        .expect("the registry is served");
    assert_eq!(registry.data.title, "Example instance");
    DimensionRegistryStorage::put(&mut instance, registry.clone());
    DimensionRegistryStorage::delete(&mut instance, &RegistryId("example".into()));
    assert_eq!(DimensionRegistryStorage::list(&instance), vec![registry]);

    let refused = instance.refused_writes();
    assert_eq!(refused.len(), 7, "{refused:#?}");
    assert_eq!(refused[0].entity, "thalamus.definition.SignalDefinition");
    assert_eq!(refused[0].identity, "service.api.request_failure_ratio");
    assert_eq!(refused[0].operation, "put");
    assert_eq!(refused[1].operation, "put");
    assert_eq!(refused[2].operation, "delete");
    assert_eq!(refused[3].entity, "thalamus.dimension.Dimension");
    assert_eq!(refused[5].entity, "thalamus.dimension.DimensionRegistry");
    let text = refused[0].to_string();
    assert!(text.contains("one copy"), "{text}");
    assert!(text.contains("service.api.request_failure_ratio"), "{text}");
}

#[test]
fn series_key_matches_d10_vectors() {
    let id = SignalId("platform.namespace.request_failure_ratio".into());
    for (pairs, expected) in [
        (vec![], "platform.namespace.request_failure_ratio"),
        (
            vec![("namespace", "shared-1"), ("cluster", "prod-a")],
            r#"platform.namespace.request_failure_ratio{cluster="prod-a",namespace="shared-1"}"#,
        ),
        (
            vec![("b", "x"), ("a_2", "y"), ("a", "z")],
            r#"platform.namespace.request_failure_ratio{a="z",a_2="y",b="x"}"#,
        ),
        (
            vec![("case", "say \"hi\"")],
            r#"platform.namespace.request_failure_ratio{case="say \"hi\""}"#,
        ),
        (
            vec![("path", "C:\\tmp\\x")],
            r#"platform.namespace.request_failure_ratio{path="C:\\tmp\\x"}"#,
        ),
        (
            vec![("line", "one\ntwo")],
            r#"platform.namespace.request_failure_ratio{line="one\ntwo"}"#,
        ),
        (
            vec![("empty", "")],
            r#"platform.namespace.request_failure_ratio{empty=""}"#,
        ),
        (
            vec![("text", "a,b=c}{")],
            r#"platform.namespace.request_failure_ratio{text="a,b=c}{"}"#,
        ),
    ] {
        assert_eq!(
            series_key(&id, &dims(&pairs)),
            SeriesKey(expected.to_owned()),
            "{pairs:?}"
        );
    }
}

#[test]
fn pooled_on_a_non_ratio_is_refused_at_load() {
    let gauge = with(&with(RATIO, "kind", "kind: gauge"), "unit", "unit: s");
    let errors = refused(&gauge, |rule| {
        matches!(
            rule,
            Rule::Baseline {
                refusal: thalamus_core::baseline::Refusal::PooledNeedsRatio { .. }
            }
        )
    });
    assert!(shown(&errors).contains("pooled"));

    let two_offsets = with(
        &gauge,
        "evaluation",
        "evaluation: { rule: threshold, baselines: [offset_1d, offset_7d] }",
    );
    refused(&two_offsets, |rule| matches!(rule, Rule::Baseline { .. }));

    let higher = with(
        &gauge,
        "evaluation",
        "evaluation: { rule: threshold, baselines: [higher_of, offset_1d, offset_7d] }",
    );
    load(&higher).expect("higher_of over two offsets loads on a gauge");

    // Instant signals take fixed only, in every state.
    let muted = with(
        MUTED,
        "evaluation",
        "evaluation: { rule: threshold, baselines: [offset_1d] }",
    );
    let proposed = with(
        &with(
            &with(&muted, "lifecycle", "lifecycle: proposed"),
            "mute_reason",
            "",
        ),
        "mute_until",
        "",
    );
    for text in [&muted, &proposed] {
        refused(text, |rule| matches!(rule, Rule::InstantNeedsFixed { .. }));
    }
}

#[test]
fn the_fixture_instance_loads_and_serves_every_definition() {
    let instance = instance();
    let ids: Vec<String> = SignalDefinitionStorage::list(&instance)
        .into_iter()
        .map(|snapshot| snapshot.data.signal_id.0)
        .collect();
    assert_eq!(
        ids,
        [
            "service.api.request_failure_ratio",
            "service.batch.duration",
            "service.queue.depth",
            "test.run.pass_ratio",
        ]
    );
    let ratio = instance
        .definition("service.api.request_failure_ratio")
        .expect("loaded");
    assert_eq!(ratio.file(), RATIO_FILE);
    let data = ratio.data();
    assert_eq!(data.rev, 3);
    assert_eq!(data.window.0, "PT6H");
    assert_eq!(data.cadence.0, "PT1H");
    assert_eq!(
        data.evaluation
            .warn
            .as_ref()
            .and_then(|warn| warn.abs.clone()),
        Some(Decimal("0.02".into()))
    );
    assert_eq!(
        data.evaluation
            .warn
            .as_ref()
            .and_then(|warn| warn.x.clone()),
        Some(Decimal("2".into()))
    );
    assert_eq!(DimensionStorage::list(&instance).len(), 4);
    let endpoint =
        DimensionStorage::get(&instance, &DimensionKey("endpoint".into())).expect("registered");
    assert_eq!(endpoint.data.parent, Some(DimensionKey("component".into())));

    // The same id in two files is refused, naming the second file.
    let errors = Instance::load(
        (REGISTRY_FILE, REGISTRY),
        &[(RATIO_FILE, RATIO), (CASE_FILE, RATIO)],
    )
    .expect_err("one definition per id");
    assert!(
        errors.iter().any(|error| error.file == CASE_FILE
            && matches!(error.rule, Rule::DuplicateDefinition { .. })),
        "{errors:#?}"
    );
    // A muted definition carries its reason and expiry, and only a muted one does.
    refused(&with(MUTED, "mute_until", ""), |rule| {
        matches!(rule, Rule::Invariant { .. })
    });
    refused(&with(RATIO, "mute_reason", "mute_reason: noisy"), |rule| {
        matches!(rule, Rule::Invariant { .. })
    });
}

fn registry_refused(text: &str, matches: impl Fn(&Rule) -> bool) {
    let errors = Registry::parse(REGISTRY_FILE, text).expect_err("the registry is refused");
    assert!(
        errors.iter().any(|error| matches(&error.rule)),
        "no expected rule among {errors:#?}"
    );
    for error in &errors {
        assert!(error.to_string().contains(REGISTRY_FILE), "{error}");
    }
}

fn registry_of(dimensions: &str) -> String {
    format!("registry_id: example\ntitle: Example\ndimensions:\n{dimensions}")
}

/// Adversary pass 1, finding 13: the baseline plan runs for the states that are sampled
/// (`Active`, `Muted`) and for no other, so a `Proposed` or `Retired` definition whose baseline
/// is not yet one thalamus computes still loads.
#[test]
fn baseline_plan_runs_only_for_sampled_states() {
    let gauge = with(&with(RATIO, "kind", "kind: gauge"), "unit", "unit: s");
    refused(&gauge, |rule| matches!(rule, Rule::Baseline { .. }));
    for lifecycle in ["proposed", "retired"] {
        let text = with(&gauge, "lifecycle", &format!("lifecycle: {lifecycle}"));
        let loaded = load(&text).unwrap_or_else(|errors| panic!("{lifecycle}: {}", shown(&errors)));
        assert_eq!(
            loaded.data().evaluation.baselines,
            [
                thalamus_model::definition::BaselineKind::Pooled,
                thalamus_model::definition::BaselineKind::Offset1d,
                thalamus_model::definition::BaselineKind::Offset7d,
            ]
        );
    }
}

/// Adversary pass 1, finding 1 (coordinator decision 1): no YAML mapping the loader reads takes
/// a key twice, whichever map it is.
#[test]
fn a_key_given_twice_is_refused_in_every_mapping() {
    registry_refused(
        &registry_of("  suite: { registry: open }\n  suite: { registry: open }\n"),
        |rule| matches!(rule, Rule::Syntax { message } if message.contains("suite")),
    );
    refused(
        &with(RATIO, "query", "query:\n  value: a\n  value: a"),
        |rule| matches!(rule, Rule::Syntax { message } if message.contains("value")),
    );
    refused(
        &with(RATIO, "objective", "objective: { target: 1, target: 2 }"),
        |rule| matches!(rule, Rule::Syntax { .. }),
    );
    refused(&format!("{RATIO}title: again\n"), |rule| {
        matches!(rule, Rule::Syntax { .. })
    });
}

/// Adversary pass 1, findings 4 and 5 (coordinator decision 4): every duration a definition
/// declares is one the adapters place: no years or months, not zero, at most what fits before
/// the earliest window end.
#[test]
fn every_duration_field_is_a_placeable_length() {
    for field in ["window", "cadence"] {
        for value in [
            "P1M",
            "P1Y",
            "P1Y2M",
            "0h",
            "0w",
            "PT0S",
            "P0D",
            "PT0H0M",
            "99999999999999999999h",
            "P99999999999999999999D",
            "P15250284452W",
        ] {
            refused(
                &with(RATIO, field, &format!("{field}: \"{value}\"")),
                |rule| matches!(rule, Rule::Duration { .. }),
            );
        }
        // Stored in the canonical RFC 3339 Appendix A form (adversary pass 2, finding 2).
        for (value, iso) in [
            ("P1W2D", "P9D"),
            ("PT1M", "PT1M"),
            ("2w", "P2W"),
            ("0006h", "PT6H"),
            ("PT1H1S", "PT1H0M1S"),
            ("P1W1DT1H1M1S", "P8DT1H1M1S"),
            ("P1DT1H1S", "P1DT1H0M1S"),
            ("P0001W", "P1W"),
        ] {
            let loaded = load(&with(RATIO, field, &format!("{field}: \"{value}\"")))
                .unwrap_or_else(|errors| panic!("{field} {value}: {}", shown(&errors)));
            let held = if field == "window" {
                &loaded.data().window
            } else {
                &loaded.data().cadence
            };
            assert_eq!(held.0, iso);
        }
    }
    let muted = |window: &str| {
        with(
            MUTED,
            "evaluation",
            &format!(
                "evaluation: {{ rule: threshold, baselines: [fixed], fixed: \"0\", min_trend_window: \"{window}\" }}"
            ),
        )
    };
    for value in ["P1M", "0h"] {
        refused(&muted(value), |rule| {
            matches!(
                rule,
                Rule::Duration {
                    field: "evaluation.min_trend_window",
                    ..
                }
            )
        });
    }
    let loaded = load(&muted("PT1H1S")).unwrap_or_else(|errors| panic!("{}", shown(&errors)));
    assert_eq!(
        loaded
            .data()
            .evaluation
            .min_trend_window
            .as_ref()
            .map(|window| window.0.as_str()),
        Some("PT1H0M1S")
    );
    // The longest window the adapters place at every end from 1970 on starts at 0000-01-01
    // (adversary pass 2, finding 1).
    load(&with(RATIO, "window", "window: P719528D"))
        .unwrap_or_else(|errors| panic!("{}", shown(&errors)));
    for value in ["P719529D", "P800000D", "P110000W"] {
        refused(
            &with(RATIO, "window", &format!("window: {value}")),
            |rule| {
                matches!(
                    rule,
                    Rule::Duration {
                        field: "window",
                        ..
                    }
                )
            },
        );
    }
}

/// Adversary pass 1, finding 3 (coordinator decision 3).
#[test]
fn signal_id_first_segment_is_the_family() {
    let errors = refused(
        &with(RATIO, "family", "family: platform"),
        |rule| matches!(rule, Rule::IdFamily { family, .. } if family == "platform"),
    );
    assert!(shown(&errors).contains("service.api.request_failure_ratio"));
}

/// Adversary pass 1, finding 6 (coordinator decision 5): a key whose parent chain reaches itself.
#[test]
fn a_parent_cycle_is_refused() {
    for (dimensions, key) in [
        ("  a: { registry: open, parent: a }\n", "a"),
        (
            "  a: { registry: open, parent: b }\n  b: { registry: open, parent: a }\n",
            "a",
        ),
        (
            "  a: { registry: open, parent: c }\n  b: { registry: open, parent: a }\n  \
             c: { registry: open, parent: b }\n  d: { registry: open, parent: a }\n",
            "b",
        ),
    ] {
        registry_refused(
            &registry_of(dimensions),
            |rule| matches!(rule, Rule::ParentCycle { key: refused } if refused == key),
        );
    }
    Registry::parse(
        REGISTRY_FILE,
        &registry_of("  a: { registry: open }\n  b: { registry: open, parent: a }\n  c: { registry: open, parent: b }\n"),
    )
    .expect("a parent chain without a cycle loads");
}

/// Adversary pass 1, finding 7 (coordinator decision 6): a `file:` locator is a relative path
/// inside the instance directory.
#[test]
fn a_file_locator_stays_inside_the_instance() {
    for locator in [
        "/etc/passwd",
        "../other/components.yaml",
        "registries/../../other/components.yaml",
        "registries/..",
        "https://example/components.yaml",
        "https:components.yaml",
        "C:/registries/components.yaml",
        "registries\\components.yaml",
        "\\\\host\\share\\components.yaml",
        "./registries/components.yaml",
        "registries/./components.yaml",
        ".",
        "registries/.",
    ] {
        registry_refused(
            &registry_of(&format!("  component: {{ registry: 'file:{locator}' }}\n")),
            |rule| matches!(rule, Rule::LocatorOutsideInstance { locator: refused, .. } if refused == locator),
        );
    }
    for locator in [
        "components.yaml",
        "registries/components.yaml",
        "registries/.components.yaml",
    ] {
        Registry::parse(
            REGISTRY_FILE,
            &registry_of(&format!("  component: {{ registry: 'file:{locator}' }}\n")),
        )
        .unwrap_or_else(|errors| panic!("{locator}: {}", shown(&errors)));
    }
}

/// Adversary pass 1, finding 8, and pass 2, finding 3 (coordinator decisions): a loaded objective
/// is JSON the model's own reader reads back, also inside the served `Definitions` body, three
/// levels above it, so no deeper than 61 levels.
#[test]
fn an_objective_reads_back_as_model_json() {
    let objective = |text: &str| with(RATIO, "objective", &format!("objective: {text}"));
    for number in [".nan", ".NaN", ".inf", "-.inf", "+.inf"] {
        refused(&objective(&format!("{{ target: {number} }}")), |rule| {
            matches!(rule, Rule::Objective { .. })
        });
    }
    let nested = |depth: usize| format!("{}1{}", "[".repeat(depth), "]".repeat(depth));
    for depth in [62, 64, 65] {
        refused(&objective(&nested(depth)), |rule| {
            matches!(rule, Rule::Objective { .. })
        });
    }
    for text in [
        nested(61),
        "{ target: 0.995, window: 28d rolling, burn: [{ rate: 14.4, long: 1h }], on: true, x: null, big: 1e300, n: -3 }".to_owned(),
    ] {
        let loaded = load(&objective(&text)).unwrap_or_else(|errors| panic!("{}", shown(&errors)));
        let value = loaded.data().objective.as_ref().expect("kept");
        let mut written = String::new();
        thalamus_model::json::push_value(&mut written, value);
        assert_eq!(
            thalamus_model::json::parse(&written).as_ref(),
            Ok(value),
            "{written}"
        );
    }
}
