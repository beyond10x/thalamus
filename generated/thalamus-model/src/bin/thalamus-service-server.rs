// generated from thalamus v1
// model digest 827e774bfdab74d78674ef1f90b801f077677fc03bc8c865ffa778e5b79217a0
// contract digest 13c4b1fcc0534ac1d06b37453f62fcff2b8a5610397a25feaf2dc9a5f5254988
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
eprintln!("unmet startup obligations:\ncommand behaviour: thalamus.assertion.AskAssertion\ncommand behaviour: thalamus.definition.ProposeDefinition\ncommand behaviour: thalamus.definition.ReviseDefinition\ncommand behaviour: thalamus.dimension.RegisterDimension\ncommand behaviour: thalamus.evaluation.RecordEvaluation\ncommand behaviour: thalamus.observation.RecordSample\nview query: thalamus.evaluation.SeriesVerdicts\nview query: thalamus.observation.SeriesSamples");
std::process::ExitCode::FAILURE
}
