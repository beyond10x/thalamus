//! Adversary pass 2 on `story:m1-instance-registry`: the corrections for pass 1's findings, and
//! what they newly refuse or admit.
//!
//! Each case drives `thalamus_core::registry` from a document outside the loader: the Connectors
//! connection refs Connectors mints and its contract fixtures, the published schema's
//! `format: duration` (RFC 3339 Appendix A), the served `Definitions` view read back by the
//! model's own JSON reader, and the signals concept page.

use thalamus_core::registry::{Definition, FileValues, Instance, LoadError, Registry, Rule};
use thalamus_core::since::{Since, SinceRefusal, parse_since};
use thalamus_model::behaviour::Generated;
use thalamus_model::definition::obligations::DefinitionsQuery;
use thalamus_model::dimension::DimensionKey;

const REGISTRY_FILE: &str = "dimensions.yaml";
const REGISTRY: &str = include_str!("fixtures/instance/dimensions.yaml");
const RATIO: &str =
    include_str!("fixtures/instance/definitions/service.api.request_failure_ratio.yaml");
const CASE_FILE: &str = "definitions/case.yaml";
const SIGNALS_PAGE: &str = include_str!("../../../website/docs/concepts/signals.md");

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

fn shown(errors: &[LoadError]) -> String {
    errors
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join("\n")
}

fn registry_of(dimensions: &str) -> String {
    format!("registry_id: example\ntitle: Example\ndimensions:\n{dimensions}")
}

fn with_connection(connection: &str) -> String {
    with(
        RATIO,
        "source",
        &format!("source: {{ adapter: prometheus, connection: \"{connection}\" }}"),
    )
}

// ---------------------------------------------------------------------------------------------
// Connection ids.

/// The connection refs Connectors itself mints and names: `qualified("conn", authority)` in
/// `connectors-host/src/local/registry/lifecycle.rs:364` (`conn_<uuid>_<uuid>`, simple form),
/// the evidence ref in `docs/evidence/gitlab-current-20261002/revalidate-after-restart.json`, and
/// the `connection` values of `contracts/cli/v1alpha1/fixtures/values.json`,
/// `contracts/service/delegation.md` and `contracts/discovery/mediated_route/v1alpha1/semantics.md`.
/// Each is a Connectors connection id, so each loads.
#[test]
fn adv_p2_connectors_connection_refs_load() {
    let mut refused = Vec::new();
    for connection in [
        "conn_4496354b35f441489edcd2312ee0207f_914058c2a80e48b4920c04160e6811b8",
        "forge-local/connection-1",
        "configured-main",
        "connection-1",
        "conn-team",
        "conn_child_7",
        "c1",
    ] {
        if let Err(errors) = load(&with_connection(connection)) {
            refused.push(format!("{connection}: {}", shown(&errors)));
        }
    }
    assert!(refused.is_empty(), "{}", refused.join("\n"));
}

/// The correction's reason is that a connection id is never a URL. The WHATWG URL standard's
/// special schemes are `ftp`, `file`, `http`, `https`, `ws` and `wss`: for each of them a parser
/// reads `<scheme>:host/path` as `<scheme>://host/path`, exactly as it reads `https:host/path`.
/// The correction refuses three of the six.
#[test]
fn adv_p2_every_special_url_scheme_is_refused() {
    let mut loaded = Vec::new();
    for connection in [
        "ftp:metrics.example/api",
        "ws:metrics.example/api",
        "wss:metrics.example/api",
        "WSS:metrics.example",
    ] {
        if load(&with_connection(connection)).is_ok() {
            loaded.push(connection);
        }
    }
    assert!(
        loaded.is_empty(),
        "connection ids a WHATWG parser reads as URLs loaded: {loaded:?}"
    );
}

// ---------------------------------------------------------------------------------------------
// `file:` locators.

/// Nested and unicode locators are relative paths inside the instance and load; the instance
/// file stores them as written.
#[test]
fn adv_p2_nested_and_unicode_locators_load() {
    for locator in [
        "registries/teams.yaml",
        "registries/2026/teams.yaml",
        "registries/équipes.yaml",
        "チーム.yaml",
        "registries/teams.v2.yaml",
        "registries/..teams.yaml",
    ] {
        let text = registry_of(&format!("  component: {{ registry: 'file:{locator}' }}\n"));
        let registry = Registry::parse(REGISTRY_FILE, &text)
            .unwrap_or_else(|errors| panic!("{locator}: {}", shown(&errors)));
        let values = &registry.dimension("component").expect("registered").values;
        assert_eq!(
            format!("{values:?}"),
            format!("File(Locator({locator:?}))"),
            "{locator}"
        );
    }
}

