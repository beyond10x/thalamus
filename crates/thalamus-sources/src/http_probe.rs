//! HTTP probe reads: the availability of one catalog read operation, read through Connectors.
//!
//! Thalamus holds no HTTP client and no secret (design § 5.2): a probe is one Connectors read.
//! Connectors has no operation that requests an arbitrary URL and reports its status and latency,
//! so a probe invokes one read operation a catalog provider admits and reads whether the target
//! served it. Latency is not read: no catalog answer carries one.
//!
//! One extractor, `http_probe.up`, kind `gauge`, unit `1`. The definition names the operation in
//! `query.operation` and its input as a JSON object in `query.input` (`{}` when absent). The
//! adapter declares no fixed operation, so two checks keep a probe a catalog read (design § 11
//! D21):
//!
//! - the id ends in `.get` or `.list` (every such operation of the catalog selection sets of
//!   Connectors v0.45.0 is `effect: read`); any other name reads `unknown` and nothing runs;
//! - the `operations describe` answer names profile [`CATALOG_READ_PROFILE`], which Connectors'
//!   catalog provider gives exactly its `effect: read` operations (`adapters/catalog/src/lib.rs`,
//!   `declare`: a write is `mutation`, and every other adapter names its own operation profile, such as `kubernetes-get` or `promql-range`, never `generic-http`).
//!   Otherwise the read is `unknown` and the operation is described and never invoked. Only the
//!   catalog reads 404 as the target failing to serve: a non-catalog 404 (a kubernetes resource
//!   that does not exist) says nothing about availability. The invoke pins the revision of the
//!   description checked. The description carries no `effect` field; the profile is its
//!   one-to-one image in the catalog.
//!
//! What a read reads (Connectors v0.45.0 `apps/connectors/src/local.rs`, `owner_failure`):
//!
//! | Connectors answers | read |
//! |---|---|
//! | `ok`, the catalog result's `status` 2xx | 1, `raw` the status |
//! | `service_failure`, service code `not_found`, at `dispatch` (the target answered 404 or 410) | 0, `raw` the refusal |
//! | `timeout` at `dispatch` (a sent request's deadline passed) | 0, `raw` the refusal |
//! | any other refusal: at `admission` (`not_found`, `not_granted`, …), `rate_limited`, `forbidden` or `unauthorized` (the target answered, the credential was refused), `unavailable` at `readiness` | `unknown`, naming the refusal |
//! | an answer without a 2xx `status` | `unknown` |
//!
//! Connectors v0.45.0 reports a target's 5xx as its own `unavailable` at `readiness`, the
//! envelope of the host's own unavailability (`crates/connectors-host/src/local/runtime.rs`, "a
//! 5xx reads as unavailability"), so a 5xx reads `unknown`, never 0: an instrument failure is not
//! a down target. A known reading carries no reason, so the refusal behind a 0 is its `raw` text.
//! No `dims` key is derived; a declared one stays unset. No answer exposes a retention start.

use serde_json::{Map, Value};
use thalamus_model::definition::SignalDefinitionData;
use thalamus_model::observation::{CompleteReading, Reading, RunId};
use thalamus_model::primitives::{Decimal, Timestamp};

use crate::adapter::{Adapter, Read, unknown, window_ending};
use crate::connectors::{ConnectorsError, ReadClient};

/// The extractor this adapter reads.
pub const UP: &str = "http_probe.up";

/// The last segments of an operation id the probe admits as a read.
const READ_VERBS: [&str; 2] = ["get", "list"];

/// The profile Connectors' catalog provider describes each of its reads with; a catalog write is
/// `mutation`, and every other adapter names its own operation profile.
pub const CATALOG_READ_PROFILE: &str = "generic-http";

/// The HTTP probe adapter.
pub fn adapter() -> Option<Box<dyn Adapter>> {
    Some(Box::new(HttpProbe))
}

struct HttpProbe;

impl Adapter for HttpProbe {
    fn read_operations(&self) -> &[&str] {
        &[]
    }

    fn operations_for<'a>(&'a self, definition: &'a SignalDefinitionData) -> Vec<&'a str> {
        operation(definition).ok().into_iter().collect()
    }

    fn read(
        &self,
        client: &ReadClient<'_>,
        definition: &SignalDefinitionData,
        end: &Timestamp,
        _run_id: &RunId,
    ) -> Result<Vec<Read>, ConnectorsError> {
        let nothing = |why: String| Ok(vec![unknown(definition, end, why)]);
        if definition.extractor.0 != UP {
            return nothing(format!(
                "the http probe adapter has no extractor {}; it reads {UP}",
                definition.extractor.0
            ));
        }
        if let Some(unit) = &definition.unit
            && unit.0 != "1"
        {
            return nothing(format!(
                "{UP} is 1 or 0 with unit 1, and the definition declares unit {}",
                unit.0
            ));
        }
        if let Err(why) = window_ending(&definition.window, end) {
            return nothing(why);
        }
        let (operation, input) =
            match operation(definition).and_then(|op| Ok((op, input(definition)?))) {
                Ok(found) => found,
                Err(why) => return nothing(why),
            };
        let call = format!("{}.{operation}", definition.source.adapter.0);
        let read = match client.invoke_if(&definition.source, operation, &input, &catalog_read) {
            Ok(document) => match status(&document) {
                Ok(status) => known(definition, end, "1", status.to_string()),
                Err(why) => unknown(definition, end, why),
            },
            Err(refused) => match down(&refused, &call) {
                Some(why) => known(definition, end, "0", why),
                None => return Err(refused),
            },
        };
        Ok(vec![read])
    }
}

