//! The one Connectors client every adapter reads through.
//!
//! Thalamus holds no secret: it runs the `connectors` binary, which keeps every credential and
//! signs every call, and names only an adapter instance and a connection id. The client passes
//! the environment of its own process through unchanged and puts nothing on the command line but
//! the words of a read. It is read-only against the systems it watches: an adapter declares the
//! operations it reads with, and [`ReadClient::invoke`] refuses every other operation before it
//! runs anything.
//!
//! A read runs as cortex runs one: list the adapter's connections (following `next_cursor` page
//! by page until the connection is found) and revalidate one that is not
//! `ready`, describe the operation for its schema and revision, invoke it. A refusal arrives as a
//! JSON envelope on stderr and becomes [`ConnectorsError::Refused`] with Connectors' `code` and
//! `stage`. Evidence that lapsed (`not_granted` at `admission`) is revalidated once and the read
//! retried; a `timeout` at `admission` is retried once. So one read is at most three invocations.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde_json::Value;
use thalamus_model::definition::Source;

/// The environment variable naming the `connectors` binary when no `--connectors` is given.
pub const CONNECTORS_ENV: &str = "THALAMUS_CONNECTORS";

/// Why a read through Connectors answered nothing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConnectorsError {
    /// The operation is not one the adapter declared as a read. Nothing was run.
    NotARead {
        /// The operation asked for.
        operation: String,
    },
    /// Connectors refused, with its own `code` and `stage`.
    Refused {
        /// The call refused, such as `metrics.series.query` or `connections list`.
        call: String,
        /// Connectors' refusal code, such as `not_granted` or `timeout`.
        code: String,
        /// The stage that refused, such as `admission` or `dispatch`.
        stage: String,
        /// The adapter's own code beside a `service_failure`, such as `not_found` or
        /// `rate_limited` (`error.data.service_code`), when Connectors gave one.
        service_code: Option<String>,
    },
    /// The adapter refused the operation's description, so the operation was described and never
    /// invoked.
    NotAdmitted {
        /// The call described, such as `forge.project.get`.
        call: String,
        /// Why the adapter refused the description.
        reason: String,
    },
    /// The connection is not among the adapter instance's connections.
    NoConnection {
        /// The adapter instance.
        adapter: String,
        /// The connection id looked for.
        connection: String,
    },
    /// `connectors` could not be run, or answered nothing a read can use.
    Unavailable {
        /// The call that failed.
        call: String,
        /// What went wrong.
        detail: String,
    },
}

impl std::fmt::Display for ConnectorsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotARead { operation } => write!(
                f,
                "operation {operation:?} is not a declared read; refused before running connectors"
            ),
            Self::Refused {
                call,
                code,
                stage,
                service_code,
            } => {
                write!(f, "{call}: connectors refused with code {code}")?;
                if let Some(service_code) = service_code {
                    write!(f, " (service code {service_code})")?;
                }
                write!(f, " at stage {stage}")
            }
            Self::NotAdmitted { call, reason } => {
                write!(f, "{call}: described and not invoked: {reason}")
            }
            Self::NoConnection {
                adapter,
                connection,
            } => write!(
                f,
                "connectors lists no connection {connection} for adapter {adapter}"
            ),
            Self::Unavailable { call, detail } => write!(f, "{call}: connectors {detail}"),
        }
    }
}

impl std::error::Error for ConnectorsError {}

/// The `connectors` binary thalamus reads through.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Connectors {
    bin: PathBuf,
}

impl Connectors {
    /// Reads through the binary at `bin`.
    pub fn new(bin: impl Into<PathBuf>) -> Self {
        Self { bin: bin.into() }
    }

    /// The binary the command line names (`--connectors`), else [`CONNECTORS_ENV`], else
    /// `connectors` on `PATH`.
    pub fn locate(flag: Option<PathBuf>) -> Self {
        let bin = flag
            .or_else(|| std::env::var_os(CONNECTORS_ENV).map(PathBuf::from))
            .unwrap_or_else(|| PathBuf::from("connectors"));
        Self::new(bin)
    }

    /// The binary this client runs.
    pub fn bin(&self) -> &Path {
        &self.bin
    }

