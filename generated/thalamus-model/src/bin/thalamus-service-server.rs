// generated from thalamus v1
// model digest 9f981109ead94ade6b2b664c0234683eabf8a4e37dbb57b0fab92ac76ace7832
// contract digest e31e759ce615e6d5358d020e748a76c36fee60b1207d5e97f762de8391357b06
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
