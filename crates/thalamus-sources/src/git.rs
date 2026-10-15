//! Git reads: repository records of a GitLab-hosted project, read through the Connectors GitLab
//! catalog provider and turned into counts and ratios.
//!
//! Connectors has no local Git adapter and no GitHub provider; the repository surface it serves is
//! the GitLab catalog provider's list reads, one page per call, at most [`PER_PAGE`] records a
//! page. Each answer is `{status, body, provenance}`, the body being the page's records as GitLab
//! returns them. GitLab's offset paging ends at a page shorter than the page size, so a read walks
//! pages `1, 2, …` until one is short, and at most [`PAGE_LIMIT`] of them.
//!
//! A definition names its records in its `query` map:
//!
//! | name | holds |
//! |---|---|
//! | `project` | the GitLab project id or full path; required |
//! | `records` | `merge_requests`, `pipelines`, `deployments`, `commits` or `tags`; required |
//! | `time` | the record field the window is placed on; each kind admits its own, the first the default (table below) |
//! | `filter` | a JSON object of list parameters sent to GitLab as written (`{"state": "merged"}`); it may not set `id`, the paging parameters or the window parameters, which the adapter sets |
//! | `match` | a JSON object: a record matches when every named field (a dotted path) holds the given value, or one of the given list of values. Optional for `git.count`, required for `git.ratio` |
//!
//! | records | operation | `time` (default first) and the parameters bounding it |
//! |---|---|---|
//! | `merge_requests` | `merge_requests.list` | `merged_at`, `created_at`, `updated_at`: `<field stem>_after`, `<field stem>_before` |
//! | `pipelines` | `pipelines.list` | `created_at`, `updated_at`: as above |
//! | `deployments` | `deployments.list` | `updated_at`, `finished_at`: as above |
//! | `commits` | `commits.list` | `committed_date`: `since`, `until` |
//! | `tags` | `tags.list` | `commit.committed_date`: none, so every tag is read |
//!
//! GitLab narrows each page to the window where it can; the adapter places every record again,
//! inside `[start, end)` of the window, by its own `time` field.
//!
//! Two extractors:
//!
//! - `git.count`: the records in the window that match, one read per series.
//! - `git.ratio`: the records in the window that match over all records in the window, per series:
//!   a decimal with unit `1`; the population `n` is the record count, in the unit of the
//!   definition's `denominator`. Both sides come from the one walk, so every matching series has
//!   its population, and a series with no matching record reads 0 over a population the adapter
//!   read whole, but only when some record of the window matches (design § 11 D20): a window in
//!   which no record matches reads `unknown` in every series, with no population ([`NO_MATCH`]), since a
//!   misspelt or mistyped `match` value looks the same. A window holding no record reads
//!   `unknown` with population 0.
//!
//! A `dims` key is set only from a top-level field the record carries, named for a dimension the
//! definition declares, holding text, a number or a boolean (L9); a null or absent field sets no
//! key. With declared dimensions and no matching record, a count has no series and reads one
//! `unknown`.
//!
//! What reads `unknown`, never 0: a refusal or failure before the first page (Connectors' code and
//! stage, through the registry); a record answered twice (the paging shifted under the read); a
//! record in the walk without its identity, or without a `time` field that is RFC 3339; a record
//! in the window without a field `match` names; a dimension field holding an object or a list. A
//! walk that stops before the window is covered (a page refused or unreadable after the first, or
//! the page budget spent) is cut: a count of one series is `partial` with bound `lower` (unknown
//! when it counted nothing), and a count of many series or a ratio is one `unknown` naming the cut.
//!
//! `raw` is unset: no read is a producer's scalar. No answer exposes the source's retention
//! start, so `retention_start` stays unset.

use std::collections::{BTreeMap, BTreeSet};

use serde_json::{Map, Value, json};
use thalamus_model::definition::{SignalDefinitionData, Unit};
use thalamus_model::observation::{
    Bound, CompleteReading, PartialReading, Reading, RunId, TimeWindow, UnreadReading,
};
use thalamus_model::primitives::{Decimal, Timestamp};
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;

use crate::adapter::{Adapter, Population, Read, query_digest, unknown, window_ending};
use crate::connectors::{ConnectorsError, ReadClient};

/// The records asked for a page: the catalog provider's ceiling for every list read.
pub const PER_PAGE: usize = 100;