    /// A client for an adapter that reads with `operations` and nothing else.
    pub fn reads<'a>(&'a self, operations: &'a [&'a str]) -> ReadClient<'a> {
        ReadClient {
            connectors: self,
            operations,
        }
    }

    /// The JSON document `connectors` answered, success or refusal. A refusal's envelope goes to
    /// stderr with nothing on stdout.
    fn answer(&self, args: &[&str], call: &str) -> Result<Value, ConnectorsError> {
        let out = Command::new(&self.bin)
            .args(args)
            .args(["--output", "json"])
            .output()
            .map_err(|e| ConnectorsError::Unavailable {
                call: call.to_string(),
                detail: format!("could not be run ({}): {e}", self.bin.display()),
            })?;
        let answer = if out.stdout.iter().all(u8::is_ascii_whitespace) {
            &out.stderr
        } else {
            &out.stdout
        };
        serde_json::from_slice(answer).map_err(|e| ConnectorsError::Unavailable {
            call: call.to_string(),
            detail: format!(
                "answered no JSON document (exit {}): {e}",
                out.status
                    .code()
                    .map_or_else(|| "by signal".to_string(), |c| c.to_string())
            ),
        })
    }

    /// The `result` of a successful answer, or the refusal it carries.
    fn json(&self, args: &[&str], call: &str) -> Result<Value, ConnectorsError> {
        envelope(self.answer(args, call)?, call)
    }

    /// The listing of `connection` among `adapter`'s connections, following `next_cursor` through
    /// every page until it is found or the listing ends.
    fn listed(&self, adapter: &str, connection: &str) -> Result<Value, ConnectorsError> {
        const CALL: &str = "connections list";
        let unusable = |detail: String| ConnectorsError::Unavailable {
            call: CALL.to_string(),
            detail,
        };
        let mut cursor: Option<String> = None;
        let mut seen = BTreeSet::new();
        loop {
            let mut args = vec!["connections", "list", "--adapter", adapter];
            if let Some(cursor) = &cursor {
                args.extend(["--cursor", cursor.as_str()]);
            }
            let page = self.json(&args, CALL)?;
            let entries = page["connections"]
                .as_array()
                .ok_or_else(|| unusable("listed no `connections` array".to_string()))?;
            if let Some(found) = entries
                .iter()
                .find(|c| c["connection"].as_str() == Some(connection))
            {
                return Ok(found.clone());
            }
            match page.get("next_cursor") {
                None | Some(Value::Null) => {
                    return Err(ConnectorsError::NoConnection {
                        adapter: adapter.to_string(),
                        connection: connection.to_string(),
                    });
                }
                Some(Value::String(next)) if seen.insert(next.clone()) => {
                    cursor = Some(next.clone());
                }
                Some(Value::String(next)) => {
                    return Err(unusable(format!("listed cursor {next:?} a second time")));
                }
                Some(other) => {
                    return Err(unusable(format!(
                        "listed a cursor that is not text: {other}"
                    )));
                }
            }
        }
    }

    /// Revalidates `connection`; unless `force`, only when it is not `ready`.
    fn revalidate(
        &self,
        adapter: &str,
        connection: &str,
        force: bool,
    ) -> Result<(), ConnectorsError> {
        let found = self.listed(adapter, connection)?;
        if !force && found["state"].as_str() == Some("ready") {
            return Ok(());
        }
        let revision = found["revision"]
            .as_str()
            .ok_or_else(|| ConnectorsError::Unavailable {
                call: "connections list".to_string(),
                detail: format!("listed connection {connection} without a revision"),
            })?;
        self.json(
            &[
                "connections",
                "revalidate",
                "--adapter",
                adapter,
                "--connection",
                connection,
                "--expected-revision",
                revision,
            ],
            "connections revalidate",
        )
        .map(|_| ())
    }

    fn invoke_once(
        &self,
        adapter: &str,
        connection: &str,
        operation: &str,
        input: &str,
        admit: &dyn Fn(&Value) -> Result<(), String>,
    ) -> Result<Value, ConnectorsError> {
        let call = format!("{adapter}.{operation}");
        let described = self.json(
            &[
                "operations",
                "describe",
                "--adapter",
                adapter,
                "--operation",
                operation,
            ],
            "operations describe",
        )?;
        admit(&described).map_err(|reason| ConnectorsError::NotAdmitted {
            call: call.clone(),
            reason,
        })?;
        let field = |name: &str| {
            described[name].as_str().map(str::to_string).ok_or_else(|| {
                ConnectorsError::Unavailable {
                    call: call.clone(),
                    detail: format!("described the operation without a {name}"),
                }
            })
        };
        let (schema, revision) = (field("schema")?, field("revision")?);
        self.json(
            &[
                "operations",
                "invoke",
                "--adapter",
                adapter,
                "--connection",
                connection,
                "--operation",
                operation,
                "--schema",
                &schema,
                "--revision",
                &revision,
                "--input-json",
                input,
            ],
            &call,
        )
    }
}

