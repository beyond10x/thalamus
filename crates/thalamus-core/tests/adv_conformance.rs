//! Adversary cases for the conformance harness and the definition and dimension obligations.
//!
//! Three groups:
//! - the harness's complete-subject snapshot held to the ESS reference runner's semantics
//!   (`ess-conformance` `runtime.ts` `snapshotSubject` and `response.ts` `admitSubjectRow`):
//!   exactly one matching row, held to its complete declared shape;
//! - planted storage defects the harness must turn red on the real suite;
//! - obligation behaviour the synthesized suite never reaches: revisions from `Active` and
//!   `Muted`, a revision's fresh stamp, the input guard against a retired subject, two revisions.

use thalamus_core::conformance::{self, Ports, Report};
use thalamus_model::assertion::{AssertionId, AssertionSnapshot};
use thalamus_model::behaviour::{
    AssertionStorage, DailyRollupStorage, DimensionRegistryStorage, DimensionStorage,
    SignalDefinitionStorage, SignalEvaluationStorage, SignalSampleStorage, TransitionStorage,
};
use thalamus_model::definition::{SignalDefinitionSnapshot, SignalDefinitionState, SignalId};
use thalamus_model::dimension::{
    DimensionKey, DimensionRegistrySnapshot, DimensionSnapshot, RegistryId,
};
use thalamus_model::evaluation::{EvaluationId, SignalEvaluationSnapshot};
use thalamus_model::json;
use thalamus_model::observation::{SampleId, SignalSampleSnapshot};
use thalamus_model::primitives::Timestamp;
use thalamus_model::rollup::{DailyRollupSnapshot, RollupId};
use thalamus_model::server::memory::MemoryPorts;
use thalamus_model::server::wire;
use thalamus_model::transition::{TransitionId, TransitionSnapshot};

fn suite() -> String {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../spec/suite.json");
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

fn failures(report: &Report) -> String {
    report
        .failed
        .iter()
        .map(|failure| format!("{}: {}", failure.scenario, failure.reason))
        .collect::<Vec<_>>()
        .join("\n")
}

/// A one-scenario suite of `steps`.
fn probe(steps: &[String]) -> String {
    format!(
        r#"{{"provenance": {{"scenario_initial_state": "empty"}},
            "scenarios": {{"probe": {{"purpose": "p", "source": [], "steps": [{}]}}}}}}"#,
        steps.join(",")
    )
}

fn run_probe(steps: &[String]) -> Report {
    let report = conformance::run(&probe(steps), &|_| true, MemoryPorts::default)
        .expect("the probe suite runs");
    assert_eq!(report.ran, ["probe"]);
    report
}

fn step(text: &str) -> String {
    text.to_owned()
}

// ---------------------------------------------------------------------------------------------
// A. The complete-subject snapshot.
// ---------------------------------------------------------------------------------------------

fn declare(registry: &str) -> String {
    format!(
        r#"{{"step": "execute_command", "command": "thalamus.dimension.DeclareRegistry",
            "actor": "thalamus.dimension.InstanceOperator",
            "input": {{"registry_id": {{"kind": "literal", "value": "{registry}"}},
                      "title": {{"kind": "literal", "value": "t"}}}}}}"#
    )
}

fn registry_snapshot(subject: &str, extra_fields: &str, title_type: &str) -> String {
    format!(
        r#"{{"step": "snapshot_complete_subject", "view": "thalamus.dimension.Registries",
            "subject": {{"registry_id": {{"kind": "literal", "value": "{subject}"}}}},
            "shape": {{"identity_field": "registry_id",
                      "fields": [{{"name": "registry_id", "type": "thalamus.dimension.RegistryId"}},
                                 {{"name": "title", "type": "{title_type}"}},
                                 {{"name": "state", "type": "thalamus.dimension.DimensionRegistry.State"}}{extra_fields}],
                      "declarations": {{"thalamus.dimension.RegistryId": {{"kind": "newtype", "of": "String"}},
                                       "thalamus.dimension.DimensionRegistry.State": {{"kind": "enum", "variants": ["Declared"]}}}}}}}}"#
    )
}