/// A locator names a registry *file* in the instance (spec `ValueSource`, signals page). The
/// correction refuses a trailing `/` (an empty segment), which names a directory; `.` and a
/// trailing `/.` name a directory just the same, the instance directory itself among them.
#[test]
fn adv_p2_a_locator_naming_a_directory_is_refused() {
    let mut loaded = Vec::new();
    for locator in [
        "registries/",
        "registries//teams.yaml",
        ".",
        "./",
        "registries/.",
    ] {
        let text = registry_of(&format!("  component: {{ registry: 'file:{locator}' }}\n"));
        if Registry::parse(REGISTRY_FILE, &text).is_ok() {
            loaded.push(locator);
        }
    }
    assert!(
        loaded.is_empty(),
        "locators naming a directory loaded: {loaded:?}"
    );
}

/// Every refusal of the locator rule, one input per clause of `is_instance_path`, each reported
/// as `LocatorOutsideInstance` naming the locator.
#[test]
fn adv_p2_each_locator_clause_refuses_on_its_own() {
    for locator in [
        "/registries/teams.yaml",
        "registries/../teams.yaml",
        "..",
        "teams:yaml",
        "registries\\teams.yaml",
        "registries//teams.yaml",
        "registries/",
    ] {
        let text = registry_of(&format!("  component: {{ registry: 'file:{locator}' }}\n"));
        let errors =
            Registry::parse(REGISTRY_FILE, &text).expect_err(&format!("{locator} is refused"));
        assert!(
            errors.iter().any(|error| matches!(
                &error.rule,
                Rule::LocatorOutsideInstance { key, locator: refused }
                    if key == "component" && refused == locator
            )),
            "{locator}: {}",
            shown(&errors)
        );
    }
}

// ---------------------------------------------------------------------------------------------
// Repeated keys, nested.

/// A key repeated in a nested map, in a map inside a sequence, or in a struct-shaped entry is
/// refused wherever the loader reads one.
#[test]
fn adv_p2_repeated_keys_are_refused_at_every_depth() {
    for (key, line) in [
        ("objective", "objective: { burn: [{ rate: 1, rate: 2 }] }"),
        ("objective", "objective: { a: { b: 1, b: 2 } }"),
        (
            "source",
            "source: { adapter: prometheus, adapter: sql, connection: metrics-main }",
        ),
        ("links", "links: { components: [api], components: [web] }"),
        (
            "evaluation",
            "evaluation:\n  rule: ratio_vs_pooled_baseline\n  baselines: [pooled]\n  warn: { x: 2, x: 3 }",
        ),
        (
            "denominator",
            "denominator: { unit: \"{request}\", min_n: 20, min_n: 30, since: first_sample }",
        ),
    ] {
        let text = with(RATIO, key, line);
        if let Ok(definition) = load(&text) {
            panic!("{line}: loaded {:?}", definition.data().signal_id.0);
        }
    }
    for dimensions in [
        "  suite: { registry: open, registry: 'file:registries/suites.yaml' }\n",
        "  suite:\n    registry: open\n    otel_attribute: a\n    otel_attribute: b\n",
    ] {
        assert!(
            Registry::parse(REGISTRY_FILE, &registry_of(dimensions)).is_err(),
            "{dimensions}"
        );
    }
    assert!(
        FileValues::parse("registries/x.yaml", "- { value: a, value: b }\n").is_err(),
        "a registry entry naming value twice loaded"
    );
}

// ---------------------------------------------------------------------------------------------
// Parent chains.

