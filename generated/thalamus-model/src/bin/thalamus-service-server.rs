// generated from thalamus v1
// model digest 6f2848b89ff6190d89c005ba09023b83056dbdfb2fe499f9e414dc63d7d866bd
// contract digest 52593762eb7c5911c4f82160d91abef9b7c9217fd30b29c6b5cb7e3ac76f0515
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
