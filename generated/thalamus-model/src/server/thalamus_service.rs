// generated from thalamus v1
// model digest 73865b5d22d3d964014ec5c52cb6db35af4760818bbeefd77df0a2551b8fd3ed
// contract digest 093adf023da1db0be3662fe2c9b1542d25415a595e3023e37377255b2d0a3061
// do not edit: regenerate with `ess synthesize --layout crate`

//! The `thalamus-service` component of `thalamus` v1, on the wire.
//!
//! The specification says this component's callers are not deployed with it, so its surface
//! exists on a wire. Which wire is derived rather than chosen: the one contract this model
//! projects for a command surface is the `OpenAPI` document, and an `OpenAPI` document is an
//! HTTP contract. The document is beside this file, served verbatim at `/openapi.json`.

use crate::server::{entry, http, json, wire};

/// The contract this surface answers, byte for byte as `generated/` commits it.
///
/// Embedded rather than rebuilt at run time: a server that regenerated its own contract could
/// publish one the repository never reviewed.
pub const OPENAPI: &str = include_str!("thalamus-service.openapi.json");

/// The prose the same model produced, byte for byte as the documentation projection wrote it.
pub const DOCS: &str = include_str!("thalamus-service.docs.md");

/// Every route this surface answers, in path order.
///
/// The same set the `OpenAPI` document declares, plus the two documents about the surface
/// itself, which no specification construct names and nothing can therefore derive. A path
/// absent from this table is answered with `404`, including one the document declares and this
/// table forgot — which is the failure a table computed twice would hide.
pub const ROUTES: &[(&str, &str)] = &[
    ("POST", "/assertions/commands/answer"),
    ("POST", "/assertions/commands/ask"),
    ("GET", "/assertions/views/assertions"),
    ("GET", "/assertions/views/open"),
    ("POST", "/definitions/commands/activate"),
    ("POST", "/definitions/commands/mute"),
    ("POST", "/definitions/commands/propose"),
    ("POST", "/definitions/commands/retire"),
    ("POST", "/definitions/commands/revise"),
    ("POST", "/definitions/commands/unmute"),
    ("GET", "/definitions/views/active"),
    ("GET", "/definitions/views/definitions"),
    ("POST", "/dimensions/commands/declare-registry"),
    ("POST", "/dimensions/commands/register-dimension"),
    ("GET", "/dimensions/views/dimensions"),
    ("GET", "/dimensions/views/registries"),
    ("GET", "/docs"),
    ("POST", "/evaluations/commands/record"),
    ("GET", "/evaluations/views/evaluations"),
    ("GET", "/evaluations/views/transitions"),
    ("GET", "/evaluations/views/verdicts"),
    ("GET", "/openapi.json"),
    ("POST", "/rollups/commands/record"),
    ("GET", "/rollups/views/rollups"),
    ("GET", "/rollups/views/series"),
    ("POST", "/samples/commands/record"),
    ("GET", "/samples/views/samples"),
    ("GET", "/samples/views/series"),
    ("POST", "/transitions/commands/record"),
    ("GET", "/transitions/views/series"),
    ("GET", "/transitions/views/transitions"),
];

/// What this process says about itself as it starts, before it answers anything.
///
/// Three lines of JSON on standard output, in this order, every member of them derived from the
/// specification — except `runtime`, which is appended by the emitted code below and holds what
/// is true of *this process*: the language it was synthesised into, and the address it bound.
/// Everything outside `runtime` is the same in every language this plan is emitted into, and
/// `cargo xtask synth --check` starts both and compares them.
pub const STARTUP: &[&str] = &[
    "{\"log\":\"ess/1\",\"event\":\"system.starting\",\"system\":\"thalamus\",\"version\":\"v1\",\"model_digest\":\"73865b5d22d3d964014ec5c52cb6db35af4760818bbeefd77df0a2551b8fd3ed\",\"contract_digest\":\"093adf023da1db0be3662fe2c9b1542d25415a595e3023e37377255b2d0a3061\",\"components\":[\"thalamus-service\"],\"capabilities\":{\"generated\":169,\"obligations\":11,\"refused\":0}",
    "{\"log\":\"ess/1\",\"event\":\"surface.serving\",\"component\":\"thalamus-service\",\"reached_by\":\"network\",\"transport\":\"http/1.1\",\"routes\":31,\"paths\":[{\"method\":\"POST\",\"path\":\"/assertions/commands/answer\",\"serves\":\"command\",\"name\":\"thalamus.assertion.AnswerAssertion\"},{\"method\":\"POST\",\"path\":\"/assertions/commands/ask\",\"serves\":\"command\",\"name\":\"thalamus.assertion.AskAssertion\"},{\"method\":\"GET\",\"path\":\"/assertions/views/assertions\",\"serves\":\"view\",\"name\":\"thalamus.assertion.Assertions\"},{\"method\":\"GET\",\"path\":\"/assertions/views/open\",\"serves\":\"view\",\"name\":\"thalamus.assertion.OpenAssertions\"},{\"method\":\"POST\",\"path\":\"/definitions/commands/activate\",\"serves\":\"command\",\"name\":\"thalamus.definition.ActivateDefinition\"},{\"method\":\"POST\",\"path\":\"/definitions/commands/mute\",\"serves\":\"command\",\"name\":\"thalamus.definition.MuteDefinition\"},{\"method\":\"POST\",\"path\":\"/definitions/commands/propose\",\"serves\":\"command\",\"name\":\"thalamus.definition.ProposeDefinition\"},{\"method\":\"POST\",\"path\":\"/definitions/commands/retire\",\"serves\":\"command\",\"name\":\"thalamus.definition.RetireDefinition\"},{\"method\":\"POST\",\"path\":\"/definitions/commands/revise\",\"serves\":\"command\",\"name\":\"thalamus.definition.ReviseDefinition\"},{\"method\":\"POST\",\"path\":\"/definitions/commands/unmute\",\"serves\":\"command\",\"name\":\"thalamus.definition.UnmuteDefinition\"},{\"method\":\"GET\",\"path\":\"/definitions/views/active\",\"serves\":\"view\",\"name\":\"thalamus.definition.ActiveDefinitions\"},{\"method\":\"GET\",\"path\":\"/definitions/views/definitions\",\"serves\":\"view\",\"name\":\"thalamus.definition.Definitions\"},{\"method\":\"POST\",\"path\":\"/dimensions/commands/declare-registry\",\"serves\":\"command\",\"name\":\"thalamus.dimension.DeclareRegistry\"},{\"method\":\"POST\",\"path\":\"/dimensions/commands/register-dimension\",\"serves\":\"command\",\"name\":\"thalamus.dimension.RegisterDimension\"},{\"method\":\"GET\",\"path\":\"/dimensions/views/dimensions\",\"serves\":\"view\",\"name\":\"thalamus.dimension.Dimensions\"},{\"method\":\"GET\",\"path\":\"/dimensions/views/registries\",\"serves\":\"view\",\"name\":\"thalamus.dimension.Registries\"},{\"method\":\"GET\",\"path\":\"/docs\",\"serves\":\"documentation\",\"name\":\"docs\"},{\"method\":\"POST\",\"path\":\"/evaluations/commands/record\",\"serves\":\"command\",\"name\":\"thalamus.evaluation.RecordEvaluation\"},{\"method\":\"GET\",\"path\":\"/evaluations/views/evaluations\",\"serves\":\"view\",\"name\":\"thalamus.evaluation.Evaluations\"},{\"method\":\"GET\",\"path\":\"/evaluations/views/transitions\",\"serves\":\"view\",\"name\":\"thalamus.evaluation.VerdictChanges\"},{\"method\":\"GET\",\"path\":\"/evaluations/views/verdicts\",\"serves\":\"view\",\"name\":\"thalamus.evaluation.SeriesVerdicts\"},{\"method\":\"GET\",\"path\":\"/openapi.json\",\"serves\":\"contract\",\"name\":\"openapi\"},{\"method\":\"POST\",\"path\":\"/rollups/commands/record\",\"serves\":\"command\",\"name\":\"thalamus.rollup.RecordRollup\"},{\"method\":\"GET\",\"path\":\"/rollups/views/rollups\",\"serves\":\"view\",\"name\":\"thalamus.rollup.Rollups\"},{\"method\":\"GET\",\"path\":\"/rollups/views/series\",\"serves\":\"view\",\"name\":\"thalamus.rollup.SeriesRollups\"},{\"method\":\"POST\",\"path\":\"/samples/commands/record\",\"serves\":\"command\",\"name\":\"thalamus.observation.RecordSample\"},{\"method\":\"GET\",\"path\":\"/samples/views/samples\",\"serves\":\"view\",\"name\":\"thalamus.observation.Samples\"},{\"method\":\"GET\",\"path\":\"/samples/views/series\",\"serves\":\"view\",\"name\":\"thalamus.observation.SeriesSamples\"},{\"method\":\"POST\",\"path\":\"/transitions/commands/record\",\"serves\":\"command\",\"name\":\"thalamus.transition.RecordTransition\"},{\"method\":\"GET\",\"path\":\"/transitions/views/series\",\"serves\":\"view\",\"name\":\"thalamus.transition.SeriesTransitions\"},{\"method\":\"GET\",\"path\":\"/transitions/views/transitions\",\"serves\":\"view\",\"name\":\"thalamus.transition.Transitions\"}]",
    "{\"log\":\"ess/1\",\"event\":\"system.ready\",\"system\":\"thalamus\",\"surfaces\":1",
];

/// Writes the startup record, with this process's own facts closing each line.
fn announce(address: &std::net::SocketAddr) {
    for facts in STARTUP {
        let mut line = String::from(*facts);
        line.push_str(",\"runtime\":{\"address\":");
        json::push_text(&mut line, &address.to_string());
        line.push_str(",\"language\":\"rust\",\"port\":");
        json::push_integer(&mut line, i64::from(address.port()));
        line.push_str("}}");
        println!("{line}");
    }
}

/// Serves `thalamus-service` at `address`, and does not return while it can answer.
///
/// `address` may name port `0`, which binds an ephemeral port; the startup record says which one
/// was taken, because a caller that cannot learn the port cannot make a request.
///
/// It chooses no realization. Every command reaches the port, and a port over unimplemented
/// obligations answers the typed refusal this surface reports as `501` — the honest empty
/// state rather than a server that pretends.
///
/// `authenticate` is the realization's: it says who each request was sent by, or `None`, and
/// [`dispatch`] checks that caller's grant before the command runs. Nothing here reads an
/// actor from the request itself.
///
/// One connection at a time: each is dropped after [`http::READ_TIMEOUT`] without a byte, or
/// [`http::WRITE_TIMEOUT`] of a stalled write, and whatever fails on one connection — a caller
/// that hung up before reading its answer, a reset, a failed accept — ends that connection only.
///
/// # Errors
///
/// What binding the address refuses: the address is taken, or the port is privileged.
pub fn serve<ThalamusServiceBehaviors>(system: &mut crate::system::System<ThalamusServiceBehaviors>, address: &str, authenticate: impl Fn(&http::Request) -> Option<crate::actor::Caller>) -> std::io::Result<()>
where
    ThalamusServiceBehaviors: crate::assertion::obligations::AnswerAssertionBehavior + crate::assertion::obligations::AskAssertionBehavior + crate::definition::obligations::ActivateDefinitionBehavior + crate::definition::obligations::MuteDefinitionBehavior + crate::definition::obligations::ProposeDefinitionBehavior + crate::definition::obligations::RetireDefinitionBehavior + crate::definition::obligations::ReviseDefinitionBehavior + crate::definition::obligations::UnmuteDefinitionBehavior + crate::dimension::obligations::DeclareRegistryBehavior + crate::dimension::obligations::RegisterDimensionBehavior + crate::evaluation::obligations::RecordEvaluationBehavior + crate::observation::obligations::RecordSampleBehavior + crate::rollup::obligations::RecordRollupBehavior + crate::transition::obligations::RecordTransitionBehavior + crate::assertion::obligations::AssertionsQuery + crate::assertion::obligations::OpenAssertionsQuery + crate::definition::obligations::ActiveDefinitionsQuery + crate::definition::obligations::DefinitionsQuery + crate::dimension::obligations::DimensionsQuery + crate::dimension::obligations::RegistriesQuery + crate::evaluation::obligations::EvaluationsQuery + crate::evaluation::obligations::SeriesVerdictsQuery + crate::evaluation::obligations::VerdictChangesQuery + crate::observation::obligations::SamplesQuery + crate::observation::obligations::SeriesSamplesQuery + crate::rollup::obligations::RollupsQuery + crate::rollup::obligations::SeriesRollupsQuery + crate::transition::obligations::SeriesTransitionsQuery + crate::transition::obligations::TransitionsQuery,
{ serve_with_static(system, address, authenticate, None) }

/// Serves the declared surface, with files only for paths outside its route table.
///
/// # Errors
/// Returns a listener or static-root error before announcing readiness.
pub fn serve_with_static<ThalamusServiceBehaviors>(system: &mut crate::system::System<ThalamusServiceBehaviors>, address: &str, authenticate: impl Fn(&http::Request) -> Option<crate::actor::Caller>, static_root: Option<&std::path::Path>) -> std::io::Result<()>
where
    ThalamusServiceBehaviors: crate::assertion::obligations::AnswerAssertionBehavior + crate::assertion::obligations::AskAssertionBehavior + crate::definition::obligations::ActivateDefinitionBehavior + crate::definition::obligations::MuteDefinitionBehavior + crate::definition::obligations::ProposeDefinitionBehavior + crate::definition::obligations::RetireDefinitionBehavior + crate::definition::obligations::ReviseDefinitionBehavior + crate::definition::obligations::UnmuteDefinitionBehavior + crate::dimension::obligations::DeclareRegistryBehavior + crate::dimension::obligations::RegisterDimensionBehavior + crate::evaluation::obligations::RecordEvaluationBehavior + crate::observation::obligations::RecordSampleBehavior + crate::rollup::obligations::RecordRollupBehavior + crate::transition::obligations::RecordTransitionBehavior + crate::assertion::obligations::AssertionsQuery + crate::assertion::obligations::OpenAssertionsQuery + crate::definition::obligations::ActiveDefinitionsQuery + crate::definition::obligations::DefinitionsQuery + crate::dimension::obligations::DimensionsQuery + crate::dimension::obligations::RegistriesQuery + crate::evaluation::obligations::EvaluationsQuery + crate::evaluation::obligations::SeriesVerdictsQuery + crate::evaluation::obligations::VerdictChangesQuery + crate::observation::obligations::SamplesQuery + crate::observation::obligations::SeriesSamplesQuery + crate::rollup::obligations::RollupsQuery + crate::rollup::obligations::SeriesRollupsQuery + crate::transition::obligations::SeriesTransitionsQuery + crate::transition::obligations::TransitionsQuery,
{
    let static_root = static_root.map(std::fs::canonicalize).transpose()?;
    if static_root.as_ref().is_some_and(|root| !root.is_dir()) { return Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, "static root is not a directory")); }
    let listener = std::net::TcpListener::bind(address)?;
    announce(&listener.local_addr()?);
    for connection in listener.incoming() {
        // What fails on one connection ends that connection, never this loop: a caller that
        // gave up before it was accepted, or before it read its answer, is that caller's affair.
        let Ok(connection) = connection else {
            // An accept the listener itself failed (no descriptor left) would fail again at
            // once: pause before the next.
            std::thread::sleep(std::time::Duration::from_millis(10));
            continue;
        };
        let _ = connection.set_read_timeout(Some(http::READ_TIMEOUT));
        let _ = connection.set_write_timeout(Some(http::WRITE_TIMEOUT));
        let mut reader = std::io::BufReader::new(connection);
        let (answer, refused) = match http::read(&mut reader) {
            Ok(request) => {
                if !ROUTES.iter().any(|(_, path)| *path == request.path) {
                    if let Some(root) = &static_root {
                        let _ = crate::server::static_assets::answer(reader.get_mut(), root, &request);
                        continue;
                    }
                }
                (dispatch(system, authenticate(&request).as_ref(), &request), false)
            },
            Err(refusal) => (refusal, true),
        };
        let mut stream = reader.into_inner();
        if http::write(&mut stream, &answer).is_ok() && refused {
            http::linger(&mut stream);
        }
    }
    Ok(())
}

/// Answers one request.
///
/// A path this table does not hold is a `404` naming where the whole table is published; a
/// path it holds under a different method is a `405` naming the one it answers. Neither is a
/// status the contract declares, and neither should be: both are facts about a transport rather
/// than about any command.
///
/// Public so a caller can hand it a request it built itself: [`serve`] is this function behind a
/// socket, and nothing else.
///
/// `caller` is who the realization authenticated the request as, or `None`. Every command
/// checks its grant before it runs, and answers the standard refusal when the caller is none
/// or is an actor the specification does not grant the command.
pub fn dispatch<ThalamusServiceBehaviors>(system: &mut crate::system::System<ThalamusServiceBehaviors>, caller: Option<&crate::actor::Caller>, request: &http::Request) -> http::Response
where
    ThalamusServiceBehaviors: crate::assertion::obligations::AnswerAssertionBehavior + crate::assertion::obligations::AskAssertionBehavior + crate::definition::obligations::ActivateDefinitionBehavior + crate::definition::obligations::MuteDefinitionBehavior + crate::definition::obligations::ProposeDefinitionBehavior + crate::definition::obligations::RetireDefinitionBehavior + crate::definition::obligations::ReviseDefinitionBehavior + crate::definition::obligations::UnmuteDefinitionBehavior + crate::dimension::obligations::DeclareRegistryBehavior + crate::dimension::obligations::RegisterDimensionBehavior + crate::evaluation::obligations::RecordEvaluationBehavior + crate::observation::obligations::RecordSampleBehavior + crate::rollup::obligations::RecordRollupBehavior + crate::transition::obligations::RecordTransitionBehavior + crate::assertion::obligations::AssertionsQuery + crate::assertion::obligations::OpenAssertionsQuery + crate::definition::obligations::ActiveDefinitionsQuery + crate::definition::obligations::DefinitionsQuery + crate::dimension::obligations::DimensionsQuery + crate::dimension::obligations::RegistriesQuery + crate::evaluation::obligations::EvaluationsQuery + crate::evaluation::obligations::SeriesVerdictsQuery + crate::evaluation::obligations::VerdictChangesQuery + crate::observation::obligations::SamplesQuery + crate::observation::obligations::SeriesSamplesQuery + crate::rollup::obligations::RollupsQuery + crate::rollup::obligations::SeriesRollupsQuery + crate::transition::obligations::SeriesTransitionsQuery + crate::transition::obligations::TransitionsQuery,
{
    match request.path.as_str() {
        "/assertions/commands/answer" => {
            if request.method != "POST" {
                return http::method_not_allowed("POST");
            }
            if let Err(actor) = admit(caller, "thalamus.assertion.AnswerAssertion") {
                return not_granted(actor);
            }
            serve_thalamus_assertion_answer_assertion(system, &request.body)
        }
        "/assertions/commands/ask" => {
            if request.method != "POST" {
                return http::method_not_allowed("POST");
            }
            if let Err(actor) = admit(caller, "thalamus.assertion.AskAssertion") {
                return not_granted(actor);
            }
            serve_thalamus_assertion_ask_assertion(system, &request.body)
        }
        "/assertions/views/assertions" => {
            if request.method != "GET" {
                return http::method_not_allowed("GET");
            }
            http::answer(run_thalamus_assertion_assertions(system))
        }
        "/assertions/views/open" => {
            if request.method != "GET" {
                return http::method_not_allowed("GET");
            }
            http::answer(run_thalamus_assertion_open_assertions(system))
        }
        "/definitions/commands/activate" => {
            if request.method != "POST" {
                return http::method_not_allowed("POST");
            }
            if let Err(actor) = admit(caller, "thalamus.definition.ActivateDefinition") {
                return not_granted(actor);
            }
            serve_thalamus_definition_activate_definition(system, &request.body)
        }
        "/definitions/commands/mute" => {
            if request.method != "POST" {
                return http::method_not_allowed("POST");
            }
            if let Err(actor) = admit(caller, "thalamus.definition.MuteDefinition") {
                return not_granted(actor);
            }
            serve_thalamus_definition_mute_definition(system, &request.body)
        }
        "/definitions/commands/propose" => {
            if request.method != "POST" {
                return http::method_not_allowed("POST");
            }
            if let Err(actor) = admit(caller, "thalamus.definition.ProposeDefinition") {
                return not_granted(actor);
            }
            serve_thalamus_definition_propose_definition(system, &request.body)
        }
        "/definitions/commands/retire" => {
            if request.method != "POST" {
                return http::method_not_allowed("POST");
            }
            if let Err(actor) = admit(caller, "thalamus.definition.RetireDefinition") {
                return not_granted(actor);
            }
            serve_thalamus_definition_retire_definition(system, &request.body)
        }
        "/definitions/commands/revise" => {
            if request.method != "POST" {
                return http::method_not_allowed("POST");
            }
            if let Err(actor) = admit(caller, "thalamus.definition.ReviseDefinition") {
                return not_granted(actor);
            }
            serve_thalamus_definition_revise_definition(system, &request.body)
        }
        "/definitions/commands/unmute" => {
            if request.method != "POST" {
                return http::method_not_allowed("POST");
            }
            if let Err(actor) = admit(caller, "thalamus.definition.UnmuteDefinition") {
                return not_granted(actor);
            }
            serve_thalamus_definition_unmute_definition(system, &request.body)
        }
        "/definitions/views/active" => {
            if request.method != "GET" {
                return http::method_not_allowed("GET");
            }
            http::answer(run_thalamus_definition_active_definitions(system))
        }
        "/definitions/views/definitions" => {
            if request.method != "GET" {
                return http::method_not_allowed("GET");
            }
            http::answer(run_thalamus_definition_definitions(system))
        }
        "/dimensions/commands/declare-registry" => {
            if request.method != "POST" {
                return http::method_not_allowed("POST");
            }
            if let Err(actor) = admit(caller, "thalamus.dimension.DeclareRegistry") {
                return not_granted(actor);
            }
            serve_thalamus_dimension_declare_registry(system, &request.body)
        }
        "/dimensions/commands/register-dimension" => {
            if request.method != "POST" {
                return http::method_not_allowed("POST");
            }
            if let Err(actor) = admit(caller, "thalamus.dimension.RegisterDimension") {
                return not_granted(actor);
            }
            serve_thalamus_dimension_register_dimension(system, &request.body)
        }
        "/dimensions/views/dimensions" => {
            if request.method != "GET" {
                return http::method_not_allowed("GET");
            }
            http::answer(run_thalamus_dimension_dimensions(system))
        }
        "/dimensions/views/registries" => {
            if request.method != "GET" {
                return http::method_not_allowed("GET");
            }
            http::answer(run_thalamus_dimension_registries(system))
        }
        "/docs" => {
            if request.method != "GET" {
                return http::method_not_allowed("GET");
            }
            http::Response::new(200, http::MARKDOWN, DOCS)
        }
        "/evaluations/commands/record" => {
            if request.method != "POST" {
                return http::method_not_allowed("POST");
            }
            if let Err(actor) = admit(caller, "thalamus.evaluation.RecordEvaluation") {
                return not_granted(actor);
            }
            serve_thalamus_evaluation_record_evaluation(system, &request.body)
        }
        "/evaluations/views/evaluations" => {
            if request.method != "GET" {
                return http::method_not_allowed("GET");
            }
            http::answer(run_thalamus_evaluation_evaluations(system))
        }
        "/evaluations/views/transitions" => {
            if request.method != "GET" {
                return http::method_not_allowed("GET");
            }
            http::answer(run_thalamus_evaluation_verdict_changes(system))
        }
        "/evaluations/views/verdicts" => {
            if request.method != "GET" {
                return http::method_not_allowed("GET");
            }
            serve_thalamus_evaluation_series_verdicts(system, &request.query)
        }
        "/openapi.json" => {
            if request.method != "GET" {
                return http::method_not_allowed("GET");
            }
            http::Response::new(200, http::JSON, OPENAPI)
        }
        "/rollups/commands/record" => {
            if request.method != "POST" {
                return http::method_not_allowed("POST");
            }
            if let Err(actor) = admit(caller, "thalamus.rollup.RecordRollup") {
                return not_granted(actor);
            }
            serve_thalamus_rollup_record_rollup(system, &request.body)
        }
        "/rollups/views/rollups" => {
            if request.method != "GET" {
                return http::method_not_allowed("GET");
            }
            http::answer(run_thalamus_rollup_rollups(system))
        }
        "/rollups/views/series" => {
            if request.method != "GET" {
                return http::method_not_allowed("GET");
            }
            serve_thalamus_rollup_series_rollups(system, &request.query)
        }
        "/samples/commands/record" => {
            if request.method != "POST" {
                return http::method_not_allowed("POST");
            }
            if let Err(actor) = admit(caller, "thalamus.observation.RecordSample") {
                return not_granted(actor);
            }
            serve_thalamus_observation_record_sample(system, &request.body)
        }
        "/samples/views/samples" => {
            if request.method != "GET" {
                return http::method_not_allowed("GET");
            }
            http::answer(run_thalamus_observation_samples(system))
        }
        "/samples/views/series" => {
            if request.method != "GET" {
                return http::method_not_allowed("GET");
            }
            serve_thalamus_observation_series_samples(system, &request.query)
        }
        "/transitions/commands/record" => {
            if request.method != "POST" {
                return http::method_not_allowed("POST");
            }
            if let Err(actor) = admit(caller, "thalamus.transition.RecordTransition") {
                return not_granted(actor);
            }
            serve_thalamus_transition_record_transition(system, &request.body)
        }
        "/transitions/views/series" => {
            if request.method != "GET" {
                return http::method_not_allowed("GET");
            }
            serve_thalamus_transition_series_transitions(system, &request.query)
        }
        "/transitions/views/transitions" => {
            if request.method != "GET" {
                return http::method_not_allowed("GET");
            }
            http::answer(run_thalamus_transition_transitions(system))
        }
        other => http::Response::refusal(
            404,
            &format!("`{other}` is not a path this surface declares; `GET /openapi.json` publishes every one that is"),
        ),
    }
}

