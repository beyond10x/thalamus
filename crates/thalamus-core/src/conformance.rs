//! The conformance harness: runs the scenarios of a suite written by `ess verify conform
//! synthesize` (`spec/suite.json`) in process, against the generated behaviours over the storage
//! a caller supplies.
//!
//! Every command and view goes through the generated transport-free entry point
//! (`thalamus_model::server::thalamus_service::handle`), so a scenario checks the grant, the
//! declared input and the declared outcome exactly as the HTTP routes do. Each command decides at
//! one fixed instant, the command clock this harness forces through the context port, and every
//! `now_offset` value is counted from it.
//!
//! A suite value is written as a semantic value (`1.0` for an `Integer`, a number for a
//! `Decimal`); the harness renders it into the published wire form by the type the committed
//! `OpenAPI` document declares for the field, and compares the same way.
//!
//! Nothing is skipped: an unknown step, step member, value kind, expectation, predicate form,
//! field shape or initial state fails the scenario or the run.

use std::collections::BTreeMap;
use std::panic::{AssertUnwindSafe, catch_unwind};

use thalamus_model::actor::{Actor, Caller};
use thalamus_model::behaviour::{
    AssertionStorage, Context, DimensionRegistryStorage, DimensionStorage, Generated,
    SignalDefinitionStorage, SignalEvaluationStorage, SignalSampleStorage,
};
use thalamus_model::json::{self, Value};
use thalamus_model::obligation::UnmetObligation;
use thalamus_model::ports::thalamus_service::ThalamusService;
use thalamus_model::primitives::Timestamp;
use thalamus_model::server::entry::Refused;
use thalamus_model::server::thalamus_service::handle;
use thalamus_model::system::System;
use thalamus_model::{assertion, definition, dimension, evaluation, observation};

/// The published wire contract the scenario values are rendered into.
const OPENAPI: &str =
    include_str!("../../../generated/thalamus-model/src/server/thalamus-service.openapi.json");

/// The instant every command of a scenario decides at, in seconds from the Unix epoch:
/// 2026-01-01T00:00:00Z.
const CLOCK: i64 = 1_767_225_600;

/// Every storage port a generated behaviour or query of this system reads or writes.
pub trait Storage:
    AssertionStorage
    + SignalDefinitionStorage
    + DimensionStorage
    + DimensionRegistryStorage
    + SignalEvaluationStorage
    + SignalSampleStorage
{
}

impl<S> Storage for S where
    S: AssertionStorage
        + SignalDefinitionStorage
        + DimensionStorage
        + DimensionRegistryStorage
        + SignalEvaluationStorage
        + SignalSampleStorage
{
}

/// The ports a scenario runs over: the caller's storage, the forced command clock, and the
/// behaviours the synthesis plan owes, met by `crate::behaviour`.
pub struct Ports<S> {
    /// Where every entity is stored.
    pub storage: S,
    /// The instant each command decides at; `None` is a context with no clock.
    pub clock: Option<Timestamp>,
}

macro_rules! delegate_storage {
    ($port:ident, $identity:ty, $snapshot:ty) => {
        impl<S: $port> $port for Ports<S> {
            fn get(&self, identity: &$identity) -> Option<$snapshot> {
                $port::get(&self.storage, identity)
            }

            fn put(&mut self, snapshot: $snapshot) {
                $port::put(&mut self.storage, snapshot);
            }

            fn delete(&mut self, identity: &$identity) {
                $port::delete(&mut self.storage, identity);
            }

            fn list(&self) -> Vec<$snapshot> {
                $port::list(&self.storage)
            }
        }
    };
}

delegate_storage!(
    AssertionStorage,
    assertion::AssertionId,
    assertion::AssertionSnapshot
);
delegate_storage!(
    SignalDefinitionStorage,
    definition::SignalId,
    definition::SignalDefinitionSnapshot
);
delegate_storage!(
    DimensionStorage,
    dimension::DimensionKey,
    dimension::DimensionSnapshot
);
delegate_storage!(
    DimensionRegistryStorage,
    dimension::RegistryId,
    dimension::DimensionRegistrySnapshot
);
delegate_storage!(
    SignalEvaluationStorage,
    evaluation::EvaluationId,
    evaluation::SignalEvaluationSnapshot
);
delegate_storage!(
    SignalSampleStorage,
    observation::SampleId,
    observation::SignalSampleSnapshot
);

impl<S> Context for Ports<S> {
    fn command_clock(&mut self) -> Option<Timestamp> {
        self.clock.clone()
    }
}

impl<S: SignalDefinitionStorage> definition::obligations::ProposeDefinitionBehavior for Ports<S> {
    fn propose_definition(
        &mut self,
        input: definition::ProposeDefinition,
    ) -> Result<definition::ProposeDefinitionOutcome, UnmetObligation> {
        crate::behaviour::definition::propose_definition(self, input)
    }
}

impl<S: SignalDefinitionStorage> definition::obligations::ReviseDefinitionBehavior for Ports<S> {
    fn revise_definition(
        &mut self,
        input: definition::ReviseDefinition,
    ) -> Result<definition::ReviseDefinitionOutcome, UnmetObligation> {
        crate::behaviour::definition::revise_definition(self, input)
    }
}

impl<S: DimensionStorage + DimensionRegistryStorage>
    dimension::obligations::RegisterDimensionBehavior for Ports<S>
{
    fn register_dimension(
        &mut self,
        input: dimension::RegisterDimension,
    ) -> Result<dimension::RegisterDimensionOutcome, UnmetObligation> {
        crate::behaviour::dimension::register_dimension(self, input)
    }
}

// The obligations of the observation, evaluation and assertion domains are not met yet: each
// answers the generated typed refusal, which fails any scenario that reaches it.

impl<S> assertion::obligations::AskAssertionBehavior for Ports<S> {
    fn ask_assertion(
        &mut self,
        input: assertion::AskAssertion,
    ) -> Result<assertion::AskAssertionOutcome, UnmetObligation> {
        assertion::obligations::AskAssertionBehavior::ask_assertion(
            &mut assertion::obligations::Unimplemented,
            input,
        )
    }
}

impl<S> evaluation::obligations::RecordEvaluationBehavior for Ports<S> {
    fn record_evaluation(
        &mut self,
        input: evaluation::RecordEvaluation,
    ) -> Result<evaluation::RecordEvaluationOutcome, UnmetObligation> {
        evaluation::obligations::RecordEvaluationBehavior::record_evaluation(
            &mut evaluation::obligations::Unimplemented,
            input,
        )
    }
}

impl<S> observation::obligations::RecordSampleBehavior for Ports<S> {
    fn record_sample(
        &mut self,
        input: observation::RecordSample,
    ) -> Result<observation::RecordSampleOutcome, UnmetObligation> {
        observation::obligations::RecordSampleBehavior::record_sample(
            &mut observation::obligations::Unimplemented,
            input,
        )
    }
}

impl<S> evaluation::obligations::SeriesVerdictsQuery for Ports<S> {
    fn series_verdicts(
        &self,
        series: observation::SeriesKey,
    ) -> Result<Vec<evaluation::SeriesVerdicts>, UnmetObligation> {
        evaluation::obligations::SeriesVerdictsQuery::series_verdicts(
            &evaluation::obligations::Unimplemented,
            series,
        )
    }
}