/// The most pages one read walks before it reads the window as cut.
pub const PAGE_LIMIT: usize = 50;

/// The fractional digits a quotient that does not terminate keeps.
pub const RATIO_SCALE: u32 = 18;

/// The reason of a ratio read whose window holds no record.
pub const EMPTY_POPULATION: &str = "empty population: the window holds no record";

/// The reason of a ratio read whose window holds records and none of them matches (design § 11
/// D20).
pub const NO_MATCH: &str =
    "no record in the window matched `match`; a matcher that matches nothing is not read as 0";

/// The Connectors operations this adapter reads with, all list reads of the catalog provider.
const READS: [&str; 5] = [
    "merge_requests.list",
    "pipelines.list",
    "deployments.list",
    "commits.list",
    "tags.list",
];

/// Parameters a definition's `filter` may not set: the adapter sets them, or they change paging.
const RESERVED: [&str; 5] = ["id", "page", "per_page", "pagination", "page_token"];

/// The Git adapter.
pub fn adapter() -> Option<Box<dyn Adapter>> {
    Some(Box::new(Git))
}

/// Reads `git.count` and `git.ratio` through the GitLab catalog provider's list reads.
#[derive(Debug, Clone, Copy, Default)]
pub struct Git;

impl Adapter for Git {
    fn read_operations(&self) -> &[&str] {
        &READS
    }

    fn read(
        &self,
        client: &ReadClient<'_>,
        definition: &SignalDefinitionData,
        end: &Timestamp,
        _run_id: &RunId,
    ) -> Result<Vec<Read>, ConnectorsError> {
        let extractor = definition.extractor.0.as_str();
        let window = match window_ending(&definition.window, end) {
            Ok(window) => window,
            Err(why) => {
                return Ok(vec![unknown(
                    definition,
                    end,
                    format!("{extractor}: {why}"),
                )]);
            }
        };
        let reads = Reads {
            definition,
            window,
            extractor,
        };
        let plan = match Plan::parse(definition) {
            Ok(plan) => plan,
            Err(why) => return Ok(vec![reads.unknown(BTreeMap::new(), why, None)]),
        };
        let bounds = match Bounds::of(&reads.window) {
            Ok(bounds) => bounds,
            Err(why) => return Ok(vec![reads.unknown(BTreeMap::new(), why, None)]),
        };
        let walk = walk(PAGE_LIMIT, |page| {
            fetch(client, definition, &plan, &reads.window, page)
        })?;
        let placed = match place(&walk.records, &plan, &bounds, &reads.declared()) {
            Ok(placed) => placed,
            Err(why) => return Ok(vec![reads.unknown(BTreeMap::new(), why, None)]),
        };
        Ok(match plan.extractor {
            Extractor::Count => count(&reads, &placed, walk.cut),
            Extractor::Ratio => ratio(&reads, &placed, walk.cut),
        })
    }
}

/// One kind of record: the list read that answers it, the field naming each record, and the
/// fields its window may be placed on.
struct Kind {
    records: &'static str,
    operation: &'static str,
    identity: &'static str,
    times: &'static [Time],
}

/// A field a window is placed on, with the list parameters bounding it, when GitLab has any.
struct Time {
    field: &'static str,
    bounds: Option<(&'static str, &'static str)>,
}