/// Runs one command or view of `thalamus-service` by its qualified name, with no transport.
///
/// The same decoding, refusals and rendering the HTTP routes use — each route and this function call
/// one `run_*` function — so a conformance runner or an in-process caller drives the system
/// without a socket and without a dispatch table of its own. `Ok` is the declared outcome, as the
/// route's body renders it; a view without parameters ignores `input`, as its `GET` route ignores a body, and a view with
/// parameters reads them from `input`, an object keyed by their wire names, as its route reads
/// them from the query string.
///
/// # Errors
///
/// [`entry::Refused::Unknown`] naming `name` when this surface declares no command or view
/// by it; [`entry::Refused::Input`] when `input` is not the command's declared input (the
/// route's `400`); [`entry::Refused::Unmet`] when the port reports an unmet obligation, and
/// [`entry::Refused::Undelivered`] when the command took effect and delivering what it published
/// failed (the route's `501`, with `committed` `false` and `true`).
/// [`entry::Refused::NotGranted`] when `caller` — who the realization authenticated the call
/// as — is none, or is an actor the specification does not grant the command; checked before
/// the command runs (the route's `403`).
pub fn handle<ThalamusServiceBehaviors>(system: &mut crate::system::System<ThalamusServiceBehaviors>, caller: Option<&crate::actor::Caller>, name: &str, input: json::Value) -> Result<json::Value, entry::Refused>
where
    ThalamusServiceBehaviors: crate::assertion::obligations::AnswerAssertionBehavior + crate::assertion::obligations::AskAssertionBehavior + crate::definition::obligations::ActivateDefinitionBehavior + crate::definition::obligations::MuteDefinitionBehavior + crate::definition::obligations::ProposeDefinitionBehavior + crate::definition::obligations::RetireDefinitionBehavior + crate::definition::obligations::ReviseDefinitionBehavior + crate::definition::obligations::UnmuteDefinitionBehavior + crate::dimension::obligations::DeclareRegistryBehavior + crate::dimension::obligations::RegisterDimensionBehavior + crate::evaluation::obligations::RecordEvaluationBehavior + crate::observation::obligations::RecordSampleBehavior + crate::rollup::obligations::RecordRollupBehavior + crate::transition::obligations::RecordTransitionBehavior + crate::assertion::obligations::AssertionsQuery + crate::assertion::obligations::OpenAssertionsQuery + crate::definition::obligations::ActiveDefinitionsQuery + crate::definition::obligations::DefinitionsQuery + crate::dimension::obligations::DimensionsQuery + crate::dimension::obligations::RegistriesQuery + crate::evaluation::obligations::EvaluationsQuery + crate::evaluation::obligations::SeriesVerdictsQuery + crate::evaluation::obligations::VerdictChangesQuery + crate::observation::obligations::SamplesQuery + crate::observation::obligations::SeriesSamplesQuery + crate::rollup::obligations::RollupsQuery + crate::rollup::obligations::SeriesRollupsQuery + crate::transition::obligations::SeriesTransitionsQuery + crate::transition::obligations::TransitionsQuery,
{
    let answered = match name {
        "thalamus.assertion.AnswerAssertion" => match admit(caller, "thalamus.assertion.AnswerAssertion") {
            Ok(()) => run_thalamus_assertion_answer_assertion(system, &input),
            Err(actor) => Err(entry::Refused::NotGranted(actor.map(str::to_owned))),
        },
        "thalamus.assertion.AskAssertion" => match admit(caller, "thalamus.assertion.AskAssertion") {
            Ok(()) => run_thalamus_assertion_ask_assertion(system, &input),
            Err(actor) => Err(entry::Refused::NotGranted(actor.map(str::to_owned))),
        },
        "thalamus.assertion.Assertions" => run_thalamus_assertion_assertions(system),
        "thalamus.assertion.OpenAssertions" => run_thalamus_assertion_open_assertions(system),
        "thalamus.definition.ActivateDefinition" => match admit(caller, "thalamus.definition.ActivateDefinition") {
            Ok(()) => run_thalamus_definition_activate_definition(system, &input),
            Err(actor) => Err(entry::Refused::NotGranted(actor.map(str::to_owned))),
        },
        "thalamus.definition.ActiveDefinitions" => run_thalamus_definition_active_definitions(system),
        "thalamus.definition.Definitions" => run_thalamus_definition_definitions(system),
        "thalamus.definition.MuteDefinition" => match admit(caller, "thalamus.definition.MuteDefinition") {
            Ok(()) => run_thalamus_definition_mute_definition(system, &input),
            Err(actor) => Err(entry::Refused::NotGranted(actor.map(str::to_owned))),
        },
        "thalamus.definition.ProposeDefinition" => match admit(caller, "thalamus.definition.ProposeDefinition") {
            Ok(()) => run_thalamus_definition_propose_definition(system, &input),
            Err(actor) => Err(entry::Refused::NotGranted(actor.map(str::to_owned))),
        },
        "thalamus.definition.RetireDefinition" => match admit(caller, "thalamus.definition.RetireDefinition") {
            Ok(()) => run_thalamus_definition_retire_definition(system, &input),
            Err(actor) => Err(entry::Refused::NotGranted(actor.map(str::to_owned))),
        },
        "thalamus.definition.ReviseDefinition" => match admit(caller, "thalamus.definition.ReviseDefinition") {
            Ok(()) => run_thalamus_definition_revise_definition(system, &input),
            Err(actor) => Err(entry::Refused::NotGranted(actor.map(str::to_owned))),
        },
        "thalamus.definition.UnmuteDefinition" => match admit(caller, "thalamus.definition.UnmuteDefinition") {
            Ok(()) => run_thalamus_definition_unmute_definition(system, &input),
            Err(actor) => Err(entry::Refused::NotGranted(actor.map(str::to_owned))),
        },
        "thalamus.dimension.DeclareRegistry" => match admit(caller, "thalamus.dimension.DeclareRegistry") {
            Ok(()) => run_thalamus_dimension_declare_registry(system, &input),
            Err(actor) => Err(entry::Refused::NotGranted(actor.map(str::to_owned))),
        },
        "thalamus.dimension.Dimensions" => run_thalamus_dimension_dimensions(system),
        "thalamus.dimension.RegisterDimension" => match admit(caller, "thalamus.dimension.RegisterDimension") {
            Ok(()) => run_thalamus_dimension_register_dimension(system, &input),
            Err(actor) => Err(entry::Refused::NotGranted(actor.map(str::to_owned))),
        },
        "thalamus.dimension.Registries" => run_thalamus_dimension_registries(system),
        "thalamus.evaluation.Evaluations" => run_thalamus_evaluation_evaluations(system),
        "thalamus.evaluation.RecordEvaluation" => match admit(caller, "thalamus.evaluation.RecordEvaluation") {
            Ok(()) => run_thalamus_evaluation_record_evaluation(system, &input),
            Err(actor) => Err(entry::Refused::NotGranted(actor.map(str::to_owned))),
        },
        "thalamus.evaluation.SeriesVerdicts" => run_thalamus_evaluation_series_verdicts(system, &input),
        "thalamus.evaluation.VerdictChanges" => run_thalamus_evaluation_verdict_changes(system),
        "thalamus.observation.RecordSample" => match admit(caller, "thalamus.observation.RecordSample") {
            Ok(()) => run_thalamus_observation_record_sample(system, &input),
            Err(actor) => Err(entry::Refused::NotGranted(actor.map(str::to_owned))),
        },
        "thalamus.observation.Samples" => run_thalamus_observation_samples(system),
        "thalamus.observation.SeriesSamples" => run_thalamus_observation_series_samples(system, &input),
        "thalamus.rollup.RecordRollup" => match admit(caller, "thalamus.rollup.RecordRollup") {
            Ok(()) => run_thalamus_rollup_record_rollup(system, &input),
            Err(actor) => Err(entry::Refused::NotGranted(actor.map(str::to_owned))),
        },
        "thalamus.rollup.Rollups" => run_thalamus_rollup_rollups(system),
        "thalamus.rollup.SeriesRollups" => run_thalamus_rollup_series_rollups(system, &input),
        "thalamus.transition.RecordTransition" => match admit(caller, "thalamus.transition.RecordTransition") {
            Ok(()) => run_thalamus_transition_record_transition(system, &input),
            Err(actor) => Err(entry::Refused::NotGranted(actor.map(str::to_owned))),
        },
        "thalamus.transition.SeriesTransitions" => run_thalamus_transition_series_transitions(system, &input),
        "thalamus.transition.Transitions" => run_thalamus_transition_transitions(system),
        other => return Err(entry::Refused::Unknown(other.to_owned())),
    };
    let (_, body) = answered?;
    Ok(entry::read(&body))
}

/// Nothing, where `caller` may invoke `command`; otherwise the actor the standard refusal names,
/// `None` where the request was authenticated as no actor.
///
/// Checked before the command runs, on the caller the realization authenticated the request
/// as and never on anything the request says about itself. Public, so code that drives the system
/// in process checks the grant exactly as every route does.
pub fn admit(caller: Option<&crate::actor::Caller>, command: &str) -> Result<(), Option<&'static str>> {
    match caller {
        Some(caller) if caller.may(command) => Ok(()),
        Some(caller) => Err(Some(caller.actor.name())),
        None => Err(None),
    }
}

/// The standard refusal for an actor no grant admits, as the contract declares it: `403`,
/// `{"refused": "not granted", "actor": <name or null>}`.
fn not_granted(actor: Option<&str>) -> http::Response {
    let mut body = String::from("{");
    json::member(&mut body, "refused");
    json::push_text(&mut body, "not granted");
    json::member(&mut body, "actor");
    match actor {
        Some(actor) => json::push_text(&mut body, actor),
        None => body.push_str("null"),
    }
    body.push('}');
    http::Response::new(403, http::JSON, body)
}

/// `POST` `thalamus.assertion.AnswerAssertion`: reads the declared input, runs the port, answers the declared outcome.
fn serve_thalamus_assertion_answer_assertion<ThalamusServiceBehaviors>(system: &mut crate::system::System<ThalamusServiceBehaviors>, body: &[u8]) -> http::Response
where
    ThalamusServiceBehaviors: crate::assertion::obligations::AnswerAssertionBehavior + crate::assertion::obligations::AskAssertionBehavior + crate::definition::obligations::ActivateDefinitionBehavior + crate::definition::obligations::MuteDefinitionBehavior + crate::definition::obligations::ProposeDefinitionBehavior + crate::definition::obligations::RetireDefinitionBehavior + crate::definition::obligations::ReviseDefinitionBehavior + crate::definition::obligations::UnmuteDefinitionBehavior + crate::dimension::obligations::DeclareRegistryBehavior + crate::dimension::obligations::RegisterDimensionBehavior + crate::evaluation::obligations::RecordEvaluationBehavior + crate::observation::obligations::RecordSampleBehavior + crate::rollup::obligations::RecordRollupBehavior + crate::transition::obligations::RecordTransitionBehavior + crate::assertion::obligations::AssertionsQuery + crate::assertion::obligations::OpenAssertionsQuery + crate::definition::obligations::ActiveDefinitionsQuery + crate::definition::obligations::DefinitionsQuery + crate::dimension::obligations::DimensionsQuery + crate::dimension::obligations::RegistriesQuery + crate::evaluation::obligations::EvaluationsQuery + crate::evaluation::obligations::SeriesVerdictsQuery + crate::evaluation::obligations::VerdictChangesQuery + crate::observation::obligations::SamplesQuery + crate::observation::obligations::SeriesSamplesQuery + crate::rollup::obligations::RollupsQuery + crate::rollup::obligations::SeriesRollupsQuery + crate::transition::obligations::SeriesTransitionsQuery + crate::transition::obligations::TransitionsQuery,
{
    let text = match std::str::from_utf8(body) {
        Ok(text) => text,
        Err(error) => {
            return http::Response::refusal(400, &format!("the body is not UTF-8: {error}"));
        }
    };
    let value = match json::parse(text) {
        Ok(value) => value,
        Err(error) => {
            return http::Response::refusal(400, &format!("the body is not JSON: {error}"));
        }
    };
    http::answer(run_thalamus_assertion_answer_assertion(system, &value))
}

/// `thalamus.assertion.AnswerAssertion` from its input as a JSON value: decode, run the port, render the declared outcome.
///
/// The one path the `POST` route and [`handle`] share. A decoding failure is located under
/// `body`, as the route reports it.
fn run_thalamus_assertion_answer_assertion<ThalamusServiceBehaviors>(system: &mut crate::system::System<ThalamusServiceBehaviors>, value: &json::Value) -> Result<(u16, String), entry::Refused>
where
    ThalamusServiceBehaviors: crate::assertion::obligations::AnswerAssertionBehavior + crate::assertion::obligations::AskAssertionBehavior + crate::definition::obligations::ActivateDefinitionBehavior + crate::definition::obligations::MuteDefinitionBehavior + crate::definition::obligations::ProposeDefinitionBehavior + crate::definition::obligations::RetireDefinitionBehavior + crate::definition::obligations::ReviseDefinitionBehavior + crate::definition::obligations::UnmuteDefinitionBehavior + crate::dimension::obligations::DeclareRegistryBehavior + crate::dimension::obligations::RegisterDimensionBehavior + crate::evaluation::obligations::RecordEvaluationBehavior + crate::observation::obligations::RecordSampleBehavior + crate::rollup::obligations::RecordRollupBehavior + crate::transition::obligations::RecordTransitionBehavior + crate::assertion::obligations::AssertionsQuery + crate::assertion::obligations::OpenAssertionsQuery + crate::definition::obligations::ActiveDefinitionsQuery + crate::definition::obligations::DefinitionsQuery + crate::dimension::obligations::DimensionsQuery + crate::dimension::obligations::RegistriesQuery + crate::evaluation::obligations::EvaluationsQuery + crate::evaluation::obligations::SeriesVerdictsQuery + crate::evaluation::obligations::VerdictChangesQuery + crate::observation::obligations::SamplesQuery + crate::observation::obligations::SeriesSamplesQuery + crate::rollup::obligations::RollupsQuery + crate::rollup::obligations::SeriesRollupsQuery + crate::transition::obligations::SeriesTransitionsQuery + crate::transition::obligations::TransitionsQuery,
{
    let input = match wire::decode_command_thalamus_assertion_answer_assertion(value, "body") {
        Ok(input) => input,
        Err(error) => {
            // `400` and not `422`: this is a body the schema decides, which is the difference
            // between fixing a value and fixing a serialiser.
            return Err(entry::Refused::Input(format!("{error}")));
        }
    };
    let outcome = match system.thalamus_service.answer_assertion(input) {
        Ok(outcome) => outcome,
        Err(unmet) => return Err(entry::Refused::Unmet(format!("{unmet}"))),
    };
    // Deliver what this command published to every binding that reacts to it, then take it
    // off the log: a long-running server keeps nothing from one request to the next.
    let delivered = system.pump();
    let _ = system.take_published();
    if let Err(failure) = delivered {
        return Err(entry::Refused::Undelivered(format!("delivering what the command published: {failure}")));
    }
    Ok(answer_thalamus_assertion_answer_assertion(&outcome))
}

/// One declared outcome of `thalamus.assertion.AnswerAssertion`: the branch that was taken, every event it published in
/// publication order, the declared error where there is one, and that error's own payload —
/// with the status the contract declares for that branch.
fn answer_thalamus_assertion_answer_assertion(outcome: &crate::assertion::AnswerAssertionOutcome) -> (u16, String) {
    let mut body = String::from("{");
    let status = match outcome {
        crate::assertion::AnswerAssertionOutcome::ReasonMissing { error, .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "reason-missing");
            json::member(&mut body, "published");
            body.push('[');
            body.push(']');
            json::member(&mut body, "error");
            json::push_text(&mut body, "thalamus.assertion.ReasonRequired");
            json::member(&mut body, "payload");
            wire::encode_error_thalamus_assertion_reason_required(error, &mut body);
            422
        }
        crate::assertion::AnswerAssertionOutcome::ContradictsVerdict { error, .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "contradicts-verdict");
            json::member(&mut body, "published");
            body.push('[');
            body.push(']');
            json::member(&mut body, "error");
            json::push_text(&mut body, "thalamus.assertion.ResultContradictsVerdict");
            json::member(&mut body, "payload");
            wire::encode_error_thalamus_assertion_result_contradicts_verdict(error, &mut body);
            422
        }
        crate::assertion::AnswerAssertionOutcome::Answered { assertion_answered, .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "answered");
            json::member(&mut body, "published");
            body.push('[');
            body.push('{');
            json::member(&mut body, "event");
            json::push_text(&mut body, "thalamus.assertion.AssertionAnswered");
            json::member(&mut body, "payload");
            wire::encode_event_thalamus_assertion_assertion_answered(assertion_answered, &mut body);
            body.push('}');
            body.push(']');
            202
        }
        crate::assertion::AnswerAssertionOutcome::WrongState { error, .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "wrong-state");
            json::member(&mut body, "published");
            body.push('[');
            body.push(']');
            json::member(&mut body, "error");
            json::push_text(&mut body, "thalamus.assertion.AssertionStateConflict");
            json::member(&mut body, "payload");
            wire::encode_error_thalamus_assertion_assertion_state_conflict(error, &mut body);
            409
        }
        crate::assertion::AnswerAssertionOutcome::NoSuchAssertion { error, .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "no-such-assertion");
            json::member(&mut body, "published");
            body.push('[');
            body.push(']');
            json::member(&mut body, "error");
            json::push_text(&mut body, "thalamus.assertion.UnknownAssertion");
            json::member(&mut body, "payload");
            wire::encode_error_thalamus_assertion_unknown_assertion(error, &mut body);
            404
        }
    };
    body.push('}');
    (status, body)
}

/// `POST` `thalamus.assertion.AskAssertion`: reads the declared input, runs the port, answers the declared outcome.
fn serve_thalamus_assertion_ask_assertion<ThalamusServiceBehaviors>(system: &mut crate::system::System<ThalamusServiceBehaviors>, body: &[u8]) -> http::Response
where
    ThalamusServiceBehaviors: crate::assertion::obligations::AnswerAssertionBehavior + crate::assertion::obligations::AskAssertionBehavior + crate::definition::obligations::ActivateDefinitionBehavior + crate::definition::obligations::MuteDefinitionBehavior + crate::definition::obligations::ProposeDefinitionBehavior + crate::definition::obligations::RetireDefinitionBehavior + crate::definition::obligations::ReviseDefinitionBehavior + crate::definition::obligations::UnmuteDefinitionBehavior + crate::dimension::obligations::DeclareRegistryBehavior + crate::dimension::obligations::RegisterDimensionBehavior + crate::evaluation::obligations::RecordEvaluationBehavior + crate::observation::obligations::RecordSampleBehavior + crate::rollup::obligations::RecordRollupBehavior + crate::transition::obligations::RecordTransitionBehavior + crate::assertion::obligations::AssertionsQuery + crate::assertion::obligations::OpenAssertionsQuery + crate::definition::obligations::ActiveDefinitionsQuery + crate::definition::obligations::DefinitionsQuery + crate::dimension::obligations::DimensionsQuery + crate::dimension::obligations::RegistriesQuery + crate::evaluation::obligations::EvaluationsQuery + crate::evaluation::obligations::SeriesVerdictsQuery + crate::evaluation::obligations::VerdictChangesQuery + crate::observation::obligations::SamplesQuery + crate::observation::obligations::SeriesSamplesQuery + crate::rollup::obligations::RollupsQuery + crate::rollup::obligations::SeriesRollupsQuery + crate::transition::obligations::SeriesTransitionsQuery + crate::transition::obligations::TransitionsQuery,
{
    let text = match std::str::from_utf8(body) {
        Ok(text) => text,
        Err(error) => {
            return http::Response::refusal(400, &format!("the body is not UTF-8: {error}"));
        }
    };
    let value = match json::parse(text) {
        Ok(value) => value,
        Err(error) => {
            return http::Response::refusal(400, &format!("the body is not JSON: {error}"));
        }
    };
    http::answer(run_thalamus_assertion_ask_assertion(system, &value))
}

/// `thalamus.assertion.AskAssertion` from its input as a JSON value: decode, run the port, render the declared outcome.
///
/// The one path the `POST` route and [`handle`] share. A decoding failure is located under
/// `body`, as the route reports it.
fn run_thalamus_assertion_ask_assertion<ThalamusServiceBehaviors>(system: &mut crate::system::System<ThalamusServiceBehaviors>, value: &json::Value) -> Result<(u16, String), entry::Refused>
where
    ThalamusServiceBehaviors: crate::assertion::obligations::AnswerAssertionBehavior + crate::assertion::obligations::AskAssertionBehavior + crate::definition::obligations::ActivateDefinitionBehavior + crate::definition::obligations::MuteDefinitionBehavior + crate::definition::obligations::ProposeDefinitionBehavior + crate::definition::obligations::RetireDefinitionBehavior + crate::definition::obligations::ReviseDefinitionBehavior + crate::definition::obligations::UnmuteDefinitionBehavior + crate::dimension::obligations::DeclareRegistryBehavior + crate::dimension::obligations::RegisterDimensionBehavior + crate::evaluation::obligations::RecordEvaluationBehavior + crate::observation::obligations::RecordSampleBehavior + crate::rollup::obligations::RecordRollupBehavior + crate::transition::obligations::RecordTransitionBehavior + crate::assertion::obligations::AssertionsQuery + crate::assertion::obligations::OpenAssertionsQuery + crate::definition::obligations::ActiveDefinitionsQuery + crate::definition::obligations::DefinitionsQuery + crate::dimension::obligations::DimensionsQuery + crate::dimension::obligations::RegistriesQuery + crate::evaluation::obligations::EvaluationsQuery + crate::evaluation::obligations::SeriesVerdictsQuery + crate::evaluation::obligations::VerdictChangesQuery + crate::observation::obligations::SamplesQuery + crate::observation::obligations::SeriesSamplesQuery + crate::rollup::obligations::RollupsQuery + crate::rollup::obligations::SeriesRollupsQuery + crate::transition::obligations::SeriesTransitionsQuery + crate::transition::obligations::TransitionsQuery,
{
    let input = match wire::decode_command_thalamus_assertion_ask_assertion(value, "body") {
        Ok(input) => input,
        Err(error) => {
            // `400` and not `422`: this is a body the schema decides, which is the difference
            // between fixing a value and fixing a serialiser.
            return Err(entry::Refused::Input(format!("{error}")));
        }
    };
    let outcome = match system.thalamus_service.ask_assertion(input) {
        Ok(outcome) => outcome,
        Err(unmet) => return Err(entry::Refused::Unmet(format!("{unmet}"))),
    };
    // Deliver what this command published to every binding that reacts to it, then take it
    // off the log: a long-running server keeps nothing from one request to the next.
    let delivered = system.pump();
    let _ = system.take_published();
    if let Err(failure) = delivered {
        return Err(entry::Refused::Undelivered(format!("delivering what the command published: {failure}")));
    }
    Ok(answer_thalamus_assertion_ask_assertion(&outcome))
}