impl<S> observation::obligations::SeriesSamplesQuery for Ports<S> {
    fn series_samples(
        &self,
        series: observation::SeriesKey,
    ) -> Result<Vec<observation::SeriesSamples>, UnmetObligation> {
        observation::obligations::SeriesSamplesQuery::series_samples(
            &observation::obligations::Unimplemented,
            series,
        )
    }
}

/// What a run answered: every scenario it ran, in suite order, and each one that failed.
#[derive(Debug, Default)]
pub struct Report {
    /// The id of every scenario run.
    pub ran: Vec<String>,
    /// Every scenario that failed, with the first reason.
    pub failed: Vec<Failure>,
}

/// One failed scenario.
#[derive(Debug)]
pub struct Failure {
    /// The scenario id.
    pub scenario: String,
    /// Why it failed.
    pub reason: String,
}

/// Runs every scenario of `suite` (the text of `spec/suite.json`) whose id `select` admits, each
/// over fresh storage from `storage`.
///
/// # Errors
///
/// A message when the suite is not one this harness can run at all: not JSON, no scenario map, or
/// an initial state other than `empty`. A scenario that fails is reported in [`Report::failed`].
pub fn run<S, F>(
    suite: &str,
    select: &dyn Fn(&str) -> bool,
    mut storage: F,
) -> Result<Report, String>
where
    S: Storage,
    F: FnMut() -> S,
{
    let suite = json::parse(suite).map_err(|error| format!("the suite is not JSON: {error}"))?;
    let schemas = Schemas::load()?;
    let initial = suite
        .member("provenance")
        .and_then(|provenance| provenance.member("scenario_initial_state"));
    match initial {
        None => {}
        Some(Value::Text(state)) if state == "empty" => {}
        Some(other) => {
            return Err(format!(
                "unsupported scenario initial state {}",
                show(other)
            ));
        }
    }
    let Some(Value::Object(scenarios)) = suite.member("scenarios") else {
        return Err("the suite holds no scenario map".to_owned());
    };
    let mut report = Report::default();
    for (id, scenario) in scenarios {
        if !select(id) {
            continue;
        }
        report.ran.push(id.clone());
        let ports = Ports {
            storage: storage(),
            clock: Some(Timestamp(timestamp(CLOCK))),
        };
        let outcome = catch_unwind(AssertUnwindSafe(|| {
            Scenario::new(ports, &schemas).play(scenario)
        }));
        let reason = match outcome {
            Ok(Ok(())) => continue,
            Ok(Err(reason)) => reason,
            Err(panic) => panic
                .downcast_ref::<String>()
                .cloned()
                .or_else(|| panic.downcast_ref::<&str>().map(|text| (*text).to_owned()))
                .map_or_else(|| "panicked".to_owned(), |text| format!("panicked: {text}")),
        };
        report.failed.push(Failure {
            scenario: id.clone(),
            reason,
        });
    }
    Ok(report)
}

/// What the last command answered.
enum Answered {
    /// The grant refused the caller before the command ran; carries the actor the refusal names.
    NotGranted(Option<String>),
    /// A declared outcome.
    Outcome {
        outcome: String,
        error: Option<String>,
        /// The error's payload, or null where no error was answered.
        payload: Value,
        events: Vec<(String, Value)>,
    },
}

struct Last {
    command: String,
    answered: Answered,
}

impl Last {
    fn events(&self) -> &[(String, Value)] {
        match &self.answered {
            Answered::NotGranted(_) => &[],
            Answered::Outcome { events, .. } => events,
        }
    }

    fn outcome(&self) -> Result<(&str, Option<&str>, &Value), String> {
        match &self.answered {
            Answered::NotGranted(actor) => Err(format!(
                "{} was refused as not granted to {}",
                self.command,
                actor.as_deref().unwrap_or("no actor")
            )),
            Answered::Outcome {
                outcome,
                error,
                payload,
                ..
            } => Ok((outcome, error.as_deref(), payload)),
        }
    }

    fn event(&self, name: &str) -> Result<&Value, String> {
        self.events()
            .iter()
            .find(|(published, _)| published == name)
            .map(|(_, payload)| payload)
            .ok_or_else(|| format!("{} published no {name}", self.command))
    }
}

/// One subject's row, as `snapshot_complete_subject` took it.
struct Snapshot {
    shape: SubjectShape,
    identity: Value,
    row: Value,
}

struct Scenario<'a, S: Storage> {
    system: System<Generated<Ports<S>>>,
    schemas: &'a Schemas,
    captured: BTreeMap<String, Value>,
    observed: Vec<(String, Value)>,
    last: Option<Last>,
    views: BTreeMap<String, Vec<Value>>,
    /// The view the last `query_view` read, unless a command has run since.
    queried: Option<String>,
    snapshots: BTreeMap<String, Snapshot>,
}

impl<'a, S: Storage> Scenario<'a, S> {
    fn new(ports: Ports<S>, schemas: &'a Schemas) -> Self {
        Self {
            system: System::new(ThalamusService::new(Generated::new(ports))),
            schemas,
            captured: BTreeMap::new(),
            observed: Vec::new(),
            last: None,
            views: BTreeMap::new(),
            queried: None,
            snapshots: BTreeMap::new(),
        }
    }

    fn play(mut self, scenario: &Value) -> Result<(), String> {
        keys(scenario, &["purpose", "source", "steps"], "a scenario")?;
        let Some(Value::Array(steps)) = scenario.member("steps") else {
            return Err("the scenario holds no steps".to_owned());
        };
        for (index, step) in steps.iter().enumerate() {
            self.step(step)
                .map_err(|reason| format!("step {index} ({}): {reason}", show(step)))?;
        }
        Ok(())
    }

    fn last(&self) -> Result<&Last, String> {
        self.last
            .as_ref()
            .ok_or_else(|| "no command ran".to_owned())
    }

    fn step(&mut self, step: &Value) -> Result<(), String> {
        let kind = text_at(step, "step")?;
        let allowed: &[&str] = match kind {
            "execute_command" => &["step", "command", "actor", "input"],
            "expect_outcome" => &["step", "outcome"],
            "expect_error" => &["step", "error", "fields"],
            "expect_no_error" | "expect_no_events" => &["step"],
            "expect_event" | "expect_event_values" => &["step", "event", "payload", "shape"],
            "expect_no_event" => &["step", "event"],
            "expect_not_granted" => &["step", "actor", "unpublished"],
            "capture_instance" => &["step", "instance", "entity", "event", "field"],
            "query_view" => &["step", "view"],
            "expect_view" => &["step", "view", "expectation"],
            "snapshot_complete_subject" => &["step", "view", "subject", "shape"],
            "expect_complete_subject_unchanged" => &["step", "view"],
            other => return Err(format!("unsupported step {other}")),
        };
        keys(step, allowed, kind)?;
        match kind {
            "execute_command" => self.execute(step),
            "expect_outcome" => self.expect_outcome(step),
            "expect_error" => self.expect_error(step),
            "expect_no_error" => match self.last()?.outcome()? {
                (_, None, _) => Ok(()),
                (outcome, Some(error), _) => {
                    Err(format!("outcome {outcome} answered the error {error}"))
                }
            },
            "expect_no_events" => match self.last()?.events() {
                [] => Ok(()),
                events => Err(format!("published {}", names(events))),
            },
            "expect_no_event" => {
                let name = text_at(step, "event")?;
                let last = self.last()?;
                if last.events().iter().any(|(published, _)| published == name) {
                    return Err(format!("{} published {name}", last.command));
                }
                Ok(())
            }
            "expect_event" | "expect_event_values" => self.expect_event(step, kind),
            "expect_not_granted" => self.expect_not_granted(step),
            "capture_instance" => {
                let event = text_at(step, "event")?;
                let field = text_at(step, "field")?;
                // The captured value is an instance of `entity`, named by that entity's own field.
                self.schemas.entity_field(text_at(step, "entity")?, field)?;
                let value = self
                    .last()?
                    .event(event)?
                    .member(field)
                    .cloned()
                    .ok_or_else(|| format!("{event} carries no {field}"))?;
                let instance = text_at(step, "instance")?;
                if self.captured.contains_key(instance) {
                    return Err(format!("{instance} is already bound"));
                }
                self.captured.insert(instance.to_owned(), value);
                Ok(())
            }
            "query_view" => {
                let view = text_at(step, "view")?;
                let rows = self.rows(view)?;
                self.views.insert(view.to_owned(), rows);
                self.queried = Some(view.to_owned());
                Ok(())
            }
            "expect_view" => self.expect_view(step),
            "snapshot_complete_subject" => self.snapshot(step),
            "expect_complete_subject_unchanged" => self.unchanged(step),
            _ => unreachable!("every admitted step has an arm"),
        }
    }