const KINDS: [Kind; 5] = [
    Kind {
        records: "merge_requests",
        operation: "merge_requests.list",
        identity: "id",
        times: &[
            Time {
                field: "merged_at",
                bounds: Some(("merged_after", "merged_before")),
            },
            Time {
                field: "created_at",
                bounds: Some(("created_after", "created_before")),
            },
            Time {
                field: "updated_at",
                bounds: Some(("updated_after", "updated_before")),
            },
        ],
    },
    Kind {
        records: "pipelines",
        operation: "pipelines.list",
        identity: "id",
        times: &[
            Time {
                field: "created_at",
                bounds: Some(("created_after", "created_before")),
            },
            Time {
                field: "updated_at",
                bounds: Some(("updated_after", "updated_before")),
            },
        ],
    },
    Kind {
        records: "deployments",
        operation: "deployments.list",
        identity: "id",
        times: &[
            Time {
                field: "updated_at",
                bounds: Some(("updated_after", "updated_before")),
            },
            Time {
                field: "finished_at",
                bounds: Some(("finished_after", "finished_before")),
            },
        ],
    },
    Kind {
        records: "commits",
        operation: "commits.list",
        identity: "id",
        times: &[Time {
            field: "committed_date",
            bounds: Some(("since", "until")),
        }],
    },
    Kind {
        records: "tags",
        operation: "tags.list",
        identity: "name",
        times: &[Time {
            field: "commit.committed_date",
            bounds: None,
        }],
    },
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Extractor {
    Count,
    Ratio,
}

/// What a definition asks, checked before anything is invoked.
struct Plan {
    extractor: Extractor,
    project: String,
    kind: &'static Kind,
    time: &'static Time,
    filter: Map<String, Value>,
    /// Each field path a matching record holds, with the values it may hold.
    matches: Vec<(String, Vec<Value>)>,
}

impl Plan {
    fn parse(definition: &SignalDefinitionData) -> Result<Plan, String> {
        let extractor = match definition.extractor.0.as_str() {
            "git.count" => Extractor::Count,
            "git.ratio" => Extractor::Ratio,
            _ => {
                return Err("no such git extractor; git reads git.count and git.ratio".to_string());
            }
        };
        if extractor == Extractor::Ratio
            && let Some(Unit(unit)) = &definition.unit
            && unit != "1"
        {
            return Err(format!(
                "a ratio has unit 1; the definition declares unit {unit:?}"
            ));
        }
        let text = |name: &str| definition.query.get(name).map(|text| text.0.as_str());
        let project = match text("project") {
            Some(project) if !project.trim().is_empty() => project.to_string(),
            _ => return Err("the definition has no query `project`".to_string()),
        };
        let records = text("records").ok_or("the definition has no query `records`")?;
        let kind = KINDS
            .iter()
            .find(|kind| kind.records == records)
            .ok_or_else(|| {
                format!(
                    "records `{records}` is not a kind git reads; it reads {}",
                    KINDS.map(|kind| kind.records).join(", ")
                )
            })?;
        let time = match text("time") {
            None => &kind.times[0],
            Some(field) => kind
                .times
                .iter()
                .find(|time| time.field == field)
                .ok_or_else(|| {
                    format!(
                        "time `{field}` is not a field {records} are placed on; they take {}",
                        kind.times
                            .iter()
                            .map(|time| time.field)
                            .collect::<Vec<_>>()
                            .join(", ")
                    )
                })?,
        };
        let filter = match text("filter") {
            None => Map::new(),
            Some(text) => object(text, "filter")?,
        };
        let window_parameters = kind
            .times
            .iter()
            .filter_map(|time| time.bounds)
            .flat_map(|(after, before)| [after, before]);
        let mut reserved: Vec<&str> = RESERVED.to_vec();
        reserved.extend(window_parameters);
        if let Some(key) = filter.keys().find(|key| reserved.contains(&key.as_str())) {
            return Err(format!(
                "the query `filter` sets `{key}`, which the adapter sets; a filter may not set {}",
                reserved.join(", ")
            ));
        }
        let matches = match text("match") {
            None if extractor == Extractor::Ratio => {
                return Err(
                    "git.ratio reads query `match`, the records counted over all records, and the \
                     definition has none"
                        .to_string(),
                );
            }
            None => Vec::new(),
            Some(text) => {
                let object = object(text, "match")?;
                if object.is_empty() {
                    return Err("the query `match` names no field".to_string());
                }
                object
                    .into_iter()
                    .map(|(path, wanted)| match wanted {
                        Value::Array(values) if values.is_empty() => Err(format!(
                            "the query `match` gives field `{path}` an empty list of values"
                        )),
                        Value::Array(values) => Ok((path, values)),
                        Value::Object(_) => Err(format!(
                            "the query `match` gives field `{path}` an object; it takes a value \
                             or a list of values"
                        )),
                        value => Ok((path, vec![value])),
                    })
                    .collect::<Result<_, _>>()?
            }
        };
        Ok(Plan {
            extractor,
            project,
            kind,
            time,
            filter,
            matches,
        })
    }
}

/// The JSON object in the query `name`, or why there is none.
fn object(text: &str, name: &str) -> Result<Map<String, Value>, String> {
    match serde_json::from_str::<Value>(text) {
        Ok(Value::Object(object)) => Ok(object),
        Ok(_) => Err(format!("the query `{name}` is JSON but not an object")),
        Err(e) => Err(format!("the query `{name}` is not a JSON object: {e}")),
    }
}

/// What every read of one definition at one window end shares.
struct Reads<'a> {
    definition: &'a SignalDefinitionData,
    window: TimeWindow,
    extractor: &'a str,
}