/// A cycle through every key of the registry (so a walk bounded one short of the key count
/// misses it), a cycle of four, a chain into an unregistered key, and a key that leads into a
/// cycle it is not on.
#[test]
fn adv_p2_parent_chains_of_three_and_more() {
    let cycle_keys = |dimensions: &str| -> Vec<String> {
        let errors = Registry::parse(REGISTRY_FILE, &registry_of(dimensions))
            .expect_err("the registry is refused");
        errors
            .iter()
            .filter_map(|error| match &error.rule {
                Rule::ParentCycle { key } => Some(key.clone()),
                _ => None,
            })
            .collect()
    };
    assert_eq!(
        cycle_keys(
            "  a: { registry: open, parent: b }\n  b: { registry: open, parent: c }\n  \
             c: { registry: open, parent: a }\n"
        ),
        ["a", "b", "c"]
    );
    assert_eq!(
        cycle_keys(
            "  a: { registry: open, parent: b }\n  b: { registry: open, parent: c }\n  \
             c: { registry: open, parent: d }\n  d: { registry: open, parent: a }\n"
        ),
        ["a", "b", "c", "d"]
    );
    assert_eq!(
        cycle_keys(
            "  a: { registry: open, parent: b }\n  b: { registry: open, parent: a }\n  \
             d: { registry: open, parent: a }\n"
        ),
        ["a", "b"]
    );
    let errors = Registry::parse(
        REGISTRY_FILE,
        &registry_of(
            "  a: { registry: open, parent: b }\n  b: { registry: open, parent: c }\n  \
             c: { registry: open, parent: missing }\n",
        ),
    )
    .expect_err("a chain into an unregistered key is refused");
    let rules: Vec<String> = errors
        .iter()
        .map(|error| match &error.rule {
            Rule::UnknownParent { key, parent } => format!("unknown {key}->{parent}"),
            Rule::ParentCycle { key } => format!("cycle {key}"),
            other => format!("other {other}"),
        })
        .collect();
    assert_eq!(rules, ["unknown c->missing"]);
    let chain = Registry::parse(
        REGISTRY_FILE,
        &registry_of(
            "  a: { registry: open }\n  b: { registry: open, parent: a }\n  \
             c: { registry: open, parent: b }\n  d: { registry: open, parent: c }\n  \
             e: { registry: open, parent: d }\n",
        ),
    )
    .unwrap_or_else(|errors| panic!("a five-key chain: {}", shown(&errors)));
    assert_eq!(
        chain
            .dimension("e")
            .and_then(|dimension| dimension.parent.as_ref())
            .map(|parent| parent.0.as_str()),
        Some("d")
    );
}

// ---------------------------------------------------------------------------------------------
// Objective depth.

/// 61 and 62 counted as `thalamus_model::json::parse` counts them, through maps as well as
/// sequences, and through a YAML tag, which adds no JSON level.
///
/// Expectation changed by the coordinator's decision on this pass's finding 3 (2026-10-15): the
/// cap is 61 levels, so that the served `Definitions` body, which puts the objective three levels
/// deep, stays within the reader's 64. This case first expected 64 to load and 65 to be refused.
#[test]
fn adv_p2_objective_depth_matches_the_reader_at_61_and_62() {
    let objective = |text: &str| with(RATIO, "objective", &format!("objective: {text}"));
    let maps = |depth: usize| format!("{}1{}", "{a: ".repeat(depth), "}".repeat(depth));
    let tagged = |depth: usize| format!("!t {}1{}", "[".repeat(depth), "]".repeat(depth));
    for text in [maps(61), tagged(61)] {
        let loaded = load(&objective(&text)).unwrap_or_else(|errors| panic!("{}", shown(&errors)));
        let value = loaded.data().objective.as_ref().expect("kept");
        let mut written = String::new();
        thalamus_model::json::push_value(&mut written, value);
        assert_eq!(thalamus_model::json::parse(&written).as_ref(), Ok(value));
    }
    for text in [maps(62), tagged(62)] {
        let errors = load(&objective(&text)).expect_err("62 levels are refused");
        assert!(
            errors
                .iter()
                .any(|error| matches!(error.rule, Rule::Objective { .. })),
            "{}",
            shown(&errors)
        );
    }
}

/// The objective is never written alone: the generated `Definitions` view, which
/// `Generated<Instance>` answers from the loaded files, is served as
/// `{"rows":[{…,"objective":…}]}` (`thalamus_service.rs` `run_thalamus_definition_definitions`),
/// three levels above the objective. An objective the loader admits at 62 to 64 levels makes
/// that body one the model's own reader refuses.
#[test]
fn adv_p2_the_served_definitions_view_reads_back() {
    let nested = |depth: usize| format!("{}1{}", "[".repeat(depth), "]".repeat(depth));
    let mut unreadable = Vec::new();
    for depth in [61, 62, 64] {
        let text = with(RATIO, "objective", &format!("objective: {}", nested(depth)));
        let Ok(instance) = Instance::load((REGISTRY_FILE, REGISTRY), &[(CASE_FILE, &text)]) else {
            continue;
        };
        let rows = Generated::new(instance)
            .definitions()
            .expect("the view answers");
        let mut body = String::from("{");
        thalamus_model::json::member(&mut body, "rows");
        body.push('[');
        for row in &rows {
            thalamus_model::server::wire::encode_view_thalamus_definition_definitions(
                row, &mut body,
            );
        }
        body.push_str("]}");
        if let Err(error) = thalamus_model::json::parse(&body) {
            unreadable.push(format!(
                "objective nested {depth} deep loaded; the served Definitions body is refused: {error}"
            ));
        }
    }
    assert!(unreadable.is_empty(), "{}", unreadable.join("\n"));
}

