// generated from thalamus v1
// model digest 6f2e9f65e0ef065a249a30bb38bae16ef166a5b45577b7a8f20509f1f3eddea7
// contract digest 067f632c99cd362c9c2d5e3d29f9b5d30e6604352424ed4eca28522b738edbee
// do not edit: regenerate with `ess synthesize --layout crate`

//! Ephemeral generated server. Durable storage and production authentication remain ports.
use clap::Parser;

#[derive(Parser)]
struct Options {
#[arg(long, default_value = "127.0.0.1:8080")]
listen: String,
#[arg(long, default_value = "none", value_parser = ["none", "actor-header"])]
callers: String,
#[arg(long = "static")]
static_root: Option<std::path::PathBuf>,
}

fn main() -> std::process::ExitCode {
let options = Options::parse();
let _ = options;
eprintln!("unmet startup obligations:\ncommand behaviour: thalamus.assertion.AskAssertion\ncommand behaviour: thalamus.definition.ProposeDefinition\ncommand behaviour: thalamus.definition.ReviseDefinition\ncommand behaviour: thalamus.dimension.RegisterDimension\ncommand behaviour: thalamus.evaluation.RecordEvaluation\ncommand behaviour: thalamus.observation.RecordSample\ncommand behaviour: thalamus.transition.RecordTransition\nview query: thalamus.evaluation.SeriesVerdicts\nview query: thalamus.observation.SeriesSamples\nview query: thalamus.rollup.SeriesRollups\nview query: thalamus.transition.SeriesTransitions");
std::process::ExitCode::FAILURE
}