impl Reads<'_> {
    fn read(
        &self,
        dims: BTreeMap<String, String>,
        reading: Reading,
        denominator: Option<Population>,
    ) -> Read {
        Read {
            dims,
            reading,
            window: self.window.clone(),
            denominator,
            adapter: self.definition.source.adapter.clone(),
            connection: self.definition.source.connection.clone(),
            query_sha256: query_digest(&self.definition.query),
            raw: None,
            retention_start: None,
        }
    }

    /// An `unknown` read saying `reason`, prefixed with the extractor.
    fn unknown(
        &self,
        dims: BTreeMap<String, String>,
        reason: String,
        denominator: Option<Population>,
    ) -> Read {
        let reason = format!("{}: {reason}", self.extractor);
        self.read(
            dims,
            Reading::Unknown(UnreadReading { reason }),
            denominator,
        )
    }

    /// The dimension keys the definition declares.
    fn declared(&self) -> Vec<&str> {
        self.definition
            .dimensions
            .iter()
            .flatten()
            .map(|key| key.0.as_str())
            .collect()
    }

    /// A population of `n` records, in the unit of the definition's `denominator`.
    fn population(&self, n: usize) -> Option<Population> {
        let unit = self
            .definition
            .denominator
            .as_ref()
            .map_or_else(|| Unit("1".to_string()), |spec| spec.unit.clone());
        i64::try_from(n).ok().map(|n| Population { n, unit })
    }
}

/// The window's bounds as instants.
struct Bounds {
    start: OffsetDateTime,
    end: OffsetDateTime,
}

impl Bounds {
    fn of(window: &TimeWindow) -> Result<Bounds, String> {
        let at = |text: &str| {
            OffsetDateTime::parse(text, &Rfc3339)
                .map_err(|e| format!("window bound {text:?} is not RFC 3339: {e}"))
        };
        Ok(Bounds {
            start: at(&window.start.0)?,
            end: at(&window.end.0)?,
        })
    }

    /// Whether `at` lies in `[start, end)`.
    fn holds(&self, at: OffsetDateTime) -> bool {
        self.start <= at && at < self.end
    }
}

/// The records a walk read, and why it stopped before the window was covered, when it did.
#[derive(Debug)]
struct Walk {
    records: Vec<Value>,
    cut: Option<String>,
}

/// Reads pages `1..=limit` through `fetch` until one is shorter than [`PER_PAGE`]. A failure of the
/// first page is the read's failure; a later one cuts the walk, as does the budget running out.
fn walk(
    limit: usize,
    mut fetch: impl FnMut(usize) -> Result<Vec<Value>, ConnectorsError>,
) -> Result<Walk, ConnectorsError> {
    let mut records = Vec::new();
    for page in 1..=limit {
        match fetch(page) {
            Err(failure) if page == 1 => return Err(failure),
            Err(failure) => {
                return Ok(Walk {
                    records,
                    cut: Some(format!(
                        "page {page} was not read ({failure}); the window is not covered"
                    )),
                });
            }
            Ok(body) => {
                let short = body.len() < PER_PAGE;
                records.extend(body);
                if short {
                    return Ok(Walk { records, cut: None });
                }
            }
        }
    }
    Ok(Walk {
        records,
        cut: Some(format!(
            "the page budget of {limit} pages of {PER_PAGE} ran out before the window was covered"
        )),
    })
}

/// Page `page` of the plan's records over `window`: the body's records.
fn fetch(
    client: &ReadClient<'_>,
    definition: &SignalDefinitionData,
    plan: &Plan,
    window: &TimeWindow,
    page: usize,
) -> Result<Vec<Value>, ConnectorsError> {
    let mut input = plan.filter.clone();
    input.insert("id".to_string(), json!(plan.project));
    if let Some((after, before)) = plan.time.bounds {
        input.insert(after.to_string(), json!(window.start.0));
        input.insert(before.to_string(), json!(window.end.0));
    }
    input.insert("per_page".to_string(), json!(PER_PAGE));
    input.insert("page".to_string(), json!(page));
    let operation = plan.kind.operation;
    let document = client.invoke(&definition.source, operation, &Value::Object(input))?;
    body(&document).map_err(|detail| ConnectorsError::Unavailable {
        call: format!("{}.{operation}", definition.source.adapter.0),
        detail: format!("answered page {page} with no list of records: {detail}"),
    })
}

