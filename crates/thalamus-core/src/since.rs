//! `denominator.since`: the earliest moment a series' population could contain anything (design
//! § 2.2), from one of the three origins design § 11 D18 admits.
//!
//! | form | origin |
//! |---|---|
//! | `first_sample` | the timestamp of the first stored sample of the series |
//! | an RFC 3339 timestamp | that timestamp |
//! | `<key>.created` | the `created` of the sample's value of `<key>` in its `file:` registry |
//!
//! `dimension.created`, the design's own spelling, names the definition's one declared dimension
//! whose registry is `file:`. Whatever the form, the origin is never earlier than the source's
//! retention start, because the source cannot be read before it. An origin that cannot be had
//! resolves to none, so `age_s` is unknown and an offset baseline is `UNKNOWN`, never 0. Pure:
//! the caller passes the first stored sample time and the retention start in.

use std::collections::BTreeMap;
use std::fmt;

use thalamus_model::dimension::{DimensionKey, DimensionValue, ValueSource};
use thalamus_model::primitives::Timestamp;

use crate::behaviour::observation::Instant;
use crate::registry::{Definition, FileValues, Registry};

/// The forms `denominator.since` admits, as every refusal of one names them.
pub const ADMITTED_FORMS: &str = "denominator.since admits first_sample, a fixed RFC 3339 \
     timestamp, or <key>.created for a declared dimension whose registry is file: \
     (dimension.created when exactly one is)";

/// The suffix of the dimension form.
const CREATED: &str = ".created";

/// The design's spelling of the dimension form, naming no key.
const SHORTHAND: &str = "dimension";

/// A `denominator.since` as it was admitted at load.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Since {
    /// The first stored sample of the series.
    FirstSample,
    /// A fixed instant.
    Fixed(Timestamp),
    /// The `created` of the sample's value of `key` in its `file:` registry.
    DimensionCreated {
        /// The dimension key.
        key: String,
    },
}

/// Why a `denominator.since` is refused at load.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SinceRefusal {
    /// None of the three forms.
    Form {
        /// The text.
        value: String,
    },
    /// `<key>.created` names a dimension the definition does not declare.
    NotDeclared {
        /// The key.
        key: String,
    },
    /// `<key>.created` names a dimension whose registry is not `file:`, so no entry of it carries
    /// `created`.
    NotFile {
        /// The key.
        key: String,
        /// Its registry kind: `open`, or `connectors`.
        registry: &'static str,
    },
    /// `dimension.created`, and more than one declared dimension has a `file:` registry.
    Ambiguous {
        /// The declared keys whose registry is `file:`.
        keys: Vec<String>,
    },
    /// `dimension.created`, and no declared dimension has a `file:` registry.
    NoFileDimension,
}

impl fmt::Display for SinceRefusal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SinceRefusal::Form { value } => write!(f, "`{value}` is none of the admitted forms"),
            SinceRefusal::NotDeclared { key } => {
                write!(
                    f,
                    "`{key}{CREATED}` names a dimension the definition does not declare"
                )
            }
            SinceRefusal::NotFile { key, registry } => write!(
                f,
                "`{key}{CREATED}` names a dimension whose registry is {registry}; only a file: \
                 registry entry carries created"
            ),
            SinceRefusal::Ambiguous { keys } => write!(
                f,
                "`{SHORTHAND}{CREATED}` is ambiguous: declared dimensions [{}] all have a file: \
                 registry; name one as <key>{CREATED}",
                keys.join(", ")
            ),
            SinceRefusal::NoFileDimension => write!(
                f,
                "`{SHORTHAND}{CREATED}` names no dimension: no declared dimension has a file: \
                 registry"
            ),
        }?;
        write!(f, "; {ADMITTED_FORMS}")
    }
}

impl std::error::Error for SinceRefusal {}

/// The [`Since`] a `denominator.since` text names, for a definition declaring `declared` over
/// `registry`.
///
/// # Errors
///
/// A [`SinceRefusal`] for any text outside the three forms D18 admits.
pub fn parse_since(
    text: &str,
    declared: Option<&[DimensionKey]>,
    registry: &Registry,
) -> Result<Since, SinceRefusal> {
    if text == "first_sample" {
        return Ok(Since::FirstSample);
    }
    if Instant::parse(text).is_some() {
        return Ok(Since::Fixed(Timestamp(text.to_owned())));
    }
    let Some(name) = text.strip_suffix(CREATED) else {
        return Err(SinceRefusal::Form {
            value: text.to_owned(),
        });
    };
    let declared = declared.unwrap_or_default();
    let is_declared = |key: &str| declared.iter().any(|declared| declared.0 == key);
    if name == SHORTHAND && !is_declared(name) {
        let keys: Vec<String> = declared
            .iter()
            .filter(|key| {
                registry
                    .dimension(&key.0)
                    .is_some_and(|dimension| matches!(dimension.values, ValueSource::File(_)))
            })
            .map(|key| key.0.clone())
            .collect();
        return match keys.as_slice() {
            [] => Err(SinceRefusal::NoFileDimension),
            [key] => Ok(Since::DimensionCreated { key: key.clone() }),
            _ => Err(SinceRefusal::Ambiguous { keys }),
        };
    }
    if name.is_empty() || name.contains('.') {
        return Err(SinceRefusal::Form {
            value: text.to_owned(),
        });
    }
    let not_declared = || SinceRefusal::NotDeclared {
        key: name.to_owned(),
    };
    if !is_declared(name) {
        return Err(not_declared());
    }
    match registry.dimension(name).map(|dimension| &dimension.values) {
        Some(ValueSource::File(_)) => Ok(Since::DimensionCreated {
            key: name.to_owned(),
        }),
        Some(ValueSource::Open) => Err(SinceRefusal::NotFile {
            key: name.to_owned(),
            registry: "open",
        }),
        Some(ValueSource::Connectors(_)) => Err(SinceRefusal::NotFile {
            key: name.to_owned(),
            registry: "connectors",
        }),
        None => Err(not_declared()),
    }
}

/// The origin of the series of `definition` with `dims`: its admitted [`Since`] resolved, and
/// never earlier than `retention_start` when the adapter reported one.
///
/// `registry_values` holds the parsed `file:` registries by dimension key; `first_stored` is the
/// timestamp of the first stored sample of the series, `None` when none is stored yet.
///
/// `None` when the definition declares no `since`, when no sample is stored yet for
/// `first_sample`, when the sample's value has no entry carrying `created`, or when an instant
/// involved names no RFC 3339 instant.
#[must_use]
pub fn resolve_since(
    definition: &Definition,
    dims: &BTreeMap<String, DimensionValue>,
    registry_values: &BTreeMap<String, FileValues>,
    first_stored: Option<Timestamp>,
    retention_start: Option<Timestamp>,
) -> Option<Timestamp> {
    let origin = match definition.since()? {
        Since::FirstSample => first_stored?,
        Since::Fixed(at) => at.clone(),
        Since::DimensionCreated { key } => {
            let value = dims.get(key)?;
            registry_values.get(key)?.created(&value.0)?.clone()
        }
    };
    let at = Instant::parse(&origin.0)?;
    match retention_start {
        None => Some(origin),
        Some(start) => {
            if at < Instant::parse(&start.0)? {
                Some(start)
            } else {
                Some(origin)
            }
        }
    }
}
