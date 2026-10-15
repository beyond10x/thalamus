//! Kubernetes object counts, read through a Connectors `kubernetes` adapter instance.
//!
//! One extractor, `kubernetes.count`: the number of listed objects of one kind that match the
//! definition's predicate, one read per configured namespace. The definition's `query` names
//!
//! - `kind`: the kind `resources.list` lists (`pods`, `services`, `deployments`, …);
//! - `predicate`, optional: `<field>=<value>` (or `==`) or `<field>!=<value>`, `<field>` a dotted
//!   path into the object such as `status.phase`. Without one every listed object counts. Any
//!   other spelling is refused, never read as a literal value (`Predicate::parse`).
//!
//! A definition is never answered with no read: when no configured namespace exists, the answer
//! is one `unknown` read.
//!
//! The namespaces are the configured ones Connectors lists with `namespaces.list`; each read's
//! `namespace` is the name in that Namespace object's metadata. No other dimension is set: a
//! `tenant` is never derived from a namespace (design § 1 L9, `rule: source_only`), and `cluster`
//! is not taken from the connection. The adapter computes no delta and no trend; the signal kind
//! comes from the definition (L8).
//!
//! A list whose continuation could not be finished reads `partial` with bound `lower`. An object
//! lacking the field the predicate reads makes its namespace's read `unknown`, never a 0 and
//! never a match. A refused or failed read is `unknown` naming Connectors' code and stage. Only
//! the two list reads are declared, so the client refuses every other operation, `pods.exec`
//! included, before running anything.

use std::collections::{BTreeMap, BTreeSet};

use serde_json::{Map, Value, json};
use thalamus_model::definition::{SignalDefinitionData, Source};
use thalamus_model::observation::{Bound, CompleteReading, PartialReading, Reading, RunId};
use thalamus_model::primitives::{Decimal, Timestamp};

use crate::adapter::{Adapter, Read, unknown, window_ending};
use crate::connectors::{ConnectorsError, ReadClient};

/// The configured namespaces present in the cluster.
pub const NAMESPACES_LIST: &str = "namespaces.list";
/// One page of objects of one kind in one namespace.
pub const RESOURCES_LIST: &str = "resources.list";
/// The one extractor this adapter answers.
pub const COUNT: &str = "kubernetes.count";
/// The dimension key every read carries.
pub const NAMESPACE: &str = "namespace";

const OPERATIONS: [&str; 2] = [NAMESPACES_LIST, RESOURCES_LIST];
/// The largest page `resources.list` and `namespaces.list` admit.
const PAGE_LIMIT: u64 = 100;
/// Pages read of one list before it counts as unfinished.
const MAX_PAGES: usize = 10_000;

/// The Kubernetes adapter.
pub fn adapter() -> Option<Box<dyn Adapter>> {
    Some(Box::new(Kubernetes))
}

/// Counts Kubernetes objects through `namespaces.list` and `resources.list`.
#[derive(Debug, Clone, Copy, Default)]
pub struct Kubernetes;

impl Adapter for Kubernetes {
    fn read_operations(&self) -> &[&str] {
        &OPERATIONS
    }

    fn read(
        &self,
        client: &ReadClient<'_>,
        definition: &SignalDefinitionData,
        end: &Timestamp,
        _run_id: &RunId,
    ) -> Result<Vec<Read>, ConnectorsError> {
        if window_ending(&definition.window, end).is_err() {
            return Ok(vec![unknown(definition, end, "kubernetes.count not read")]);
        }
        let count = match Count::of(definition) {
            Ok(count) => count,
            Err(reason) => return Ok(vec![unknown(definition, end, reason)]),
        };
        let source = &definition.source;
        let namespaces = list(client, source, NAMESPACES_LIST, Map::new())?;
        let mut reads = Vec::new();
        let mut names = BTreeSet::new();
        let mut unnamed = false;
        for namespace in &namespaces.items {
            match namespace["metadata"]["name"].as_str() {
                Some(name) if !name.is_empty() => {
                    names.insert(name.to_string());
                }
                _ => unnamed = true,
            }
        }
        for name in &names {
            let reading = match list(client, source, RESOURCES_LIST, count.input(name)) {
                Ok(objects) => count.reading(&objects),
                Err(failure) => unread(failure.to_string()),
            };
            let mut read = unknown(definition, end, String::new());
            read.dims = BTreeMap::from([(NAMESPACE.to_string(), name.clone())]);
            read.reading = reading;
            reads.push(read);
        }
        if let Some(why) = namespaces.unfinished {
            reads.push(unknown(
                definition,
                end,
                format!("namespace listing unfinished, so namespaces may be missing: {why}"),
            ));
        } else if unnamed {
            reads.push(unknown(
                definition,
                end,
                "namespace listing held a namespace without a name",
            ));
        }
        if reads.is_empty() {
            // A definition is never answered with nothing: no read would be no sample and no
            // verdict, so a configuration naming no existing namespace would be invisible.
            reads.push(unknown(
                definition,
                end,
                "no configured namespace exists in the cluster",
            ));
        }
        Ok(reads)
    }
}