    /// The value a scenario value stands for.
    fn resolve(&self, spec: &Value) -> Result<Value, String> {
        let kind = text_at(spec, "kind")?;
        match kind {
            "literal" => {
                keys(spec, &["kind", "value"], "a literal value")?;
                member(spec, "value").cloned()
            }
            "instance" => {
                keys(spec, &["kind", "instance"], "an instance value")?;
                let name = text_at(spec, "instance")?;
                self.captured
                    .get(name)
                    .cloned()
                    .ok_or_else(|| format!("nothing was captured as {name}"))
            }
            "observed" => {
                keys(spec, &["kind", "event", "field"], "an observed value")?;
                let event = text_at(spec, "event")?;
                let field = text_at(spec, "field")?;
                let (_, payload) = self
                    .observed
                    .iter()
                    .rev()
                    .find(|(published, _)| published == event)
                    .ok_or_else(|| format!("no {event} was observed"))?;
                payload
                    .member(field)
                    .cloned()
                    .ok_or_else(|| format!("the observed {event} carries no {field}"))
            }
            "now_offset" => {
                keys(spec, &["kind", "seconds"], "a now_offset value")?;
                let seconds = match member(spec, "seconds")? {
                    Value::Number(spelling) => spelling
                        .parse::<i64>()
                        .map_err(|_| format!("now_offset seconds {spelling} is not an integer"))?,
                    other => return Err(format!("now_offset seconds {}", show(other))),
                };
                let at = CLOCK
                    .checked_add(seconds)
                    .ok_or_else(|| format!("now_offset {seconds} overflows"))?;
                Ok(Value::Text(timestamp(at)))
            }
            other => Err(format!("unsupported value kind {other}")),
        }
    }

    /// Resolves every member of `fields` and renders it by the type `component` declares for it.
    fn rendered(
        &self,
        component: &str,
        fields: &Value,
        values: bool,
    ) -> Result<Vec<(String, Value)>, String> {
        let Value::Object(members) = fields else {
            return Err(format!(
                "expected an object of fields, found {}",
                show(fields)
            ));
        };
        members
            .iter()
            .map(|(field, written)| {
                let value = if values {
                    self.resolve(written)?
                } else {
                    written.clone()
                };
                let schema = self.schemas.property(component, field)?;
                Ok((field.clone(), self.schemas.render(schema, value)?))
            })
            .collect()
    }

    fn execute(&mut self, step: &Value) -> Result<(), String> {
        let command = text_at(step, "command")?.to_owned();
        let caller = match step.member("actor") {
            None | Some(Value::Null) => None,
            Some(Value::Text(name)) => Some(caller(name)?),
            Some(other) => return Err(format!("actor {}", show(other))),
        };
        let input = match step.member("input") {
            None => Vec::new(),
            Some(fields) => self.rendered(&format!("{command}.Input"), fields, true)?,
        };
        let answered = match handle(
            &mut self.system,
            caller.as_ref(),
            &command,
            Value::Object(input),
        ) {
            Ok(body) => answered(&body)?,
            Err(Refused::NotGranted(actor)) => Answered::NotGranted(actor),
            Err(refused) => {
                return Err(format!(
                    "{command} answered no declared outcome ({}): {refused}",
                    refused.status()
                ));
            }
        };
        if let Answered::Outcome { events, .. } = &answered {
            self.observed.extend(events.iter().cloned());
        }
        self.queried = None;
        self.last = Some(Last { command, answered });
        Ok(())
    }

    fn expect_outcome(&self, step: &Value) -> Result<(), String> {
        let expected = member(step, "outcome")?;
        keys(expected, &["command", "outcome"], "an expected outcome")?;
        let command = text_at(expected, "command")?;
        let wanted = text_at(expected, "outcome")?;
        let last = self.last()?;
        if last.command != command {
            return Err(format!(
                "the last command was {}, not {command}",
                last.command
            ));
        }
        let (outcome, ..) = last.outcome()?;
        if outcome != wanted {
            return Err(format!("{command} took {outcome}, not {wanted}"));
        }
        Ok(())
    }

    fn expect_error(&self, step: &Value) -> Result<(), String> {
        let wanted = text_at(step, "error")?;
        let (outcome, error, payload) = self.last()?.outcome()?;
        if error != Some(wanted) {
            return Err(format!(
                "outcome {outcome} answered {}, not {wanted}",
                error.unwrap_or("no error")
            ));
        }
        if let Some(fields) = step.member("fields") {
            for (field, value) in self.rendered(&format!("{wanted}.Error"), fields, false)? {
                let got = payload.member(&field).unwrap_or(&Value::Null);
                if !same(got, &value) {
                    return Err(format!(
                        "{wanted}.{field} is {}, not {}",
                        show(got),
                        show(&value)
                    ));
                }
            }
        }
        Ok(())
    }

    fn expect_event(&self, step: &Value, kind: &str) -> Result<(), String> {
        let name = text_at(step, "event")?;
        let payload = self.last()?.event(name)?;
        // In `expect_event` each value is the literal the field carries; in `expect_event_values`
        // it is a scenario value to resolve, as an input's is.
        let values = kind == "expect_event_values";
        if let Some(fields) = step.member("payload") {
            for (field, value) in self.rendered(&format!("{name}.Event"), fields, values)? {
                let got = payload.member(&field).unwrap_or(&Value::Null);
                if !same(got, &value) {
                    return Err(format!(
                        "{name}.{field} is {}, not {}",
                        show(got),
                        show(&value)
                    ));
                }
            }
        }
        if let Some(shape) = step.member("shape") {
            check_shape(name, payload, shape)?;
        }
        Ok(())
    }

    fn expect_not_granted(&self, step: &Value) -> Result<(), String> {
        let wanted = match member(step, "actor")? {
            Value::Null => None,
            Value::Text(actor) => Some(actor.as_str()),
            other => return Err(format!("actor {}", show(other))),
        };
        let last = self.last()?;
        match &last.answered {
            Answered::NotGranted(actor) if actor.as_deref() == wanted => {}
            Answered::NotGranted(actor) => {
                return Err(format!(
                    "{} was refused naming {}, not {}",
                    last.command,
                    actor.as_deref().unwrap_or("no actor"),
                    wanted.unwrap_or("no actor")
                ));
            }
            Answered::Outcome { outcome, .. } => {
                return Err(format!(
                    "{} ran and took {outcome}; no grant refused it",
                    last.command
                ));
            }
        }
        let Value::Array(unpublished) = member(step, "unpublished")? else {
            return Err("unpublished is not a list".to_owned());
        };
        for event in unpublished {
            let Value::Text(event) = event else {
                return Err(format!("unpublished event {}", show(event)));
            };
            if last
                .events()
                .iter()
                .any(|(published, _)| published == event)
            {
                return Err(format!("{} published {event}", last.command));
            }
        }
        Ok(())
    }