/// One declared outcome of `thalamus.assertion.AskAssertion`: the branch that was taken, every event it published in
/// publication order, the declared error where there is one, and that error's own payload —
/// with the status the contract declares for that branch.
fn answer_thalamus_assertion_ask_assertion(outcome: &crate::assertion::AskAssertionOutcome) -> (u16, String) {
    let mut body = String::from("{");
    let status = match outcome {
        crate::assertion::AskAssertionOutcome::NoSignals { .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "no-signals");
            json::member(&mut body, "published");
            body.push('[');
            body.push(']');
            json::member(&mut body, "error");
            json::push_text(&mut body, "thalamus.assertion.NoSignals");
            422
        }
        crate::assertion::AskAssertionOutcome::Asked { response, assertion_asked, .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "asked");
            json::member(&mut body, "published");
            body.push('[');
            body.push('{');
            json::member(&mut body, "event");
            json::push_text(&mut body, "thalamus.assertion.AssertionAsked");
            json::member(&mut body, "payload");
            wire::encode_event_thalamus_assertion_assertion_asked(assertion_asked, &mut body);
            body.push('}');
            body.push(']');
            json::member(&mut body, "response");
            wire::encode_response_thalamus_assertion_ask_assertion(response, &mut body);
            200
        }
    };
    body.push('}');
    (status, body)
}

/// `GET` `thalamus.assertion.Assertions` at `read_your_writes` consistency: every row the owed projection holds.
///
/// The one path the `GET` route and [`handle`] share.
fn run_thalamus_assertion_assertions<ThalamusServiceBehaviors>(system: &crate::system::System<ThalamusServiceBehaviors>) -> Result<(u16, String), entry::Refused>
where
    ThalamusServiceBehaviors: crate::assertion::obligations::AnswerAssertionBehavior + crate::assertion::obligations::AskAssertionBehavior + crate::definition::obligations::ActivateDefinitionBehavior + crate::definition::obligations::MuteDefinitionBehavior + crate::definition::obligations::ProposeDefinitionBehavior + crate::definition::obligations::RetireDefinitionBehavior + crate::definition::obligations::ReviseDefinitionBehavior + crate::definition::obligations::UnmuteDefinitionBehavior + crate::dimension::obligations::DeclareRegistryBehavior + crate::dimension::obligations::RegisterDimensionBehavior + crate::evaluation::obligations::RecordEvaluationBehavior + crate::observation::obligations::RecordSampleBehavior + crate::rollup::obligations::RecordRollupBehavior + crate::transition::obligations::RecordTransitionBehavior + crate::assertion::obligations::AssertionsQuery + crate::assertion::obligations::OpenAssertionsQuery + crate::definition::obligations::ActiveDefinitionsQuery + crate::definition::obligations::DefinitionsQuery + crate::dimension::obligations::DimensionsQuery + crate::dimension::obligations::RegistriesQuery + crate::evaluation::obligations::EvaluationsQuery + crate::evaluation::obligations::SeriesVerdictsQuery + crate::evaluation::obligations::VerdictChangesQuery + crate::observation::obligations::SamplesQuery + crate::observation::obligations::SeriesSamplesQuery + crate::rollup::obligations::RollupsQuery + crate::rollup::obligations::SeriesRollupsQuery + crate::transition::obligations::SeriesTransitionsQuery + crate::transition::obligations::TransitionsQuery,
{
    match system.thalamus_service.assertions() {
        Ok(rows) => {
            let mut body = String::from("{");
            json::member(&mut body, "rows");
            body.push('[');
            for (position, row) in rows.iter().enumerate() {
                if position > 0 {
                    body.push(',');
                }
                wire::encode_view_thalamus_assertion_assertions(row, &mut body);
            }
            body.push(']');
            body.push('}');
            Ok((200, body))
        }
        Err(unmet) => Err(entry::Refused::Unmet(format!("{unmet}"))),
    }
}

/// `GET` `thalamus.assertion.OpenAssertions` at `read_your_writes` consistency: every row the owed projection holds.
///
/// The one path the `GET` route and [`handle`] share.
fn run_thalamus_assertion_open_assertions<ThalamusServiceBehaviors>(system: &crate::system::System<ThalamusServiceBehaviors>) -> Result<(u16, String), entry::Refused>
where
    ThalamusServiceBehaviors: crate::assertion::obligations::AnswerAssertionBehavior + crate::assertion::obligations::AskAssertionBehavior + crate::definition::obligations::ActivateDefinitionBehavior + crate::definition::obligations::MuteDefinitionBehavior + crate::definition::obligations::ProposeDefinitionBehavior + crate::definition::obligations::RetireDefinitionBehavior + crate::definition::obligations::ReviseDefinitionBehavior + crate::definition::obligations::UnmuteDefinitionBehavior + crate::dimension::obligations::DeclareRegistryBehavior + crate::dimension::obligations::RegisterDimensionBehavior + crate::evaluation::obligations::RecordEvaluationBehavior + crate::observation::obligations::RecordSampleBehavior + crate::rollup::obligations::RecordRollupBehavior + crate::transition::obligations::RecordTransitionBehavior + crate::assertion::obligations::AssertionsQuery + crate::assertion::obligations::OpenAssertionsQuery + crate::definition::obligations::ActiveDefinitionsQuery + crate::definition::obligations::DefinitionsQuery + crate::dimension::obligations::DimensionsQuery + crate::dimension::obligations::RegistriesQuery + crate::evaluation::obligations::EvaluationsQuery + crate::evaluation::obligations::SeriesVerdictsQuery + crate::evaluation::obligations::VerdictChangesQuery + crate::observation::obligations::SamplesQuery + crate::observation::obligations::SeriesSamplesQuery + crate::rollup::obligations::RollupsQuery + crate::rollup::obligations::SeriesRollupsQuery + crate::transition::obligations::SeriesTransitionsQuery + crate::transition::obligations::TransitionsQuery,
{
    match system.thalamus_service.open_assertions() {
        Ok(rows) => {
            let mut body = String::from("{");
            json::member(&mut body, "rows");
            body.push('[');
            for (position, row) in rows.iter().enumerate() {
                if position > 0 {
                    body.push(',');
                }
                wire::encode_view_thalamus_assertion_open_assertions(row, &mut body);
            }
            body.push(']');
            body.push('}');
            Ok((200, body))
        }
        Err(unmet) => Err(entry::Refused::Unmet(format!("{unmet}"))),
    }
}

/// `POST` `thalamus.definition.ActivateDefinition`: reads the declared input, runs the port, answers the declared outcome.
fn serve_thalamus_definition_activate_definition<ThalamusServiceBehaviors>(system: &mut crate::system::System<ThalamusServiceBehaviors>, body: &[u8]) -> http::Response
where
    ThalamusServiceBehaviors: crate::assertion::obligations::AnswerAssertionBehavior + crate::assertion::obligations::AskAssertionBehavior + crate::definition::obligations::ActivateDefinitionBehavior + crate::definition::obligations::MuteDefinitionBehavior + crate::definition::obligations::ProposeDefinitionBehavior + crate::definition::obligations::RetireDefinitionBehavior + crate::definition::obligations::ReviseDefinitionBehavior + crate::definition::obligations::UnmuteDefinitionBehavior + crate::dimension::obligations::DeclareRegistryBehavior + crate::dimension::obligations::RegisterDimensionBehavior + crate::evaluation::obligations::RecordEvaluationBehavior + crate::observation::obligations::RecordSampleBehavior + crate::rollup::obligations::RecordRollupBehavior + crate::transition::obligations::RecordTransitionBehavior + crate::assertion::obligations::AssertionsQuery + crate::assertion::obligations::OpenAssertionsQuery + crate::definition::obligations::ActiveDefinitionsQuery + crate::definition::obligations::DefinitionsQuery + crate::dimension::obligations::DimensionsQuery + crate::dimension::obligations::RegistriesQuery + crate::evaluation::obligations::EvaluationsQuery + crate::evaluation::obligations::SeriesVerdictsQuery + crate::evaluation::obligations::VerdictChangesQuery + crate::observation::obligations::SamplesQuery + crate::observation::obligations::SeriesSamplesQuery + crate::rollup::obligations::RollupsQuery + crate::rollup::obligations::SeriesRollupsQuery + crate::transition::obligations::SeriesTransitionsQuery + crate::transition::obligations::TransitionsQuery,
{
    let text = match std::str::from_utf8(body) {
        Ok(text) => text,
        Err(error) => {
            return http::Response::refusal(400, &format!("the body is not UTF-8: {error}"));
        }
    };
    let value = match json::parse(text) {
        Ok(value) => value,
        Err(error) => {
            return http::Response::refusal(400, &format!("the body is not JSON: {error}"));
        }
    };
    http::answer(run_thalamus_definition_activate_definition(system, &value))
}

/// `thalamus.definition.ActivateDefinition` from its input as a JSON value: decode, run the port, render the declared outcome.
///
/// The one path the `POST` route and [`handle`] share. A decoding failure is located under
/// `body`, as the route reports it.
fn run_thalamus_definition_activate_definition<ThalamusServiceBehaviors>(system: &mut crate::system::System<ThalamusServiceBehaviors>, value: &json::Value) -> Result<(u16, String), entry::Refused>
where
    ThalamusServiceBehaviors: crate::assertion::obligations::AnswerAssertionBehavior + crate::assertion::obligations::AskAssertionBehavior + crate::definition::obligations::ActivateDefinitionBehavior + crate::definition::obligations::MuteDefinitionBehavior + crate::definition::obligations::ProposeDefinitionBehavior + crate::definition::obligations::RetireDefinitionBehavior + crate::definition::obligations::ReviseDefinitionBehavior + crate::definition::obligations::UnmuteDefinitionBehavior + crate::dimension::obligations::DeclareRegistryBehavior + crate::dimension::obligations::RegisterDimensionBehavior + crate::evaluation::obligations::RecordEvaluationBehavior + crate::observation::obligations::RecordSampleBehavior + crate::rollup::obligations::RecordRollupBehavior + crate::transition::obligations::RecordTransitionBehavior + crate::assertion::obligations::AssertionsQuery + crate::assertion::obligations::OpenAssertionsQuery + crate::definition::obligations::ActiveDefinitionsQuery + crate::definition::obligations::DefinitionsQuery + crate::dimension::obligations::DimensionsQuery + crate::dimension::obligations::RegistriesQuery + crate::evaluation::obligations::EvaluationsQuery + crate::evaluation::obligations::SeriesVerdictsQuery + crate::evaluation::obligations::VerdictChangesQuery + crate::observation::obligations::SamplesQuery + crate::observation::obligations::SeriesSamplesQuery + crate::rollup::obligations::RollupsQuery + crate::rollup::obligations::SeriesRollupsQuery + crate::transition::obligations::SeriesTransitionsQuery + crate::transition::obligations::TransitionsQuery,
{
    let input = match wire::decode_command_thalamus_definition_activate_definition(value, "body") {
        Ok(input) => input,
        Err(error) => {
            // `400` and not `422`: this is a body the schema decides, which is the difference
            // between fixing a value and fixing a serialiser.
            return Err(entry::Refused::Input(format!("{error}")));
        }
    };
    let outcome = match system.thalamus_service.activate_definition(input) {
        Ok(outcome) => outcome,
        Err(unmet) => return Err(entry::Refused::Unmet(format!("{unmet}"))),
    };
    // Deliver what this command published to every binding that reacts to it, then take it
    // off the log: a long-running server keeps nothing from one request to the next.
    let delivered = system.pump();
    let _ = system.take_published();
    if let Err(failure) = delivered {
        return Err(entry::Refused::Undelivered(format!("delivering what the command published: {failure}")));
    }
    Ok(answer_thalamus_definition_activate_definition(&outcome))
}

/// One declared outcome of `thalamus.definition.ActivateDefinition`: the branch that was taken, every event it published in
/// publication order, the declared error where there is one, and that error's own payload —
/// with the status the contract declares for that branch.
fn answer_thalamus_definition_activate_definition(outcome: &crate::definition::ActivateDefinitionOutcome) -> (u16, String) {
    let mut body = String::from("{");
    let status = match outcome {
        crate::definition::ActivateDefinitionOutcome::Incomplete { error, .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "incomplete");
            json::member(&mut body, "published");
            body.push('[');
            body.push(']');
            json::member(&mut body, "error");
            json::push_text(&mut body, "thalamus.definition.IncompleteDefinition");
            json::member(&mut body, "payload");
            wire::encode_error_thalamus_definition_incomplete_definition(error, &mut body);
            409
        }
        crate::definition::ActivateDefinitionOutcome::Activated { definition_activated, .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "activated");
            json::member(&mut body, "published");
            body.push('[');
            body.push('{');
            json::member(&mut body, "event");
            json::push_text(&mut body, "thalamus.definition.DefinitionActivated");
            json::member(&mut body, "payload");
            wire::encode_event_thalamus_definition_definition_activated(definition_activated, &mut body);
            body.push('}');
            body.push(']');
            202
        }
        crate::definition::ActivateDefinitionOutcome::WrongState { error, .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "wrong-state");
            json::member(&mut body, "published");
            body.push('[');
            body.push(']');
            json::member(&mut body, "error");
            json::push_text(&mut body, "thalamus.definition.DefinitionStateConflict");
            json::member(&mut body, "payload");
            wire::encode_error_thalamus_definition_definition_state_conflict(error, &mut body);
            409
        }
        crate::definition::ActivateDefinitionOutcome::NoSuchDefinition { error, .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "no-such-definition");
            json::member(&mut body, "published");
            body.push('[');
            body.push(']');
            json::member(&mut body, "error");
            json::push_text(&mut body, "thalamus.definition.UnknownDefinition");
            json::member(&mut body, "payload");
            wire::encode_error_thalamus_definition_unknown_definition(error, &mut body);
            404
        }
    };
    body.push('}');
    (status, body)
}

/// `POST` `thalamus.definition.MuteDefinition`: reads the declared input, runs the port, answers the declared outcome.
fn serve_thalamus_definition_mute_definition<ThalamusServiceBehaviors>(system: &mut crate::system::System<ThalamusServiceBehaviors>, body: &[u8]) -> http::Response
where
    ThalamusServiceBehaviors: crate::assertion::obligations::AnswerAssertionBehavior + crate::assertion::obligations::AskAssertionBehavior + crate::definition::obligations::ActivateDefinitionBehavior + crate::definition::obligations::MuteDefinitionBehavior + crate::definition::obligations::ProposeDefinitionBehavior + crate::definition::obligations::RetireDefinitionBehavior + crate::definition::obligations::ReviseDefinitionBehavior + crate::definition::obligations::UnmuteDefinitionBehavior + crate::dimension::obligations::DeclareRegistryBehavior + crate::dimension::obligations::RegisterDimensionBehavior + crate::evaluation::obligations::RecordEvaluationBehavior + crate::observation::obligations::RecordSampleBehavior + crate::rollup::obligations::RecordRollupBehavior + crate::transition::obligations::RecordTransitionBehavior + crate::assertion::obligations::AssertionsQuery + crate::assertion::obligations::OpenAssertionsQuery + crate::definition::obligations::ActiveDefinitionsQuery + crate::definition::obligations::DefinitionsQuery + crate::dimension::obligations::DimensionsQuery + crate::dimension::obligations::RegistriesQuery + crate::evaluation::obligations::EvaluationsQuery + crate::evaluation::obligations::SeriesVerdictsQuery + crate::evaluation::obligations::VerdictChangesQuery + crate::observation::obligations::SamplesQuery + crate::observation::obligations::SeriesSamplesQuery + crate::rollup::obligations::RollupsQuery + crate::rollup::obligations::SeriesRollupsQuery + crate::transition::obligations::SeriesTransitionsQuery + crate::transition::obligations::TransitionsQuery,
{
    let text = match std::str::from_utf8(body) {
        Ok(text) => text,
        Err(error) => {
            return http::Response::refusal(400, &format!("the body is not UTF-8: {error}"));
        }
    };
    let value = match json::parse(text) {
        Ok(value) => value,
        Err(error) => {
            return http::Response::refusal(400, &format!("the body is not JSON: {error}"));
        }
    };
    http::answer(run_thalamus_definition_mute_definition(system, &value))
}

/// `thalamus.definition.MuteDefinition` from its input as a JSON value: decode, run the port, render the declared outcome.
///
/// The one path the `POST` route and [`handle`] share. A decoding failure is located under
/// `body`, as the route reports it.
fn run_thalamus_definition_mute_definition<ThalamusServiceBehaviors>(system: &mut crate::system::System<ThalamusServiceBehaviors>, value: &json::Value) -> Result<(u16, String), entry::Refused>
where
    ThalamusServiceBehaviors: crate::assertion::obligations::AnswerAssertionBehavior + crate::assertion::obligations::AskAssertionBehavior + crate::definition::obligations::ActivateDefinitionBehavior + crate::definition::obligations::MuteDefinitionBehavior + crate::definition::obligations::ProposeDefinitionBehavior + crate::definition::obligations::RetireDefinitionBehavior + crate::definition::obligations::ReviseDefinitionBehavior + crate::definition::obligations::UnmuteDefinitionBehavior + crate::dimension::obligations::DeclareRegistryBehavior + crate::dimension::obligations::RegisterDimensionBehavior + crate::evaluation::obligations::RecordEvaluationBehavior + crate::observation::obligations::RecordSampleBehavior + crate::rollup::obligations::RecordRollupBehavior + crate::transition::obligations::RecordTransitionBehavior + crate::assertion::obligations::AssertionsQuery + crate::assertion::obligations::OpenAssertionsQuery + crate::definition::obligations::ActiveDefinitionsQuery + crate::definition::obligations::DefinitionsQuery + crate::dimension::obligations::DimensionsQuery + crate::dimension::obligations::RegistriesQuery + crate::evaluation::obligations::EvaluationsQuery + crate::evaluation::obligations::SeriesVerdictsQuery + crate::evaluation::obligations::VerdictChangesQuery + crate::observation::obligations::SamplesQuery + crate::observation::obligations::SeriesSamplesQuery + crate::rollup::obligations::RollupsQuery + crate::rollup::obligations::SeriesRollupsQuery + crate::transition::obligations::SeriesTransitionsQuery + crate::transition::obligations::TransitionsQuery,
{
    let input = match wire::decode_command_thalamus_definition_mute_definition(value, "body") {
        Ok(input) => input,
        Err(error) => {
            // `400` and not `422`: this is a body the schema decides, which is the difference
            // between fixing a value and fixing a serialiser.
            return Err(entry::Refused::Input(format!("{error}")));
        }
    };
    let outcome = match system.thalamus_service.mute_definition(input) {
        Ok(outcome) => outcome,
        Err(unmet) => return Err(entry::Refused::Unmet(format!("{unmet}"))),
    };
    // Deliver what this command published to every binding that reacts to it, then take it
    // off the log: a long-running server keeps nothing from one request to the next.
    let delivered = system.pump();
    let _ = system.take_published();
    if let Err(failure) = delivered {
        return Err(entry::Refused::Undelivered(format!("delivering what the command published: {failure}")));
    }
    Ok(answer_thalamus_definition_mute_definition(&outcome))
}

/// One declared outcome of `thalamus.definition.MuteDefinition`: the branch that was taken, every event it published in
/// publication order, the declared error where there is one, and that error's own payload —
/// with the status the contract declares for that branch.
fn answer_thalamus_definition_mute_definition(outcome: &crate::definition::MuteDefinitionOutcome) -> (u16, String) {
    let mut body = String::from("{");
    let status = match outcome {
        crate::definition::MuteDefinitionOutcome::ExpiryPassed { error, .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "expiry-passed");
            json::member(&mut body, "published");
            body.push('[');
            body.push(']');
            json::member(&mut body, "error");
            json::push_text(&mut body, "thalamus.definition.MuteExpiryPassed");
            json::member(&mut body, "payload");
            wire::encode_error_thalamus_definition_mute_expiry_passed(error, &mut body);
            422
        }
        crate::definition::MuteDefinitionOutcome::Muted { definition_muted, .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "muted");
            json::member(&mut body, "published");
            body.push('[');
            body.push('{');
            json::member(&mut body, "event");
            json::push_text(&mut body, "thalamus.definition.DefinitionMuted");
            json::member(&mut body, "payload");
            wire::encode_event_thalamus_definition_definition_muted(definition_muted, &mut body);
            body.push('}');
            body.push(']');
            202
        }
        crate::definition::MuteDefinitionOutcome::WrongState { error, .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "wrong-state");
            json::member(&mut body, "published");
            body.push('[');
            body.push(']');
            json::member(&mut body, "error");
            json::push_text(&mut body, "thalamus.definition.DefinitionStateConflict");
            json::member(&mut body, "payload");
            wire::encode_error_thalamus_definition_definition_state_conflict(error, &mut body);
            409
        }
        crate::definition::MuteDefinitionOutcome::NoSuchDefinition { error, .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "no-such-definition");
            json::member(&mut body, "published");
            body.push('[');
            body.push(']');
            json::member(&mut body, "error");
            json::push_text(&mut body, "thalamus.definition.UnknownDefinition");
            json::member(&mut body, "payload");
            wire::encode_error_thalamus_definition_unknown_definition(error, &mut body);
            404
        }
    };
    body.push('}');
    (status, body)
}

/// `POST` `thalamus.definition.ProposeDefinition`: reads the declared input, runs the port, answers the declared outcome.
fn serve_thalamus_definition_propose_definition<ThalamusServiceBehaviors>(system: &mut crate::system::System<ThalamusServiceBehaviors>, body: &[u8]) -> http::Response
where
    ThalamusServiceBehaviors: crate::assertion::obligations::AnswerAssertionBehavior + crate::assertion::obligations::AskAssertionBehavior + crate::definition::obligations::ActivateDefinitionBehavior + crate::definition::obligations::MuteDefinitionBehavior + crate::definition::obligations::ProposeDefinitionBehavior + crate::definition::obligations::RetireDefinitionBehavior + crate::definition::obligations::ReviseDefinitionBehavior + crate::definition::obligations::UnmuteDefinitionBehavior + crate::dimension::obligations::DeclareRegistryBehavior + crate::dimension::obligations::RegisterDimensionBehavior + crate::evaluation::obligations::RecordEvaluationBehavior + crate::observation::obligations::RecordSampleBehavior + crate::rollup::obligations::RecordRollupBehavior + crate::transition::obligations::RecordTransitionBehavior + crate::assertion::obligations::AssertionsQuery + crate::assertion::obligations::OpenAssertionsQuery + crate::definition::obligations::ActiveDefinitionsQuery + crate::definition::obligations::DefinitionsQuery + crate::dimension::obligations::DimensionsQuery + crate::dimension::obligations::RegistriesQuery + crate::evaluation::obligations::EvaluationsQuery + crate::evaluation::obligations::SeriesVerdictsQuery + crate::evaluation::obligations::VerdictChangesQuery + crate::observation::obligations::SamplesQuery + crate::observation::obligations::SeriesSamplesQuery + crate::rollup::obligations::RollupsQuery + crate::rollup::obligations::SeriesRollupsQuery + crate::transition::obligations::SeriesTransitionsQuery + crate::transition::obligations::TransitionsQuery,
{
    let text = match std::str::from_utf8(body) {
        Ok(text) => text,
        Err(error) => {
            return http::Response::refusal(400, &format!("the body is not UTF-8: {error}"));
        }
    };
    let value = match json::parse(text) {
        Ok(value) => value,
        Err(error) => {
            return http::Response::refusal(400, &format!("the body is not JSON: {error}"));
        }
    };
    http::answer(run_thalamus_definition_propose_definition(system, &value))
}