/// What one `kubernetes.count` definition asks for.
struct Count {
    kind: String,
    predicate: Option<Predicate>,
}

impl Count {
    /// The count `definition` declares, or the reason it declares none this adapter can read.
    fn of(definition: &SignalDefinitionData) -> Result<Count, String> {
        if definition.extractor.0 != COUNT {
            return Err(format!(
                "the kubernetes adapter has no extractor {}; it answers {COUNT}",
                definition.extractor.0
            ));
        }
        let declares_namespace = definition
            .dimensions
            .iter()
            .flatten()
            .any(|key| key.0 == NAMESPACE);
        if !declares_namespace {
            return Err(format!(
                "{COUNT} reads one series per namespace; the definition declares no dimension {NAMESPACE}"
            ));
        }
        if let Some(key) = definition
            .query
            .keys()
            .find(|k| !matches!(k.as_str(), "kind" | "predicate"))
        {
            return Err(format!(
                "{COUNT} reads query keys kind and predicate, not {key:?}"
            ));
        }
        let kind = match definition.query.get("kind") {
            Some(kind) if !kind.0.is_empty() => kind.0.clone(),
            _ => return Err(format!("{COUNT} query names no kind")),
        };
        let predicate = definition
            .query
            .get("predicate")
            .map(|text| Predicate::parse(&text.0))
            .transpose()?;
        Ok(Count { kind, predicate })
    }

    /// The `resources.list` input for `namespace`, without the paging fields.
    fn input(&self, namespace: &str) -> Map<String, Value> {
        let mut input = Map::new();
        input.insert("namespace".to_string(), json!(namespace));
        input.insert("kind".to_string(), json!(self.kind));
        input
    }

    /// The reading of one namespace's listed objects.
    fn reading(&self, objects: &Listing) -> Reading {
        let mut matched: u64 = 0;
        for object in &objects.items {
            let hit = match &self.predicate {
                None => true,
                Some(predicate) => match predicate.test(object) {
                    Ok(hit) => hit,
                    Err(Unanswered::NotScalar) => {
                        return unread(format!(
                            "field is not a scalar: {} of an object of kind {} is an object or a list",
                            predicate.path.join("."),
                            self.kind
                        ));
                    }
                    Err(Unanswered::Absent) => {
                        return unread(format!(
                            "{} not projected: an object of kind {} has no {}",
                            if predicate.path.first().map(String::as_str) == Some("status") {
                                "status"
                            } else {
                                "field"
                            },
                            self.kind,
                            predicate.path.join(".")
                        ));
                    }
                },
            };
            matched += u64::from(hit);
        }
        let value = Decimal(matched.to_string());
        match objects.unfinished {
            None => Reading::Ok(CompleteReading { value }),
            Some(_) => Reading::Partial(PartialReading {
                value,
                bound: Bound::Lower,
            }),
        }
    }
}

/// Whether `c` may stand in a predicate value: ASCII `[A-Za-z0-9._:/-]`.
///
/// An allowlist, so every operator, quote, glob, pattern, separator, whitespace, invisible or
/// non-ASCII character is refused without being named: a value is one exact literal.
fn value_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | ':' | '/' | '-')
}

/// Why an object answers no predicate.
enum Unanswered {
    /// The field is absent or `null`.
    Absent,
    /// The field is an object or a list, which no exact literal equals.
    NotScalar,
}

/// `<path>=<value>`, `<path>==<value>` or `<path>!=<value>`.
struct Predicate {
    path: Vec<String>,
    value: String,
    equal: bool,
}