    /// The rows `view` serves now.
    fn rows(&mut self, view: &str) -> Result<Vec<Value>, String> {
        let body =
            handle(&mut self.system, None, view, Value::Object(Vec::new())).map_err(|refused| {
                format!("{view} answered no rows ({}): {refused}", refused.status())
            })?;
        match body.member("rows") {
            Some(Value::Array(rows)) => Ok(rows.clone()),
            _ => Err(format!("{view} answered {}", show(&body))),
        }
    }

    fn expect_view(&self, step: &Value) -> Result<(), String> {
        let view = text_at(step, "view")?;
        // Rows read before the last command say nothing about what it did.
        if self.queried.as_deref() != Some(view) {
            return Err(format!(
                "an expectation over {view} needs a query of {view} after the last command"
            ));
        }
        let rows = self
            .views
            .get(view)
            .ok_or_else(|| format!("{view} was not queried"))?;
        let expectation = member(step, "expectation")?;
        let expect = text_at(expectation, "expect")?;
        match expect {
            "contains" | "excludes" => {
                keys(expectation, &["expect", "fields"], "a view expectation")?;
                let fields =
                    self.rendered(&format!("{view}.Row"), member(expectation, "fields")?, true)?;
                let matches = |row: &Value| {
                    fields.iter().all(|(field, value)| {
                        same(row.member(field).unwrap_or(&Value::Null), value)
                    })
                };
                let held = rows.iter().any(matches);
                match (expect, held) {
                    ("contains", false) => Err(format!(
                        "{view} holds no row with {}: {}",
                        show(&Value::Object(fields)),
                        show(&Value::Array(rows.clone()))
                    )),
                    ("excludes", true) => Err(format!(
                        "{view} holds a row with {}: {}",
                        show(&Value::Object(fields)),
                        show(&Value::Array(rows.clone()))
                    )),
                    _ => Ok(()),
                }
            }
            "satisfies" => {
                keys(expectation, &["expect", "predicate"], "a view expectation")?;
                let predicate = member(expectation, "predicate")?;
                if rows.is_empty() {
                    return Err(format!("{view} holds no rows"));
                }
                for row in rows {
                    match truth(predicate, row)? {
                        Some(true) => {}
                        Some(false) => {
                            return Err(format!("{} fails for {}", show(predicate), show(row)));
                        }
                        None => {
                            return Err(format!(
                                "{} is undecided for {}",
                                show(predicate),
                                show(row)
                            ));
                        }
                    }
                }
                Ok(())
            }
            other => Err(format!("unsupported expectation {other}")),
        }
    }

    /// The one row of the view the step names, as the `query_view` just before it read it, whose
    /// identity field holds `identity`, held to the subject's complete declared shape.
    fn subject_row(
        &self,
        view: &str,
        shape: &SubjectShape,
        identity: &Value,
    ) -> Result<Value, String> {
        if self.queried.as_deref() != Some(view) {
            return Err(format!(
                "a subject snapshot of {view} needs a query of {view} after the last command"
            ));
        }
        let rows = self
            .views
            .get(view)
            .ok_or_else(|| format!("{view} was not queried"))?;
        let matched: Vec<&Value> = rows
            .iter()
            .filter(|row| {
                same(
                    row.member(&shape.identity_field).unwrap_or(&Value::Null),
                    identity,
                )
            })
            .collect();
        let [row] = matched.as_slice() else {
            return Err(format!(
                "the subject {} = {} matched {} rows of {view}, want exactly one",
                shape.identity_field,
                show(identity),
                matched.len()
            ));
        };
        shape
            .admit(row)
            .map_err(|reason| format!("the subject row of {view} {}: {reason}", show(row)))?;
        Ok((*row).clone())
    }

    fn snapshot(&mut self, step: &Value) -> Result<(), String> {
        let view = text_at(step, "view")?.to_owned();
        let shape = SubjectShape::decode(member(step, "shape")?)?;
        let subject = member(step, "subject")?;
        keys(subject, &[shape.identity_field.as_str()], "a subject")?;
        let identity = self.schemas.render(
            self.schemas
                .property(&format!("{view}.Row"), &shape.identity_field)?,
            self.resolve(member(subject, &shape.identity_field)?)?,
        )?;
        let row = self.subject_row(&view, &shape, &identity)?;
        self.snapshots.insert(
            view,
            Snapshot {
                shape,
                identity,
                row,
            },
        );
        Ok(())
    }

    fn unchanged(&self, step: &Value) -> Result<(), String> {
        let view = text_at(step, "view")?;
        let Some(snapshot) = self.snapshots.get(view) else {
            return Err(format!("no snapshot of {view} was taken"));
        };
        let after = self.subject_row(view, &snapshot.shape, &snapshot.identity)?;
        if !same(&snapshot.row, &after) {
            return Err(format!(
                "{view} changed from {} to {}",
                show(&snapshot.row),
                show(&after)
            ));
        }
        Ok(())
    }
}

/// A declaration of a subject shape's type graph.
enum Declared {
    Newtype(String),
    Enum(Vec<String>),
    Struct(Vec<(String, String)>),
    /// The tag member, and each label with the type it carries, `None` for a unit variant.
    Union(String, Vec<(String, Option<String>)>),
}

/// The complete declared shape of a subject's row, as `snapshot_complete_subject` carries it:
/// every field with its type, and the declarations those types name.
struct SubjectShape {
    identity_field: String,
    fields: Vec<(String, String)>,
    declarations: Vec<(String, Declared)>,
}

/// The primitive types a subject field may hold.
const PRIMITIVES: [&str; 10] = [
    "String",
    "Boolean",
    "Integer",
    "Decimal",
    "Binary64",
    "Timestamp",
    "Duration",
    "Uuid",
    "Bytes",
    "Json",
];

/// The name and type of each `{name, type}` in a list.
fn typed_fields(fields: &Value, what: &str) -> Result<Vec<(String, String)>, String> {
    let Value::Array(fields) = fields else {
        return Err(format!("{what} is not a list"));
    };
    let mut typed: Vec<(String, String)> = Vec::new();
    for field in fields {
        keys(field, &["name", "type"], what)?;
        let name = text_at(field, "name")?;
        if name.is_empty() || typed.iter().any(|(held, _)| held == name) {
            return Err(format!("{what} has an empty or repeated field {name:?}"));
        }
        typed.push((name.to_owned(), text_at(field, "type")?.to_owned()));
    }
    Ok(typed)
}

