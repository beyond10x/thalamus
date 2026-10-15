//! The adapter trait: one source kind turning Connectors answers into reads.
//!
//! An adapter answers one [`Read`] per series of a definition at one window end. It never builds
//! a sample: `since` and `age_s` of the denominator, the sample id and the rest of the provenance
//! are the pipeline's (design § 2.2, § 11 D18).

use std::collections::BTreeMap;

use sha2::{Digest, Sha256};
use thalamus_model::definition::{AdapterId, ConnectionId, QueryText, SignalDefinitionData, Unit};
use thalamus_model::observation::{QueryDigest, Reading, RunId, TimeWindow, UnreadReading};
use thalamus_model::primitives::{Duration, Timestamp};
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;

use crate::connectors::{ConnectorsError, ReadClient};

/// One source kind. The registry chooses it by the definition `extractor` prefix.
pub trait Adapter {
    /// The Connectors operations this adapter reads with. The client it is handed refuses every
    /// other operation before running anything.
    fn read_operations(&self) -> &[&str];

    /// The operations this adapter reads `definition` with: by default every operation of
    /// [`Adapter::read_operations`]. An adapter whose definition names the operation it reads
    /// (the HTTP probe) narrows it to that one, and to none when the name is not a read.
    fn operations_for<'a>(&'a self, definition: &'a SignalDefinitionData) -> Vec<&'a str> {
        let _ = definition;
        self.read_operations().to_vec()
    }

    /// One read per series of `definition` for the window ending at `end`.
    ///
    /// A [`ConnectorsError`] is not lost: the registry turns it into one `unknown` read naming
    /// Connectors' code and stage.
    fn read(
        &self,
        client: &ReadClient<'_>,
        definition: &SignalDefinitionData,
        end: &Timestamp,
        run_id: &RunId,
    ) -> Result<Vec<Read>, ConnectorsError>;
}

/// The population a reading was taken over, as the source counted it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Population {
    /// How many.
    pub n: i64,
    /// Of what: the unit of the definition's `denominator`.
    pub unit: Unit,
}

/// What an adapter read for one series.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Read {
    /// The series' dimension values, keyed by the definition's dimension keys. A key the source
    /// does not carry is absent.
    pub dims: BTreeMap<String, String>,
    /// The value and its quality; `unknown` carries a reason and no value.
    pub reading: Reading,
    /// The span the reading covers.
    pub window: TimeWindow,
    /// The population behind a ratio or count, when the source gave one.
    pub denominator: Option<Population>,
    /// The Connectors adapter instance read through.
    pub adapter: AdapterId,
    /// The Connectors connection read through; an id, never a credential.
    pub connection: ConnectionId,
    /// SHA-256 of the definition's query, as [`query_digest`] computes it.
    pub query_sha256: QueryDigest,
    /// The producer's own text of the scalar parsed into the reading, when there is one.
    pub raw: Option<String>,
    /// The earliest moment the source still holds, only when the source exposes one.
    pub retention_start: Option<Timestamp>,
}

/// An `unknown` read of `definition` for the window ending at `end`, saying `reason`.
///
/// It has no dims, no denominator, no raw text and no retention start. If the definition's window
/// cannot be placed before `end`, the window is the instant `end` and the reason says why.
pub fn unknown(
    definition: &SignalDefinitionData,
    end: &Timestamp,
    reason: impl Into<String>,
) -> Read {
    let mut reason = reason.into();
    let window = window_ending(&definition.window, end).unwrap_or_else(|why| {
        reason = format!("{reason}; {why}");
        TimeWindow {
            start: end.clone(),
            end: end.clone(),
        }
    });
    Read {
        dims: BTreeMap::new(),
        reading: Reading::Unknown(UnreadReading { reason }),
        window,
        denominator: None,
        adapter: definition.source.adapter.clone(),
        connection: definition.source.connection.clone(),
        query_sha256: query_digest(&definition.query),
        raw: None,
        retention_start: None,
    }
}

/// The digest of a definition's `query`: SHA-256 of its canonical JSON, an object of every name and
/// text, keys sorted, no whitespace, lower-case hex.
///
/// One rule for every map, one entry or many. Hashing a lone entry's bare text would let that
/// text, spelled as the canonical JSON of another map, share that map's digest; the canonical
/// JSON of two different maps never coincides.
pub fn query_digest(query: &BTreeMap<String, QueryText>) -> QueryDigest {
    let map: BTreeMap<&str, &str> = query
        .iter()
        .map(|(k, v)| (k.as_str(), v.0.as_str()))
        .collect();
    let text = serde_json::to_string(&map).unwrap_or_default();
    let digest = Sha256::digest(text.as_bytes());
    QueryDigest(digest.iter().map(|b| format!("{b:02x}")).collect())
}

/// The window `[end - window, end]`.
///
/// `end` is RFC 3339. `window` is an ISO 8601 duration of weeks (`P2W`) or of days, hours,
/// minutes and whole seconds (`P1DT6H`, `PT30M`); years and months are refused, their length
/// depending on the calendar.
pub fn window_ending(window: &Duration, end: &Timestamp) -> Result<TimeWindow, String> {
    let at = OffsetDateTime::parse(&end.0, &Rfc3339)
        .map_err(|e| format!("window end {:?} is not RFC 3339: {e}", end.0))?;
    let seconds = iso8601_seconds(&window.0).ok_or_else(|| {
        format!(
            "window {:?} is not an ISO 8601 duration of days, hours, minutes and seconds",
            window.0
        )
    })?;
    let start = at
        .checked_sub(time::Duration::seconds(seconds))
        .and_then(|start| start.format(&Rfc3339).ok())
        .ok_or_else(|| {
            format!(
                "window {:?} reaches before any representable instant",
                window.0
            )
        })?;
    Ok(TimeWindow {
        start: Timestamp(start),
        end: end.clone(),
    })
}

/// The length of an ISO 8601 duration in seconds; `None` for anything else, for years and months,
/// for fractions and for a duration with no component.
fn iso8601_seconds(text: &str) -> Option<i64> {
    let rest = text.strip_prefix('P')?;
    let (date, time) = match rest.split_once('T') {
        Some((date, time)) if !time.is_empty() => (date, Some(time)),
        Some(_) => return None,
        None => (rest, None),
    };
    let mut total: i64 = 0;
    let mut components = 0;
    let mut add = |part: &str, units: &[(char, i64)]| -> Option<()> {
        let mut number = String::new();
        let mut allowed = units.iter();
        for c in part.chars() {
            if c.is_ascii_digit() {
                number.push(c);
                continue;
            }
            let (_, scale) = allowed.by_ref().find(|(unit, _)| *unit == c)?;
            let n: i64 = number.parse().ok()?;
            total = total.checked_add(n.checked_mul(*scale)?)?;
            number.clear();
            components += 1;
        }
        number.is_empty().then_some(())
    };
    add(date, &[('W', 604_800), ('D', 86_400)])?;
    if let Some(time) = time {
        add(time, &[('H', 3_600), ('M', 60), ('S', 1)])?;
    }
    (components > 0).then_some(total)
}
