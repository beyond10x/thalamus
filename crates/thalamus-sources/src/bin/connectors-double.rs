//! A recorded stand-in for the `connectors` binary, for tests. It answers `connections list`,
//! `connections revalidate`, `operations describe`, `operations invoke` and any other call from
//! recorded JSON files, and holds and reads no credential.
//!
//! Where it reads and writes:
//!
//! - recordings: the directory `CONNECTORS_DOUBLE_FIXTURES` names, else the directory it was started
//!   from (its `argv[0]`, so a test can link `connectors` to it beside the recordings and leave the
//!   environment of the process under test untouched);
//! - the call log: the file `CONNECTORS_DOUBLE_LOG` names, else `calls.jsonl` in the recordings
//!   directory. Each call appends one line `{"recording", "argv", "env"}`; `env` holds the names of
//!   the environment variables it received, never their values.
//!
//! Where it holds a call to what the real CLI does:
//!
//! - it answers only a call that asks for `--output json`; any other call exits 70, since the real
//!   CLI would answer in its human form, which nothing is recorded in;
//! - `operations invoke` must carry the `--schema` and `--revision` of the current description of
//!   the same adapter and operation: the answer the latest `operations describe` received, or
//!   before any, the one the first would receive. Otherwise it refuses on stderr with code
//!   `stale_description` (or `description_unavailable` when there is no such description) at stage
//!   `observation`, next action `refresh_description`, exit 1, before reading the invoke recording.
//!
//! A call's recording is named by its leading words and the values of `--adapter`, `--connection`
//! and `--operation`: the words joined by `+`, then `+a=<adapter>`, `+c=<connection>`,
//! `+o=<operation>` for each flag present, then `.json`. Every byte of a word or value outside
//! `[A-Za-z0-9._-]` is written `%XX`, so `+`, `=` and `%` never stand for themselves inside one
//! and two different calls never read one recording. `operations invoke --adapter metrics
//! --connection main --operation series.query` reads
//! `operations+invoke+a=metrics+c=main+o=series.query.json`. Calls that differ only in other flags
//! (`--cursor`, `--input-json`) share a recording and are told apart by `call` and `input`.
//!
//! A recording is one answer or a list of them:
//!
//! ```json
//! {"call": 1, "input": {"query": "up"}, "exit": 1, "stdout": {…}, "stderr": {…}}
//! ```
//!
//! The first answer whose `call` (1-based count of this recording's calls, this one included) and
//! `input` (every key equal in the `--input-json` document) match, or that names neither, is
//! written: `stdout` to standard output and `stderr` to standard error, each as one JSON line, and
//! the process exits with `exit`, else 1 when there is a `stderr` document and 0 when there is not.
//! A call no answer matches exits 70 naming its recording.

use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use serde_json::{Value, json};

fn main() -> ExitCode {
    let mut args = std::env::args_os();
    let started = args.next().map(PathBuf::from).unwrap_or_default();
    let argv: Vec<String> = args.map(|a| a.to_string_lossy().into_owned()).collect();

    let fixtures = std::env::var_os("CONNECTORS_DOUBLE_FIXTURES")
        .map(PathBuf::from)
        .unwrap_or_else(|| started.parent().unwrap_or(Path::new(".")).to_path_buf());
    let log = std::env::var_os("CONNECTORS_DOUBLE_LOG")
        .map(PathBuf::from)
        .unwrap_or_else(|| fixtures.join("calls.jsonl"));

    let (json_output, call_argv) = without_output_json(&argv);
    let recording = recording(&call_argv);
    let call = previous_calls(&log, &recording) + 1;
    if let Err(e) = append(&log, &recording, &argv) {
        eprintln!("connectors-double: cannot append to {}: {e}", log.display());
        return ExitCode::from(70);
    }
    if !json_output {
        eprintln!(
            "connectors-double: recordings are JSON; this call did not ask for --output json"
        );
        return ExitCode::from(70);
    }
    if words(&call_argv) == ["operations", "invoke"]
        && let Some(refused) = stale_description(&fixtures, &log, &call_argv)
    {
        return refused;
    }

    let path = fixtures.join(&recording);
    let answers = match read_answers(&path) {
        Ok(Some(answers)) => answers,
        Ok(None) => {
            eprintln!("connectors-double: no recording {recording}");
            return ExitCode::from(70);
        }
        Err(e) => {
            eprintln!("connectors-double: {} is not JSON: {e}", path.display());
            return ExitCode::from(70);
        }
    };
    let input =
        flag(&call_argv, "--input-json").and_then(|text| serde_json::from_str::<Value>(text).ok());
    let Some(answer) = answers.iter().find(|a| matches(a, call, input.as_ref())) else {
        eprintln!("connectors-double: no answer in {recording} for call {call}");
        return ExitCode::from(70);
    };

    if let Some(out) = answer.get("stdout") {
        println!("{out}");
    }
    if let Some(err) = answer.get("stderr") {
        eprintln!("{err}");
    }
    let default = if answer.get("stderr").is_some() { 1 } else { 0 };
    let exit = answer
        .get("exit")
        .and_then(Value::as_u64)
        .and_then(|e| u8::try_from(e).ok())
        .unwrap_or(default);
    ExitCode::from(exit)
}