fn query_registries() -> String {
    step(r#"{"step": "query_view", "view": "thalamus.dimension.Registries"}"#)
}

fn registries_unchanged() -> String {
    step(
        r#"{"step": "expect_complete_subject_unchanged", "view": "thalamus.dimension.Registries"}"#,
    )
}

/// The reference runner fails a subject snapshot that matches no row ("matched 0 rows, want
/// exactly one"); a refusal that "leaves the row as the first call made it" says nothing when
/// there is no row.
#[test]
fn a_snapshot_of_a_subject_the_view_does_not_serve_fails() {
    let report = run_probe(&[
        declare("r1"),
        query_registries(),
        registry_snapshot("absent", "", "String"),
        declare("r1"),
        query_registries(),
        registries_unchanged(),
    ]);
    assert_eq!(
        report.failed.len(),
        1,
        "a snapshot of a subject no row carries passed"
    );
}

/// The reference runner holds the snapshot row to its complete declared shape: a declared,
/// non-optional field the row does not serve fails it.
#[test]
fn a_snapshot_row_lacking_a_declared_field_fails() {
    let report = run_probe(&[
        declare("r1"),
        query_registries(),
        registry_snapshot("r1", r#",{"name": "owner", "type": "String"}"#, "String"),
    ]);
    assert_eq!(
        report.failed.len(),
        1,
        "a snapshot row lacking the declared field owner passed"
    );
}

/// The reference runner holds each served field to its declared type: a string `title` declared
/// `Integer` fails it.
#[test]
fn a_snapshot_row_breaking_a_declared_field_type_fails() {
    let report = run_probe(&[
        declare("r1"),
        query_registries(),
        registry_snapshot("r1", "", "Integer"),
    ]);
    assert_eq!(
        report.failed.len(),
        1,
        "a snapshot row whose title is not the declared Integer passed"
    );
}

// ---------------------------------------------------------------------------------------------
// B. Planted storage defects on the real suite.
// ---------------------------------------------------------------------------------------------

#[derive(Clone, Copy)]
enum Defect {
    /// `list` answers nothing for definitions and dimensions; `get` and `put` work.
    ViewsBlind,
    /// A definition already stored is never replaced.
    LostUpdates,
    /// A dimension is stored without its parent.
    ParentDropped,
    /// A retired definition reads as absent.
    RetiredHidden,
}

struct Planted {
    inner: MemoryPorts,
    defect: Defect,
}

macro_rules! plain {
    ($port:ident, $identity:ty, $snapshot:ty) => {
        impl $port for Planted {
            fn get(&self, identity: &$identity) -> Option<$snapshot> {
                $port::get(&self.inner, identity)
            }
            fn put(&mut self, snapshot: $snapshot) {
                $port::put(&mut self.inner, snapshot);
            }
            fn delete(&mut self, identity: &$identity) {
                $port::delete(&mut self.inner, identity);
            }
            fn list(&self) -> Vec<$snapshot> {
                $port::list(&self.inner)
            }
        }
    };
}

plain!(AssertionStorage, AssertionId, AssertionSnapshot);
plain!(
    DimensionRegistryStorage,
    RegistryId,
    DimensionRegistrySnapshot
);
plain!(
    SignalEvaluationStorage,
    EvaluationId,
    SignalEvaluationSnapshot
);
plain!(SignalSampleStorage, SampleId, SignalSampleSnapshot);
plain!(TransitionStorage, TransitionId, TransitionSnapshot);
plain!(DailyRollupStorage, RollupId, DailyRollupSnapshot);

impl SignalDefinitionStorage for Planted {
    fn get(&self, identity: &SignalId) -> Option<SignalDefinitionSnapshot> {
        let held = SignalDefinitionStorage::get(&self.inner, identity);
        match self.defect {
            Defect::RetiredHidden => {
                held.filter(|held| held.state != SignalDefinitionState::Retired)
            }
            _ => held,
        }
    }
    fn put(&mut self, snapshot: SignalDefinitionSnapshot) {
        if matches!(self.defect, Defect::LostUpdates)
            && SignalDefinitionStorage::get(&self.inner, &snapshot.data.signal_id).is_some()
        {
            return;
        }
        SignalDefinitionStorage::put(&mut self.inner, snapshot);
    }
    fn delete(&mut self, identity: &SignalId) {
        SignalDefinitionStorage::delete(&mut self.inner, identity);
    }
    fn list(&self) -> Vec<SignalDefinitionSnapshot> {
        match self.defect {
            Defect::ViewsBlind => Vec::new(),
            _ => SignalDefinitionStorage::list(&self.inner),
        }
    }
}

impl DimensionStorage for Planted {
    fn get(&self, identity: &DimensionKey) -> Option<DimensionSnapshot> {
        DimensionStorage::get(&self.inner, identity)
    }
    fn put(&mut self, mut snapshot: DimensionSnapshot) {
        if matches!(self.defect, Defect::ParentDropped) {
            snapshot.data.parent = None;
        }
        DimensionStorage::put(&mut self.inner, snapshot);
    }
    fn delete(&mut self, identity: &DimensionKey) {
        DimensionStorage::delete(&mut self.inner, identity);
    }
    fn list(&self) -> Vec<DimensionSnapshot> {
        match self.defect {
            Defect::ViewsBlind => Vec::new(),
            _ => DimensionStorage::list(&self.inner),
        }
    }
}

fn planted_run(defect: Defect, scenarios: &[&str]) -> Report {
    let report = conformance::run(&suite(), &|id| scenarios.contains(&id), || Planted {
        inner: MemoryPorts::default(),
        defect,
    })
    .expect("the suite runs");
    assert_eq!(
        report.ran.len(),
        scenarios.len(),
        "a named scenario is not in the suite"
    );
    report
}

fn assert_all_red(defect: Defect, scenarios: &[&str]) {
    let report = planted_run(defect, scenarios);
    let red: Vec<&str> = report.failed.iter().map(|f| f.scenario.as_str()).collect();
    for scenario in scenarios {
        assert!(
            red.contains(scenario),
            "{scenario} passed over a planted defect; failed were:\n{}",
            failures(&report)
        );
    }
}

#[test]
fn lost_definition_updates_turn_the_run_red() {
    assert_all_red(
        Defect::LostUpdates,
        &[
            "thalamus.definition.ReviseDefinition/outcome/revised",
            "thalamus.definition.SignalDefinition/transition/activate/by/thalamus.definition.ActivateDefinition/activated",
        ],
    );
}

#[test]
fn a_dropped_parent_turns_the_run_red() {
    assert_all_red(
        Defect::ParentDropped,
        &["thalamus.dimension.RegisterDimension/outcome/registered"],
    );
}

#[test]
fn a_hidden_retired_definition_turns_the_run_red() {
    assert_all_red(
        Defect::RetiredHidden,
        &["thalamus.definition.ReviseDefinition/outcome/retired"],
    );
}

/// Storage whose views serve nothing (a projection that never caught up) still has to fail the
/// two "leaves the row as the first call made it" scenarios: their only check of the row is the
/// complete-subject snapshot.
#[test]
fn views_that_serve_nothing_turn_the_already_scenarios_red() {
    assert_all_red(
        Defect::ViewsBlind,
        &[
            "thalamus.definition.ProposeDefinition/outcome/already-defined",
            "thalamus.dimension.RegisterDimension/outcome/already-registered",
        ],
    );
}

// ---------------------------------------------------------------------------------------------
// C. Obligation behaviour the synthesized suite never reaches.
// ---------------------------------------------------------------------------------------------

fn lit(value: &str) -> String {
    format!(r#"{{"kind": "literal", "value": {value}}}"#)
}

/// The suite's semantic input for a definition body; `kind` and `baselines` vary.
fn body(signal: &str, title: &str, kind: &str, baselines: &str, author: &str) -> String {
    format!(
        r#"{{"author": {author}, "cadence": {cadence}, "denominator": {denominator},
            "dimensions": {dimensions}, "evaluation": {evaluation}, "extractor": {extractor},
            "family": {family}, "history": {history}, "kind": {kind}, "links": {links},
            "objective": {objective}, "owner": {owner}, "query": {query},
            "signal_id": {signal}, "source": {source}, "title": {title}, "unit": {unit},
            "window": {window}}}"#,
        author = lit(&format!("\"{author}\"")),
        cadence = lit("\"PT1S\""),
        denominator =
            lit(r#"{"min_n": 1.0, "since": "denominator.since", "unit": "denominator.unit"}"#),
        dimensions = lit(r#"["dimensionsj0"]"#),
        evaluation = lit(&format!(
            r#"{{"baselines": {baselines}, "crit": {{"abs": 2.0, "min_events": 1.0, "x": 2.0}},
                "min_trend_window": "PT1S", "rule": "evaluation.rule",
                "warn": {{"abs": 1.0, "min_events": 1.0, "x": 1.0}}}}"#
        )),
        extractor = lit("\"extractor\""),
        family = lit("\"family\""),
        history = lit("\"source\""),
        kind = lit(&format!("\"{kind}\"")),
        links = lit(r#"{"components": ["c"], "issues": ["i"]}"#),
        objective = lit(r#"{"objective": "objective"}"#),
        owner = lit("\"owner\""),
        query = lit(r#"{"query": "query.0"}"#),
        signal = lit(&format!("\"{signal}\"")),
        source = lit(r#"{"adapter": "sourcejadapter", "connection": "source.connection"}"#),
        title = lit(&format!("\"{title}\"")),
        unit = lit("\"unit\""),
        window = lit("\"PT1S\""),
    )
}

fn definition_command(command: &str, input: &str) -> String {
    format!(
        r#"{{"step": "execute_command", "command": "thalamus.definition.{command}",
            "actor": "thalamus.definition.InstanceOperator", "input": {input}}}"#
    )
}

fn by_id(command: &str, signal: &str) -> String {
    definition_command(
        command,
        &format!(r#"{{"signal_id": {}}}"#, lit(&format!("\"{signal}\""))),
    )
}

fn outcome(command: &str, outcome: &str) -> String {
    format!(
        r#"{{"step": "expect_outcome", "outcome": {{"command": "thalamus.definition.{command}",
            "outcome": "{outcome}"}}}}"#
    )
}