impl Predicate {
    /// The predicate `text` states, or why it states none.
    ///
    /// The grammar is closed: a text it does not take whole is refused, never read with the rest
    /// as a literal value, which would answer a complete count of another question (often 0).
    /// `==` is `=`, as in a Kubernetes field selector. A field is dot-separated segments of
    /// `[A-Za-z0-9_-]`; a value is non-empty and every character passes [`value_char`], so a
    /// selector conjunction, a set, comparison, pattern or glob operator, a quote, whitespace and
    /// any invisible or non-ASCII character are refused.
    fn parse(text: &str) -> Result<Predicate, String> {
        let refuse = |why: &str| Err(format!("{COUNT} predicate {text:?} {why}"));
        let (path, value, equal) = if let Some((path, value)) = text.split_once("!=") {
            (path, value, false)
        } else if let Some((path, value)) = text.split_once("==") {
            (path, value, true)
        } else if let Some((path, value)) = text.split_once('=') {
            (path, value, true)
        } else {
            return refuse("is not <field>=<value>, <field>==<value> or <field>!=<value>");
        };
        let path: Vec<String> = path.trim().split('.').map(str::to_string).collect();
        let segment = |s: &String| {
            !s.is_empty()
                && s.chars()
                    .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
        };
        if !path.iter().all(segment) {
            return refuse("names no field as a dotted path of [A-Za-z0-9_-] segments");
        }
        let value = value.trim();
        if value.is_empty() {
            return refuse("compares with an empty value");
        }
        if let Some(c) = value.chars().find(|c| !value_char(*c)) {
            return refuse(&format!(
                "has {c:?} in its value; a value is ASCII [A-Za-z0-9._:/-] only, compared exactly"
            ));
        }
        Ok(Predicate {
            path,
            value: value.to_string(),
            equal,
        })
    }

    /// Whether `object` matches, or why it answers nothing. Only a scalar field is compared.
    fn test(&self, object: &Value) -> Result<bool, Unanswered> {
        let field = self
            .path
            .iter()
            .try_fold(object, |at, key| at.as_object()?.get(key))
            .ok_or(Unanswered::Absent)?;
        // Every JSON kind is named, so a new one cannot fall through into a comparison.
        let text = match field {
            Value::Null => return Err(Unanswered::Absent),
            Value::Object(_) | Value::Array(_) => return Err(Unanswered::NotScalar),
            Value::String(text) => text.clone(),
            Value::Bool(flag) => flag.to_string(),
            Value::Number(number) => number.to_string(),
        };
        Ok((text == self.value) == self.equal)
    }
}

/// Every item of a paged list, and why it is unfinished when it is.
struct Listing {
    items: Vec<Value>,
    unfinished: Option<String>,
}

/// Every page of `operation` with `input`, following `next_cursor`.
///
/// A failure of the first page is the caller's error. Once a page has been read, a later failure
/// leaves the listing unfinished: what was listed is still a lower bound.
fn list(
    client: &ReadClient<'_>,
    source: &Source,
    operation: &str,
    input: Map<String, Value>,
) -> Result<Listing, ConnectorsError> {
    let mut items = Vec::new();
    let mut cursor: Option<String> = None;
    let mut seen = BTreeSet::new();
    for page in 0..MAX_PAGES {
        let mut call = input.clone();
        call.insert("limit".to_string(), json!(PAGE_LIMIT));
        if let Some(cursor) = &cursor {
            call.insert("cursor".to_string(), json!(cursor));
        }
        let answer = match client.invoke(source, operation, &Value::Object(call)) {
            Ok(answer) => answer,
            Err(failure) if page == 0 => return Err(failure),
            Err(failure) => return Ok(unfinished(items, failure.to_string())),
        };
        let Some(listed) = answer["items"].as_array() else {
            let detail = "answered a page without an items list".to_string();
            if page == 0 {
                return Err(ConnectorsError::Unavailable {
                    call: format!("{}.{operation}", source.adapter.0),
                    detail,
                });
            }
            return Ok(unfinished(items, detail));
        };
        items.extend(listed.iter().cloned());
        match &answer["next_cursor"] {
            Value::Null if answer["complete"] == Value::Bool(true) => {
                return Ok(Listing {
                    items,
                    unfinished: None,
                });
            }
            Value::Null => {
                return Ok(unfinished(
                    items,
                    "the provider did not finish and gave no continuation".to_string(),
                ));
            }
            Value::String(next) if seen.insert(next.clone()) => cursor = Some(next.clone()),
            Value::String(next) => {
                return Ok(unfinished(
                    items,
                    format!("continuation {next:?} came back a second time"),
                ));
            }
            other => {
                return Ok(unfinished(
                    items,
                    format!("continuation is not text: {other}"),
                ));
            }
        }
    }
    Ok(unfinished(
        items,
        format!("more than {MAX_PAGES} pages listed"),
    ))
}

fn unfinished(items: Vec<Value>, why: String) -> Listing {
    Listing {
        items,
        unfinished: Some(why),
    }
}

fn unread(reason: String) -> Reading {
    Reading::Unknown(thalamus_model::observation::UnreadReading { reason })
}