impl SubjectShape {
    /// Reads a shape, refusing an unknown member, an unknown declaration kind, a type it cannot
    /// resolve, a declaration no field reaches, and an identity that is undeclared or optional.
    fn decode(shape: &Value) -> Result<Self, String> {
        keys(
            shape,
            &["identity_field", "fields", "declarations"],
            "a subject shape",
        )?;
        let identity_field = text_at(shape, "identity_field")?.to_owned();
        let fields = typed_fields(member(shape, "fields")?, "the subject fields")?;
        let Value::Object(bodies) = member(shape, "declarations")? else {
            return Err("the subject declarations are not an object".to_owned());
        };
        let mut declarations = Vec::new();
        for (name, body) in bodies {
            let at = format!("the declaration {name}");
            let declared = match text_at(body, "kind")? {
                "newtype" => {
                    keys(body, &["kind", "of"], &at)?;
                    Declared::Newtype(text_at(body, "of")?.to_owned())
                }
                "enum" => {
                    keys(body, &["kind", "variants"], &at)?;
                    let Value::Array(variants) = member(body, "variants")? else {
                        return Err(format!("{at}: variants is not a list"));
                    };
                    Declared::Enum(
                        variants
                            .iter()
                            .map(|variant| match variant {
                                Value::Text(variant) => Ok(variant.clone()),
                                other => Err(format!("{at}: variant {}", show(other))),
                            })
                            .collect::<Result<_, _>>()?,
                    )
                }
                "struct" => {
                    keys(body, &["kind", "fields"], &at)?;
                    Declared::Struct(typed_fields(member(body, "fields")?, &at)?)
                }
                "union" => {
                    keys(body, &["kind", "tag", "variants"], &at)?;
                    let Value::Object(variants) = member(body, "variants")? else {
                        return Err(format!("{at}: variants is not an object"));
                    };
                    Declared::Union(
                        text_at(body, "tag")?.to_owned(),
                        variants
                            .iter()
                            .map(|(label, carried)| match carried {
                                Value::Null => Ok((label.clone(), None)),
                                Value::Text(carried) => Ok((label.clone(), Some(carried.clone()))),
                                other => {
                                    Err(format!("{at}: variant {label} carries {}", show(other)))
                                }
                            })
                            .collect::<Result<_, _>>()?,
                    )
                }
                other => return Err(format!("{at}: unsupported kind {other}")),
            };
            declarations.push((name.clone(), declared));
        }
        let shape = Self {
            identity_field,
            fields,
            declarations,
        };
        let mut used = Vec::new();
        for (_, ty) in &shape.fields {
            shape.reach(ty, &mut used, 0)?;
        }
        if used.len() != shape.declarations.len() {
            return Err("the subject shape carries a declaration no field reaches".to_owned());
        }
        let Some((_, identity)) = shape
            .fields
            .iter()
            .find(|(name, _)| *name == shape.identity_field)
        else {
            return Err(format!(
                "the subject identity {} is not a declared field",
                shape.identity_field
            ));
        };
        let mut ty = identity.as_str();
        loop {
            if ty.starts_with("Optional<") {
                return Err("the subject identity is optional".to_owned());
            }
            match shape.declared(ty) {
                Some(Declared::Newtype(of)) => ty = of,
                _ => break,
            }
        }
        Ok(shape)
    }

    fn declared(&self, name: &str) -> Option<&Declared> {
        self.declarations
            .iter()
            .find(|(declared, _)| declared == name)
            .map(|(_, declared)| declared)
    }

    /// Every declaration `ty` reaches, into `used`; an unresolvable type fails.
    fn reach(&self, ty: &str, used: &mut Vec<String>, depth: usize) -> Result<(), String> {
        if depth > 128 {
            return Err("the subject type graph is too deep".to_owned());
        }
        if let Some(inner) = wrapped(ty, "Optional<").or_else(|| wrapped(ty, "List<")) {
            return self.reach(inner, used, depth + 1);
        }
        if let Some(inner) = wrapped(ty, "Map<String, ") {
            return self.reach(inner, used, depth + 1);
        }
        if PRIMITIVES.contains(&ty) {
            return Ok(());
        }
        let Some(declared) = self.declared(ty) else {
            return Err(format!("the subject shape cannot resolve the type {ty}"));
        };
        if used.iter().any(|name| name == ty) {
            return Ok(());
        }
        used.push(ty.to_owned());
        match declared {
            Declared::Newtype(of) => self.reach(of, used, depth + 1),
            Declared::Enum(_) => Ok(()),
            Declared::Struct(fields) => fields
                .iter()
                .try_for_each(|(_, field)| self.reach(field, used, depth + 1)),
            Declared::Union(_, variants) => variants
                .iter()
                .filter_map(|(_, carried)| carried.as_deref())
                .try_for_each(|carried| self.reach(carried, used, depth + 1)),
        }
    }

    /// Requires every declared field of `row`, each of its declared type, and no field the shape
    /// does not declare.
    fn admit(&self, row: &Value) -> Result<(), String> {
        let Value::Object(members) = row else {
            return Err("the row is not an object".to_owned());
        };
        if let Some((field, _)) = members
            .iter()
            .find(|(field, _)| !self.fields.iter().any(|(declared, _)| declared == field))
        {
            return Err(format!(
                "it serves {field}, which the shape does not declare"
            ));
        }
        for (field, ty) in &self.fields {
            self.value(ty, row.member(field), 0)
                .map_err(|reason| format!("{field} ({ty}): {reason}"))?;
        }
        Ok(())
    }

    /// Requires `value` (`None` where absent) to be one of `ty`.
    fn value(&self, ty: &str, value: Option<&Value>, depth: usize) -> Result<(), String> {
        if depth > 128 {
            return Err("the value is too deep".to_owned());
        }
        if let Some(inner) = wrapped(ty, "Optional<") {
            return match value {
                None | Some(Value::Null) => Ok(()),
                held => self.value(inner, held, depth + 1),
            };
        }
        let value = match value {
            None => return Err("absent".to_owned()),
            Some(Value::Null) => return Err("null".to_owned()),
            Some(value) => value,
        };
        let refuse = || Err(format!("{} is not a {ty}", show(value)));
        if let Some(declared) = self.declared(ty) {
            return match (declared, value) {
                (Declared::Newtype(of), _) => self.value(of, Some(value), depth + 1),
                (Declared::Enum(variants), Value::Text(text)) if variants.contains(text) => Ok(()),
                (Declared::Struct(fields), Value::Object(members)) => {
                    if let Some((name, _)) = members
                        .iter()
                        .find(|(name, _)| !fields.iter().any(|(declared, _)| declared == name))
                    {
                        return Err(format!("{ty} carries {name}, which it does not declare"));
                    }
                    fields.iter().try_for_each(|(name, field)| {
                        self.value(field, value.member(name), depth + 1)
                            .map_err(|reason| format!("{ty}.{name}: {reason}"))
                    })
                }
                (Declared::Union(tag, variants), Value::Object(members)) => {
                    let Some(Value::Text(label)) = value.member(tag) else {
                        return refuse();
                    };
                    let Some((_, carried)) =
                        variants.iter().find(|(declared, _)| declared == label)
                    else {
                        return refuse();
                    };
                    let key = if tag == "value" { "content" } else { "value" };
                    if members.iter().any(|(name, _)| name != tag && name != key) {
                        return refuse();
                    }
                    match carried {
                        None if value.member(key).is_none() => Ok(()),
                        None => refuse(),
                        Some(carried) => self.value(carried, value.member(key), depth + 1),
                    }
                }
                _ => refuse(),
            };
        }
        if let Some(inner) = wrapped(ty, "List<") {
            let Value::Array(items) = value else {
                return refuse();
            };
            return items
                .iter()
                .try_for_each(|item| self.value(inner, Some(item), depth + 1));
        }
        if let Some(inner) = wrapped(ty, "Map<String, ") {
            let Value::Object(members) = value else {
                return refuse();
            };
            return members
                .iter()
                .try_for_each(|(_, item)| self.value(inner, Some(item), depth + 1));
        }
        let admitted = match ty {
            "String" | "Timestamp" | "Duration" => matches!(value, Value::Text(_)),
            "Boolean" => matches!(value, Value::Bool(_)),
            // An `Integer` crosses this wire as a number spelled without fraction or exponent.
            "Integer" => {
                matches!(value, Value::Number(spelling) if spelling.parse::<i64>().is_ok())
            }
            // A `Decimal` crosses this wire only as a decimal string, never a JSON number
            // (design § 11 D11).
            "Decimal" => matches!(value, Value::Text(text) if decimal(text)),
            "Binary64" => matches!(value, Value::Number(_)),
            "Uuid" => matches!(value, Value::Text(text) if uuid(text)),
            "Bytes" => matches!(value, Value::Text(text) if base64(text)),
            // `Json` holds any JSON value.
            "Json" => true,
            _ => return Err(format!("unsupported type {ty}")),
        };
        if admitted { Ok(()) } else { refuse() }
    }
}