/// `thalamus.definition.ProposeDefinition` from its input as a JSON value: decode, run the port, render the declared outcome.
///
/// The one path the `POST` route and [`handle`] share. A decoding failure is located under
/// `body`, as the route reports it.
fn run_thalamus_definition_propose_definition<ThalamusServiceBehaviors>(system: &mut crate::system::System<ThalamusServiceBehaviors>, value: &json::Value) -> Result<(u16, String), entry::Refused>
where
    ThalamusServiceBehaviors: crate::assertion::obligations::AnswerAssertionBehavior + crate::assertion::obligations::AskAssertionBehavior + crate::definition::obligations::ActivateDefinitionBehavior + crate::definition::obligations::MuteDefinitionBehavior + crate::definition::obligations::ProposeDefinitionBehavior + crate::definition::obligations::RetireDefinitionBehavior + crate::definition::obligations::ReviseDefinitionBehavior + crate::definition::obligations::UnmuteDefinitionBehavior + crate::dimension::obligations::DeclareRegistryBehavior + crate::dimension::obligations::RegisterDimensionBehavior + crate::evaluation::obligations::RecordEvaluationBehavior + crate::observation::obligations::RecordSampleBehavior + crate::rollup::obligations::RecordRollupBehavior + crate::transition::obligations::RecordTransitionBehavior + crate::assertion::obligations::AssertionsQuery + crate::assertion::obligations::OpenAssertionsQuery + crate::definition::obligations::ActiveDefinitionsQuery + crate::definition::obligations::DefinitionsQuery + crate::dimension::obligations::DimensionsQuery + crate::dimension::obligations::RegistriesQuery + crate::evaluation::obligations::EvaluationsQuery + crate::evaluation::obligations::SeriesVerdictsQuery + crate::evaluation::obligations::VerdictChangesQuery + crate::observation::obligations::SamplesQuery + crate::observation::obligations::SeriesSamplesQuery + crate::rollup::obligations::RollupsQuery + crate::rollup::obligations::SeriesRollupsQuery + crate::transition::obligations::SeriesTransitionsQuery + crate::transition::obligations::TransitionsQuery,
{
    let input = match wire::decode_command_thalamus_definition_propose_definition(value, "body") {
        Ok(input) => input,
        Err(error) => {
            // `400` and not `422`: this is a body the schema decides, which is the difference
            // between fixing a value and fixing a serialiser.
            return Err(entry::Refused::Input(format!("{error}")));
        }
    };
    let outcome = match system.thalamus_service.propose_definition(input) {
        Ok(outcome) => outcome,
        Err(unmet) => return Err(entry::Refused::Unmet(format!("{unmet}"))),
    };
    // Deliver what this command published to every binding that reacts to it, then take it
    // off the log: a long-running server keeps nothing from one request to the next.
    let delivered = system.pump();
    let _ = system.take_published();
    if let Err(failure) = delivered {
        return Err(entry::Refused::Undelivered(format!("delivering what the command published: {failure}")));
    }
    Ok(answer_thalamus_definition_propose_definition(&outcome))
}

/// One declared outcome of `thalamus.definition.ProposeDefinition`: the branch that was taken, every event it published in
/// publication order, the declared error where there is one, and that error's own payload —
/// with the status the contract declares for that branch.
fn answer_thalamus_definition_propose_definition(outcome: &crate::definition::ProposeDefinitionOutcome) -> (u16, String) {
    let mut body = String::from("{");
    let status = match outcome {
        crate::definition::ProposeDefinitionOutcome::InstantNotFixed { error, .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "instant-not-fixed");
            json::member(&mut body, "published");
            body.push('[');
            body.push(']');
            json::member(&mut body, "error");
            json::push_text(&mut body, "thalamus.definition.InstantNeedsFixedBaseline");
            json::member(&mut body, "payload");
            wire::encode_error_thalamus_definition_instant_needs_fixed_baseline(error, &mut body);
            422
        }
        crate::definition::ProposeDefinitionOutcome::RollingNMissing { error, .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "rolling-n-missing");
            json::member(&mut body, "published");
            body.push('[');
            body.push(']');
            json::member(&mut body, "error");
            json::push_text(&mut body, "thalamus.definition.RollingWindowMissing");
            json::member(&mut body, "payload");
            wire::encode_error_thalamus_definition_rolling_window_missing(error, &mut body);
            422
        }
        crate::definition::ProposeDefinitionOutcome::RollingNUnused { error, .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "rolling-n-unused");
            json::member(&mut body, "published");
            body.push('[');
            body.push(']');
            json::member(&mut body, "error");
            json::push_text(&mut body, "thalamus.definition.RollingWindowUnused");
            json::member(&mut body, "payload");
            wire::encode_error_thalamus_definition_rolling_window_unused(error, &mut body);
            422
        }
        crate::definition::ProposeDefinitionOutcome::FixedNumberMissing { error, .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "fixed-number-missing");
            json::member(&mut body, "published");
            body.push('[');
            body.push(']');
            json::member(&mut body, "error");
            json::push_text(&mut body, "thalamus.definition.FixedNumberMissing");
            json::member(&mut body, "payload");
            wire::encode_error_thalamus_definition_fixed_number_missing(error, &mut body);
            422
        }
        crate::definition::ProposeDefinitionOutcome::FixedNumberUnused { error, .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "fixed-number-unused");
            json::member(&mut body, "published");
            body.push('[');
            body.push(']');
            json::member(&mut body, "error");
            json::push_text(&mut body, "thalamus.definition.FixedNumberUnused");
            json::member(&mut body, "payload");
            wire::encode_error_thalamus_definition_fixed_number_unused(error, &mut body);
            422
        }
        crate::definition::ProposeDefinitionOutcome::PersistenceBelowOne { error, .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "persistence-below-one");
            json::member(&mut body, "published");
            body.push('[');
            body.push(']');
            json::member(&mut body, "error");
            json::push_text(&mut body, "thalamus.definition.PersistenceTooSmall");
            json::member(&mut body, "payload");
            wire::encode_error_thalamus_definition_persistence_too_small(error, &mut body);
            422
        }
        crate::definition::ProposeDefinitionOutcome::MappingMissing { error, .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "mapping-missing");
            json::member(&mut body, "published");
            body.push('[');
            body.push(']');
            json::member(&mut body, "error");
            json::push_text(&mut body, "thalamus.definition.MappingMissing");
            json::member(&mut body, "payload");
            wire::encode_error_thalamus_definition_mapping_missing(error, &mut body);
            422
        }
        crate::definition::ProposeDefinitionOutcome::MappingUnused { error, .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "mapping-unused");
            json::member(&mut body, "published");
            body.push('[');
            body.push(']');
            json::member(&mut body, "error");
            json::push_text(&mut body, "thalamus.definition.MappingUnused");
            json::member(&mut body, "payload");
            wire::encode_error_thalamus_definition_mapping_unused(error, &mut body);
            422
        }
        crate::definition::ProposeDefinitionOutcome::ThresholdsUnused { error, .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "thresholds-unused");
            json::member(&mut body, "published");
            body.push('[');
            body.push(']');
            json::member(&mut body, "error");
            json::push_text(&mut body, "thalamus.definition.ThresholdsUnused");
            json::member(&mut body, "payload");
            wire::encode_error_thalamus_definition_thresholds_unused(error, &mut body);
            422
        }
        crate::definition::ProposeDefinitionOutcome::Proposed { definition_proposed, .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "proposed");
            json::member(&mut body, "published");
            body.push('[');
            body.push('{');
            json::member(&mut body, "event");
            json::push_text(&mut body, "thalamus.definition.DefinitionProposed");
            json::member(&mut body, "payload");
            wire::encode_event_thalamus_definition_definition_proposed(definition_proposed, &mut body);
            body.push('}');
            body.push(']');
            202
        }
        crate::definition::ProposeDefinitionOutcome::AlreadyDefined { error, .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "already-defined");
            json::member(&mut body, "published");
            body.push('[');
            body.push(']');
            json::member(&mut body, "error");
            json::push_text(&mut body, "thalamus.definition.DefinitionExists");
            json::member(&mut body, "payload");
            wire::encode_error_thalamus_definition_definition_exists(error, &mut body);
            409
        }
    };
    body.push('}');
    (status, body)
}

/// `POST` `thalamus.definition.RetireDefinition`: reads the declared input, runs the port, answers the declared outcome.
fn serve_thalamus_definition_retire_definition<ThalamusServiceBehaviors>(system: &mut crate::system::System<ThalamusServiceBehaviors>, body: &[u8]) -> http::Response
where
    ThalamusServiceBehaviors: crate::assertion::obligations::AnswerAssertionBehavior + crate::assertion::obligations::AskAssertionBehavior + crate::definition::obligations::ActivateDefinitionBehavior + crate::definition::obligations::MuteDefinitionBehavior + crate::definition::obligations::ProposeDefinitionBehavior + crate::definition::obligations::RetireDefinitionBehavior + crate::definition::obligations::ReviseDefinitionBehavior + crate::definition::obligations::UnmuteDefinitionBehavior + crate::dimension::obligations::DeclareRegistryBehavior + crate::dimension::obligations::RegisterDimensionBehavior + crate::evaluation::obligations::RecordEvaluationBehavior + crate::observation::obligations::RecordSampleBehavior + crate::rollup::obligations::RecordRollupBehavior + crate::transition::obligations::RecordTransitionBehavior + crate::assertion::obligations::AssertionsQuery + crate::assertion::obligations::OpenAssertionsQuery + crate::definition::obligations::ActiveDefinitionsQuery + crate::definition::obligations::DefinitionsQuery + crate::dimension::obligations::DimensionsQuery + crate::dimension::obligations::RegistriesQuery + crate::evaluation::obligations::EvaluationsQuery + crate::evaluation::obligations::SeriesVerdictsQuery + crate::evaluation::obligations::VerdictChangesQuery + crate::observation::obligations::SamplesQuery + crate::observation::obligations::SeriesSamplesQuery + crate::rollup::obligations::RollupsQuery + crate::rollup::obligations::SeriesRollupsQuery + crate::transition::obligations::SeriesTransitionsQuery + crate::transition::obligations::TransitionsQuery,
{
    let text = match std::str::from_utf8(body) {
        Ok(text) => text,
        Err(error) => {
            return http::Response::refusal(400, &format!("the body is not UTF-8: {error}"));
        }
    };
    let value = match json::parse(text) {
        Ok(value) => value,
        Err(error) => {
            return http::Response::refusal(400, &format!("the body is not JSON: {error}"));
        }
    };
    http::answer(run_thalamus_definition_retire_definition(system, &value))
}

/// `thalamus.definition.RetireDefinition` from its input as a JSON value: decode, run the port, render the declared outcome.
///
/// The one path the `POST` route and [`handle`] share. A decoding failure is located under
/// `body`, as the route reports it.
fn run_thalamus_definition_retire_definition<ThalamusServiceBehaviors>(system: &mut crate::system::System<ThalamusServiceBehaviors>, value: &json::Value) -> Result<(u16, String), entry::Refused>
where
    ThalamusServiceBehaviors: crate::assertion::obligations::AnswerAssertionBehavior + crate::assertion::obligations::AskAssertionBehavior + crate::definition::obligations::ActivateDefinitionBehavior + crate::definition::obligations::MuteDefinitionBehavior + crate::definition::obligations::ProposeDefinitionBehavior + crate::definition::obligations::RetireDefinitionBehavior + crate::definition::obligations::ReviseDefinitionBehavior + crate::definition::obligations::UnmuteDefinitionBehavior + crate::dimension::obligations::DeclareRegistryBehavior + crate::dimension::obligations::RegisterDimensionBehavior + crate::evaluation::obligations::RecordEvaluationBehavior + crate::observation::obligations::RecordSampleBehavior + crate::rollup::obligations::RecordRollupBehavior + crate::transition::obligations::RecordTransitionBehavior + crate::assertion::obligations::AssertionsQuery + crate::assertion::obligations::OpenAssertionsQuery + crate::definition::obligations::ActiveDefinitionsQuery + crate::definition::obligations::DefinitionsQuery + crate::dimension::obligations::DimensionsQuery + crate::dimension::obligations::RegistriesQuery + crate::evaluation::obligations::EvaluationsQuery + crate::evaluation::obligations::SeriesVerdictsQuery + crate::evaluation::obligations::VerdictChangesQuery + crate::observation::obligations::SamplesQuery + crate::observation::obligations::SeriesSamplesQuery + crate::rollup::obligations::RollupsQuery + crate::rollup::obligations::SeriesRollupsQuery + crate::transition::obligations::SeriesTransitionsQuery + crate::transition::obligations::TransitionsQuery,
{
    let input = match wire::decode_command_thalamus_definition_retire_definition(value, "body") {
        Ok(input) => input,
        Err(error) => {
            // `400` and not `422`: this is a body the schema decides, which is the difference
            // between fixing a value and fixing a serialiser.
            return Err(entry::Refused::Input(format!("{error}")));
        }
    };
    let outcome = match system.thalamus_service.retire_definition(input) {
        Ok(outcome) => outcome,
        Err(unmet) => return Err(entry::Refused::Unmet(format!("{unmet}"))),
    };
    // Deliver what this command published to every binding that reacts to it, then take it
    // off the log: a long-running server keeps nothing from one request to the next.
    let delivered = system.pump();
    let _ = system.take_published();
    if let Err(failure) = delivered {
        return Err(entry::Refused::Undelivered(format!("delivering what the command published: {failure}")));
    }
    Ok(answer_thalamus_definition_retire_definition(&outcome))
}

/// One declared outcome of `thalamus.definition.RetireDefinition`: the branch that was taken, every event it published in
/// publication order, the declared error where there is one, and that error's own payload —
/// with the status the contract declares for that branch.
fn answer_thalamus_definition_retire_definition(outcome: &crate::definition::RetireDefinitionOutcome) -> (u16, String) {
    let mut body = String::from("{");
    let status = match outcome {
        crate::definition::RetireDefinitionOutcome::Retired { definition_retired, .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "retired");
            json::member(&mut body, "published");
            body.push('[');
            body.push('{');
            json::member(&mut body, "event");
            json::push_text(&mut body, "thalamus.definition.DefinitionRetired");
            json::member(&mut body, "payload");
            wire::encode_event_thalamus_definition_definition_retired(definition_retired, &mut body);
            body.push('}');
            body.push(']');
            202
        }
        crate::definition::RetireDefinitionOutcome::WrongState { error, .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "wrong-state");
            json::member(&mut body, "published");
            body.push('[');
            body.push(']');
            json::member(&mut body, "error");
            json::push_text(&mut body, "thalamus.definition.DefinitionStateConflict");
            json::member(&mut body, "payload");
            wire::encode_error_thalamus_definition_definition_state_conflict(error, &mut body);
            409
        }
        crate::definition::RetireDefinitionOutcome::NoSuchDefinition { error, .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "no-such-definition");
            json::member(&mut body, "published");
            body.push('[');
            body.push(']');
            json::member(&mut body, "error");
            json::push_text(&mut body, "thalamus.definition.UnknownDefinition");
            json::member(&mut body, "payload");
            wire::encode_error_thalamus_definition_unknown_definition(error, &mut body);
            404
        }
    };
    body.push('}');
    (status, body)
}

/// `POST` `thalamus.definition.ReviseDefinition`: reads the declared input, runs the port, answers the declared outcome.
fn serve_thalamus_definition_revise_definition<ThalamusServiceBehaviors>(system: &mut crate::system::System<ThalamusServiceBehaviors>, body: &[u8]) -> http::Response
where
    ThalamusServiceBehaviors: crate::assertion::obligations::AnswerAssertionBehavior + crate::assertion::obligations::AskAssertionBehavior + crate::definition::obligations::ActivateDefinitionBehavior + crate::definition::obligations::MuteDefinitionBehavior + crate::definition::obligations::ProposeDefinitionBehavior + crate::definition::obligations::RetireDefinitionBehavior + crate::definition::obligations::ReviseDefinitionBehavior + crate::definition::obligations::UnmuteDefinitionBehavior + crate::dimension::obligations::DeclareRegistryBehavior + crate::dimension::obligations::RegisterDimensionBehavior + crate::evaluation::obligations::RecordEvaluationBehavior + crate::observation::obligations::RecordSampleBehavior + crate::rollup::obligations::RecordRollupBehavior + crate::transition::obligations::RecordTransitionBehavior + crate::assertion::obligations::AssertionsQuery + crate::assertion::obligations::OpenAssertionsQuery + crate::definition::obligations::ActiveDefinitionsQuery + crate::definition::obligations::DefinitionsQuery + crate::dimension::obligations::DimensionsQuery + crate::dimension::obligations::RegistriesQuery + crate::evaluation::obligations::EvaluationsQuery + crate::evaluation::obligations::SeriesVerdictsQuery + crate::evaluation::obligations::VerdictChangesQuery + crate::observation::obligations::SamplesQuery + crate::observation::obligations::SeriesSamplesQuery + crate::rollup::obligations::RollupsQuery + crate::rollup::obligations::SeriesRollupsQuery + crate::transition::obligations::SeriesTransitionsQuery + crate::transition::obligations::TransitionsQuery,
{
    let text = match std::str::from_utf8(body) {
        Ok(text) => text,
        Err(error) => {
            return http::Response::refusal(400, &format!("the body is not UTF-8: {error}"));
        }
    };
    let value = match json::parse(text) {
        Ok(value) => value,
        Err(error) => {
            return http::Response::refusal(400, &format!("the body is not JSON: {error}"));
        }
    };
    http::answer(run_thalamus_definition_revise_definition(system, &value))
}

/// `thalamus.definition.ReviseDefinition` from its input as a JSON value: decode, run the port, render the declared outcome.
///
/// The one path the `POST` route and [`handle`] share. A decoding failure is located under
/// `body`, as the route reports it.
fn run_thalamus_definition_revise_definition<ThalamusServiceBehaviors>(system: &mut crate::system::System<ThalamusServiceBehaviors>, value: &json::Value) -> Result<(u16, String), entry::Refused>
where
    ThalamusServiceBehaviors: crate::assertion::obligations::AnswerAssertionBehavior + crate::assertion::obligations::AskAssertionBehavior + crate::definition::obligations::ActivateDefinitionBehavior + crate::definition::obligations::MuteDefinitionBehavior + crate::definition::obligations::ProposeDefinitionBehavior + crate::definition::obligations::RetireDefinitionBehavior + crate::definition::obligations::ReviseDefinitionBehavior + crate::definition::obligations::UnmuteDefinitionBehavior + crate::dimension::obligations::DeclareRegistryBehavior + crate::dimension::obligations::RegisterDimensionBehavior + crate::evaluation::obligations::RecordEvaluationBehavior + crate::observation::obligations::RecordSampleBehavior + crate::rollup::obligations::RecordRollupBehavior + crate::transition::obligations::RecordTransitionBehavior + crate::assertion::obligations::AssertionsQuery + crate::assertion::obligations::OpenAssertionsQuery + crate::definition::obligations::ActiveDefinitionsQuery + crate::definition::obligations::DefinitionsQuery + crate::dimension::obligations::DimensionsQuery + crate::dimension::obligations::RegistriesQuery + crate::evaluation::obligations::EvaluationsQuery + crate::evaluation::obligations::SeriesVerdictsQuery + crate::evaluation::obligations::VerdictChangesQuery + crate::observation::obligations::SamplesQuery + crate::observation::obligations::SeriesSamplesQuery + crate::rollup::obligations::RollupsQuery + crate::rollup::obligations::SeriesRollupsQuery + crate::transition::obligations::SeriesTransitionsQuery + crate::transition::obligations::TransitionsQuery,
{
    let input = match wire::decode_command_thalamus_definition_revise_definition(value, "body") {
        Ok(input) => input,
        Err(error) => {
            // `400` and not `422`: this is a body the schema decides, which is the difference
            // between fixing a value and fixing a serialiser.
            return Err(entry::Refused::Input(format!("{error}")));
        }
    };
    let outcome = match system.thalamus_service.revise_definition(input) {
        Ok(outcome) => outcome,
        Err(unmet) => return Err(entry::Refused::Unmet(format!("{unmet}"))),
    };
    // Deliver what this command published to every binding that reacts to it, then take it
    // off the log: a long-running server keeps nothing from one request to the next.
    let delivered = system.pump();
    let _ = system.take_published();
    if let Err(failure) = delivered {
        return Err(entry::Refused::Undelivered(format!("delivering what the command published: {failure}")));
    }
    Ok(answer_thalamus_definition_revise_definition(&outcome))
}

