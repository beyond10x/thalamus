// generated from thalamus v1
// model digest 73865b5d22d3d964014ec5c52cb6db35af4760818bbeefd77df0a2551b8fd3ed
// contract digest 093adf023da1db0be3662fe2c9b1542d25415a595e3023e37377255b2d0a3061
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