/// The `result` of a successful envelope, or its refusal as [`ConnectorsError::Refused`]. A
/// document that is neither (no `ok: true` with a `result`, no `ok: false`) is
/// [`ConnectorsError::Unavailable`].
fn envelope(doc: Value, call: &str) -> Result<Value, ConnectorsError> {
    match doc.get("ok") {
        Some(Value::Bool(true)) => {
            return doc
                .get("result")
                .cloned()
                .ok_or_else(|| ConnectorsError::Unavailable {
                    call: call.to_string(),
                    detail: "answered success without a result".to_string(),
                });
        }
        Some(Value::Bool(false)) => {}
        _ => {
            return Err(ConnectorsError::Unavailable {
                call: call.to_string(),
                detail: "answered a document that is neither a success nor a refusal".to_string(),
            });
        }
    }
    let error = &doc["error"];
    let text = |v: &Value| v.as_str().map(str::to_string);
    Err(ConnectorsError::Refused {
        call: call.to_string(),
        code: text(&error["data"]["code"])
            .or_else(|| text(&error["code"]))
            .unwrap_or_else(|| "unspecified".to_string()),
        stage: text(&error["data"]["stage"]).unwrap_or_else(|| "unspecified".to_string()),
        service_code: text(&error["data"]["service_code"]),
    })
}

fn refused_at_admission(error: &ConnectorsError, wanted: &str) -> bool {
    matches!(error, ConnectorsError::Refused { code, stage, .. }
        if code == wanted && stage == "admission")
}

/// A [`Connectors`] scoped to the reads one adapter declared.
#[derive(Debug, Clone, Copy)]
pub struct ReadClient<'a> {
    connectors: &'a Connectors,
    operations: &'a [&'a str],
}

impl ReadClient<'_> {
    /// The operations this client admits.
    pub fn operations(&self) -> &[&str] {
        self.operations
    }

    /// Invokes the read `operation` on `source` with `input`, and answers the adapter's own result.
    pub fn invoke(
        &self,
        source: &Source,
        operation: &str,
        input: &Value,
    ) -> Result<Value, ConnectorsError> {
        self.invoke_if(source, operation, input, &|_| Ok(()))
    }

    /// As [`ReadClient::invoke`], but the operation is invoked only when `admit` accepts its
    /// `operations describe` answer; otherwise [`ConnectorsError::NotAdmitted`]. The invoke pins
    /// the schema and revision of the very description `admit` accepted.
    pub fn invoke_if(
        &self,
        source: &Source,
        operation: &str,
        input: &Value,
        admit: &dyn Fn(&Value) -> Result<(), String>,
    ) -> Result<Value, ConnectorsError> {
        if !self.operations.contains(&operation) {
            return Err(ConnectorsError::NotARead {
                operation: operation.to_string(),
            });
        }
        let (adapter, connection) = (source.adapter.0.as_str(), source.connection.0.as_str());
        let input = serde_json::to_string(input).map_err(|e| ConnectorsError::Unavailable {
            call: format!("{adapter}.{operation}"),
            detail: format!("input is not JSON: {e}"),
        })?;
        let connectors = self.connectors;
        connectors.revalidate(adapter, connection, false)?;
        let (mut renew, mut timeout) = (true, true);
        let answer = loop {
            match connectors.invoke_once(adapter, connection, operation, &input, admit) {
                Err(e) if renew && refused_at_admission(&e, "not_granted") => {
                    renew = false;
                    connectors.revalidate(adapter, connection, true)?;
                }
                Err(e) if timeout && refused_at_admission(&e, "timeout") => timeout = false,
                other => break other?,
            }
        };
        // The CLI wraps an answer as `{adapter, operation, revision, result}`, `result` being the
        // adapter's JSON document in a string. Anything else is no document a read can use.
        let unusable = |detail: String| ConnectorsError::Unavailable {
            call: format!("{adapter}.{operation}"),
            detail,
        };
        let document = match answer.get("result") {
            Some(Value::String(text)) => serde_json::from_str(text)
                .map_err(|e| unusable(format!("answered a result that is not JSON: {e}")))?,
            Some(inner) => inner.clone(),
            None => return Err(unusable("answered without an adapter document".to_string())),
        };
        if document.is_object() || document.is_array() {
            Ok(document)
        } else {
            Err(unusable(format!(
                "answered an adapter document that is neither an object nor a list: {document}"
            )))
        }
    }
}