fn contains(view: &str, fields: &str) -> String {
    format!(
        r#"{{"step": "query_view", "view": "thalamus.definition.{view}"}},
           {{"step": "expect_view", "view": "thalamus.definition.{view}",
             "expectation": {{"expect": "contains", "fields": {fields}}}}}"#
    )
}

fn propose(signal: &str) -> [String; 2] {
    [
        definition_command(
            "ProposeDefinition",
            &body(signal, "first", "gauge", "[]", "author-1"),
        ),
        outcome("ProposeDefinition", "proposed"),
    ]
}

fn revise(signal: &str, title: &str) -> [String; 2] {
    [
        definition_command(
            "ReviseDefinition",
            &body(signal, title, "gauge", "[]", "author-2"),
        ),
        outcome("ReviseDefinition", "revised"),
    ]
}

/// `revised` moves nothing (`updates:`, no `moves:`); the lifecycle declares no way back to
/// `Proposed`, so a revised Active definition stays Active and stays in `ActiveDefinitions`.
#[test]
fn a_revised_active_definition_stays_active() {
    let mut steps = Vec::new();
    steps.extend(propose("sig_a"));
    steps.push(by_id("ActivateDefinition", "sig_a"));
    steps.push(outcome("ActivateDefinition", "activated"));
    steps.extend(revise("sig_a", "second"));
    steps.push(contains(
        "Definitions",
        &format!(
            r#"{{"signal_id": {}, "state": {}, "rev": {}, "title": {}}}"#,
            lit("\"sig_a\""),
            lit("\"Active\""),
            lit("2.0"),
            lit("\"second\"")
        ),
    ));
    steps.push(contains(
        "ActiveDefinitions",
        &format!(
            r#"{{"signal_id": {}, "rev": {}}}"#,
            lit("\"sig_a\""),
            lit("2.0")
        ),
    ));
    let report = run_probe(&steps);
    assert!(report.failed.is_empty(), "{}", failures(&report));
}