/// The records of a catalog answer `{status, body, provenance}`.
fn body(document: &Value) -> Result<Vec<Value>, String> {
    match document["status"].as_u64() {
        Some(200..=299) => {}
        Some(status) => return Err(format!("status {status}")),
        None => return Err("no `status`".to_string()),
    }
    let records = document["body"]
        .as_array()
        .ok_or("a `body` that is not a list")?;
    if let Some(other) = records.iter().find(|record| !record.is_object()) {
        return Err(format!("a record that is not an object: {other}"));
    }
    Ok(records.clone())
}

/// One record placed in the window: its series and whether it matches.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Placed {
    dims: BTreeMap<String, String>,
    matches: bool,
}

/// The records of a walk inside the window, or why the walk reads nothing.
fn place(
    records: &[Value],
    plan: &Plan,
    bounds: &Bounds,
    declared: &[&str],
) -> Result<Vec<Placed>, String> {
    let identity = plan.kind.identity;
    let time = plan.time.field;
    let mut seen = BTreeSet::new();
    let mut placed = Vec::new();
    for record in records {
        let id = match record.get(identity) {
            Some(Value::String(text)) => text.clone(),
            Some(Value::Number(n)) => n.to_string(),
            _ => {
                return Err(format!(
                    "a record carries no `{identity}`, so a record answered twice cannot be told \
                     apart: {record}"
                ));
            }
        };
        if !seen.insert(id.clone()) {
            return Err(format!(
                "record {identity} {id} was answered twice; the pages shifted under the read"
            ));
        }
        let at = path(record, time)
            .and_then(Value::as_str)
            .and_then(|text| OffsetDateTime::parse(text, &Rfc3339).ok())
            .ok_or_else(|| {
                format!(
                    "record {identity} {id} has no `{time}` in RFC 3339, so it cannot be placed in \
                     the window"
                )
            })?;
        if !bounds.holds(at) {
            continue;
        }
        let mut matches = true;
        for (field, wanted) in &plan.matches {
            let value = path(record, field).ok_or_else(|| {
                format!(
                    "record {identity} {id} carries no field `{field}`, which the query `match` \
                     names"
                )
            })?;
            matches &= wanted.contains(value);
        }
        let mut dims = BTreeMap::new();
        for key in declared {
            match record.get(*key) {
                None | Some(Value::Null) => {}
                Some(Value::String(text)) => {
                    dims.insert((*key).to_string(), text.clone());
                }
                Some(value @ (Value::Number(_) | Value::Bool(_))) => {
                    dims.insert((*key).to_string(), value.to_string());
                }
                Some(other) => {
                    return Err(format!(
                        "record {identity} {id} holds {other} in `{key}`, which is no dimension \
                         value"
                    ));
                }
            }
        }
        placed.push(Placed { dims, matches });
    }
    Ok(placed)
}

/// The value at a dotted `path` of `record`; `None` when a step is absent. A null met on the way
/// is the value (a merge request with no `merge_user` has a null `merge_user.username`).
fn path<'a>(record: &'a Value, path: &str) -> Option<&'a Value> {
    path.split('.').try_fold(record, |value, step| {
        if value.is_null() {
            Some(value)
        } else {
            value.as_object()?.get(step)
        }
    })
}

fn count(reads: &Reads<'_>, placed: &[Placed], cut: Option<String>) -> Vec<Read> {
    let matched: Vec<&Placed> = placed.iter().filter(|p| p.matches).collect();
    if reads.declared().is_empty() {
        let value = Decimal(matched.len().to_string());
        let reading = match cut {
            None => Reading::Ok(CompleteReading { value }),
            Some(why) if matched.is_empty() => {
                return vec![reads.unknown(
                    BTreeMap::new(),
                    format!(
                        "{why}; no record counted before the cut, and a bound of 0 says nothing"
                    ),
                    None,
                )];
            }
            // One series: a cut walk holds at least the records it counted.
            Some(_) => Reading::Partial(PartialReading {
                value,
                bound: Bound::Lower,
            }),
        };
        return vec![reads.read(BTreeMap::new(), reading, None)];
    }
    if let Some(why) = cut {
        return vec![reads.unknown(
            BTreeMap::new(),
            format!("{why}; series past the cut are not read"),
            None,
        )];
    }
    if matched.is_empty() {
        return vec![reads.unknown(
            BTreeMap::new(),
            "no record in the window matched, so there is no series to count".to_string(),
            None,
        )];
    }
    let mut counts: BTreeMap<&BTreeMap<String, String>, usize> = BTreeMap::new();
    for p in matched {
        *counts.entry(&p.dims).or_default() += 1;
    }
    counts
        .into_iter()
        .map(|(dims, n)| {
            let reading = Reading::Ok(CompleteReading {
                value: Decimal(n.to_string()),
            });
            reads.read(dims.clone(), reading, None)
        })
        .collect()
}