/// The type inside `prefix…>`, where `ty` is one.
fn wrapped<'t>(ty: &'t str, prefix: &str) -> Option<&'t str> {
    ty.strip_prefix(prefix)?.strip_suffix('>')
}

/// `true` for a UUID in canonical hyphenated form, in either case.
fn uuid(text: &str) -> bool {
    text.len() == 36
        && text.bytes().enumerate().all(|(index, byte)| match index {
            8 | 13 | 18 | 23 => byte == b'-',
            _ => byte.is_ascii_hexdigit(),
        })
}

/// `true` for padded standard base64.
fn base64(text: &str) -> bool {
    let body = text.trim_end_matches('=');
    text.len().is_multiple_of(4)
        && text.len() - body.len() <= 2
        && body
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'+' || byte == b'/')
}

/// The caller a scenario names by the actor's qualified name.
fn caller(name: &str) -> Result<Caller, String> {
    Actor::ALL
        .iter()
        .find(|actor| actor.name() == name)
        .map(|actor| Caller { actor: *actor })
        .ok_or_else(|| format!("unknown actor {name}"))
}

/// A declared outcome, as `handle` renders it.
fn answered(body: &Value) -> Result<Answered, String> {
    keys(
        body,
        &["outcome", "published", "error", "payload"],
        "an outcome",
    )?;
    let outcome = text_at(body, "outcome")?.to_owned();
    let Value::Array(published) = member(body, "published")? else {
        return Err(format!("the outcome published {}", show(body)));
    };
    let events = published
        .iter()
        .map(|event| {
            keys(event, &["event", "payload"], "a published event")?;
            Ok((
                text_at(event, "event")?.to_owned(),
                member(event, "payload")?.clone(),
            ))
        })
        .collect::<Result<Vec<_>, String>>()?;
    let error = match body.member("error") {
        None => None,
        Some(Value::Text(error)) => Some(error.clone()),
        Some(other) => return Err(format!("error {}", show(other))),
    };
    Ok(Answered::Outcome {
        outcome,
        error,
        payload: body.member("payload").cloned().unwrap_or(Value::Null),
        events,
    })
}

/// Holds a published payload to the shape the suite states for its event: a field is present
/// unless the shape marks it `optional`, a present value is never null and is of the stated kind
/// or one of the stated variants, and the payload carries no field the shape does not state.
fn check_shape(event: &str, payload: &Value, shape: &Value) -> Result<(), String> {
    let (Value::Object(fields), Value::Object(descriptors)) = (payload, shape) else {
        return Err(format!(
            "{event}: payload {} or shape {} is not an object",
            show(payload),
            show(shape)
        ));
    };
    if let Some((field, _)) = fields
        .iter()
        .find(|(field, _)| !descriptors.iter().any(|(declared, _)| declared == field))
    {
        return Err(format!(
            "{event} carries {field}, which its shape does not state"
        ));
    }
    for (field, descriptor) in descriptors {
        let at = format!("{event}.{field}");
        keys(descriptor, &["holds", "kind", "optional", "variants"], &at)?;
        let optional = match descriptor.member("optional") {
            None => false,
            Some(Value::Bool(optional)) => *optional,
            Some(other) => return Err(format!("{at}: unsupported optional {}", show(other))),
        };
        let admits: Box<dyn Fn(&Value) -> bool> = match text_at(descriptor, "holds")? {
            "primitive" => {
                if descriptor.member("variants").is_some() {
                    return Err(format!("{at}: a primitive shape with variants"));
                }
                match text_at(descriptor, "kind")? {
                    "string" => Box::new(|value| matches!(value, Value::Text(_))),
                    "integer" => Box::new(
                        |value| matches!(value, Value::Number(spelling) if spelling.parse::<i64>().is_ok()),
                    ),
                    "decimal" => {
                        Box::new(|value| matches!(value, Value::Text(text) if decimal(text)))
                    }
                    "timestamp" => Box::new(
                        |value| matches!(value, Value::Text(text) if instant(text).is_some()),
                    ),
                    other => return Err(format!("{at}: unsupported kind {other}")),
                }
            }
            "enum" => {
                if descriptor.member("kind").is_some() {
                    return Err(format!("{at}: an enum shape with a kind"));
                }
                let Value::Array(variants) = member(descriptor, "variants")? else {
                    return Err(format!("{at}: variants is not a list"));
                };
                let variants = variants.clone();
                Box::new(move |value| matches!(value, Value::Text(_)) && variants.contains(value))
            }
            other => return Err(format!("{at}: unsupported holds {other}")),
        };
        match fields.iter().find(|(name, _)| name == field) {
            None if optional => {}
            None => return Err(format!("{event} lacks {field}")),
            Some((_, value)) if admits(value) => {}
            Some((_, value)) => {
                return Err(format!(
                    "{at}: {} is not of the stated shape {}",
                    show(value),
                    show(descriptor)
                ));
            }
        }
    }
    Ok(())
}

/// The truth of an ESS predicate over one row: `true`, `false`, or `None` where it reads
/// something the row does not hold.
fn truth(predicate: &Value, row: &Value) -> Result<Option<bool>, String> {
    match predicate {
        Value::Text(atom) => atom_truth(atom, row),
        Value::Object(members) => match members.as_slice() {
            [(combinator, Value::Array(parts))] if combinator == "all" || combinator == "any" => {
                let truths = parts
                    .iter()
                    .map(|part| truth(part, row))
                    .collect::<Result<Vec<_>, _>>()?;
                let (decides, otherwise) = if combinator == "all" {
                    (false, true)
                } else {
                    (true, false)
                };
                Ok(if truths.contains(&Some(decides)) {
                    Some(decides)
                } else if truths.contains(&None) {
                    None
                } else {
                    Some(otherwise)
                })
            }
            [(not, part)] if not == "not" => Ok(truth(part, row)?.map(|truth| !truth)),
            _ => Err(format!("unsupported predicate form {}", show(predicate))),
        },
        other => Err(format!("unsupported predicate form {}", show(other))),
    }
}