/// One declared outcome of `thalamus.definition.ReviseDefinition`: the branch that was taken, every event it published in
/// publication order, the declared error where there is one, and that error's own payload —
/// with the status the contract declares for that branch.
fn answer_thalamus_definition_revise_definition(outcome: &crate::definition::ReviseDefinitionOutcome) -> (u16, String) {
    let mut body = String::from("{");
    let status = match outcome {
        crate::definition::ReviseDefinitionOutcome::Retired { error, .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "retired");
            json::member(&mut body, "published");
            body.push('[');
            body.push(']');
            json::member(&mut body, "error");
            json::push_text(&mut body, "thalamus.definition.RetiredDefinition");
            json::member(&mut body, "payload");
            wire::encode_error_thalamus_definition_retired_definition(error, &mut body);
            409
        }
        crate::definition::ReviseDefinitionOutcome::InstantNotFixed { error, .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "instant-not-fixed");
            json::member(&mut body, "published");
            body.push('[');
            body.push(']');
            json::member(&mut body, "error");
            json::push_text(&mut body, "thalamus.definition.InstantNeedsFixedBaseline");
            json::member(&mut body, "payload");
            wire::encode_error_thalamus_definition_instant_needs_fixed_baseline(error, &mut body);
            422
        }
        crate::definition::ReviseDefinitionOutcome::RollingNMissing { error, .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "rolling-n-missing");
            json::member(&mut body, "published");
            body.push('[');
            body.push(']');
            json::member(&mut body, "error");
            json::push_text(&mut body, "thalamus.definition.RollingWindowMissing");
            json::member(&mut body, "payload");
            wire::encode_error_thalamus_definition_rolling_window_missing(error, &mut body);
            422
        }
        crate::definition::ReviseDefinitionOutcome::RollingNUnused { error, .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "rolling-n-unused");
            json::member(&mut body, "published");
            body.push('[');
            body.push(']');
            json::member(&mut body, "error");
            json::push_text(&mut body, "thalamus.definition.RollingWindowUnused");
            json::member(&mut body, "payload");
            wire::encode_error_thalamus_definition_rolling_window_unused(error, &mut body);
            422
        }
        crate::definition::ReviseDefinitionOutcome::FixedNumberMissing { error, .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "fixed-number-missing");
            json::member(&mut body, "published");
            body.push('[');
            body.push(']');
            json::member(&mut body, "error");
            json::push_text(&mut body, "thalamus.definition.FixedNumberMissing");
            json::member(&mut body, "payload");
            wire::encode_error_thalamus_definition_fixed_number_missing(error, &mut body);
            422
        }
        crate::definition::ReviseDefinitionOutcome::FixedNumberUnused { error, .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "fixed-number-unused");
            json::member(&mut body, "published");
            body.push('[');
            body.push(']');
            json::member(&mut body, "error");
            json::push_text(&mut body, "thalamus.definition.FixedNumberUnused");
            json::member(&mut body, "payload");
            wire::encode_error_thalamus_definition_fixed_number_unused(error, &mut body);
            422
        }
        crate::definition::ReviseDefinitionOutcome::PersistenceBelowOne { error, .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "persistence-below-one");
            json::member(&mut body, "published");
            body.push('[');
            body.push(']');
            json::member(&mut body, "error");
            json::push_text(&mut body, "thalamus.definition.PersistenceTooSmall");
            json::member(&mut body, "payload");
            wire::encode_error_thalamus_definition_persistence_too_small(error, &mut body);
            422
        }
        crate::definition::ReviseDefinitionOutcome::MappingMissing { error, .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "mapping-missing");
            json::member(&mut body, "published");
            body.push('[');
            body.push(']');
            json::member(&mut body, "error");
            json::push_text(&mut body, "thalamus.definition.MappingMissing");
            json::member(&mut body, "payload");
            wire::encode_error_thalamus_definition_mapping_missing(error, &mut body);
            422
        }
        crate::definition::ReviseDefinitionOutcome::MappingUnused { error, .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "mapping-unused");
            json::member(&mut body, "published");
            body.push('[');
            body.push(']');
            json::member(&mut body, "error");
            json::push_text(&mut body, "thalamus.definition.MappingUnused");
            json::member(&mut body, "payload");
            wire::encode_error_thalamus_definition_mapping_unused(error, &mut body);
            422
        }
        crate::definition::ReviseDefinitionOutcome::ThresholdsUnused { error, .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "thresholds-unused");
            json::member(&mut body, "published");
            body.push('[');
            body.push(']');
            json::member(&mut body, "error");
            json::push_text(&mut body, "thalamus.definition.ThresholdsUnused");
            json::member(&mut body, "payload");
            wire::encode_error_thalamus_definition_thresholds_unused(error, &mut body);
            422
        }
        crate::definition::ReviseDefinitionOutcome::Revised { definition_revised, .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "revised");
            json::member(&mut body, "published");
            body.push('[');
            body.push('{');
            json::member(&mut body, "event");
            json::push_text(&mut body, "thalamus.definition.DefinitionRevised");
            json::member(&mut body, "payload");
            wire::encode_event_thalamus_definition_definition_revised(definition_revised, &mut body);
            body.push('}');
            body.push(']');
            202
        }
        crate::definition::ReviseDefinitionOutcome::NoSuchDefinition { error, .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "no-such-definition");
            json::member(&mut body, "published");
            body.push('[');
            body.push(']');
            json::member(&mut body, "error");
            json::push_text(&mut body, "thalamus.definition.UnknownDefinition");
            json::member(&mut body, "payload");
            wire::encode_error_thalamus_definition_unknown_definition(error, &mut body);
            404
        }
    };
    body.push('}');
    (status, body)
}

/// `POST` `thalamus.definition.UnmuteDefinition`: reads the declared input, runs the port, answers the declared outcome.
fn serve_thalamus_definition_unmute_definition<ThalamusServiceBehaviors>(system: &mut crate::system::System<ThalamusServiceBehaviors>, body: &[u8]) -> http::Response
where
    ThalamusServiceBehaviors: crate::assertion::obligations::AnswerAssertionBehavior + crate::assertion::obligations::AskAssertionBehavior + crate::definition::obligations::ActivateDefinitionBehavior + crate::definition::obligations::MuteDefinitionBehavior + crate::definition::obligations::ProposeDefinitionBehavior + crate::definition::obligations::RetireDefinitionBehavior + crate::definition::obligations::ReviseDefinitionBehavior + crate::definition::obligations::UnmuteDefinitionBehavior + crate::dimension::obligations::DeclareRegistryBehavior + crate::dimension::obligations::RegisterDimensionBehavior + crate::evaluation::obligations::RecordEvaluationBehavior + crate::observation::obligations::RecordSampleBehavior + crate::rollup::obligations::RecordRollupBehavior + crate::transition::obligations::RecordTransitionBehavior + crate::assertion::obligations::AssertionsQuery + crate::assertion::obligations::OpenAssertionsQuery + crate::definition::obligations::ActiveDefinitionsQuery + crate::definition::obligations::DefinitionsQuery + crate::dimension::obligations::DimensionsQuery + crate::dimension::obligations::RegistriesQuery + crate::evaluation::obligations::EvaluationsQuery + crate::evaluation::obligations::SeriesVerdictsQuery + crate::evaluation::obligations::VerdictChangesQuery + crate::observation::obligations::SamplesQuery + crate::observation::obligations::SeriesSamplesQuery + crate::rollup::obligations::RollupsQuery + crate::rollup::obligations::SeriesRollupsQuery + crate::transition::obligations::SeriesTransitionsQuery + crate::transition::obligations::TransitionsQuery,
{
    let text = match std::str::from_utf8(body) {
        Ok(text) => text,
        Err(error) => {
            return http::Response::refusal(400, &format!("the body is not UTF-8: {error}"));
        }
    };
    let value = match json::parse(text) {
        Ok(value) => value,
        Err(error) => {
            return http::Response::refusal(400, &format!("the body is not JSON: {error}"));
        }
    };
    http::answer(run_thalamus_definition_unmute_definition(system, &value))
}

/// `thalamus.definition.UnmuteDefinition` from its input as a JSON value: decode, run the port, render the declared outcome.
///
/// The one path the `POST` route and [`handle`] share. A decoding failure is located under
/// `body`, as the route reports it.
fn run_thalamus_definition_unmute_definition<ThalamusServiceBehaviors>(system: &mut crate::system::System<ThalamusServiceBehaviors>, value: &json::Value) -> Result<(u16, String), entry::Refused>
where
    ThalamusServiceBehaviors: crate::assertion::obligations::AnswerAssertionBehavior + crate::assertion::obligations::AskAssertionBehavior + crate::definition::obligations::ActivateDefinitionBehavior + crate::definition::obligations::MuteDefinitionBehavior + crate::definition::obligations::ProposeDefinitionBehavior + crate::definition::obligations::RetireDefinitionBehavior + crate::definition::obligations::ReviseDefinitionBehavior + crate::definition::obligations::UnmuteDefinitionBehavior + crate::dimension::obligations::DeclareRegistryBehavior + crate::dimension::obligations::RegisterDimensionBehavior + crate::evaluation::obligations::RecordEvaluationBehavior + crate::observation::obligations::RecordSampleBehavior + crate::rollup::obligations::RecordRollupBehavior + crate::transition::obligations::RecordTransitionBehavior + crate::assertion::obligations::AssertionsQuery + crate::assertion::obligations::OpenAssertionsQuery + crate::definition::obligations::ActiveDefinitionsQuery + crate::definition::obligations::DefinitionsQuery + crate::dimension::obligations::DimensionsQuery + crate::dimension::obligations::RegistriesQuery + crate::evaluation::obligations::EvaluationsQuery + crate::evaluation::obligations::SeriesVerdictsQuery + crate::evaluation::obligations::VerdictChangesQuery + crate::observation::obligations::SamplesQuery + crate::observation::obligations::SeriesSamplesQuery + crate::rollup::obligations::RollupsQuery + crate::rollup::obligations::SeriesRollupsQuery + crate::transition::obligations::SeriesTransitionsQuery + crate::transition::obligations::TransitionsQuery,
{
    let input = match wire::decode_command_thalamus_definition_unmute_definition(value, "body") {
        Ok(input) => input,
        Err(error) => {
            // `400` and not `422`: this is a body the schema decides, which is the difference
            // between fixing a value and fixing a serialiser.
            return Err(entry::Refused::Input(format!("{error}")));
        }
    };
    let outcome = match system.thalamus_service.unmute_definition(input) {
        Ok(outcome) => outcome,
        Err(unmet) => return Err(entry::Refused::Unmet(format!("{unmet}"))),
    };
    // Deliver what this command published to every binding that reacts to it, then take it
    // off the log: a long-running server keeps nothing from one request to the next.
    let delivered = system.pump();
    let _ = system.take_published();
    if let Err(failure) = delivered {
        return Err(entry::Refused::Undelivered(format!("delivering what the command published: {failure}")));
    }
    Ok(answer_thalamus_definition_unmute_definition(&outcome))
}

/// One declared outcome of `thalamus.definition.UnmuteDefinition`: the branch that was taken, every event it published in
/// publication order, the declared error where there is one, and that error's own payload —
/// with the status the contract declares for that branch.
fn answer_thalamus_definition_unmute_definition(outcome: &crate::definition::UnmuteDefinitionOutcome) -> (u16, String) {
    let mut body = String::from("{");
    let status = match outcome {
        crate::definition::UnmuteDefinitionOutcome::Unmuted { definition_unmuted, .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "unmuted");
            json::member(&mut body, "published");
            body.push('[');
            body.push('{');
            json::member(&mut body, "event");
            json::push_text(&mut body, "thalamus.definition.DefinitionUnmuted");
            json::member(&mut body, "payload");
            wire::encode_event_thalamus_definition_definition_unmuted(definition_unmuted, &mut body);
            body.push('}');
            body.push(']');
            202
        }
        crate::definition::UnmuteDefinitionOutcome::WrongState { error, .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "wrong-state");
            json::member(&mut body, "published");
            body.push('[');
            body.push(']');
            json::member(&mut body, "error");
            json::push_text(&mut body, "thalamus.definition.DefinitionStateConflict");
            json::member(&mut body, "payload");
            wire::encode_error_thalamus_definition_definition_state_conflict(error, &mut body);
            409
        }
        crate::definition::UnmuteDefinitionOutcome::NoSuchDefinition { error, .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "no-such-definition");
            json::member(&mut body, "published");
            body.push('[');
            body.push(']');
            json::member(&mut body, "error");
            json::push_text(&mut body, "thalamus.definition.UnknownDefinition");
            json::member(&mut body, "payload");
            wire::encode_error_thalamus_definition_unknown_definition(error, &mut body);
            404
        }
    };
    body.push('}');
    (status, body)
}

/// `GET` `thalamus.definition.ActiveDefinitions` at `read_your_writes` consistency: every row the owed projection holds.
///
/// The one path the `GET` route and [`handle`] share.
fn run_thalamus_definition_active_definitions<ThalamusServiceBehaviors>(system: &crate::system::System<ThalamusServiceBehaviors>) -> Result<(u16, String), entry::Refused>
where
    ThalamusServiceBehaviors: crate::assertion::obligations::AnswerAssertionBehavior + crate::assertion::obligations::AskAssertionBehavior + crate::definition::obligations::ActivateDefinitionBehavior + crate::definition::obligations::MuteDefinitionBehavior + crate::definition::obligations::ProposeDefinitionBehavior + crate::definition::obligations::RetireDefinitionBehavior + crate::definition::obligations::ReviseDefinitionBehavior + crate::definition::obligations::UnmuteDefinitionBehavior + crate::dimension::obligations::DeclareRegistryBehavior + crate::dimension::obligations::RegisterDimensionBehavior + crate::evaluation::obligations::RecordEvaluationBehavior + crate::observation::obligations::RecordSampleBehavior + crate::rollup::obligations::RecordRollupBehavior + crate::transition::obligations::RecordTransitionBehavior + crate::assertion::obligations::AssertionsQuery + crate::assertion::obligations::OpenAssertionsQuery + crate::definition::obligations::ActiveDefinitionsQuery + crate::definition::obligations::DefinitionsQuery + crate::dimension::obligations::DimensionsQuery + crate::dimension::obligations::RegistriesQuery + crate::evaluation::obligations::EvaluationsQuery + crate::evaluation::obligations::SeriesVerdictsQuery + crate::evaluation::obligations::VerdictChangesQuery + crate::observation::obligations::SamplesQuery + crate::observation::obligations::SeriesSamplesQuery + crate::rollup::obligations::RollupsQuery + crate::rollup::obligations::SeriesRollupsQuery + crate::transition::obligations::SeriesTransitionsQuery + crate::transition::obligations::TransitionsQuery,
{
    match system.thalamus_service.active_definitions() {
        Ok(rows) => {
            let mut body = String::from("{");
            json::member(&mut body, "rows");
            body.push('[');
            for (position, row) in rows.iter().enumerate() {
                if position > 0 {
                    body.push(',');
                }
                wire::encode_view_thalamus_definition_active_definitions(row, &mut body);
            }
            body.push(']');
            body.push('}');
            Ok((200, body))
        }
        Err(unmet) => Err(entry::Refused::Unmet(format!("{unmet}"))),
    }
}

/// `GET` `thalamus.definition.Definitions` at `read_your_writes` consistency: every row the owed projection holds.
///
/// The one path the `GET` route and [`handle`] share.
fn run_thalamus_definition_definitions<ThalamusServiceBehaviors>(system: &crate::system::System<ThalamusServiceBehaviors>) -> Result<(u16, String), entry::Refused>
where
    ThalamusServiceBehaviors: crate::assertion::obligations::AnswerAssertionBehavior + crate::assertion::obligations::AskAssertionBehavior + crate::definition::obligations::ActivateDefinitionBehavior + crate::definition::obligations::MuteDefinitionBehavior + crate::definition::obligations::ProposeDefinitionBehavior + crate::definition::obligations::RetireDefinitionBehavior + crate::definition::obligations::ReviseDefinitionBehavior + crate::definition::obligations::UnmuteDefinitionBehavior + crate::dimension::obligations::DeclareRegistryBehavior + crate::dimension::obligations::RegisterDimensionBehavior + crate::evaluation::obligations::RecordEvaluationBehavior + crate::observation::obligations::RecordSampleBehavior + crate::rollup::obligations::RecordRollupBehavior + crate::transition::obligations::RecordTransitionBehavior + crate::assertion::obligations::AssertionsQuery + crate::assertion::obligations::OpenAssertionsQuery + crate::definition::obligations::ActiveDefinitionsQuery + crate::definition::obligations::DefinitionsQuery + crate::dimension::obligations::DimensionsQuery + crate::dimension::obligations::RegistriesQuery + crate::evaluation::obligations::EvaluationsQuery + crate::evaluation::obligations::SeriesVerdictsQuery + crate::evaluation::obligations::VerdictChangesQuery + crate::observation::obligations::SamplesQuery + crate::observation::obligations::SeriesSamplesQuery + crate::rollup::obligations::RollupsQuery + crate::rollup::obligations::SeriesRollupsQuery + crate::transition::obligations::SeriesTransitionsQuery + crate::transition::obligations::TransitionsQuery,
{
    match system.thalamus_service.definitions() {
        Ok(rows) => {
            let mut body = String::from("{");
            json::member(&mut body, "rows");
            body.push('[');
            for (position, row) in rows.iter().enumerate() {
                if position > 0 {
                    body.push(',');
                }
                wire::encode_view_thalamus_definition_definitions(row, &mut body);
            }
            body.push(']');
            body.push('}');
            Ok((200, body))
        }
        Err(unmet) => Err(entry::Refused::Unmet(format!("{unmet}"))),
    }
}

/// `POST` `thalamus.dimension.DeclareRegistry`: reads the declared input, runs the port, answers the declared outcome.
fn serve_thalamus_dimension_declare_registry<ThalamusServiceBehaviors>(system: &mut crate::system::System<ThalamusServiceBehaviors>, body: &[u8]) -> http::Response
where
    ThalamusServiceBehaviors: crate::assertion::obligations::AnswerAssertionBehavior + crate::assertion::obligations::AskAssertionBehavior + crate::definition::obligations::ActivateDefinitionBehavior + crate::definition::obligations::MuteDefinitionBehavior + crate::definition::obligations::ProposeDefinitionBehavior + crate::definition::obligations::RetireDefinitionBehavior + crate::definition::obligations::ReviseDefinitionBehavior + crate::definition::obligations::UnmuteDefinitionBehavior + crate::dimension::obligations::DeclareRegistryBehavior + crate::dimension::obligations::RegisterDimensionBehavior + crate::evaluation::obligations::RecordEvaluationBehavior + crate::observation::obligations::RecordSampleBehavior + crate::rollup::obligations::RecordRollupBehavior + crate::transition::obligations::RecordTransitionBehavior + crate::assertion::obligations::AssertionsQuery + crate::assertion::obligations::OpenAssertionsQuery + crate::definition::obligations::ActiveDefinitionsQuery + crate::definition::obligations::DefinitionsQuery + crate::dimension::obligations::DimensionsQuery + crate::dimension::obligations::RegistriesQuery + crate::evaluation::obligations::EvaluationsQuery + crate::evaluation::obligations::SeriesVerdictsQuery + crate::evaluation::obligations::VerdictChangesQuery + crate::observation::obligations::SamplesQuery + crate::observation::obligations::SeriesSamplesQuery + crate::rollup::obligations::RollupsQuery + crate::rollup::obligations::SeriesRollupsQuery + crate::transition::obligations::SeriesTransitionsQuery + crate::transition::obligations::TransitionsQuery,
{
    let text = match std::str::from_utf8(body) {
        Ok(text) => text,
        Err(error) => {
            return http::Response::refusal(400, &format!("the body is not UTF-8: {error}"));
        }
    };
    let value = match json::parse(text) {
        Ok(value) => value,
        Err(error) => {
            return http::Response::refusal(400, &format!("the body is not JSON: {error}"));
        }
    };
    http::answer(run_thalamus_dimension_declare_registry(system, &value))
}

/// `thalamus.dimension.DeclareRegistry` from its input as a JSON value: decode, run the port, render the declared outcome.
///
/// The one path the `POST` route and [`handle`] share. A decoding failure is located under
/// `body`, as the route reports it.
fn run_thalamus_dimension_declare_registry<ThalamusServiceBehaviors>(system: &mut crate::system::System<ThalamusServiceBehaviors>, value: &json::Value) -> Result<(u16, String), entry::Refused>
where
    ThalamusServiceBehaviors: crate::assertion::obligations::AnswerAssertionBehavior + crate::assertion::obligations::AskAssertionBehavior + crate::definition::obligations::ActivateDefinitionBehavior + crate::definition::obligations::MuteDefinitionBehavior + crate::definition::obligations::ProposeDefinitionBehavior + crate::definition::obligations::RetireDefinitionBehavior + crate::definition::obligations::ReviseDefinitionBehavior + crate::definition::obligations::UnmuteDefinitionBehavior + crate::dimension::obligations::DeclareRegistryBehavior + crate::dimension::obligations::RegisterDimensionBehavior + crate::evaluation::obligations::RecordEvaluationBehavior + crate::observation::obligations::RecordSampleBehavior + crate::rollup::obligations::RecordRollupBehavior + crate::transition::obligations::RecordTransitionBehavior + crate::assertion::obligations::AssertionsQuery + crate::assertion::obligations::OpenAssertionsQuery + crate::definition::obligations::ActiveDefinitionsQuery + crate::definition::obligations::DefinitionsQuery + crate::dimension::obligations::DimensionsQuery + crate::dimension::obligations::RegistriesQuery + crate::evaluation::obligations::EvaluationsQuery + crate::evaluation::obligations::SeriesVerdictsQuery + crate::evaluation::obligations::VerdictChangesQuery + crate::observation::obligations::SamplesQuery + crate::observation::obligations::SeriesSamplesQuery + crate::rollup::obligations::RollupsQuery + crate::rollup::obligations::SeriesRollupsQuery + crate::transition::obligations::SeriesTransitionsQuery + crate::transition::obligations::TransitionsQuery,
{
    let input = match wire::decode_command_thalamus_dimension_declare_registry(value, "body") {
        Ok(input) => input,
        Err(error) => {
            // `400` and not `422`: this is a body the schema decides, which is the difference
            // between fixing a value and fixing a serialiser.
            return Err(entry::Refused::Input(format!("{error}")));
        }
    };
    let outcome = match system.thalamus_service.declare_registry(input) {
        Ok(outcome) => outcome,
        Err(unmet) => return Err(entry::Refused::Unmet(format!("{unmet}"))),
    };
    // Deliver what this command published to every binding that reacts to it, then take it
    // off the log: a long-running server keeps nothing from one request to the next.
    let delivered = system.pump();
    let _ = system.take_published();
    if let Err(failure) = delivered {
        return Err(entry::Refused::Undelivered(format!("delivering what the command published: {failure}")));
    }
    Ok(answer_thalamus_dimension_declare_registry(&outcome))
}

/// One declared outcome of `thalamus.dimension.DeclareRegistry`: the branch that was taken, every event it published in
/// publication order, the declared error where there is one, and that error's own payload —
/// with the status the contract declares for that branch.
fn answer_thalamus_dimension_declare_registry(outcome: &crate::dimension::DeclareRegistryOutcome) -> (u16, String) {
    let mut body = String::from("{");
    let status = match outcome {
        crate::dimension::DeclareRegistryOutcome::Declared { registry_declared, .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "declared");
            json::member(&mut body, "published");
            body.push('[');
            body.push('{');
            json::member(&mut body, "event");
            json::push_text(&mut body, "thalamus.dimension.RegistryDeclared");
            json::member(&mut body, "payload");
            wire::encode_event_thalamus_dimension_registry_declared(registry_declared, &mut body);
            body.push('}');
            body.push(']');
            202
        }
        crate::dimension::DeclareRegistryOutcome::AlreadyDeclared { error, .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "already-declared");
            json::member(&mut body, "published");
            body.push('[');
            body.push(']');
            json::member(&mut body, "error");
            json::push_text(&mut body, "thalamus.dimension.RegistryExists");
            json::member(&mut body, "payload");
            wire::encode_error_thalamus_dimension_registry_exists(error, &mut body);
            409
        }
    };
    body.push('}');
    (status, body)
}

/// `POST` `thalamus.dimension.RegisterDimension`: reads the declared input, runs the port, answers the declared outcome.
fn serve_thalamus_dimension_register_dimension<ThalamusServiceBehaviors>(system: &mut crate::system::System<ThalamusServiceBehaviors>, body: &[u8]) -> http::Response
where
    ThalamusServiceBehaviors: crate::assertion::obligations::AnswerAssertionBehavior + crate::assertion::obligations::AskAssertionBehavior + crate::definition::obligations::ActivateDefinitionBehavior + crate::definition::obligations::MuteDefinitionBehavior + crate::definition::obligations::ProposeDefinitionBehavior + crate::definition::obligations::RetireDefinitionBehavior + crate::definition::obligations::ReviseDefinitionBehavior + crate::definition::obligations::UnmuteDefinitionBehavior + crate::dimension::obligations::DeclareRegistryBehavior + crate::dimension::obligations::RegisterDimensionBehavior + crate::evaluation::obligations::RecordEvaluationBehavior + crate::observation::obligations::RecordSampleBehavior + crate::rollup::obligations::RecordRollupBehavior + crate::transition::obligations::RecordTransitionBehavior + crate::assertion::obligations::AssertionsQuery + crate::assertion::obligations::OpenAssertionsQuery + crate::definition::obligations::ActiveDefinitionsQuery + crate::definition::obligations::DefinitionsQuery + crate::dimension::obligations::DimensionsQuery + crate::dimension::obligations::RegistriesQuery + crate::evaluation::obligations::EvaluationsQuery + crate::evaluation::obligations::SeriesVerdictsQuery + crate::evaluation::obligations::VerdictChangesQuery + crate::observation::obligations::SamplesQuery + crate::observation::obligations::SeriesSamplesQuery + crate::rollup::obligations::RollupsQuery + crate::rollup::obligations::SeriesRollupsQuery + crate::transition::obligations::SeriesTransitionsQuery + crate::transition::obligations::TransitionsQuery,
{
    let text = match std::str::from_utf8(body) {
        Ok(text) => text,
        Err(error) => {
            return http::Response::refusal(400, &format!("the body is not UTF-8: {error}"));
        }
    };
    let value = match json::parse(text) {
        Ok(value) => value,
        Err(error) => {
            return http::Response::refusal(400, &format!("the body is not JSON: {error}"));
        }
    };
    http::answer(run_thalamus_dimension_register_dimension(system, &value))
}