/// Whether `argv` asks for `--output json`, and `argv` without that pair.
fn without_output_json(argv: &[String]) -> (bool, Vec<String>) {
    let mut json_output = false;
    let mut rest = Vec::with_capacity(argv.len());
    let mut args = argv.iter().peekable();
    while let Some(arg) = args.next() {
        if arg == "--output" && args.peek().map(|v| v.as_str()) == Some("json") {
            args.next();
            json_output = true;
        } else {
            rest.push(arg.clone());
        }
    }
    (json_output, rest)
}

/// The leading words of a call, before its first flag.
fn words(argv: &[String]) -> Vec<&str> {
    argv.iter()
        .take_while(|a| !a.starts_with("--"))
        .map(String::as_str)
        .collect()
}

/// The value after `name` in `argv`.
fn flag<'a>(argv: &'a [String], name: &str) -> Option<&'a str> {
    argv.iter()
        .position(|a| a == name)
        .and_then(|at| argv.get(at + 1))
        .map(String::as_str)
}

/// `text` with every byte outside `[A-Za-z0-9._-]` written `%XX`.
fn encode(text: &str) -> String {
    text.bytes()
        .map(|b| {
            if b.is_ascii_alphanumeric() || matches!(b, b'.' | b'_' | b'-') {
                char::from(b).to_string()
            } else {
                format!("%{b:02X}")
            }
        })
        .collect()
}

/// The recording file a call reads.
fn recording(argv: &[String]) -> String {
    let mut name: Vec<String> = words(argv).into_iter().map(encode).collect();
    for (key, tag) in [
        ("--adapter", "a"),
        ("--connection", "c"),
        ("--operation", "o"),
    ] {
        if let Some(value) = flag(argv, key) {
            name.push(format!("{tag}={}", encode(value)));
        }
    }
    format!("{}.json", name.join("+"))
}

/// The answers of the recording at `path`; `None` when there is none.
fn read_answers(path: &Path) -> Result<Option<Vec<Value>>, serde_json::Error> {
    let Ok(text) = std::fs::read_to_string(path) else {
        return Ok(None);
    };
    Ok(Some(match serde_json::from_str::<Value>(&text)? {
        Value::Array(answers) => answers,
        answer => vec![answer],
    }))
}

/// The refusal the real CLI answers an invoke whose `--schema` and `--revision` are not those of
/// the operation's current description, or `None` when they are.
///
/// The current description is the answer the latest logged `operations describe` of the same
/// adapter and operation received, or, before any, the answer its first call would receive. A
/// description that answer does not give (no recording, a refusal, no schema or revision) is
/// unavailable.
fn stale_description(fixtures: &Path, log: &Path, argv: &[String]) -> Option<ExitCode> {
    let describe = [
        "operations",
        "describe",
        "--adapter",
        flag(argv, "--adapter").unwrap_or_default(),
        "--operation",
        flag(argv, "--operation").unwrap_or_default(),
    ]
    .map(str::to_string);
    let describe = recording(&describe);
    let latest = previous_calls(log, &describe).max(1);
    let answers = read_answers(&fixtures.join(&describe))
        .ok()
        .flatten()
        .unwrap_or_default();
    let current = answers
        .iter()
        .find(|a| matches(a, latest, None))
        .filter(|a| a["stdout"]["ok"] == Value::Bool(true))
        .and_then(|a| {
            let result = &a["stdout"]["result"];
            Some((
                result["schema"].as_str()?.to_string(),
                result["revision"].as_str()?.to_string(),
            ))
        });
    let selected = (
        flag(argv, "--schema").unwrap_or_default().to_string(),
        flag(argv, "--revision").unwrap_or_default().to_string(),
    );
    let code = match current {
        None => "description_unavailable",
        Some(current) if current == selected => return None,
        Some(_) => "stale_description",
    };
    // The real CLI's stage and next action for both codes (connectors `apps/connectors/src/local.rs`,
    // `StaleDescription | DescriptionUnavailable`).
    let envelope = json!({
        "ok": false,
        "error": {
            "code": "failure",
            "message": "the selected schema and revision are not the operation's current description",
            "data": {"kind": "operational", "code": code, "stage": "observation", "next_action": "refresh_description"}
        }
    });
    eprintln!("{envelope}");
    Some(ExitCode::from(1))
}

/// How many calls of `recording` the log already holds.
fn previous_calls(log: &Path, recording: &str) -> u64 {
    let Ok(text) = std::fs::read_to_string(log) else {
        return 0;
    };
    let count = text
        .lines()
        .filter_map(|line| serde_json::from_str::<Value>(line).ok())
        .filter(|line| line["recording"].as_str() == Some(recording))
        .count();
    u64::try_from(count).unwrap_or(u64::MAX)
}

fn append(log: &Path, recording: &str, argv: &[String]) -> std::io::Result<()> {
    let mut env: Vec<String> = std::env::vars_os()
        .map(|(name, _)| name.to_string_lossy().into_owned())
        .collect();
    env.sort();
    // One `write` per line: calls made concurrently append whole lines, never interleaved pieces.
    let line = format!(
        "{}\n",
        json!({"recording": recording, "argv": argv, "env": env})
    );
    let mut file = OpenOptions::new().create(true).append(true).open(log)?;
    file.write_all(line.as_bytes())
}

fn matches(answer: &Value, call: u64, input: Option<&Value>) -> bool {
    let call_matches = answer
        .get("call")
        .and_then(Value::as_u64)
        .is_none_or(|n| n == call);
    let input_matches = match answer.get("input").and_then(Value::as_object) {
        None => true,
        Some(wanted) => wanted
            .iter()
            .all(|(key, value)| input.and_then(|i| i.get(key)) == Some(value)),
    };
    call_matches && input_matches
}
