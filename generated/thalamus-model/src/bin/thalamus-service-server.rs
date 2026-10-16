// generated from thalamus v1
// model digest b40a14bce49e9e2836dc0e6b252937d178b3362dd200d747907375c0e882946a
// contract digest 468f6502d1569e62e77cf40bce974fd8ace2ccc86ced19dbf5164f2dcdb78110
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