/// `thalamus.dimension.RegisterDimension` from its input as a JSON value: decode, run the port, render the declared outcome.
///
/// The one path the `POST` route and [`handle`] share. A decoding failure is located under
/// `body`, as the route reports it.
fn run_thalamus_dimension_register_dimension<ThalamusServiceBehaviors>(system: &mut crate::system::System<ThalamusServiceBehaviors>, value: &json::Value) -> Result<(u16, String), entry::Refused>
where
    ThalamusServiceBehaviors: crate::assertion::obligations::AnswerAssertionBehavior + crate::assertion::obligations::AskAssertionBehavior + crate::definition::obligations::ActivateDefinitionBehavior + crate::definition::obligations::MuteDefinitionBehavior + crate::definition::obligations::ProposeDefinitionBehavior + crate::definition::obligations::RetireDefinitionBehavior + crate::definition::obligations::ReviseDefinitionBehavior + crate::definition::obligations::UnmuteDefinitionBehavior + crate::dimension::obligations::DeclareRegistryBehavior + crate::dimension::obligations::RegisterDimensionBehavior + crate::evaluation::obligations::RecordEvaluationBehavior + crate::observation::obligations::RecordSampleBehavior + crate::rollup::obligations::RecordRollupBehavior + crate::transition::obligations::RecordTransitionBehavior + crate::assertion::obligations::AssertionsQuery + crate::assertion::obligations::OpenAssertionsQuery + crate::definition::obligations::ActiveDefinitionsQuery + crate::definition::obligations::DefinitionsQuery + crate::dimension::obligations::DimensionsQuery + crate::dimension::obligations::RegistriesQuery + crate::evaluation::obligations::EvaluationsQuery + crate::evaluation::obligations::SeriesVerdictsQuery + crate::evaluation::obligations::VerdictChangesQuery + crate::observation::obligations::SamplesQuery + crate::observation::obligations::SeriesSamplesQuery + crate::rollup::obligations::RollupsQuery + crate::rollup::obligations::SeriesRollupsQuery + crate::transition::obligations::SeriesTransitionsQuery + crate::transition::obligations::TransitionsQuery,
{
    let input = match wire::decode_command_thalamus_dimension_register_dimension(value, "body") {
        Ok(input) => input,
        Err(error) => {
            // `400` and not `422`: this is a body the schema decides, which is the difference
            // between fixing a value and fixing a serialiser.
            return Err(entry::Refused::Input(format!("{error}")));
        }
    };
    let outcome = match system.thalamus_service.register_dimension(input) {
        Ok(outcome) => outcome,
        Err(unmet) => return Err(entry::Refused::Unmet(format!("{unmet}"))),
    };
    // Deliver what this command published to every binding that reacts to it, then take it
    // off the log: a long-running server keeps nothing from one request to the next.
    let delivered = system.pump();
    let _ = system.take_published();
    if let Err(failure) = delivered {
        return Err(entry::Refused::Undelivered(format!("delivering what the command published: {failure}")));
    }
    Ok(answer_thalamus_dimension_register_dimension(&outcome))
}

/// One declared outcome of `thalamus.dimension.RegisterDimension`: the branch that was taken, every event it published in
/// publication order, the declared error where there is one, and that error's own payload —
/// with the status the contract declares for that branch.
fn answer_thalamus_dimension_register_dimension(outcome: &crate::dimension::RegisterDimensionOutcome) -> (u16, String) {
    let mut body = String::from("{");
    let status = match outcome {
        crate::dimension::RegisterDimensionOutcome::NoRegistry { error, .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "no-registry");
            json::member(&mut body, "published");
            body.push('[');
            body.push(']');
            json::member(&mut body, "error");
            json::push_text(&mut body, "thalamus.dimension.UnknownRegistry");
            json::member(&mut body, "payload");
            wire::encode_error_thalamus_dimension_unknown_registry(error, &mut body);
            409
        }
        crate::dimension::RegisterDimensionOutcome::NoParent { error, .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "no-parent");
            json::member(&mut body, "published");
            body.push('[');
            body.push(']');
            json::member(&mut body, "error");
            json::push_text(&mut body, "thalamus.dimension.UnknownParent");
            json::member(&mut body, "payload");
            wire::encode_error_thalamus_dimension_unknown_parent(error, &mut body);
            409
        }
        crate::dimension::RegisterDimensionOutcome::Registered { dimension_registered, .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "registered");
            json::member(&mut body, "published");
            body.push('[');
            body.push('{');
            json::member(&mut body, "event");
            json::push_text(&mut body, "thalamus.dimension.DimensionRegistered");
            json::member(&mut body, "payload");
            wire::encode_event_thalamus_dimension_dimension_registered(dimension_registered, &mut body);
            body.push('}');
            body.push(']');
            202
        }
        crate::dimension::RegisterDimensionOutcome::AlreadyRegistered { error, .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "already-registered");
            json::member(&mut body, "published");
            body.push('[');
            body.push(']');
            json::member(&mut body, "error");
            json::push_text(&mut body, "thalamus.dimension.DimensionExists");
            json::member(&mut body, "payload");
            wire::encode_error_thalamus_dimension_dimension_exists(error, &mut body);
            409
        }
    };
    body.push('}');
    (status, body)
}

/// `GET` `thalamus.dimension.Dimensions` at `read_your_writes` consistency: every row the owed projection holds.
///
/// The one path the `GET` route and [`handle`] share.
fn run_thalamus_dimension_dimensions<ThalamusServiceBehaviors>(system: &crate::system::System<ThalamusServiceBehaviors>) -> Result<(u16, String), entry::Refused>
where
    ThalamusServiceBehaviors: crate::assertion::obligations::AnswerAssertionBehavior + crate::assertion::obligations::AskAssertionBehavior + crate::definition::obligations::ActivateDefinitionBehavior + crate::definition::obligations::MuteDefinitionBehavior + crate::definition::obligations::ProposeDefinitionBehavior + crate::definition::obligations::RetireDefinitionBehavior + crate::definition::obligations::ReviseDefinitionBehavior + crate::definition::obligations::UnmuteDefinitionBehavior + crate::dimension::obligations::DeclareRegistryBehavior + crate::dimension::obligations::RegisterDimensionBehavior + crate::evaluation::obligations::RecordEvaluationBehavior + crate::observation::obligations::RecordSampleBehavior + crate::rollup::obligations::RecordRollupBehavior + crate::transition::obligations::RecordTransitionBehavior + crate::assertion::obligations::AssertionsQuery + crate::assertion::obligations::OpenAssertionsQuery + crate::definition::obligations::ActiveDefinitionsQuery + crate::definition::obligations::DefinitionsQuery + crate::dimension::obligations::DimensionsQuery + crate::dimension::obligations::RegistriesQuery + crate::evaluation::obligations::EvaluationsQuery + crate::evaluation::obligations::SeriesVerdictsQuery + crate::evaluation::obligations::VerdictChangesQuery + crate::observation::obligations::SamplesQuery + crate::observation::obligations::SeriesSamplesQuery + crate::rollup::obligations::RollupsQuery + crate::rollup::obligations::SeriesRollupsQuery + crate::transition::obligations::SeriesTransitionsQuery + crate::transition::obligations::TransitionsQuery,
{
    match system.thalamus_service.dimensions() {
        Ok(rows) => {
            let mut body = String::from("{");
            json::member(&mut body, "rows");
            body.push('[');
            for (position, row) in rows.iter().enumerate() {
                if position > 0 {
                    body.push(',');
                }
                wire::encode_view_thalamus_dimension_dimensions(row, &mut body);
            }
            body.push(']');
            body.push('}');
            Ok((200, body))
        }
        Err(unmet) => Err(entry::Refused::Unmet(format!("{unmet}"))),
    }
}

/// `GET` `thalamus.dimension.Registries` at `read_your_writes` consistency: every row the owed projection holds.
///
/// The one path the `GET` route and [`handle`] share.
fn run_thalamus_dimension_registries<ThalamusServiceBehaviors>(system: &crate::system::System<ThalamusServiceBehaviors>) -> Result<(u16, String), entry::Refused>
where
    ThalamusServiceBehaviors: crate::assertion::obligations::AnswerAssertionBehavior + crate::assertion::obligations::AskAssertionBehavior + crate::definition::obligations::ActivateDefinitionBehavior + crate::definition::obligations::MuteDefinitionBehavior + crate::definition::obligations::ProposeDefinitionBehavior + crate::definition::obligations::RetireDefinitionBehavior + crate::definition::obligations::ReviseDefinitionBehavior + crate::definition::obligations::UnmuteDefinitionBehavior + crate::dimension::obligations::DeclareRegistryBehavior + crate::dimension::obligations::RegisterDimensionBehavior + crate::evaluation::obligations::RecordEvaluationBehavior + crate::observation::obligations::RecordSampleBehavior + crate::rollup::obligations::RecordRollupBehavior + crate::transition::obligations::RecordTransitionBehavior + crate::assertion::obligations::AssertionsQuery + crate::assertion::obligations::OpenAssertionsQuery + crate::definition::obligations::ActiveDefinitionsQuery + crate::definition::obligations::DefinitionsQuery + crate::dimension::obligations::DimensionsQuery + crate::dimension::obligations::RegistriesQuery + crate::evaluation::obligations::EvaluationsQuery + crate::evaluation::obligations::SeriesVerdictsQuery + crate::evaluation::obligations::VerdictChangesQuery + crate::observation::obligations::SamplesQuery + crate::observation::obligations::SeriesSamplesQuery + crate::rollup::obligations::RollupsQuery + crate::rollup::obligations::SeriesRollupsQuery + crate::transition::obligations::SeriesTransitionsQuery + crate::transition::obligations::TransitionsQuery,
{
    match system.thalamus_service.registries() {
        Ok(rows) => {
            let mut body = String::from("{");
            json::member(&mut body, "rows");
            body.push('[');
            for (position, row) in rows.iter().enumerate() {
                if position > 0 {
                    body.push(',');
                }
                wire::encode_view_thalamus_dimension_registries(row, &mut body);
            }
            body.push(']');
            body.push('}');
            Ok((200, body))
        }
        Err(unmet) => Err(entry::Refused::Unmet(format!("{unmet}"))),
    }
}

/// `POST` `thalamus.evaluation.RecordEvaluation`: reads the declared input, runs the port, answers the declared outcome.
fn serve_thalamus_evaluation_record_evaluation<ThalamusServiceBehaviors>(system: &mut crate::system::System<ThalamusServiceBehaviors>, body: &[u8]) -> http::Response
where
    ThalamusServiceBehaviors: crate::assertion::obligations::AnswerAssertionBehavior + crate::assertion::obligations::AskAssertionBehavior + crate::definition::obligations::ActivateDefinitionBehavior + crate::definition::obligations::MuteDefinitionBehavior + crate::definition::obligations::ProposeDefinitionBehavior + crate::definition::obligations::RetireDefinitionBehavior + crate::definition::obligations::ReviseDefinitionBehavior + crate::definition::obligations::UnmuteDefinitionBehavior + crate::dimension::obligations::DeclareRegistryBehavior + crate::dimension::obligations::RegisterDimensionBehavior + crate::evaluation::obligations::RecordEvaluationBehavior + crate::observation::obligations::RecordSampleBehavior + crate::rollup::obligations::RecordRollupBehavior + crate::transition::obligations::RecordTransitionBehavior + crate::assertion::obligations::AssertionsQuery + crate::assertion::obligations::OpenAssertionsQuery + crate::definition::obligations::ActiveDefinitionsQuery + crate::definition::obligations::DefinitionsQuery + crate::dimension::obligations::DimensionsQuery + crate::dimension::obligations::RegistriesQuery + crate::evaluation::obligations::EvaluationsQuery + crate::evaluation::obligations::SeriesVerdictsQuery + crate::evaluation::obligations::VerdictChangesQuery + crate::observation::obligations::SamplesQuery + crate::observation::obligations::SeriesSamplesQuery + crate::rollup::obligations::RollupsQuery + crate::rollup::obligations::SeriesRollupsQuery + crate::transition::obligations::SeriesTransitionsQuery + crate::transition::obligations::TransitionsQuery,
{
    let text = match std::str::from_utf8(body) {
        Ok(text) => text,
        Err(error) => {
            return http::Response::refusal(400, &format!("the body is not UTF-8: {error}"));
        }
    };
    let value = match json::parse(text) {
        Ok(value) => value,
        Err(error) => {
            return http::Response::refusal(400, &format!("the body is not JSON: {error}"));
        }
    };
    http::answer(run_thalamus_evaluation_record_evaluation(system, &value))
}

/// `thalamus.evaluation.RecordEvaluation` from its input as a JSON value: decode, run the port, render the declared outcome.
///
/// The one path the `POST` route and [`handle`] share. A decoding failure is located under
/// `body`, as the route reports it.
fn run_thalamus_evaluation_record_evaluation<ThalamusServiceBehaviors>(system: &mut crate::system::System<ThalamusServiceBehaviors>, value: &json::Value) -> Result<(u16, String), entry::Refused>
where
    ThalamusServiceBehaviors: crate::assertion::obligations::AnswerAssertionBehavior + crate::assertion::obligations::AskAssertionBehavior + crate::definition::obligations::ActivateDefinitionBehavior + crate::definition::obligations::MuteDefinitionBehavior + crate::definition::obligations::ProposeDefinitionBehavior + crate::definition::obligations::RetireDefinitionBehavior + crate::definition::obligations::ReviseDefinitionBehavior + crate::definition::obligations::UnmuteDefinitionBehavior + crate::dimension::obligations::DeclareRegistryBehavior + crate::dimension::obligations::RegisterDimensionBehavior + crate::evaluation::obligations::RecordEvaluationBehavior + crate::observation::obligations::RecordSampleBehavior + crate::rollup::obligations::RecordRollupBehavior + crate::transition::obligations::RecordTransitionBehavior + crate::assertion::obligations::AssertionsQuery + crate::assertion::obligations::OpenAssertionsQuery + crate::definition::obligations::ActiveDefinitionsQuery + crate::definition::obligations::DefinitionsQuery + crate::dimension::obligations::DimensionsQuery + crate::dimension::obligations::RegistriesQuery + crate::evaluation::obligations::EvaluationsQuery + crate::evaluation::obligations::SeriesVerdictsQuery + crate::evaluation::obligations::VerdictChangesQuery + crate::observation::obligations::SamplesQuery + crate::observation::obligations::SeriesSamplesQuery + crate::rollup::obligations::RollupsQuery + crate::rollup::obligations::SeriesRollupsQuery + crate::transition::obligations::SeriesTransitionsQuery + crate::transition::obligations::TransitionsQuery,
{
    let input = match wire::decode_command_thalamus_evaluation_record_evaluation(value, "body") {
        Ok(input) => input,
        Err(error) => {
            // `400` and not `422`: this is a body the schema decides, which is the difference
            // between fixing a value and fixing a serialiser.
            return Err(entry::Refused::Input(format!("{error}")));
        }
    };
    let outcome = match system.thalamus_service.record_evaluation(input) {
        Ok(outcome) => outcome,
        Err(unmet) => return Err(entry::Refused::Unmet(format!("{unmet}"))),
    };
    // Deliver what this command published to every binding that reacts to it, then take it
    // off the log: a long-running server keeps nothing from one request to the next.
    let delivered = system.pump();
    let _ = system.take_published();
    if let Err(failure) = delivered {
        return Err(entry::Refused::Undelivered(format!("delivering what the command published: {failure}")));
    }
    Ok(answer_thalamus_evaluation_record_evaluation(&outcome))
}

/// One declared outcome of `thalamus.evaluation.RecordEvaluation`: the branch that was taken, every event it published in
/// publication order, the declared error where there is one, and that error's own payload —
/// with the status the contract declares for that branch.
fn answer_thalamus_evaluation_record_evaluation(outcome: &crate::evaluation::RecordEvaluationOutcome) -> (u16, String) {
    let mut body = String::from("{");
    let status = match outcome {
        crate::evaluation::RecordEvaluationOutcome::UnknownSample { error, .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "unknown-sample");
            json::member(&mut body, "published");
            body.push('[');
            body.push(']');
            json::member(&mut body, "error");
            json::push_text(&mut body, "thalamus.evaluation.UnknownSample");
            json::member(&mut body, "payload");
            wire::encode_error_thalamus_evaluation_unknown_sample(error, &mut body);
            409
        }
        crate::evaluation::RecordEvaluationOutcome::ReasonMissing { error, .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "reason-missing");
            json::member(&mut body, "published");
            body.push('[');
            body.push(']');
            json::member(&mut body, "error");
            json::push_text(&mut body, "thalamus.evaluation.ReasonRequired");
            json::member(&mut body, "payload");
            wire::encode_error_thalamus_evaluation_reason_required(error, &mut body);
            422
        }
        crate::evaluation::RecordEvaluationOutcome::ScoreNotAdmitted { error, .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "score-not-admitted");
            json::member(&mut body, "published");
            body.push('[');
            body.push(']');
            json::member(&mut body, "error");
            json::push_text(&mut body, "thalamus.evaluation.ScoreNotAdmitted");
            json::member(&mut body, "payload");
            wire::encode_error_thalamus_evaluation_score_not_admitted(error, &mut body);
            422
        }
        crate::evaluation::RecordEvaluationOutcome::Recorded { response, evaluation_recorded, .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "recorded");
            json::member(&mut body, "published");
            body.push('[');
            body.push('{');
            json::member(&mut body, "event");
            json::push_text(&mut body, "thalamus.evaluation.EvaluationRecorded");
            json::member(&mut body, "payload");
            wire::encode_event_thalamus_evaluation_evaluation_recorded(evaluation_recorded, &mut body);
            body.push('}');
            body.push(']');
            json::member(&mut body, "response");
            wire::encode_response_thalamus_evaluation_record_evaluation(response, &mut body);
            200
        }
    };
    body.push('}');
    (status, body)
}

/// `GET` `thalamus.evaluation.Evaluations` at `read_your_writes` consistency: every row the owed projection holds.
///
/// The one path the `GET` route and [`handle`] share.
fn run_thalamus_evaluation_evaluations<ThalamusServiceBehaviors>(system: &crate::system::System<ThalamusServiceBehaviors>) -> Result<(u16, String), entry::Refused>
where
    ThalamusServiceBehaviors: crate::assertion::obligations::AnswerAssertionBehavior + crate::assertion::obligations::AskAssertionBehavior + crate::definition::obligations::ActivateDefinitionBehavior + crate::definition::obligations::MuteDefinitionBehavior + crate::definition::obligations::ProposeDefinitionBehavior + crate::definition::obligations::RetireDefinitionBehavior + crate::definition::obligations::ReviseDefinitionBehavior + crate::definition::obligations::UnmuteDefinitionBehavior + crate::dimension::obligations::DeclareRegistryBehavior + crate::dimension::obligations::RegisterDimensionBehavior + crate::evaluation::obligations::RecordEvaluationBehavior + crate::observation::obligations::RecordSampleBehavior + crate::rollup::obligations::RecordRollupBehavior + crate::transition::obligations::RecordTransitionBehavior + crate::assertion::obligations::AssertionsQuery + crate::assertion::obligations::OpenAssertionsQuery + crate::definition::obligations::ActiveDefinitionsQuery + crate::definition::obligations::DefinitionsQuery + crate::dimension::obligations::DimensionsQuery + crate::dimension::obligations::RegistriesQuery + crate::evaluation::obligations::EvaluationsQuery + crate::evaluation::obligations::SeriesVerdictsQuery + crate::evaluation::obligations::VerdictChangesQuery + crate::observation::obligations::SamplesQuery + crate::observation::obligations::SeriesSamplesQuery + crate::rollup::obligations::RollupsQuery + crate::rollup::obligations::SeriesRollupsQuery + crate::transition::obligations::SeriesTransitionsQuery + crate::transition::obligations::TransitionsQuery,
{
    match system.thalamus_service.evaluations() {
        Ok(rows) => {
            let mut body = String::from("{");
            json::member(&mut body, "rows");
            body.push('[');
            for (position, row) in rows.iter().enumerate() {
                if position > 0 {
                    body.push(',');
                }
                wire::encode_view_thalamus_evaluation_evaluations(row, &mut body);
            }
            body.push(']');
            body.push('}');
            Ok((200, body))
        }
        Err(unmet) => Err(entry::Refused::Unmet(format!("{unmet}"))),
    }
}

/// `GET` `thalamus.evaluation.VerdictChanges` at `read_your_writes` consistency: every row the owed projection holds.
///
/// The one path the `GET` route and [`handle`] share.
fn run_thalamus_evaluation_verdict_changes<ThalamusServiceBehaviors>(system: &crate::system::System<ThalamusServiceBehaviors>) -> Result<(u16, String), entry::Refused>
where
    ThalamusServiceBehaviors: crate::assertion::obligations::AnswerAssertionBehavior + crate::assertion::obligations::AskAssertionBehavior + crate::definition::obligations::ActivateDefinitionBehavior + crate::definition::obligations::MuteDefinitionBehavior + crate::definition::obligations::ProposeDefinitionBehavior + crate::definition::obligations::RetireDefinitionBehavior + crate::definition::obligations::ReviseDefinitionBehavior + crate::definition::obligations::UnmuteDefinitionBehavior + crate::dimension::obligations::DeclareRegistryBehavior + crate::dimension::obligations::RegisterDimensionBehavior + crate::evaluation::obligations::RecordEvaluationBehavior + crate::observation::obligations::RecordSampleBehavior + crate::rollup::obligations::RecordRollupBehavior + crate::transition::obligations::RecordTransitionBehavior + crate::assertion::obligations::AssertionsQuery + crate::assertion::obligations::OpenAssertionsQuery + crate::definition::obligations::ActiveDefinitionsQuery + crate::definition::obligations::DefinitionsQuery + crate::dimension::obligations::DimensionsQuery + crate::dimension::obligations::RegistriesQuery + crate::evaluation::obligations::EvaluationsQuery + crate::evaluation::obligations::SeriesVerdictsQuery + crate::evaluation::obligations::VerdictChangesQuery + crate::observation::obligations::SamplesQuery + crate::observation::obligations::SeriesSamplesQuery + crate::rollup::obligations::RollupsQuery + crate::rollup::obligations::SeriesRollupsQuery + crate::transition::obligations::SeriesTransitionsQuery + crate::transition::obligations::TransitionsQuery,
{
    match system.thalamus_service.verdict_changes() {
        Ok(rows) => {
            let mut body = String::from("{");
            json::member(&mut body, "rows");
            body.push('[');
            for (position, row) in rows.iter().enumerate() {
                if position > 0 {
                    body.push(',');
                }
                wire::encode_view_thalamus_evaluation_verdict_changes(row, &mut body);
            }
            body.push(']');
            body.push('}');
            Ok((200, body))
        }
        Err(unmet) => Err(entry::Refused::Unmet(format!("{unmet}"))),
    }
}