/// A revision of a Muted definition keeps it Muted with its reason and expiry: the invariant
/// "Muted iff a mute is carried" must still hold.
#[test]
fn a_revised_muted_definition_stays_muted_with_its_mute() {
    let mut steps = Vec::new();
    steps.extend(propose("sig_m"));
    steps.push(by_id("ActivateDefinition", "sig_m"));
    steps.push(outcome("ActivateDefinition", "activated"));
    steps.push(definition_command(
        "MuteDefinition",
        &format!(
            r#"{{"signal_id": {}, "reason": {}, "until": {{"kind": "now_offset", "seconds": 3600}}}}"#,
            lit("\"sig_m\""),
            lit("\"maintenance\"")
        ),
    ));
    steps.push(outcome("MuteDefinition", "muted"));
    steps.extend(revise("sig_m", "second"));
    steps.push(contains(
        "Definitions",
        &format!(
            r#"{{"signal_id": {}, "state": {}, "rev": {}, "mute_reason": {},
                 "mute_until": {{"kind": "now_offset", "seconds": 3600}}}}"#,
            lit("\"sig_m\""),
            lit("\"Muted\""),
            lit("2.0"),
            lit("\"maintenance\"")
        ),
    ));
    let report = run_probe(&steps);
    assert!(report.failed.is_empty(), "{}", failures(&report));
}