fn atom_truth(atom: &str, row: &Value) -> Result<Option<bool>, String> {
    let unsupported = || format!("unsupported predicate {atom:?}");
    let path = |text: &str| -> Result<Option<Value>, String> {
        if text.is_empty()
            || !text
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'.')
        {
            return Err(unsupported());
        }
        let mut held = row;
        for step in text.split('.') {
            match held.member(step) {
                Some(Value::Null) | None => return Ok(None),
                Some(value) => held = value,
            }
        }
        Ok(Some(held.clone()))
    };
    if let Some(inner) = atom
        .strip_prefix("defined(")
        .and_then(|rest| rest.strip_suffix(')'))
    {
        return Ok(Some(path(inner)?.is_some()));
    }
    let [field, op, literal] = atom.split_whitespace().collect::<Vec<_>>()[..] else {
        return Err(unsupported());
    };
    if !matches!(op, "==" | "!=" | ">=" | ">" | "<=" | "<") {
        return Err(unsupported());
    }
    let Some(held) = path(field)? else {
        return Ok(None);
    };
    if let Ok(bound) = literal.parse::<f64>() {
        let value = match &held {
            Value::Number(spelling) => spelling.parse::<f64>().ok(),
            Value::Text(text) if decimal(text) => text.parse::<f64>().ok(),
            _ => None,
        };
        let Some(value) = value else {
            return Ok(None);
        };
        return Ok(Some(match op {
            "==" => value == bound,
            "!=" => value != bound,
            ">=" => value >= bound,
            ">" => value > bound,
            "<=" => value <= bound,
            _ => value < bound,
        }));
    }
    if !literal
        .bytes()
        .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
    {
        return Err(unsupported());
    }
    let Value::Text(text) = &held else {
        return Ok(None);
    };
    match op {
        "==" => Ok(Some(text == literal)),
        "!=" => Ok(Some(text != literal)),
        _ => Err(unsupported()),
    }
}

/// The schemas of the published wire contract, by component name.
struct Schemas {
    components: Vec<(String, Value)>,
    /// The model's declarations (`x-ess-entities`), entities among them.
    declarations: Vec<(String, Value)>,
}

impl Schemas {
    fn load() -> Result<Self, String> {
        let document = json::parse(OPENAPI)
            .map_err(|error| format!("the OpenAPI document is not JSON: {error}"))?;
        let Some(Value::Object(components)) = document
            .member("components")
            .and_then(|components| components.member("schemas"))
        else {
            return Err("the OpenAPI document holds no component schemas".to_owned());
        };
        let Some(Value::Object(declarations)) = document.member("x-ess-entities") else {
            return Err("the OpenAPI document holds no model declarations".to_owned());
        };
        Ok(Self {
            components: components.clone(),
            declarations: declarations.clone(),
        })
    }

    /// Nothing, where `entity` is an entity the model declares and `field` one of its fields.
    fn entity_field(&self, entity: &str, field: &str) -> Result<(), String> {
        let declared = self
            .declarations
            .iter()
            .find(|(name, _)| name == entity)
            .map(|(_, declaration)| declaration)
            .filter(|declaration| {
                declaration.member("x-ess-kind") == Some(&Value::Text("entity".to_owned()))
            })
            .ok_or_else(|| format!("the model declares no entity {entity}"))?;
        declared
            .member("properties")
            .and_then(|properties| properties.member(field))
            .map(|_| ())
            .ok_or_else(|| format!("the entity {entity} declares no field {field}"))
    }

    fn named(&self, name: &str) -> Result<&Value, String> {
        self.components
            .iter()
            .find(|(component, _)| component == name)
            .map(|(_, schema)| schema)
            .ok_or_else(|| format!("the wire contract declares no {name}"))
    }

    /// `schema`, with a `$ref` followed.
    fn resolved<'s>(&'s self, schema: &'s Value) -> Result<&'s Value, String> {
        match schema.member("$ref") {
            None => Ok(schema),
            Some(Value::Text(reference)) => {
                let name = reference
                    .strip_prefix("#/components/schemas/")
                    .ok_or_else(|| format!("unsupported reference {reference}"))?;
                self.resolved(self.named(name)?)
            }
            Some(other) => Err(format!("unsupported reference {}", show(other))),
        }
    }

    /// The schema the component `component` declares for its field `field`.
    fn property(&self, component: &str, field: &str) -> Result<&Value, String> {
        self.resolved(self.named(component)?)?
            .member("properties")
            .and_then(|properties| properties.member(field))
            .ok_or_else(|| format!("{component} declares no field {field}"))
    }

    /// `value`, rendered in the wire form `schema` declares: an integral number where an
    /// `Integer` is declared, a decimal string where a `Decimal` is. What it cannot render it
    /// leaves as it is, for the generated decoder to refuse with its own path.
    fn render(&self, schema: &Value, value: Value) -> Result<Value, String> {
        let schema = self.resolved(schema)?;
        if let Some(key) = ["anyOf", "allOf", "not"]
            .into_iter()
            .find(|key| schema.member(key).is_some())
        {
            return Err(format!("unsupported schema construct {key}"));
        }
        if let Some(Value::Array(variants)) = schema.member("oneOf") {
            let Some(Value::Text(tag)) = schema.member("x-ess-union-tag") else {
                return Err(format!("unsupported untagged union {}", show(schema)));
            };
            let wanted = value.member(tag).cloned();
            for variant in variants {
                let variant = self.resolved(variant)?;
                let constant = variant
                    .member("properties")
                    .and_then(|properties| properties.member(tag))
                    .and_then(|property| property.member("const"))
                    .cloned();
                if constant.is_some() && constant == wanted {
                    return self.render(variant, value);
                }
            }
            return Ok(value);
        }
        let declared = match schema.member("type") {
            Some(Value::Text(declared)) => declared.as_str(),
            _ => return Ok(value),
        };
        Ok(match (declared, value) {
            ("integer", Value::Number(spelling)) => Value::Number(integral(spelling)),
            ("string", Value::Number(spelling))
                if schema.member("format") == Some(&Value::Text("decimal".to_owned())) =>
            {
                Value::Text(spelling)
            }
            ("object", Value::Object(members)) => Value::Object(
                members
                    .into_iter()
                    .map(|(name, member)| {
                        let declared = schema
                            .member("properties")
                            .and_then(|properties| properties.member(&name))
                            .or_else(|| match schema.member("additionalProperties") {
                                Some(Value::Object(additional)) if !additional.is_empty() => {
                                    schema.member("additionalProperties")
                                }
                                _ => None,
                            });
                        match declared {
                            Some(declared) => Ok((name, self.render(declared, member)?)),
                            None => Ok((name, member)),
                        }
                    })
                    .collect::<Result<_, String>>()?,
            ),
            ("array", Value::Array(items)) => {
                let prefix = match schema.member("prefixItems") {
                    Some(Value::Array(prefix)) => prefix.as_slice(),
                    _ => &[],
                };
                let rest = schema.member("items");
                Value::Array(
                    items
                        .into_iter()
                        .enumerate()
                        .map(|(index, item)| match prefix.get(index).or(rest) {
                            Some(declared) => self.render(declared, item),
                            None => Ok(item),
                        })
                        .collect::<Result<_, String>>()?,
                )
            }
            (_, value) => value,
        })
    }
}

/// An integral number's `Integer` spelling (`1.0` is `1`); any other spelling unchanged.
fn integral(spelling: String) -> String {
    if spelling.parse::<i64>().is_ok() {
        return spelling;
    }
    match spelling.parse::<f64>() {
        // The range check keeps the cast exact: every integral `f64` inside it is an `i64`.
        Ok(number) if number.fract() == 0.0 && number.abs() < 9.0e15 => {
            format!("{}", number as i64)
        }
        _ => spelling,
    }
}