/// A complete read of `value`, its producer text `raw`.
fn known(definition: &SignalDefinitionData, end: &Timestamp, value: &str, raw: String) -> Read {
    let mut read = unknown(definition, end, String::new());
    read.reading = Reading::Ok(CompleteReading {
        value: Decimal(value.to_string()),
    });
    read.raw = Some(raw);
    read
}

/// Whether an `operations describe` answer describes a catalog read: its `operation.profile` is
/// [`CATALOG_READ_PROFILE`].
fn catalog_read(described: &Value) -> Result<(), String> {
    match described["operation"]["profile"].as_str() {
        Some(CATALOG_READ_PROFILE) => Ok(()),
        Some(other) => Err(format!(
            "the probe invokes only a catalog read (profile {CATALOG_READ_PROFILE}), and the \
             operation is described with profile {other}"
        )),
        None => Err(format!(
            "the probe invokes only a catalog read (profile {CATALOG_READ_PROFILE}), and the \
             description names no profile"
        )),
    }
}

/// The operation `query.operation` names, when it is a read the probe admits.
fn operation(definition: &SignalDefinitionData) -> Result<&str, String> {
    let name = definition
        .query
        .get("operation")
        .map(|text| text.0.as_str())
        .ok_or_else(|| format!("{UP} reads query `operation`, and the definition has none"))?;
    let valid = !name.is_empty()
        && name.len() <= 128
        && name
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || b"-_.".contains(&c));
    match name.rsplit_once('.') {
        Some((resource, verb)) if valid && !resource.is_empty() && READ_VERBS.contains(&verb) => {
            Ok(name)
        }
        _ => Err(format!(
            "operation {name:?} is not a read the probe admits: its id must end in .{}",
            READ_VERBS.join(" or .")
        )),
    }
}

/// The input object `query.input` writes, `{}` when it is absent.
fn input(definition: &SignalDefinitionData) -> Result<Value, String> {
    let Some(text) = definition.query.get("input") else {
        return Ok(Value::Object(Map::new()));
    };
    match serde_json::from_str::<Value>(&text.0) {
        Ok(object @ Value::Object(_)) => Ok(object),
        Ok(other) => Err(format!(
            "query `input` is the operation's input object, and it is {other}"
        )),
        Err(e) => Err(format!("query `input` is not JSON: {e}")),
    }
}

/// The 2xx `status` of a catalog result, or why the answer is no served read.
fn status(document: &Value) -> Result<u64, String> {
    match document.get("status").and_then(Value::as_u64) {
        Some(status) if (200..300).contains(&status) => Ok(status),
        Some(status) => Err(format!(
            "connectors answered status {status} as a success; only a 2xx is a served read"
        )),
        None => Err("the answer names no status".to_string()),
    }
}

/// The refusal text when `refused` says the target did not serve the read `call`; `None` for
/// every refusal that leaves the target's availability unknown.
fn down(refused: &ConnectorsError, call: &str) -> Option<String> {
    let ConnectorsError::Refused {
        call: refused_call,
        code,
        stage,
        service_code,
    } = refused
    else {
        return None;
    };
    let served_not = refused_call == call
        && stage == "dispatch"
        && match code.as_str() {
            "service_failure" => service_code.as_deref() == Some("not_found"),
            "timeout" => service_code.is_none(),
            _ => false,
        };
    served_not.then(|| refused.to_string())
}

#[cfg(test)]
mod tests {
    //! Answers the Connectors contract rules out (a success without a 2xx status: the catalog
    //! turns every other status into a refusal), so no recording carries them; the adapter still
    //! reads none of them as up.

    use serde_json::json;

    use super::{ConnectorsError, down, status};

    #[test]
    fn a_success_without_a_2xx_status_is_not_up() {
        assert_eq!(status(&json!({"status": 204, "body": null})), Ok(204));
        for document in [
            json!({"status": 503, "body": null}),
            json!({"status": 302, "body": null}),
            json!({"status": "200", "body": null}),
            json!({"body": {}}),
            json!([]),
        ] {
            assert!(status(&document).is_err(), "{document}");
        }
    }

    #[test]
    fn only_a_dispatched_refusal_of_the_probed_call_is_down() {
        let refused =
            |call: &str, code: &str, stage: &str, service: Option<&str>| ConnectorsError::Refused {
                call: call.into(),
                code: code.into(),
                stage: stage.into(),
                service_code: service.map(str::to_string),
            };
        let call = "forge.project.get";
        assert!(
            down(
                &refused(call, "service_failure", "dispatch", Some("not_found")),
                call
            )
            .is_some()
        );
        assert!(down(&refused(call, "timeout", "dispatch", None), call).is_some());
        for (who, code, stage, service) in [
            (call, "service_failure", "dispatch", Some("rate_limited")),
            (call, "service_failure", "dispatch", Some("unauthorized")),
            (
                call,
                "service_failure",
                "dispatch",
                Some("upstream_protocol"),
            ),
            (call, "service_failure", "dispatch", None),
            (call, "forbidden", "dispatch", None),
            (call, "capacity", "dispatch", None),
            (call, "timeout", "admission", None),
            (call, "not_found", "admission", None),
            (call, "unavailable", "readiness", None),
            ("connections list", "timeout", "dispatch", None),
            (
                "operations describe",
                "service_failure",
                "dispatch",
                Some("not_found"),
            ),
        ] {
            assert_eq!(
                down(&refused(who, code, stage, service), call),
                None,
                "{who} {code} {stage} {service:?}"
            );
        }
        let unavailable = ConnectorsError::Unavailable {
            call: call.into(),
            detail: "could not be run".into(),
        };
        assert_eq!(down(&unavailable, call), None);
    }
}