fn ratio(reads: &Reads<'_>, placed: &[Placed], cut: Option<String>) -> Vec<Read> {
    if let Some(why) = cut {
        return vec![reads.unknown(
            BTreeMap::new(),
            format!("{why}; a ratio over part of the population has no bound"),
            None,
        )];
    }
    if placed.is_empty() {
        return vec![reads.unknown(
            BTreeMap::new(),
            EMPTY_POPULATION.to_string(),
            reads.population(0),
        )];
    }
    // Design § 11 D20: no matching record at all over a non-empty window looks the same as a
    // misspelt or mistyped `match` value, so no series is read as 0: each series reads unknown,
    // as the SQL, Prometheus and Loki adapters read it.
    if !placed.iter().any(|p| p.matches) {
        let keys: BTreeSet<&BTreeMap<String, String>> = placed.iter().map(|p| &p.dims).collect();
        return keys
            .into_iter()
            .map(|dims| reads.unknown(dims.clone(), NO_MATCH.to_string(), None))
            .collect();
    }
    // Each series: (matching records, records).
    let mut series: BTreeMap<&BTreeMap<String, String>, (usize, usize)> = BTreeMap::new();
    for p in placed {
        let (matched, all) = series.entry(&p.dims).or_default();
        *matched += usize::from(p.matches);
        *all += 1;
    }
    series
        .into_iter()
        .map(|(dims, (matched, all))| {
            let reading = Reading::Ok(CompleteReading {
                value: Decimal(quotient(matched, all)),
            });
            reads.read(dims.clone(), reading, reads.population(all))
        })
        .collect()
}