// ---------------------------------------------------------------------------------------------
// Durations.

/// RFC 3339 Appendix A `duration`, the grammar of the JSON Schema `format: duration` that the
/// generated schemas give every `Duration` field (`generated/spec-types/source.schema.json`,
/// `thalamus-service.openapi.json`):
///
/// ```text
/// dur-second = 1*DIGIT "S"
/// dur-minute = 1*DIGIT "M" [dur-second]
/// dur-hour   = 1*DIGIT "H" [dur-minute]
/// dur-time   = "T" (dur-hour / dur-minute / dur-second)
/// dur-day    = 1*DIGIT "D"
/// dur-week   = 1*DIGIT "W"
/// dur-month  = 1*DIGIT "M" [dur-day]
/// dur-year   = 1*DIGIT "Y" [dur-month]
/// dur-date   = (dur-day / dur-month / dur-year) [dur-time]
/// duration   = "P" (dur-date / dur-time / dur-week)
/// ```
fn rfc3339_duration(text: &str) -> bool {
    /// The designators read from `text`, each after one or more digits; `None` when a
    /// designator has no digits or digits trail.
    fn parts(text: &str) -> Option<Vec<char>> {
        let mut out = Vec::new();
        let mut digits = 0;
        for c in text.chars() {
            if c.is_ascii_digit() {
                digits += 1;
            } else {
                if digits == 0 {
                    return None;
                }
                out.push(c);
                digits = 0;
            }
        }
        (digits == 0).then_some(out)
    }
    let Some(rest) = text.strip_prefix('P') else {
        return false;
    };
    let (date, time) = match rest.split_once('T') {
        Some((date, time)) => (date, Some(time)),
        None => (rest, None),
    };
    let Some(date) = parts(date) else {
        return false;
    };
    let date_ok = matches!(
        date.as_slice(),
        [] | ['D'] | ['M'] | ['M', 'D'] | ['Y'] | ['Y', 'M'] | ['Y', 'M', 'D']
    );
    let time_ok = match time {
        None => true,
        Some(time) => parts(time).is_some_and(|time| {
            matches!(
                time.as_slice(),
                ['H'] | ['H', 'M'] | ['H', 'M', 'S'] | ['M'] | ['M', 'S'] | ['S']
            )
        }),
    };
    if date.as_slice() == ['W'] {
        return time.is_none();
    }
    date_ok && time_ok && !(date.is_empty() && time.is_none())
}

/// Every duration the loader admits for `window`, `cadence` and `min_trend_window` is stored as
/// a `Duration` the published schema's `format: duration` admits.
#[test]
fn adv_p2_every_admitted_duration_is_an_rfc_3339_duration() {
    for (text, valid) in [
        ("PT6H", true),
        ("P1D", true),
        ("P2W", true),
        ("P1DT12H", true),
        ("PT1H30M", true),
        ("P1W2D", false),
        ("PT1H1S", false),
        ("P1DT1H1S", false),
        ("P1W1DT1H1M1S", false),
        ("P", false),
        ("PT", false),
    ] {
        assert_eq!(rfc3339_duration(text), valid, "oracle on {text}");
    }
    let mut drift = Vec::new();
    for written in [
        "6h",
        "1d",
        "2w",
        "30s",
        "5m",
        "P1W2D",
        "PT1H1S",
        "P1DT1H1S",
        "P1W1DT1H1M1S",
        "PT0.5S",
        "pt6h",
        "p1d",
        "P1d",
        "6H",
        "1D",
    ] {
        for field in ["window", "cadence"] {
            let Ok(definition) = load(&with(RATIO, field, &format!("{field}: \"{written}\"")))
            else {
                continue;
            };
            let held = if field == "window" {
                &definition.data().window.0
            } else {
                &definition.data().cadence.0
            };
            if !rfc3339_duration(held) {
                drift.push(format!("{field} {written:?} loaded as {held:?}"));
            }
        }
    }
    assert!(
        drift.is_empty(),
        "durations outside RFC 3339 Appendix A loaded:\n{}",
        drift.join("\n")
    );
}

