//! Series keys (design § 11 D10) and the admission of a sample's `dims` (design § 2.4).
//!
//! A series is a signal id together with its dims values. Its key is
//! `<signal-id>{k1="v1",k2="v2"}`, keys ascending, values escaped as `OpenMetrics` label values;
//! a sample with no dims has the bare id as its key.
//!
//! `dims` carry only keys the instance registered and the definition declared, and only values
//! the key's registry lists: a `file:` registry's contents are passed in, an `open` one admits any
//! value. A `connectors:` registry never loads (design § 11 D17), so nothing is admitted through
//! one. Pure: no I/O.

use std::collections::BTreeMap;
use std::fmt;

use thalamus_model::definition::{SignalDefinitionData, SignalId};
use thalamus_model::dimension::{DimensionValue, ValueSource};
use thalamus_model::observation::SeriesKey;

use crate::registry::{FileValues, Registry};

/// The series key of `signal` with `dims` (D10).
#[must_use]
pub fn series_key(signal: &SignalId, dims: &BTreeMap<String, DimensionValue>) -> SeriesKey {
    let mut key = signal.0.clone();
    if !dims.is_empty() {
        key.push('{');
        for (index, (name, value)) in dims.iter().enumerate() {
            if index > 0 {
                key.push(',');
            }
            key.push_str(name);
            key.push_str("=\"");
            escape_label_value(&mut key, &value.0);
            key.push('"');
        }
        key.push('}');
    }
    SeriesKey(key)
}

/// `value` appended as an `OpenMetrics` label value: `\` as `\\`, `"` as `\"`, a line feed as `\n`.
fn escape_label_value(out: &mut String, value: &str) {
    for character in value.chars() {
        match character {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            other => out.push(other),
        }
    }
}

/// Why a sample's `dims` are not admitted.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DimsRefusal {
    /// The key is not in the instance's dimension registry.
    UnregisteredKey {
        /// The key.
        key: String,
    },
    /// The key is registered, but the definition does not declare it.
    UndeclaredKey {
        /// The key.
        key: String,
    },
    /// The value is not listed in the key's `file:` registry.
    UnlistedValue {
        /// The key.
        key: String,
        /// The value.
        value: String,
        /// The registry file.
        locator: String,
    },
    /// The key's registry is a file whose contents the caller did not pass, so no value of it
    /// can be shown to be listed.
    ValuesNotGiven {
        /// The key.
        key: String,
        /// The registry file.
        locator: String,
    },
    /// The key's registry is a Connectors inventory, which no value resolves in (D17).
    ConnectorsRegistry {
        /// The key.
        key: String,
        /// The inventory locator.
        locator: String,
    },
}

impl fmt::Display for DimsRefusal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DimsRefusal::UnregisteredKey { key } => write!(
                f,
                "dims key `{key}` is not registered in the instance's dimension registry"
            ),
            DimsRefusal::UndeclaredKey { key } => write!(
                f,
                "dims key `{key}` is registered, but the definition does not declare it"
            ),
            DimsRefusal::UnlistedValue {
                key,
                value,
                locator,
            } => write!(
                f,
                "dims value `{value}` of `{key}` is not listed in the registry file {locator}"
            ),
            DimsRefusal::ValuesNotGiven { key, locator } => write!(
                f,
                "the values of `{key}` are listed in {locator}, whose contents were not given"
            ),
            DimsRefusal::ConnectorsRegistry { key, locator } => write!(
                f,
                "`{key}` resolves in connectors:{locator}, and Connectors offers no inventory \
                 operation to resolve it (design § 11 D17)"
            ),
        }
    }
}

impl std::error::Error for DimsRefusal {}

/// Admits `dims` for a sample of `definition`: every key registered in `registry` and declared
/// by the definition, every value listed in its `file:` registry (`file_values`, the parsed
/// contents by dimension key) or admitted by `open`.
///
/// # Errors
///
/// The first [`DimsRefusal`] in key order.
pub fn admit_dims(
    registry: &Registry,
    definition: &SignalDefinitionData,
    dims: &BTreeMap<String, DimensionValue>,
    file_values: &BTreeMap<String, FileValues>,
) -> Result<(), DimsRefusal> {
    let declared = definition.dimensions.as_deref().unwrap_or_default();
    for (key, value) in dims {
        let Some(dimension) = registry.dimension(key) else {
            return Err(DimsRefusal::UnregisteredKey { key: key.clone() });
        };
        if !declared.iter().any(|declared| declared.0 == *key) {
            return Err(DimsRefusal::UndeclaredKey { key: key.clone() });
        }
        match &dimension.values {
            ValueSource::Open => {}
            ValueSource::File(locator) => {
                let Some(values) = file_values.get(key) else {
                    return Err(DimsRefusal::ValuesNotGiven {
                        key: key.clone(),
                        locator: locator.0.clone(),
                    });
                };
                if !values.contains(&value.0) {
                    return Err(DimsRefusal::UnlistedValue {
                        key: key.clone(),
                        value: value.0.clone(),
                        locator: locator.0.clone(),
                    });
                }
            }
            ValueSource::Connectors(locator) => {
                return Err(DimsRefusal::ConnectorsRegistry {
                    key: key.clone(),
                    locator: locator.0.clone(),
                });
            }
        }
    }
    Ok(())
}