/// `matched / all` for `0 < all` and `matched <= all`, in the wire grammar: [`RATIO_SCALE`]
/// fractional digits, the last rounded half away from zero, trailing zeros dropped.
fn quotient(matched: usize, all: usize) -> String {
    let (matched, all) = (matched as u128, all as u128);
    let unit = 10_u128.pow(RATIO_SCALE);
    let deeper = matched * unit * 10 / all;
    let scaled = deeper / 10 + u128::from(deeper % 10 >= 5);
    let (whole, fraction) = (scaled / unit, scaled % unit);
    let fraction = format!("{fraction:0width$}", width = RATIO_SCALE as usize);
    match fraction.trim_end_matches('0') {
        "" => whole.to_string(),
        digits => format!("{whole}.{digits}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn record(id: u64, at: &str) -> Value {
        json!({"id": id, "created_at": at, "status": "failed", "ref": "main"})
    }

    fn page(from: u64, n: usize) -> Vec<Value> {
        (0..n as u64)
            .map(|i| record(from + i, "2026-10-08T00:00:00Z"))
            .collect()
    }

    fn plan(matches: &[(&str, &str)]) -> Plan {
        Plan {
            extractor: Extractor::Count,
            project: "group/app".to_string(),
            kind: &KINDS[1],
            time: &KINDS[1].times[0],
            filter: Map::new(),
            matches: matches
                .iter()
                .map(|(k, v)| ((*k).to_string(), vec![json!(v)]))
                .collect(),
        }
    }

    fn bounds() -> Bounds {
        Bounds::of(&TimeWindow {
            start: Timestamp("2026-10-07T22:57:03Z".into()),
            end: Timestamp("2026-10-14T22:57:03Z".into()),
        })
        .expect("bounds")
    }

    #[test]
    fn walk_stops_at_a_short_page_and_is_cut_at_the_budget() {
        let mut asked = Vec::new();
        let walked = walk(PAGE_LIMIT, |n| {
            asked.push(n);
            Ok(page(n as u64 * 1000, if n < 3 { PER_PAGE } else { 7 }))
        })
        .expect("a walk");
        assert_eq!(asked, [1, 2, 3]);
        assert_eq!(walked.records.len(), 2 * PER_PAGE + 7);
        assert!(walked.cut.is_none());

        let walked = walk(4, |n| Ok(page(n as u64 * 1000, PER_PAGE))).expect("a walk");
        assert_eq!(walked.records.len(), 4 * PER_PAGE);
        let cut = walked.cut.expect("a walk past its budget is cut");
        assert!(cut.contains("budget of 4 pages"), "{cut}");

        // A full last page then an empty one: covered.
        let walked = walk(PAGE_LIMIT, |n| {
            Ok(page(n as u64 * 1000, if n == 1 { PER_PAGE } else { 0 }))
        })
        .expect("a walk");
        assert!(walked.cut.is_none());
        assert_eq!(walked.records.len(), PER_PAGE);
    }

    #[test]
    fn a_record_answered_twice_reads_nothing() {
        let records = [
            record(1, "2026-10-08T00:00:00Z"),
            record(1, "2026-10-08T00:00:00Z"),
        ];
        let why = place(&records, &plan(&[]), &bounds(), &[]).expect_err("a duplicate");
        assert!(why.contains("twice"), "{why}");
    }

    #[test]
    fn a_record_that_cannot_be_placed_reads_nothing() {
        for at in [json!(null), json!("yesterday"), json!(17)] {
            let records = [json!({"id": 1, "created_at": at})];
            let why = place(&records, &plan(&[]), &bounds(), &[]).expect_err("no time");
            assert!(why.contains("created_at"), "{why}");
        }
        let records = [json!({"created_at": "2026-10-08T00:00:00Z"})];
        let why = place(&records, &plan(&[]), &bounds(), &[]).expect_err("no identity");
        assert!(why.contains("`id`"), "{why}");
    }

    #[test]
    fn a_dimension_field_holding_an_object_reads_nothing() {
        let records =
            [json!({"id": 1, "created_at": "2026-10-08T00:00:00Z", "ref": {"name": "main"}})];
        let why = place(&records, &plan(&[]), &bounds(), &["ref"]).expect_err("an object");
        assert!(why.contains("`ref`"), "{why}");
    }

    #[test]
    fn records_outside_the_window_are_not_placed_and_need_no_match_field() {
        let records = [
            json!({"id": 1, "created_at": "2026-10-07T22:57:03Z", "status": "failed"}),
            json!({"id": 2, "created_at": "2026-10-14T22:57:03Z"}),
            json!({"id": 3, "created_at": "2026-10-09T10:00:00+02:00", "status": "success"}),
        ];
        let placed =
            place(&records, &plan(&[("status", "failed")]), &bounds(), &[]).expect("placed");
        assert_eq!(
            placed.iter().map(|p| p.matches).collect::<Vec<_>>(),
            [true, false]
        );
    }

    #[test]
    fn a_null_on_the_match_path_is_a_value_and_an_absent_step_reads_nothing() {
        let plan = plan(&[("merge_user.username", "release-bot")]);
        let records = [
            json!({"id": 1, "created_at": "2026-10-08T00:00:00Z", "merge_user": null}),
            json!({"id": 2, "created_at": "2026-10-08T00:00:00Z", "merge_user": {"username": "release-bot"}}),
        ];
        let placed = place(&records, &plan, &bounds(), &[]).expect("placed");
        assert_eq!(
            placed.iter().map(|p| p.matches).collect::<Vec<_>>(),
            [false, true]
        );
        let records = [json!({"id": 3, "created_at": "2026-10-08T00:00:00Z", "merge_user": {}})];
        let why = place(&records, &plan, &bounds(), &[]).expect_err("an absent step");
        assert!(why.contains("merge_user.username"), "{why}");
    }

    #[test]
    fn quotient_is_exact_to_its_scale() {
        assert_eq!(quotient(2, 9), "0.222222222222222222");
        assert_eq!(quotient(1, 3), "0.333333333333333333");
        assert_eq!(quotient(2, 3), "0.666666666666666667");
        assert_eq!(quotient(1, 4), "0.25");
        assert_eq!(quotient(0, 7), "0");
        assert_eq!(quotient(5, 5), "1");
        assert_eq!(quotient(1, 5000), "0.0002");
    }
}