/// The forms the correction refuses, each on its own: fractions, lower-case ISO designators,
/// upper-case shorthand units, a sign, a space.
#[test]
fn adv_p2_duration_forms_outside_the_grammar_are_refused() {
    for written in [
        "PT0.5S", "PT0,5S", "pt6h", "p1d", "P1d", "Pt6H", "6H", "1D", "+6h", "6 h", "6hh", "h6",
        "P-1D", "PT-1H", "P1DT", "P1D1D", "PT1S1S", "PT1H1H",
    ] {
        let errors = load(&with(RATIO, "window", &format!("window: \"{written}\"")))
            .expect_err(&format!("{written} is refused"));
        assert!(
            errors.iter().any(|error| matches!(
                &error.rule,
                Rule::Duration { field: "window", value } if value == written
            )),
            "{written}: {}",
            shown(&errors)
        );
    }
}

// ---------------------------------------------------------------------------------------------
// The signals concept page.

/// The page's example definition, verbatim, loads against a registry declaring its two keys,
/// with the values the page shows.
#[test]
fn adv_p2_the_signals_page_example_loads() {
    let start = SIGNALS_PAGE
        .find("```yaml\nid: ")
        .expect("the page's example block")
        + "```yaml\n".len();
    let end = start
        + SIGNALS_PAGE[start..]
            .find("```")
            .expect("the example block ends");
    let example = &SIGNALS_PAGE[start..end];
    let registry = Registry::parse(
        REGISTRY_FILE,
        &registry_of(
            "  cluster: { registry: 'file:registries/clusters.yaml', otel_attribute: k8s.cluster.name }\n  \
             namespace: { registry: open, parent: cluster }\n",
        ),
    )
    .unwrap_or_else(|errors| panic!("{}", shown(&errors)));
    let definition = Definition::parse("definitions/example.yaml", example, &registry)
        .unwrap_or_else(|errors| panic!("the page's example is refused:\n{}", shown(&errors)));
    let data = definition.data();
    assert_eq!(data.signal_id.0, "platform.namespace.request_failure_ratio");
    assert_eq!(data.family.0, "platform");
    assert_eq!(data.window.0, "PT6H");
    assert_eq!(data.cadence.0, "PT1H");
    assert_eq!(data.source.connection.0, "platform-replica");
    assert_eq!(data.rev, 1);
    assert_eq!(
        format!("{:?}", definition.state()),
        "Active",
        "lifecycle: active"
    );
}

// ---------------------------------------------------------------------------------------------
// `denominator.since` (design § 11 D18 as amended).

/// D18's first clause admits `<key>.created` for a declared dimension with a `file` registry, a
/// key named `dimension` included; the shorthand clause applies only when no declared key is
/// `dimension`. Two declared `file` keys, one of them named `dimension`: `dimension.created`
/// names that key, never an ambiguity.
#[test]
fn adv_p2_a_key_named_dimension_is_named_by_its_own_created() {
    let registry = Registry::parse(
        REGISTRY_FILE,
        &registry_of(
            "  dimension: { registry: 'file:registries/dimensions.yaml' }\n  \
             component: { registry: 'file:registries/components.yaml' }\n  \
             suite: { registry: open }\n",
        ),
    )
    .unwrap_or_else(|errors| panic!("{}", shown(&errors)));
    let declared = |keys: &[&str]| -> Vec<DimensionKey> {
        keys.iter()
            .map(|key| DimensionKey((*key).to_owned()))
            .collect()
    };
    assert_eq!(
        parse_since(
            "dimension.created",
            Some(&declared(&["dimension", "component"])),
            &registry
        ),
        Ok(Since::DimensionCreated {
            key: "dimension".to_owned()
        })
    );
    assert_eq!(
        parse_since(
            "dimension.created",
            Some(&declared(&["component", "suite"])),
            &registry
        ),
        Ok(Since::DimensionCreated {
            key: "component".to_owned()
        })
    );
}

/// A key holds no `.`, so `a.b.created` is none of the forms, not an undeclared key.
#[test]
fn adv_p2_a_dotted_name_is_no_form() {
    let registry = registry();
    assert_eq!(
        parse_since(
            "component.x.created",
            Some(&[DimensionKey("component".to_owned())]),
            &registry
        ),
        Err(SinceRefusal::Form {
            value: "component.x.created".to_owned()
        })
    );
}