/// `GET` `thalamus.evaluation.SeriesVerdicts`: reads the declared parameters from the query string by their wire names, runs the
/// port, answers its rows. A key the view does not declare is ignored.
fn serve_thalamus_evaluation_series_verdicts<ThalamusServiceBehaviors>(system: &crate::system::System<ThalamusServiceBehaviors>, query: &str) -> http::Response
where
    ThalamusServiceBehaviors: crate::assertion::obligations::AnswerAssertionBehavior + crate::assertion::obligations::AskAssertionBehavior + crate::definition::obligations::ActivateDefinitionBehavior + crate::definition::obligations::MuteDefinitionBehavior + crate::definition::obligations::ProposeDefinitionBehavior + crate::definition::obligations::RetireDefinitionBehavior + crate::definition::obligations::ReviseDefinitionBehavior + crate::definition::obligations::UnmuteDefinitionBehavior + crate::dimension::obligations::DeclareRegistryBehavior + crate::dimension::obligations::RegisterDimensionBehavior + crate::evaluation::obligations::RecordEvaluationBehavior + crate::observation::obligations::RecordSampleBehavior + crate::rollup::obligations::RecordRollupBehavior + crate::transition::obligations::RecordTransitionBehavior + crate::assertion::obligations::AssertionsQuery + crate::assertion::obligations::OpenAssertionsQuery + crate::definition::obligations::ActiveDefinitionsQuery + crate::definition::obligations::DefinitionsQuery + crate::dimension::obligations::DimensionsQuery + crate::dimension::obligations::RegistriesQuery + crate::evaluation::obligations::EvaluationsQuery + crate::evaluation::obligations::SeriesVerdictsQuery + crate::evaluation::obligations::VerdictChangesQuery + crate::observation::obligations::SamplesQuery + crate::observation::obligations::SeriesSamplesQuery + crate::rollup::obligations::RollupsQuery + crate::rollup::obligations::SeriesRollupsQuery + crate::transition::obligations::SeriesTransitionsQuery + crate::transition::obligations::TransitionsQuery,
{
    match http::query_object(query, &[("series", http::Scalar::Text)], "query") {
        Ok(value) => http::answer(run_thalamus_evaluation_series_verdicts(system, &value)),
        Err(refused) => http::Response::refusal(400, &refused),
    }
}

/// `GET` `thalamus.evaluation.SeriesVerdicts` at `read_your_writes` consistency: every row the owed projection holds for the declared
/// parameters, read from `value`, an object keyed by their wire names.
///
/// The one path the `GET` route and [`handle`] share. A decoding failure is located under
/// `query`, as the route reports it.
fn run_thalamus_evaluation_series_verdicts<ThalamusServiceBehaviors>(system: &crate::system::System<ThalamusServiceBehaviors>, value: &json::Value) -> Result<(u16, String), entry::Refused>
where
    ThalamusServiceBehaviors: crate::assertion::obligations::AnswerAssertionBehavior + crate::assertion::obligations::AskAssertionBehavior + crate::definition::obligations::ActivateDefinitionBehavior + crate::definition::obligations::MuteDefinitionBehavior + crate::definition::obligations::ProposeDefinitionBehavior + crate::definition::obligations::RetireDefinitionBehavior + crate::definition::obligations::ReviseDefinitionBehavior + crate::definition::obligations::UnmuteDefinitionBehavior + crate::dimension::obligations::DeclareRegistryBehavior + crate::dimension::obligations::RegisterDimensionBehavior + crate::evaluation::obligations::RecordEvaluationBehavior + crate::observation::obligations::RecordSampleBehavior + crate::rollup::obligations::RecordRollupBehavior + crate::transition::obligations::RecordTransitionBehavior + crate::assertion::obligations::AssertionsQuery + crate::assertion::obligations::OpenAssertionsQuery + crate::definition::obligations::ActiveDefinitionsQuery + crate::definition::obligations::DefinitionsQuery + crate::dimension::obligations::DimensionsQuery + crate::dimension::obligations::RegistriesQuery + crate::evaluation::obligations::EvaluationsQuery + crate::evaluation::obligations::SeriesVerdictsQuery + crate::evaluation::obligations::VerdictChangesQuery + crate::observation::obligations::SamplesQuery + crate::observation::obligations::SeriesSamplesQuery + crate::rollup::obligations::RollupsQuery + crate::rollup::obligations::SeriesRollupsQuery + crate::transition::obligations::SeriesTransitionsQuery + crate::transition::obligations::TransitionsQuery,
{
    let (param0,) = match wire::decode_params_thalamus_evaluation_series_verdicts(value, "query") {
        Ok(params) => params,
        Err(error) => return Err(entry::Refused::Input(format!("{error}"))),
    };
    match system.thalamus_service.series_verdicts(param0) {
        Ok(rows) => {
            let mut body = String::from("{");
            json::member(&mut body, "rows");
            body.push('[');
            for (position, row) in rows.iter().enumerate() {
                if position > 0 {
                    body.push(',');
                }
                wire::encode_view_thalamus_evaluation_series_verdicts(row, &mut body);
            }
            body.push(']');
            body.push('}');
            Ok((200, body))
        }
        Err(unmet) => Err(entry::Refused::Unmet(format!("{unmet}"))),
    }
}

/// `POST` `thalamus.rollup.RecordRollup`: reads the declared input, runs the port, answers the declared outcome.
fn serve_thalamus_rollup_record_rollup<ThalamusServiceBehaviors>(system: &mut crate::system::System<ThalamusServiceBehaviors>, body: &[u8]) -> http::Response
where
    ThalamusServiceBehaviors: crate::assertion::obligations::AnswerAssertionBehavior + crate::assertion::obligations::AskAssertionBehavior + crate::definition::obligations::ActivateDefinitionBehavior + crate::definition::obligations::MuteDefinitionBehavior + crate::definition::obligations::ProposeDefinitionBehavior + crate::definition::obligations::RetireDefinitionBehavior + crate::definition::obligations::ReviseDefinitionBehavior + crate::definition::obligations::UnmuteDefinitionBehavior + crate::dimension::obligations::DeclareRegistryBehavior + crate::dimension::obligations::RegisterDimensionBehavior + crate::evaluation::obligations::RecordEvaluationBehavior + crate::observation::obligations::RecordSampleBehavior + crate::rollup::obligations::RecordRollupBehavior + crate::transition::obligations::RecordTransitionBehavior + crate::assertion::obligations::AssertionsQuery + crate::assertion::obligations::OpenAssertionsQuery + crate::definition::obligations::ActiveDefinitionsQuery + crate::definition::obligations::DefinitionsQuery + crate::dimension::obligations::DimensionsQuery + crate::dimension::obligations::RegistriesQuery + crate::evaluation::obligations::EvaluationsQuery + crate::evaluation::obligations::SeriesVerdictsQuery + crate::evaluation::obligations::VerdictChangesQuery + crate::observation::obligations::SamplesQuery + crate::observation::obligations::SeriesSamplesQuery + crate::rollup::obligations::RollupsQuery + crate::rollup::obligations::SeriesRollupsQuery + crate::transition::obligations::SeriesTransitionsQuery + crate::transition::obligations::TransitionsQuery,
{
    let text = match std::str::from_utf8(body) {
        Ok(text) => text,
        Err(error) => {
            return http::Response::refusal(400, &format!("the body is not UTF-8: {error}"));
        }
    };
    let value = match json::parse(text) {
        Ok(value) => value,
        Err(error) => {
            return http::Response::refusal(400, &format!("the body is not JSON: {error}"));
        }
    };
    http::answer(run_thalamus_rollup_record_rollup(system, &value))
}

/// `thalamus.rollup.RecordRollup` from its input as a JSON value: decode, run the port, render the declared outcome.
///
/// The one path the `POST` route and [`handle`] share. A decoding failure is located under
/// `body`, as the route reports it.
fn run_thalamus_rollup_record_rollup<ThalamusServiceBehaviors>(system: &mut crate::system::System<ThalamusServiceBehaviors>, value: &json::Value) -> Result<(u16, String), entry::Refused>
where
    ThalamusServiceBehaviors: crate::assertion::obligations::AnswerAssertionBehavior + crate::assertion::obligations::AskAssertionBehavior + crate::definition::obligations::ActivateDefinitionBehavior + crate::definition::obligations::MuteDefinitionBehavior + crate::definition::obligations::ProposeDefinitionBehavior + crate::definition::obligations::RetireDefinitionBehavior + crate::definition::obligations::ReviseDefinitionBehavior + crate::definition::obligations::UnmuteDefinitionBehavior + crate::dimension::obligations::DeclareRegistryBehavior + crate::dimension::obligations::RegisterDimensionBehavior + crate::evaluation::obligations::RecordEvaluationBehavior + crate::observation::obligations::RecordSampleBehavior + crate::rollup::obligations::RecordRollupBehavior + crate::transition::obligations::RecordTransitionBehavior + crate::assertion::obligations::AssertionsQuery + crate::assertion::obligations::OpenAssertionsQuery + crate::definition::obligations::ActiveDefinitionsQuery + crate::definition::obligations::DefinitionsQuery + crate::dimension::obligations::DimensionsQuery + crate::dimension::obligations::RegistriesQuery + crate::evaluation::obligations::EvaluationsQuery + crate::evaluation::obligations::SeriesVerdictsQuery + crate::evaluation::obligations::VerdictChangesQuery + crate::observation::obligations::SamplesQuery + crate::observation::obligations::SeriesSamplesQuery + crate::rollup::obligations::RollupsQuery + crate::rollup::obligations::SeriesRollupsQuery + crate::transition::obligations::SeriesTransitionsQuery + crate::transition::obligations::TransitionsQuery,
{
    let input = match wire::decode_command_thalamus_rollup_record_rollup(value, "body") {
        Ok(input) => input,
        Err(error) => {
            // `400` and not `422`: this is a body the schema decides, which is the difference
            // between fixing a value and fixing a serialiser.
            return Err(entry::Refused::Input(format!("{error}")));
        }
    };
    let outcome = match system.thalamus_service.record_rollup(input) {
        Ok(outcome) => outcome,
        Err(unmet) => return Err(entry::Refused::Unmet(format!("{unmet}"))),
    };
    // Deliver what this command published to every binding that reacts to it, then take it
    // off the log: a long-running server keeps nothing from one request to the next.
    let delivered = system.pump();
    let _ = system.take_published();
    if let Err(failure) = delivered {
        return Err(entry::Refused::Undelivered(format!("delivering what the command published: {failure}")));
    }
    Ok(answer_thalamus_rollup_record_rollup(&outcome))
}

/// One declared outcome of `thalamus.rollup.RecordRollup`: the branch that was taken, every event it published in
/// publication order, the declared error where there is one, and that error's own payload —
/// with the status the contract declares for that branch.
fn answer_thalamus_rollup_record_rollup(outcome: &crate::rollup::RecordRollupOutcome) -> (u16, String) {
    let mut body = String::from("{");
    let status = match outcome {
        crate::rollup::RecordRollupOutcome::UnknownSignal { error, .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "unknown-signal");
            json::member(&mut body, "published");
            body.push('[');
            body.push(']');
            json::member(&mut body, "error");
            json::push_text(&mut body, "thalamus.rollup.UnknownSignal");
            json::member(&mut body, "payload");
            wire::encode_error_thalamus_rollup_unknown_signal(error, &mut body);
            409
        }
        crate::rollup::RecordRollupOutcome::Recorded { rollup_recorded, .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "recorded");
            json::member(&mut body, "published");
            body.push('[');
            body.push('{');
            json::member(&mut body, "event");
            json::push_text(&mut body, "thalamus.rollup.RollupRecorded");
            json::member(&mut body, "payload");
            wire::encode_event_thalamus_rollup_rollup_recorded(rollup_recorded, &mut body);
            body.push('}');
            body.push(']');
            202
        }
        crate::rollup::RecordRollupOutcome::AlreadyRolledUp { error, .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "already-rolled-up");
            json::member(&mut body, "published");
            body.push('[');
            body.push(']');
            json::member(&mut body, "error");
            json::push_text(&mut body, "thalamus.rollup.RollupExists");
            json::member(&mut body, "payload");
            wire::encode_error_thalamus_rollup_rollup_exists(error, &mut body);
            409
        }
    };
    body.push('}');
    (status, body)
}

/// `GET` `thalamus.rollup.Rollups` at `read_your_writes` consistency: every row the owed projection holds.
///
/// The one path the `GET` route and [`handle`] share.
fn run_thalamus_rollup_rollups<ThalamusServiceBehaviors>(system: &crate::system::System<ThalamusServiceBehaviors>) -> Result<(u16, String), entry::Refused>
where
    ThalamusServiceBehaviors: crate::assertion::obligations::AnswerAssertionBehavior + crate::assertion::obligations::AskAssertionBehavior + crate::definition::obligations::ActivateDefinitionBehavior + crate::definition::obligations::MuteDefinitionBehavior + crate::definition::obligations::ProposeDefinitionBehavior + crate::definition::obligations::RetireDefinitionBehavior + crate::definition::obligations::ReviseDefinitionBehavior + crate::definition::obligations::UnmuteDefinitionBehavior + crate::dimension::obligations::DeclareRegistryBehavior + crate::dimension::obligations::RegisterDimensionBehavior + crate::evaluation::obligations::RecordEvaluationBehavior + crate::observation::obligations::RecordSampleBehavior + crate::rollup::obligations::RecordRollupBehavior + crate::transition::obligations::RecordTransitionBehavior + crate::assertion::obligations::AssertionsQuery + crate::assertion::obligations::OpenAssertionsQuery + crate::definition::obligations::ActiveDefinitionsQuery + crate::definition::obligations::DefinitionsQuery + crate::dimension::obligations::DimensionsQuery + crate::dimension::obligations::RegistriesQuery + crate::evaluation::obligations::EvaluationsQuery + crate::evaluation::obligations::SeriesVerdictsQuery + crate::evaluation::obligations::VerdictChangesQuery + crate::observation::obligations::SamplesQuery + crate::observation::obligations::SeriesSamplesQuery + crate::rollup::obligations::RollupsQuery + crate::rollup::obligations::SeriesRollupsQuery + crate::transition::obligations::SeriesTransitionsQuery + crate::transition::obligations::TransitionsQuery,
{
    match system.thalamus_service.rollups() {
        Ok(rows) => {
            let mut body = String::from("{");
            json::member(&mut body, "rows");
            body.push('[');
            for (position, row) in rows.iter().enumerate() {
                if position > 0 {
                    body.push(',');
                }
                wire::encode_view_thalamus_rollup_rollups(row, &mut body);
            }
            body.push(']');
            body.push('}');
            Ok((200, body))
        }
        Err(unmet) => Err(entry::Refused::Unmet(format!("{unmet}"))),
    }
}

/// `GET` `thalamus.rollup.SeriesRollups`: reads the declared parameters from the query string by their wire names, runs the
/// port, answers its rows. A key the view does not declare is ignored.
fn serve_thalamus_rollup_series_rollups<ThalamusServiceBehaviors>(system: &crate::system::System<ThalamusServiceBehaviors>, query: &str) -> http::Response
where
    ThalamusServiceBehaviors: crate::assertion::obligations::AnswerAssertionBehavior + crate::assertion::obligations::AskAssertionBehavior + crate::definition::obligations::ActivateDefinitionBehavior + crate::definition::obligations::MuteDefinitionBehavior + crate::definition::obligations::ProposeDefinitionBehavior + crate::definition::obligations::RetireDefinitionBehavior + crate::definition::obligations::ReviseDefinitionBehavior + crate::definition::obligations::UnmuteDefinitionBehavior + crate::dimension::obligations::DeclareRegistryBehavior + crate::dimension::obligations::RegisterDimensionBehavior + crate::evaluation::obligations::RecordEvaluationBehavior + crate::observation::obligations::RecordSampleBehavior + crate::rollup::obligations::RecordRollupBehavior + crate::transition::obligations::RecordTransitionBehavior + crate::assertion::obligations::AssertionsQuery + crate::assertion::obligations::OpenAssertionsQuery + crate::definition::obligations::ActiveDefinitionsQuery + crate::definition::obligations::DefinitionsQuery + crate::dimension::obligations::DimensionsQuery + crate::dimension::obligations::RegistriesQuery + crate::evaluation::obligations::EvaluationsQuery + crate::evaluation::obligations::SeriesVerdictsQuery + crate::evaluation::obligations::VerdictChangesQuery + crate::observation::obligations::SamplesQuery + crate::observation::obligations::SeriesSamplesQuery + crate::rollup::obligations::RollupsQuery + crate::rollup::obligations::SeriesRollupsQuery + crate::transition::obligations::SeriesTransitionsQuery + crate::transition::obligations::TransitionsQuery,
{
    match http::query_object(query, &[("series", http::Scalar::Text)], "query") {
        Ok(value) => http::answer(run_thalamus_rollup_series_rollups(system, &value)),
        Err(refused) => http::Response::refusal(400, &refused),
    }
}

/// `GET` `thalamus.rollup.SeriesRollups` at `read_your_writes` consistency: every row the owed projection holds for the declared
/// parameters, read from `value`, an object keyed by their wire names.
///
/// The one path the `GET` route and [`handle`] share. A decoding failure is located under
/// `query`, as the route reports it.
fn run_thalamus_rollup_series_rollups<ThalamusServiceBehaviors>(system: &crate::system::System<ThalamusServiceBehaviors>, value: &json::Value) -> Result<(u16, String), entry::Refused>
where
    ThalamusServiceBehaviors: crate::assertion::obligations::AnswerAssertionBehavior + crate::assertion::obligations::AskAssertionBehavior + crate::definition::obligations::ActivateDefinitionBehavior + crate::definition::obligations::MuteDefinitionBehavior + crate::definition::obligations::ProposeDefinitionBehavior + crate::definition::obligations::RetireDefinitionBehavior + crate::definition::obligations::ReviseDefinitionBehavior + crate::definition::obligations::UnmuteDefinitionBehavior + crate::dimension::obligations::DeclareRegistryBehavior + crate::dimension::obligations::RegisterDimensionBehavior + crate::evaluation::obligations::RecordEvaluationBehavior + crate::observation::obligations::RecordSampleBehavior + crate::rollup::obligations::RecordRollupBehavior + crate::transition::obligations::RecordTransitionBehavior + crate::assertion::obligations::AssertionsQuery + crate::assertion::obligations::OpenAssertionsQuery + crate::definition::obligations::ActiveDefinitionsQuery + crate::definition::obligations::DefinitionsQuery + crate::dimension::obligations::DimensionsQuery + crate::dimension::obligations::RegistriesQuery + crate::evaluation::obligations::EvaluationsQuery + crate::evaluation::obligations::SeriesVerdictsQuery + crate::evaluation::obligations::VerdictChangesQuery + crate::observation::obligations::SamplesQuery + crate::observation::obligations::SeriesSamplesQuery + crate::rollup::obligations::RollupsQuery + crate::rollup::obligations::SeriesRollupsQuery + crate::transition::obligations::SeriesTransitionsQuery + crate::transition::obligations::TransitionsQuery,
{
    let (param0,) = match wire::decode_params_thalamus_rollup_series_rollups(value, "query") {
        Ok(params) => params,
        Err(error) => return Err(entry::Refused::Input(format!("{error}"))),
    };
    match system.thalamus_service.series_rollups(param0) {
        Ok(rows) => {
            let mut body = String::from("{");
            json::member(&mut body, "rows");
            body.push('[');
            for (position, row) in rows.iter().enumerate() {
                if position > 0 {
                    body.push(',');
                }
                wire::encode_view_thalamus_rollup_series_rollups(row, &mut body);
            }
            body.push(']');
            body.push('}');
            Ok((200, body))
        }
        Err(unmet) => Err(entry::Refused::Unmet(format!("{unmet}"))),
    }
}

/// `POST` `thalamus.observation.RecordSample`: reads the declared input, runs the port, answers the declared outcome.
fn serve_thalamus_observation_record_sample<ThalamusServiceBehaviors>(system: &mut crate::system::System<ThalamusServiceBehaviors>, body: &[u8]) -> http::Response
where
    ThalamusServiceBehaviors: crate::assertion::obligations::AnswerAssertionBehavior + crate::assertion::obligations::AskAssertionBehavior + crate::definition::obligations::ActivateDefinitionBehavior + crate::definition::obligations::MuteDefinitionBehavior + crate::definition::obligations::ProposeDefinitionBehavior + crate::definition::obligations::RetireDefinitionBehavior + crate::definition::obligations::ReviseDefinitionBehavior + crate::definition::obligations::UnmuteDefinitionBehavior + crate::dimension::obligations::DeclareRegistryBehavior + crate::dimension::obligations::RegisterDimensionBehavior + crate::evaluation::obligations::RecordEvaluationBehavior + crate::observation::obligations::RecordSampleBehavior + crate::rollup::obligations::RecordRollupBehavior + crate::transition::obligations::RecordTransitionBehavior + crate::assertion::obligations::AssertionsQuery + crate::assertion::obligations::OpenAssertionsQuery + crate::definition::obligations::ActiveDefinitionsQuery + crate::definition::obligations::DefinitionsQuery + crate::dimension::obligations::DimensionsQuery + crate::dimension::obligations::RegistriesQuery + crate::evaluation::obligations::EvaluationsQuery + crate::evaluation::obligations::SeriesVerdictsQuery + crate::evaluation::obligations::VerdictChangesQuery + crate::observation::obligations::SamplesQuery + crate::observation::obligations::SeriesSamplesQuery + crate::rollup::obligations::RollupsQuery + crate::rollup::obligations::SeriesRollupsQuery + crate::transition::obligations::SeriesTransitionsQuery + crate::transition::obligations::TransitionsQuery,
{
    let text = match std::str::from_utf8(body) {
        Ok(text) => text,
        Err(error) => {
            return http::Response::refusal(400, &format!("the body is not UTF-8: {error}"));
        }
    };
    let value = match json::parse(text) {
        Ok(value) => value,
        Err(error) => {
            return http::Response::refusal(400, &format!("the body is not JSON: {error}"));
        }
    };
    http::answer(run_thalamus_observation_record_sample(system, &value))
}

/// `thalamus.observation.RecordSample` from its input as a JSON value: decode, run the port, render the declared outcome.
///
/// The one path the `POST` route and [`handle`] share. A decoding failure is located under
/// `body`, as the route reports it.
fn run_thalamus_observation_record_sample<ThalamusServiceBehaviors>(system: &mut crate::system::System<ThalamusServiceBehaviors>, value: &json::Value) -> Result<(u16, String), entry::Refused>
where
    ThalamusServiceBehaviors: crate::assertion::obligations::AnswerAssertionBehavior + crate::assertion::obligations::AskAssertionBehavior + crate::definition::obligations::ActivateDefinitionBehavior + crate::definition::obligations::MuteDefinitionBehavior + crate::definition::obligations::ProposeDefinitionBehavior + crate::definition::obligations::RetireDefinitionBehavior + crate::definition::obligations::ReviseDefinitionBehavior + crate::definition::obligations::UnmuteDefinitionBehavior + crate::dimension::obligations::DeclareRegistryBehavior + crate::dimension::obligations::RegisterDimensionBehavior + crate::evaluation::obligations::RecordEvaluationBehavior + crate::observation::obligations::RecordSampleBehavior + crate::rollup::obligations::RecordRollupBehavior + crate::transition::obligations::RecordTransitionBehavior + crate::assertion::obligations::AssertionsQuery + crate::assertion::obligations::OpenAssertionsQuery + crate::definition::obligations::ActiveDefinitionsQuery + crate::definition::obligations::DefinitionsQuery + crate::dimension::obligations::DimensionsQuery + crate::dimension::obligations::RegistriesQuery + crate::evaluation::obligations::EvaluationsQuery + crate::evaluation::obligations::SeriesVerdictsQuery + crate::evaluation::obligations::VerdictChangesQuery + crate::observation::obligations::SamplesQuery + crate::observation::obligations::SeriesSamplesQuery + crate::rollup::obligations::RollupsQuery + crate::rollup::obligations::SeriesRollupsQuery + crate::transition::obligations::SeriesTransitionsQuery + crate::transition::obligations::TransitionsQuery,
{
    let input = match wire::decode_command_thalamus_observation_record_sample(value, "body") {
        Ok(input) => input,
        Err(error) => {
            // `400` and not `422`: this is a body the schema decides, which is the difference
            // between fixing a value and fixing a serialiser.
            return Err(entry::Refused::Input(format!("{error}")));
        }
    };
    let outcome = match system.thalamus_service.record_sample(input) {
        Ok(outcome) => outcome,
        Err(unmet) => return Err(entry::Refused::Unmet(format!("{unmet}"))),
    };
    // Deliver what this command published to every binding that reacts to it, then take it
    // off the log: a long-running server keeps nothing from one request to the next.
    let delivered = system.pump();
    let _ = system.take_published();
    if let Err(failure) = delivered {
        return Err(entry::Refused::Undelivered(format!("delivering what the command published: {failure}")));
    }
    Ok(answer_thalamus_observation_record_sample(&outcome))
}