/// Selection precedence (PLAN.md): input-guarded refusals before the held state. A retired
/// definition revised with an instant body under a non-fixed baseline answers
/// `instant-not-fixed`, and nothing changes.
#[test]
fn the_instant_guard_precedes_the_retired_subject() {
    let mut steps = Vec::new();
    steps.extend(propose("sig_r"));
    steps.push(by_id("RetireDefinition", "sig_r"));
    steps.push(outcome("RetireDefinition", "retired"));
    steps.push(definition_command(
        "ReviseDefinition",
        &body("sig_r", "second", "instant", r#"["offset_1d"]"#, "author-2"),
    ));
    steps.push(outcome("ReviseDefinition", "instant-not-fixed"));
    steps.push(step(
        r#"{"step": "expect_error", "error": "thalamus.definition.InstantNeedsFixedBaseline",
            "fields": {"signal_id": "sig_r"}}"#,
    ));
    steps.push(contains(
        "Definitions",
        &format!(
            r#"{{"signal_id": {}, "state": {}, "rev": {}, "title": {}}}"#,
            lit("\"sig_r\""),
            lit("\"Retired\""),
            lit("1.0"),
            lit("\"first\"")
        ),
    ));
    let report = run_probe(&steps);
    assert!(report.failed.is_empty(), "{}", failures(&report));
}

/// Two revisions: `rev` 3, and the second `DefinitionRevised` names `previous_rev` 2.
#[test]
fn a_second_revision_counts_from_the_first() {
    let mut steps = Vec::new();
    steps.extend(propose("sig_t"));
    steps.extend(revise("sig_t", "second"));
    steps.extend(revise("sig_t", "third"));
    steps.push(step(
        r#"{"step": "expect_event", "event": "thalamus.definition.DefinitionRevised",
            "payload": {"previous_rev": 2.0, "signal_id": "sig_t", "revised_by": "author-2"},
            "shape": {"previous_rev": {"holds": "primitive", "kind": "integer"},
                      "revised_by": {"holds": "primitive", "kind": "string"},
                      "signal_id": {"holds": "primitive", "kind": "string"}}}"#,
    ));
    steps.push(contains(
        "Definitions",
        &format!(
            r#"{{"signal_id": {}, "rev": {}, "title": {}}}"#,
            lit("\"sig_t\""),
            lit("3.0"),
            lit("\"third\"")
        ),
    ));
    let report = run_probe(&steps);
    assert!(report.failed.is_empty(), "{}", failures(&report));
}

/// The wire body of a definition, as `wire::decode_command_*` reads it.
fn wire_body(title: &str, author: &str) -> json::Value {
    json::parse(&format!(
        r#"{{"signal_id": "sig_s", "title": "{title}", "family": "family", "kind": "gauge",
            "unit": "unit", "dimensions": ["dimensionsj0"],
            "source": {{"adapter": "sourcejadapter", "connection": "source.connection"}},
            "extractor": "extractor", "query": {{"query": "query.0"}}, "window": "PT1S",
            "cadence": "PT1S", "history": "source",
            "denominator": {{"unit": "unit", "min_n": 1, "since": "since"}},
            "evaluation": {{"rule": "evaluation.rule", "baselines": [],
                           "warn": {{"x": "1.0", "abs": "1.0", "min_events": 1}},
                           "crit": {{"x": "2.0", "abs": "2.0", "min_events": 1}},
                           "min_trend_window": "PT1S"}},
            "objective": {{"objective": "objective"}}, "owner": "owner",
            "links": {{"components": [], "issues": []}}, "author": "{author}"}}"#
    ))
    .expect("the body is JSON")
}

/// `revised_at: {generated: true}` on `revised`: a revision is stamped with the instant it was
/// decided at, not the proposal's. The harness decides every command at one fixed instant, so
/// the suite cannot tell the two apart; this drives the obligation over two clocks.
#[test]
fn a_revision_is_stamped_with_its_own_instant() {
    let first = Timestamp("2026-01-01T00:00:00Z".to_owned());
    let later = Timestamp("2026-01-02T00:00:00Z".to_owned());
    let mut ports = Ports::new(MemoryPorts::default(), Some(first.clone()));
    let proposal = wire::decode_command_thalamus_definition_propose_definition(
        &wire_body("first", "a"),
        "body",
    )
    .expect("the proposal decodes");
    thalamus_core::behaviour::definition::propose_definition(&mut ports, proposal)
        .expect("the proposal answers");
    let id = SignalId("sig_s".to_owned());
    assert_eq!(
        SignalDefinitionStorage::get(&ports, &id)
            .expect("proposed")
            .data
            .revised_at,
        first
    );
    ports.clock = Some(later.clone());
    let revision = wire::decode_command_thalamus_definition_revise_definition(
        &wire_body("second", "b"),
        "body",
    )
    .expect("the revision decodes");
    thalamus_core::behaviour::definition::revise_definition(&mut ports, revision)
        .expect("the revision answers");
    let held = SignalDefinitionStorage::get(&ports, &id).expect("revised");
    assert_eq!(held.data.rev, 2);
    assert_eq!(
        held.data.revised_at, later,
        "the revision kept the proposal's instant"
    );
}