/// Equality of two wire values: numbers by value, objects member by member regardless of order
/// with an absent member the same as null, lists in order.
fn same(left: &Value, right: &Value) -> bool {
    match (left, right) {
        (Value::Number(left), Value::Number(right)) => {
            left == right
                || matches!((left.parse::<f64>(), right.parse::<f64>()), (Ok(left), Ok(right)) if left == right)
        }
        (Value::Object(left_members), Value::Object(right_members)) => {
            let names = left_members
                .iter()
                .chain(right_members)
                .map(|(name, _)| name);
            names.into_iter().all(|name| {
                same(
                    left.member(name).unwrap_or(&Value::Null),
                    right.member(name).unwrap_or(&Value::Null),
                )
            })
        }
        (Value::Array(left), Value::Array(right)) => {
            left.len() == right.len()
                && left
                    .iter()
                    .zip(right)
                    .all(|(left, right)| same(left, right))
        }
        (left, right) => left == right,
    }
}

/// Fails on any member of `value` not in `allowed`, so a construct this harness does not know is
/// refused rather than ignored.
fn keys(value: &Value, allowed: &[&str], what: &str) -> Result<(), String> {
    let Value::Object(members) = value else {
        return Err(format!("{what} is not an object: {}", show(value)));
    };
    match members
        .iter()
        .find(|(name, _)| !allowed.contains(&name.as_str()))
    {
        Some((name, _)) => Err(format!("unsupported member {name} of {what}")),
        None => Ok(()),
    }
}

fn member<'v>(value: &'v Value, name: &str) -> Result<&'v Value, String> {
    value
        .member(name)
        .ok_or_else(|| format!("{} has no {name}", show(value)))
}

fn text_at<'v>(value: &'v Value, name: &str) -> Result<&'v str, String> {
    match member(value, name)? {
        Value::Text(text) => Ok(text),
        other => Err(format!("{name} is {}, not a string", show(other))),
    }
}

fn names(events: &[(String, Value)]) -> String {
    events
        .iter()
        .map(|(name, _)| name.as_str())
        .collect::<Vec<_>>()
        .join(", ")
}

fn show(value: &Value) -> String {
    let mut out = String::new();
    json::push_value(&mut out, value);
    out
}

/// `true` for a decimal in the published pattern: an optional `-`, digits without a leading
/// zero, then an optional `.` and digits.
fn decimal(text: &str) -> bool {
    let digits = |part: &str| !part.is_empty() && part.bytes().all(|byte| byte.is_ascii_digit());
    let unsigned = text.strip_prefix('-').unwrap_or(text);
    let (whole, fraction) = match unsigned.split_once('.') {
        Some((whole, fraction)) => (whole, Some(fraction)),
        None => (unsigned, None),
    };
    digits(whole) && !(whole.len() > 1 && whole.starts_with('0')) && fraction.is_none_or(digits)
}

/// Days from 1970-01-01 to the civil date.
fn days_from_civil(year: i64, month: i64, day: i64) -> i64 {
    let year = if month <= 2 { year - 1 } else { year };
    let era = year.div_euclid(400);
    let of_era = year - era * 400;
    let of_year = (153 * (if month > 2 { month - 3 } else { month + 9 }) + 2) / 5 + day - 1;
    let of_cycle = of_era * 365 + of_era / 4 - of_era / 100 + of_year;
    era * 146_097 + of_cycle - 719_468
}

/// The RFC 3339 rendering, in UTC, of `seconds` from the Unix epoch.
fn timestamp(seconds: i64) -> String {
    let days = seconds.div_euclid(86_400);
    let of_day = seconds.rem_euclid(86_400);
    let shifted = days + 719_468;
    let era = shifted.div_euclid(146_097);
    let of_era = shifted - era * 146_097;
    let year_of_era = (of_era - of_era / 1460 + of_era / 36_524 - of_era / 146_096) / 365;
    let of_year = of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let march = (5 * of_year + 2) / 153;
    let day = of_year - (153 * march + 2) / 5 + 1;
    let month = if march < 10 { march + 3 } else { march - 9 };
    let year = year_of_era + era * 400 + i64::from(month <= 2);
    format!(
        "{year:04}-{month:02}-{day:02}T{:02}:{:02}:{:02}Z",
        of_day / 3600,
        of_day % 3600 / 60,
        of_day % 60
    )
}

/// The instant an RFC 3339 `date-time` names, in seconds from the Unix epoch, or `None` for any
/// other text.
fn instant(text: &str) -> Option<i64> {
    let bytes = text.as_bytes();
    let number = |from: usize, to: usize| -> Option<i64> {
        let slice = bytes.get(from..to)?;
        if !slice.iter().all(u8::is_ascii_digit) {
            return None;
        }
        std::str::from_utf8(slice).ok()?.parse().ok()
    };
    let at = |index: usize, expected: &[u8]| {
        bytes.get(index).is_some_and(|byte| expected.contains(byte))
    };
    if !(at(4, b"-") && at(7, b"-") && at(10, b"Tt") && at(13, b":") && at(16, b":")) {
        return None;
    }
    let (year, month, day) = (number(0, 4)?, number(5, 7)?, number(8, 10)?);
    let (hour, minute, second) = (number(11, 13)?, number(14, 16)?, number(17, 19)?);
    let leap = (year % 4 == 0 && year % 100 != 0) || year % 400 == 0;
    let length = match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if leap => 29,
        2 => 28,
        _ => return None,
    };
    if day < 1 || day > length || hour > 23 || minute > 59 || second > 59 {
        return None;
    }
    let mut position = 19;
    if at(position, b".") {
        let start = position + 1;
        position = start;
        while bytes.get(position).is_some_and(u8::is_ascii_digit) {
            position += 1;
        }
        if position == start {
            return None;
        }
    }
    let offset = match bytes.get(position..)? {
        b"Z" | b"z" => 0,
        [sign @ (b'+' | b'-'), _, _, b':', _, _] => {
            let (hours, minutes) = (
                number(position + 1, position + 3)?,
                number(position + 4, position + 6)?,
            );
            if hours > 23 || minutes > 59 {
                return None;
            }
            let magnitude = hours * 3600 + minutes * 60;
            if *sign == b'-' { -magnitude } else { magnitude }
        }
        _ => return None,
    };
    Some(days_from_civil(year, month, day) * 86_400 + hour * 3600 + minute * 60 + second - offset)
}

#[cfg(test)]
mod tests {
    use super::{CLOCK, instant, timestamp};

    #[test]
    fn the_clock_renders_and_reads_back_as_the_same_instant() {
        assert_eq!(timestamp(CLOCK), "2026-01-01T00:00:00Z");
        assert_eq!(timestamp(CLOCK - 1), "2025-12-31T23:59:59Z");
        assert_eq!(timestamp(CLOCK + 86_401), "2026-01-02T00:00:01Z");
        assert_eq!(timestamp(951_782_400), "2000-02-29T00:00:00Z");
        for seconds in [0, CLOCK, CLOCK - 1, CLOCK + 259_201, 951_782_400] {
            assert_eq!(instant(&timestamp(seconds)), Some(seconds));
        }
        assert_eq!(instant("2026-01-01T01:00:00+01:00"), Some(CLOCK));
        assert_eq!(instant("2026-02-30T00:00:00Z"), None);
        assert_eq!(instant("yesterday"), None);
    }
}