/// One declared outcome of `thalamus.observation.RecordSample`: the branch that was taken, every event it published in
/// publication order, the declared error where there is one, and that error's own payload —
/// with the status the contract declares for that branch.
fn answer_thalamus_observation_record_sample(outcome: &crate::observation::RecordSampleOutcome) -> (u16, String) {
    let mut body = String::from("{");
    let status = match outcome {
        crate::observation::RecordSampleOutcome::UnknownSignal { error, .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "unknown-signal");
            json::member(&mut body, "published");
            body.push('[');
            body.push(']');
            json::member(&mut body, "error");
            json::push_text(&mut body, "thalamus.observation.UnknownSignal");
            json::member(&mut body, "payload");
            wire::encode_error_thalamus_observation_unknown_signal(error, &mut body);
            409
        }
        crate::observation::RecordSampleOutcome::NotActive { error, .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "not-active");
            json::member(&mut body, "published");
            body.push('[');
            body.push(']');
            json::member(&mut body, "error");
            json::push_text(&mut body, "thalamus.observation.SignalNotActive");
            json::member(&mut body, "payload");
            wire::encode_error_thalamus_observation_signal_not_active(error, &mut body);
            409
        }
        crate::observation::RecordSampleOutcome::Recorded { response, sample_recorded, .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "recorded");
            json::member(&mut body, "published");
            body.push('[');
            body.push('{');
            json::member(&mut body, "event");
            json::push_text(&mut body, "thalamus.observation.SampleRecorded");
            json::member(&mut body, "payload");
            wire::encode_event_thalamus_observation_sample_recorded(sample_recorded, &mut body);
            body.push('}');
            body.push(']');
            json::member(&mut body, "response");
            wire::encode_response_thalamus_observation_record_sample(response, &mut body);
            200
        }
    };
    body.push('}');
    (status, body)
}

/// `GET` `thalamus.observation.Samples` at `read_your_writes` consistency: every row the owed projection holds.
///
/// The one path the `GET` route and [`handle`] share.
fn run_thalamus_observation_samples<ThalamusServiceBehaviors>(system: &crate::system::System<ThalamusServiceBehaviors>) -> Result<(u16, String), entry::Refused>
where
    ThalamusServiceBehaviors: crate::assertion::obligations::AnswerAssertionBehavior + crate::assertion::obligations::AskAssertionBehavior + crate::definition::obligations::ActivateDefinitionBehavior + crate::definition::obligations::MuteDefinitionBehavior + crate::definition::obligations::ProposeDefinitionBehavior + crate::definition::obligations::RetireDefinitionBehavior + crate::definition::obligations::ReviseDefinitionBehavior + crate::definition::obligations::UnmuteDefinitionBehavior + crate::dimension::obligations::DeclareRegistryBehavior + crate::dimension::obligations::RegisterDimensionBehavior + crate::evaluation::obligations::RecordEvaluationBehavior + crate::observation::obligations::RecordSampleBehavior + crate::rollup::obligations::RecordRollupBehavior + crate::transition::obligations::RecordTransitionBehavior + crate::assertion::obligations::AssertionsQuery + crate::assertion::obligations::OpenAssertionsQuery + crate::definition::obligations::ActiveDefinitionsQuery + crate::definition::obligations::DefinitionsQuery + crate::dimension::obligations::DimensionsQuery + crate::dimension::obligations::RegistriesQuery + crate::evaluation::obligations::EvaluationsQuery + crate::evaluation::obligations::SeriesVerdictsQuery + crate::evaluation::obligations::VerdictChangesQuery + crate::observation::obligations::SamplesQuery + crate::observation::obligations::SeriesSamplesQuery + crate::rollup::obligations::RollupsQuery + crate::rollup::obligations::SeriesRollupsQuery + crate::transition::obligations::SeriesTransitionsQuery + crate::transition::obligations::TransitionsQuery,
{
    match system.thalamus_service.samples() {
        Ok(rows) => {
            let mut body = String::from("{");
            json::member(&mut body, "rows");
            body.push('[');
            for (position, row) in rows.iter().enumerate() {
                if position > 0 {
                    body.push(',');
                }
                wire::encode_view_thalamus_observation_samples(row, &mut body);
            }
            body.push(']');
            body.push('}');
            Ok((200, body))
        }
        Err(unmet) => Err(entry::Refused::Unmet(format!("{unmet}"))),
    }
}

/// `GET` `thalamus.observation.SeriesSamples`: reads the declared parameters from the query string by their wire names, runs the
/// port, answers its rows. A key the view does not declare is ignored.
fn serve_thalamus_observation_series_samples<ThalamusServiceBehaviors>(system: &crate::system::System<ThalamusServiceBehaviors>, query: &str) -> http::Response
where
    ThalamusServiceBehaviors: crate::assertion::obligations::AnswerAssertionBehavior + crate::assertion::obligations::AskAssertionBehavior + crate::definition::obligations::ActivateDefinitionBehavior + crate::definition::obligations::MuteDefinitionBehavior + crate::definition::obligations::ProposeDefinitionBehavior + crate::definition::obligations::RetireDefinitionBehavior + crate::definition::obligations::ReviseDefinitionBehavior + crate::definition::obligations::UnmuteDefinitionBehavior + crate::dimension::obligations::DeclareRegistryBehavior + crate::dimension::obligations::RegisterDimensionBehavior + crate::evaluation::obligations::RecordEvaluationBehavior + crate::observation::obligations::RecordSampleBehavior + crate::rollup::obligations::RecordRollupBehavior + crate::transition::obligations::RecordTransitionBehavior + crate::assertion::obligations::AssertionsQuery + crate::assertion::obligations::OpenAssertionsQuery + crate::definition::obligations::ActiveDefinitionsQuery + crate::definition::obligations::DefinitionsQuery + crate::dimension::obligations::DimensionsQuery + crate::dimension::obligations::RegistriesQuery + crate::evaluation::obligations::EvaluationsQuery + crate::evaluation::obligations::SeriesVerdictsQuery + crate::evaluation::obligations::VerdictChangesQuery + crate::observation::obligations::SamplesQuery + crate::observation::obligations::SeriesSamplesQuery + crate::rollup::obligations::RollupsQuery + crate::rollup::obligations::SeriesRollupsQuery + crate::transition::obligations::SeriesTransitionsQuery + crate::transition::obligations::TransitionsQuery,
{
    match http::query_object(query, &[("series", http::Scalar::Text)], "query") {
        Ok(value) => http::answer(run_thalamus_observation_series_samples(system, &value)),
        Err(refused) => http::Response::refusal(400, &refused),
    }
}

/// `GET` `thalamus.observation.SeriesSamples` at `read_your_writes` consistency: every row the owed projection holds for the declared
/// parameters, read from `value`, an object keyed by their wire names.
///
/// The one path the `GET` route and [`handle`] share. A decoding failure is located under
/// `query`, as the route reports it.
fn run_thalamus_observation_series_samples<ThalamusServiceBehaviors>(system: &crate::system::System<ThalamusServiceBehaviors>, value: &json::Value) -> Result<(u16, String), entry::Refused>
where
    ThalamusServiceBehaviors: crate::assertion::obligations::AnswerAssertionBehavior + crate::assertion::obligations::AskAssertionBehavior + crate::definition::obligations::ActivateDefinitionBehavior + crate::definition::obligations::MuteDefinitionBehavior + crate::definition::obligations::ProposeDefinitionBehavior + crate::definition::obligations::RetireDefinitionBehavior + crate::definition::obligations::ReviseDefinitionBehavior + crate::definition::obligations::UnmuteDefinitionBehavior + crate::dimension::obligations::DeclareRegistryBehavior + crate::dimension::obligations::RegisterDimensionBehavior + crate::evaluation::obligations::RecordEvaluationBehavior + crate::observation::obligations::RecordSampleBehavior + crate::rollup::obligations::RecordRollupBehavior + crate::transition::obligations::RecordTransitionBehavior + crate::assertion::obligations::AssertionsQuery + crate::assertion::obligations::OpenAssertionsQuery + crate::definition::obligations::ActiveDefinitionsQuery + crate::definition::obligations::DefinitionsQuery + crate::dimension::obligations::DimensionsQuery + crate::dimension::obligations::RegistriesQuery + crate::evaluation::obligations::EvaluationsQuery + crate::evaluation::obligations::SeriesVerdictsQuery + crate::evaluation::obligations::VerdictChangesQuery + crate::observation::obligations::SamplesQuery + crate::observation::obligations::SeriesSamplesQuery + crate::rollup::obligations::RollupsQuery + crate::rollup::obligations::SeriesRollupsQuery + crate::transition::obligations::SeriesTransitionsQuery + crate::transition::obligations::TransitionsQuery,
{
    let (param0,) = match wire::decode_params_thalamus_observation_series_samples(value, "query") {
        Ok(params) => params,
        Err(error) => return Err(entry::Refused::Input(format!("{error}"))),
    };
    match system.thalamus_service.series_samples(param0) {
        Ok(rows) => {
            let mut body = String::from("{");
            json::member(&mut body, "rows");
            body.push('[');
            for (position, row) in rows.iter().enumerate() {
                if position > 0 {
                    body.push(',');
                }
                wire::encode_view_thalamus_observation_series_samples(row, &mut body);
            }
            body.push(']');
            body.push('}');
            Ok((200, body))
        }
        Err(unmet) => Err(entry::Refused::Unmet(format!("{unmet}"))),
    }
}

/// `POST` `thalamus.transition.RecordTransition`: reads the declared input, runs the port, answers the declared outcome.
fn serve_thalamus_transition_record_transition<ThalamusServiceBehaviors>(system: &mut crate::system::System<ThalamusServiceBehaviors>, body: &[u8]) -> http::Response
where
    ThalamusServiceBehaviors: crate::assertion::obligations::AnswerAssertionBehavior + crate::assertion::obligations::AskAssertionBehavior + crate::definition::obligations::ActivateDefinitionBehavior + crate::definition::obligations::MuteDefinitionBehavior + crate::definition::obligations::ProposeDefinitionBehavior + crate::definition::obligations::RetireDefinitionBehavior + crate::definition::obligations::ReviseDefinitionBehavior + crate::definition::obligations::UnmuteDefinitionBehavior + crate::dimension::obligations::DeclareRegistryBehavior + crate::dimension::obligations::RegisterDimensionBehavior + crate::evaluation::obligations::RecordEvaluationBehavior + crate::observation::obligations::RecordSampleBehavior + crate::rollup::obligations::RecordRollupBehavior + crate::transition::obligations::RecordTransitionBehavior + crate::assertion::obligations::AssertionsQuery + crate::assertion::obligations::OpenAssertionsQuery + crate::definition::obligations::ActiveDefinitionsQuery + crate::definition::obligations::DefinitionsQuery + crate::dimension::obligations::DimensionsQuery + crate::dimension::obligations::RegistriesQuery + crate::evaluation::obligations::EvaluationsQuery + crate::evaluation::obligations::SeriesVerdictsQuery + crate::evaluation::obligations::VerdictChangesQuery + crate::observation::obligations::SamplesQuery + crate::observation::obligations::SeriesSamplesQuery + crate::rollup::obligations::RollupsQuery + crate::rollup::obligations::SeriesRollupsQuery + crate::transition::obligations::SeriesTransitionsQuery + crate::transition::obligations::TransitionsQuery,
{
    let text = match std::str::from_utf8(body) {
        Ok(text) => text,
        Err(error) => {
            return http::Response::refusal(400, &format!("the body is not UTF-8: {error}"));
        }
    };
    let value = match json::parse(text) {
        Ok(value) => value,
        Err(error) => {
            return http::Response::refusal(400, &format!("the body is not JSON: {error}"));
        }
    };
    http::answer(run_thalamus_transition_record_transition(system, &value))
}

/// `thalamus.transition.RecordTransition` from its input as a JSON value: decode, run the port, render the declared outcome.
///
/// The one path the `POST` route and [`handle`] share. A decoding failure is located under
/// `body`, as the route reports it.
fn run_thalamus_transition_record_transition<ThalamusServiceBehaviors>(system: &mut crate::system::System<ThalamusServiceBehaviors>, value: &json::Value) -> Result<(u16, String), entry::Refused>
where
    ThalamusServiceBehaviors: crate::assertion::obligations::AnswerAssertionBehavior + crate::assertion::obligations::AskAssertionBehavior + crate::definition::obligations::ActivateDefinitionBehavior + crate::definition::obligations::MuteDefinitionBehavior + crate::definition::obligations::ProposeDefinitionBehavior + crate::definition::obligations::RetireDefinitionBehavior + crate::definition::obligations::ReviseDefinitionBehavior + crate::definition::obligations::UnmuteDefinitionBehavior + crate::dimension::obligations::DeclareRegistryBehavior + crate::dimension::obligations::RegisterDimensionBehavior + crate::evaluation::obligations::RecordEvaluationBehavior + crate::observation::obligations::RecordSampleBehavior + crate::rollup::obligations::RecordRollupBehavior + crate::transition::obligations::RecordTransitionBehavior + crate::assertion::obligations::AssertionsQuery + crate::assertion::obligations::OpenAssertionsQuery + crate::definition::obligations::ActiveDefinitionsQuery + crate::definition::obligations::DefinitionsQuery + crate::dimension::obligations::DimensionsQuery + crate::dimension::obligations::RegistriesQuery + crate::evaluation::obligations::EvaluationsQuery + crate::evaluation::obligations::SeriesVerdictsQuery + crate::evaluation::obligations::VerdictChangesQuery + crate::observation::obligations::SamplesQuery + crate::observation::obligations::SeriesSamplesQuery + crate::rollup::obligations::RollupsQuery + crate::rollup::obligations::SeriesRollupsQuery + crate::transition::obligations::SeriesTransitionsQuery + crate::transition::obligations::TransitionsQuery,
{
    let input = match wire::decode_command_thalamus_transition_record_transition(value, "body") {
        Ok(input) => input,
        Err(error) => {
            // `400` and not `422`: this is a body the schema decides, which is the difference
            // between fixing a value and fixing a serialiser.
            return Err(entry::Refused::Input(format!("{error}")));
        }
    };
    let outcome = match system.thalamus_service.record_transition(input) {
        Ok(outcome) => outcome,
        Err(unmet) => return Err(entry::Refused::Unmet(format!("{unmet}"))),
    };
    // Deliver what this command published to every binding that reacts to it, then take it
    // off the log: a long-running server keeps nothing from one request to the next.
    let delivered = system.pump();
    let _ = system.take_published();
    if let Err(failure) = delivered {
        return Err(entry::Refused::Undelivered(format!("delivering what the command published: {failure}")));
    }
    Ok(answer_thalamus_transition_record_transition(&outcome))
}

/// One declared outcome of `thalamus.transition.RecordTransition`: the branch that was taken, every event it published in
/// publication order, the declared error where there is one, and that error's own payload —
/// with the status the contract declares for that branch.
fn answer_thalamus_transition_record_transition(outcome: &crate::transition::RecordTransitionOutcome) -> (u16, String) {
    let mut body = String::from("{");
    let status = match outcome {
        crate::transition::RecordTransitionOutcome::UnknownSignal { error, .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "unknown-signal");
            json::member(&mut body, "published");
            body.push('[');
            body.push(']');
            json::member(&mut body, "error");
            json::push_text(&mut body, "thalamus.transition.UnknownSignal");
            json::member(&mut body, "payload");
            wire::encode_error_thalamus_transition_unknown_signal(error, &mut body);
            409
        }
        crate::transition::RecordTransitionOutcome::Recorded { response, transition_recorded, .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "recorded");
            json::member(&mut body, "published");
            body.push('[');
            body.push('{');
            json::member(&mut body, "event");
            json::push_text(&mut body, "thalamus.transition.TransitionRecorded");
            json::member(&mut body, "payload");
            wire::encode_event_thalamus_transition_transition_recorded(transition_recorded, &mut body);
            body.push('}');
            body.push(']');
            json::member(&mut body, "response");
            wire::encode_response_thalamus_transition_record_transition(response, &mut body);
            200
        }
    };
    body.push('}');
    (status, body)
}

/// `GET` `thalamus.transition.SeriesTransitions`: reads the declared parameters from the query string by their wire names, runs the
/// port, answers its rows. A key the view does not declare is ignored.
fn serve_thalamus_transition_series_transitions<ThalamusServiceBehaviors>(system: &crate::system::System<ThalamusServiceBehaviors>, query: &str) -> http::Response
where
    ThalamusServiceBehaviors: crate::assertion::obligations::AnswerAssertionBehavior + crate::assertion::obligations::AskAssertionBehavior + crate::definition::obligations::ActivateDefinitionBehavior + crate::definition::obligations::MuteDefinitionBehavior + crate::definition::obligations::ProposeDefinitionBehavior + crate::definition::obligations::RetireDefinitionBehavior + crate::definition::obligations::ReviseDefinitionBehavior + crate::definition::obligations::UnmuteDefinitionBehavior + crate::dimension::obligations::DeclareRegistryBehavior + crate::dimension::obligations::RegisterDimensionBehavior + crate::evaluation::obligations::RecordEvaluationBehavior + crate::observation::obligations::RecordSampleBehavior + crate::rollup::obligations::RecordRollupBehavior + crate::transition::obligations::RecordTransitionBehavior + crate::assertion::obligations::AssertionsQuery + crate::assertion::obligations::OpenAssertionsQuery + crate::definition::obligations::ActiveDefinitionsQuery + crate::definition::obligations::DefinitionsQuery + crate::dimension::obligations::DimensionsQuery + crate::dimension::obligations::RegistriesQuery + crate::evaluation::obligations::EvaluationsQuery + crate::evaluation::obligations::SeriesVerdictsQuery + crate::evaluation::obligations::VerdictChangesQuery + crate::observation::obligations::SamplesQuery + crate::observation::obligations::SeriesSamplesQuery + crate::rollup::obligations::RollupsQuery + crate::rollup::obligations::SeriesRollupsQuery + crate::transition::obligations::SeriesTransitionsQuery + crate::transition::obligations::TransitionsQuery,
{
    match http::query_object(query, &[("series", http::Scalar::Text)], "query") {
        Ok(value) => http::answer(run_thalamus_transition_series_transitions(system, &value)),
        Err(refused) => http::Response::refusal(400, &refused),
    }
}

/// `GET` `thalamus.transition.SeriesTransitions` at `read_your_writes` consistency: every row the owed projection holds for the declared
/// parameters, read from `value`, an object keyed by their wire names.
///
/// The one path the `GET` route and [`handle`] share. A decoding failure is located under
/// `query`, as the route reports it.
fn run_thalamus_transition_series_transitions<ThalamusServiceBehaviors>(system: &crate::system::System<ThalamusServiceBehaviors>, value: &json::Value) -> Result<(u16, String), entry::Refused>
where
    ThalamusServiceBehaviors: crate::assertion::obligations::AnswerAssertionBehavior + crate::assertion::obligations::AskAssertionBehavior + crate::definition::obligations::ActivateDefinitionBehavior + crate::definition::obligations::MuteDefinitionBehavior + crate::definition::obligations::ProposeDefinitionBehavior + crate::definition::obligations::RetireDefinitionBehavior + crate::definition::obligations::ReviseDefinitionBehavior + crate::definition::obligations::UnmuteDefinitionBehavior + crate::dimension::obligations::DeclareRegistryBehavior + crate::dimension::obligations::RegisterDimensionBehavior + crate::evaluation::obligations::RecordEvaluationBehavior + crate::observation::obligations::RecordSampleBehavior + crate::rollup::obligations::RecordRollupBehavior + crate::transition::obligations::RecordTransitionBehavior + crate::assertion::obligations::AssertionsQuery + crate::assertion::obligations::OpenAssertionsQuery + crate::definition::obligations::ActiveDefinitionsQuery + crate::definition::obligations::DefinitionsQuery + crate::dimension::obligations::DimensionsQuery + crate::dimension::obligations::RegistriesQuery + crate::evaluation::obligations::EvaluationsQuery + crate::evaluation::obligations::SeriesVerdictsQuery + crate::evaluation::obligations::VerdictChangesQuery + crate::observation::obligations::SamplesQuery + crate::observation::obligations::SeriesSamplesQuery + crate::rollup::obligations::RollupsQuery + crate::rollup::obligations::SeriesRollupsQuery + crate::transition::obligations::SeriesTransitionsQuery + crate::transition::obligations::TransitionsQuery,
{
    let (param0,) = match wire::decode_params_thalamus_transition_series_transitions(value, "query") {
        Ok(params) => params,
        Err(error) => return Err(entry::Refused::Input(format!("{error}"))),
    };
    match system.thalamus_service.series_transitions(param0) {
        Ok(rows) => {
            let mut body = String::from("{");
            json::member(&mut body, "rows");
            body.push('[');
            for (position, row) in rows.iter().enumerate() {
                if position > 0 {
                    body.push(',');
                }
                wire::encode_view_thalamus_transition_series_transitions(row, &mut body);
            }
            body.push(']');
            body.push('}');
            Ok((200, body))
        }
        Err(unmet) => Err(entry::Refused::Unmet(format!("{unmet}"))),
    }
}

/// `GET` `thalamus.transition.Transitions` at `read_your_writes` consistency: every row the owed projection holds.
///
/// The one path the `GET` route and [`handle`] share.
fn run_thalamus_transition_transitions<ThalamusServiceBehaviors>(system: &crate::system::System<ThalamusServiceBehaviors>) -> Result<(u16, String), entry::Refused>
where
    ThalamusServiceBehaviors: crate::assertion::obligations::AnswerAssertionBehavior + crate::assertion::obligations::AskAssertionBehavior + crate::definition::obligations::ActivateDefinitionBehavior + crate::definition::obligations::MuteDefinitionBehavior + crate::definition::obligations::ProposeDefinitionBehavior + crate::definition::obligations::RetireDefinitionBehavior + crate::definition::obligations::ReviseDefinitionBehavior + crate::definition::obligations::UnmuteDefinitionBehavior + crate::dimension::obligations::DeclareRegistryBehavior + crate::dimension::obligations::RegisterDimensionBehavior + crate::evaluation::obligations::RecordEvaluationBehavior + crate::observation::obligations::RecordSampleBehavior + crate::rollup::obligations::RecordRollupBehavior + crate::transition::obligations::RecordTransitionBehavior + crate::assertion::obligations::AssertionsQuery + crate::assertion::obligations::OpenAssertionsQuery + crate::definition::obligations::ActiveDefinitionsQuery + crate::definition::obligations::DefinitionsQuery + crate::dimension::obligations::DimensionsQuery + crate::dimension::obligations::RegistriesQuery + crate::evaluation::obligations::EvaluationsQuery + crate::evaluation::obligations::SeriesVerdictsQuery + crate::evaluation::obligations::VerdictChangesQuery + crate::observation::obligations::SamplesQuery + crate::observation::obligations::SeriesSamplesQuery + crate::rollup::obligations::RollupsQuery + crate::rollup::obligations::SeriesRollupsQuery + crate::transition::obligations::SeriesTransitionsQuery + crate::transition::obligations::TransitionsQuery,
{
    match system.thalamus_service.transitions() {
        Ok(rows) => {
            let mut body = String::from("{");
            json::member(&mut body, "rows");
            body.push('[');
            for (position, row) in rows.iter().enumerate() {
                if position > 0 {
                    body.push(',');
                }
                wire::encode_view_thalamus_transition_transitions(row, &mut body);
            }
            body.push(']');
            body.push('}');
            Ok((200, body))
        }
        Err(unmet) => Err(entry::Refused::Unmet(format!("{unmet}"))),
    }
}
