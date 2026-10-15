//! An instance's definition files and dimension registry (design § 2.1, § 2.4, § 4, § 5.2): parsed,
//! validated, and served read-only through the generated storage ports.
//!
//! Pure: the caller passes each file's name and contents in. Every refusal is a [`LoadError`]
//! naming the file and the [`Rule`] it breaks. The rules the specification leaves to this crate
//! (`UNMAPPED:` in `spec/`) are held here: the signal id grammar, registered dimension keys, the
//! UCUM grammar of every unit, and the `dims` of a sample (`crate::series::admit_dims`). The
//! decisions of design § 11 are applied as written: an empty `dimensions` list declares zero
//! dimensions and an absent one is undeclared (D8); a `connectors:` registry is refused (D17);
//! `denominator.since` admits three forms (D18, `crate::since`); the files are the one copy of a
//! definition, so the storage ports refuse every write (D19). A definition whose baseline
//! `crate::baseline::plan` refuses is refused at load, so it never loads `Active` and fails every
//! pass.

use std::collections::btree_map::Entry;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use serde::Deserialize;
use thalamus_model::behaviour::{
    DimensionRegistryStorage, DimensionStorage, SignalDefinitionStorage,
};
use thalamus_model::definition::{
    AdapterId, Author, BaselineKind, ConnectionId, DenominatorSpec, EvaluationSpec, ExtractorId,
    Family, History, Links, Owner, Persistence, QueryText, RuleId, SignalDefinitionData,
    SignalDefinitionSnapshot, SignalDefinitionState, SignalId, SignalKind, Source, Threshold, Unit,
};
use thalamus_model::dimension::{
    DimensionData, DimensionKey, DimensionRegistryData, DimensionRegistrySnapshot,
    DimensionRegistryState, DimensionRule, DimensionSnapshot, DimensionState, Locator, RegistryId,
    ValueSource,
};
use thalamus_model::json::Value as Json;
use thalamus_model::observation::{
    CompleteReading, Provenance, QueryDigest, Reading, RunId, SampleId, SeriesKey,
    SignalSampleData, TimeWindow,
};
use thalamus_model::primitives::{Decimal, Duration, Timestamp};

use crate::baseline::{self, Refusal};
use crate::behaviour::definition::{InputRefusal, input_refusals};
use crate::behaviour::observation::Instant;
use crate::since::{Since, SinceRefusal, parse_since};
use crate::unit::parse_ucum;

const LOWER: &str = "abcdefghijklmnopqrstuvwxyz0123456789_";
const REGISTRY_ID: &str = "abcdefghijklmnopqrstuvwxyz0123456789_-";
const CONNECTION: &str = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789_-.:/";

const SIGNAL_DEFINITION: &str = "thalamus.definition.SignalDefinition";
const DIMENSION: &str = "thalamus.dimension.Dimension";
const DIMENSION_REGISTRY: &str = "thalamus.dimension.DimensionRegistry";

/// The invariant a `Muted` definition and every other one keep about a mute.
const MUTE_INVARIANT: &str = "a Muted definition carries mute_reason and mute_until, and a \
                              definition in any other state carries neither";

/// The rule of the spec's first `SignalDefinition` invariant.
const REV_INVARIANT: &str = "rev >= 1";

/// One refusal at load: the file and the rule it breaks.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoadError {
    /// The file, as the caller named it.
    pub file: String,
    /// The rule.
    pub rule: Rule,
}

impl fmt::Display for LoadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}: {}", self.file, self.rule.name(), self.rule)
    }
}

impl std::error::Error for LoadError {}

/// A rule an instance file breaks.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Rule {
    /// The file is not a document of the expected shape.
    Syntax {
        /// What the parser said.
        message: String,
    },
    /// A value outside its type's character set.
    Alphabet {
        /// The field.
        field: &'static str,
        /// The value.
        value: String,
        /// The characters the type admits.
        alphabet: &'static str,
    },
    /// The id is not `<family>.<layer>.<name>`.
    SignalIdGrammar {
        /// The id.
        id: String,
    },
    /// An `Active` or `Muted` definition leaves `unit`, `dimensions`, `denominator` or `owner`
    /// undeclared (`thalamus.definition.IncompleteDefinition`).
    Incomplete {
        /// The definition.
        signal_id: String,
        /// The fields missing, in that order.
        missing: Vec<&'static str>,
    },
    /// Another invariant of `thalamus.definition.SignalDefinition`.
    Invariant {
        /// The invariant.
        invariant: &'static str,
    },
    /// A unit that is not UCUM.
    NotUcum {
        /// `unit` or `denominator.unit`.
        field: &'static str,
        /// The unit.
        unit: String,
        /// Why `parse_ucum` refused it.
        error: String,
    },
    /// A ratio whose unit is not `1`.
    RatioUnit {
        /// The unit.
        unit: String,
    },
    /// A signal of kind `instant` declares a baseline other than `fixed`
    /// (`thalamus.definition.InstantNeedsFixedBaseline`).
    InstantNeedsFixed {
        /// The definition.
        signal_id: String,
    },
    /// An input-guarded refusal `ProposeDefinition` and `ReviseDefinition` share, other than
    /// `instant-not-fixed` ([`Rule::InstantNeedsFixed`]): an N or a fixed number missing, below 1
    /// or unread, or a persistence below 1 (design § 11 D25, D29).
    Input {
        /// The definition.
        signal_id: String,
        /// Which refusal.
        refusal: InputRefusal,
    },
    /// `crate::baseline::plan` refuses the definition's baseline.
    Baseline {
        /// The refusal.
        refusal: Refusal,
    },
    /// `dimensions` names a key the registry does not hold.
    UnregisteredDimension {
        /// The key.
        key: String,
    },
    /// `dimensions` names a key twice.
    DuplicateDimension {
        /// The key.
        key: String,
    },
    /// `source.connection` is not a Connectors connection id.
    ConnectionNotIdentifier {
        /// The connection.
        connection: String,
    },
    /// A dimension's registry is a Connectors inventory (design § 11 D17).
    ConnectorsRegistry {
        /// The key.
        key: String,
        /// The inventory locator.
        locator: String,
    },
    /// A dimension's registry is none of `file:<locator>`, `open` or `connectors:<locator>`.
    RegistryForm {
        /// The key.
        key: String,
        /// The value.
        value: String,
    },
    /// A dimension's parent is not registered (`thalamus.dimension.UnknownParent`).
    UnknownParent {
        /// The key.
        key: String,
        /// The parent named.
        parent: String,
    },
    /// `denominator.since` is none of the forms of design § 11 D18.
    Since {
        /// The text.
        value: String,
        /// Why.
        refusal: SinceRefusal,
    },
    /// A duration that is neither `<n>s|m|h|d|w` nor ISO 8601.
    Duration {
        /// The field.
        field: &'static str,
        /// The value.
        value: String,
    },
    /// A number that is not a decimal.
    Decimal {
        /// The field.
        field: &'static str,
        /// The value.
        value: String,
    },
    /// A timestamp that is not RFC 3339.
    Timestamp {
        /// The field.
        field: &'static str,
        /// The value.
        value: String,
    },
    /// Two definition files share an id (`thalamus.definition.DefinitionExists`).
    DuplicateDefinition {
        /// The id.
        signal_id: String,
    },
    /// A registry file lists one value twice.
    DuplicateValue {
        /// The value.
        value: String,
    },
    /// The id's `<family>` segment is not the definition's `family`.
    IdFamily {
        /// The id.
        id: String,
        /// The `family` field.
        family: String,
    },
    /// A dimension's `parent` chain reaches the dimension itself.
    ParentCycle {
        /// The key.
        key: String,
    },
    /// A `file:` locator that is not a relative path inside the instance directory.
    LocatorOutsideInstance {
        /// The key.
        key: String,
        /// The locator.
        locator: String,
    },
    /// An `objective` that is not JSON the model reads back.
    Objective {
        /// Why.
        reason: String,
    },
}

impl Rule {
    /// The rule's name, as every error spells it.
    #[must_use]
    pub fn name(&self) -> &'static str {
        match self {
            Rule::Syntax { .. } => "document shape",
            Rule::Alphabet { .. } => "character set",
            Rule::SignalIdGrammar { .. } => "signal id grammar",
            Rule::Incomplete { .. } => "thalamus.definition.IncompleteDefinition",
            Rule::Invariant { .. } => "thalamus.definition.SignalDefinition invariant",
            Rule::NotUcum { .. } => "UCUM unit",
            Rule::RatioUnit { .. } => "ratio unit",
            Rule::InstantNeedsFixed { .. } => "thalamus.definition.InstantNeedsFixedBaseline",
            Rule::Input { refusal, .. } => match refusal {
                InputRefusal::InstantNotFixed => "thalamus.definition.InstantNeedsFixedBaseline",
                InputRefusal::RollingNMissing => "thalamus.definition.RollingWindowMissing",
                InputRefusal::RollingNUnused => "thalamus.definition.RollingWindowUnused",
                InputRefusal::FixedNumberMissing => "thalamus.definition.FixedNumberMissing",
                InputRefusal::FixedNumberUnused => "thalamus.definition.FixedNumberUnused",
                InputRefusal::PersistenceBelowOne => "thalamus.definition.PersistenceTooSmall",
            },
            Rule::Baseline { .. } => "baseline plan",
            Rule::UnregisteredDimension { .. } => "registered dimension",
            Rule::DuplicateDimension { .. } => "distinct dimensions",
            Rule::ConnectionNotIdentifier { .. } => "connection id",
            Rule::ConnectorsRegistry { .. } => "D17 connectors registry",
            Rule::RegistryForm { .. } => "registry form",
            Rule::UnknownParent { .. } => "thalamus.dimension.UnknownParent",
            Rule::Since { .. } => "D18 denominator.since",
            Rule::Duration { .. } => "duration",
            Rule::Decimal { .. } => "decimal",
            Rule::Timestamp { .. } => "timestamp",
            Rule::DuplicateDefinition { .. } => "thalamus.definition.DefinitionExists",
            Rule::DuplicateValue { .. } => "distinct registry values",
            Rule::IdFamily { .. } => "id family",
            Rule::ParentCycle { .. } => "parent chain",
            Rule::LocatorOutsideInstance { .. } => "file locator",
            Rule::Objective { .. } => "objective JSON",
        }
    }
}

impl fmt::Display for Rule {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Rule::Syntax { message } => f.write_str(message),
            Rule::Alphabet {
                field,
                value,
                alphabet,
            } => write!(
                f,
                "{field} `{value}` is empty or has a character outside `{alphabet}`"
            ),
            Rule::SignalIdGrammar { id } => write!(
                f,
                "id `{id}` is not <family>.<layer>.<name>: three non-empty segments of \
                 `{LOWER}`, joined by dots"
            ),
            Rule::Incomplete { signal_id, missing } => write!(
                f,
                "an Active or Muted definition declares unit, dimensions, denominator and owner; \
                 `{signal_id}` does not declare {}",
                missing.join(", ")
            ),
            Rule::Invariant { invariant } => f.write_str(invariant),
            Rule::NotUcum { field, unit, error } => {
                write!(f, "{field} `{unit}` is not a UCUM unit: {error}")
            }
            Rule::RatioUnit { unit } => {
                write!(
                    f,
                    "a signal of kind ratio has unit 1; this one has `{unit}`"
                )
            }
            Rule::InstantNeedsFixed { signal_id } => write!(
                f,
                "`{signal_id}` is of kind instant and is graded against fixed only; it declares \
                 another baseline"
            ),
            Rule::Input { signal_id, refusal } => match refusal {
                InputRefusal::InstantNotFixed => write!(
                    f,
                    "`{signal_id}` is of kind instant and is graded against fixed only"
                ),
                InputRefusal::RollingNMissing => write!(
                    f,
                    "`{signal_id}` names rolling_median without an evaluation.rolling_n of at \
                     least 1"
                ),
                InputRefusal::RollingNUnused => write!(
                    f,
                    "`{signal_id}` declares evaluation.rolling_n and no rolling_median baseline \
                     reads it"
                ),
                InputRefusal::FixedNumberMissing => write!(
                    f,
                    "`{signal_id}` names a fixed baseline without evaluation.fixed"
                ),
                InputRefusal::FixedNumberUnused => write!(
                    f,
                    "`{signal_id}` declares evaluation.fixed and no fixed baseline reads it"
                ),
                InputRefusal::PersistenceBelowOne => write!(
                    f,
                    "`{signal_id}` declares a persistence k, k_crit or m below 1"
                ),
            },
            Rule::Baseline { refusal } => write!(f, "the baseline is refused: {refusal}"),
            Rule::UnregisteredDimension { key } => write!(
                f,
                "dimension `{key}` is not registered in the instance's dimension registry"
            ),
            Rule::DuplicateDimension { key } => {
                write!(f, "dimension `{key}` is declared twice")
            }
            Rule::ConnectionNotIdentifier { connection } => write!(
                f,
                "source.connection `{connection}` is not a Connectors connection id: an \
                 identifier of `{CONNECTION}`, never a URL, a path or a credential"
            ),
            Rule::ConnectorsRegistry { key, locator } => write!(
                f,
                "dimension `{key}` names registry connectors:{locator}; Connectors offers no \
                 inventory operation to resolve it, so only file: and open registries load"
            ),
            Rule::RegistryForm { key, value } => write!(
                f,
                "dimension `{key}` names registry `{value}`; a registry is file:<locator> or open"
            ),
            Rule::UnknownParent { key, parent } => write!(
                f,
                "dimension `{key}` names parent `{parent}`, which is not registered"
            ),
            Rule::Since { value, refusal } => {
                write!(f, "denominator.since `{value}` is refused: {refusal}")
            }
            Rule::Duration { field, value } => write!(
                f,
                "{field} `{value}` is not a length a window can have: <n>s, <n>m, <n>h, <n>d, \
                 <n>w or an ISO 8601 duration of weeks, days, hours, minutes and seconds, more \
                 than zero, with no years or months, each unit at most once, and at most \
                 719528 days, so that a window of it ending in 1970 starts no earlier than \
                 0000-01-01T00:00:00Z"
            ),
            Rule::Decimal { field, value } => {
                write!(f, "{field} `{value}` is not a decimal number")
            }
            Rule::Timestamp { field, value } => {
                write!(f, "{field} `{value}` is not an RFC 3339 timestamp")
            }
            Rule::DuplicateDefinition { signal_id } => {
                write!(f, "`{signal_id}` is defined in another file already")
            }
            Rule::DuplicateValue { value } => write!(f, "value `{value}` is listed twice"),
            Rule::IdFamily { id, family } => write!(
                f,
                "id `{id}` is <family>.<layer>.<name>, and its first segment is not the family \
                 `{family}`"
            ),
            Rule::ParentCycle { key } => {
                write!(
                    f,
                    "dimension `{key}` reaches itself through its parent chain"
                )
            }
            Rule::LocatorOutsideInstance { key, locator } => write!(
                f,
                "dimension `{key}` names file:{locator}; a file registry is a relative path \
                 naming a file inside the instance directory: no leading /, no empty, . or .. \
                 segment, no scheme or drive, no backslash"
            ),
            Rule::Objective { reason } => write!(f, "objective {reason}"),
        }
    }
}

/// Collects the refusals of one file.
struct Errors<'a> {
    file: &'a str,
    errors: Vec<LoadError>,
}

impl<'a> Errors<'a> {
    fn new(file: &'a str) -> Self {
        Self {
            file,
            errors: Vec::new(),
        }
    }

    fn push(&mut self, rule: Rule) {
        self.errors.push(LoadError {
            file: self.file.to_owned(),
            rule,
        });
    }

    fn alphabet(&mut self, field: &'static str, value: &str, alphabet: &'static str) {
        if !in_alphabet(value, alphabet) {
            self.push(Rule::Alphabet {
                field,
                value: value.to_owned(),
                alphabet,
            });
        }
    }

    fn timestamp(&mut self, field: &'static str, value: &str) -> Timestamp {
        if Instant::parse(value).is_none() {
            self.push(Rule::Timestamp {
                field,
                value: value.to_owned(),
            });
        }
        Timestamp(value.to_owned())
    }

    fn duration(&mut self, field: &'static str, value: &str) -> Duration {
        match iso_duration(value) {
            Some((iso, seconds)) if placeable(seconds) => Duration(iso),
            _ => {
                self.push(Rule::Duration {
                    field,
                    value: value.to_owned(),
                });
                Duration(value.to_owned())
            }
        }
    }

    fn decimal(&mut self, field: &'static str, value: Option<String>) -> Option<Decimal> {
        let value = value?;
        if !is_decimal(&value) {
            self.push(Rule::Decimal {
                field,
                value: value.clone(),
            });
        }
        Some(Decimal(value))
    }

    fn finish<T>(self, value: T) -> Result<T, Vec<LoadError>> {
        if self.errors.is_empty() {
            Ok(value)
        } else {
            Err(self.errors)
        }
    }
}

fn syntax(file: &str, error: &serde_yaml_ng::Error) -> Vec<LoadError> {
    vec![LoadError {
        file: file.to_owned(),
        rule: Rule::Syntax {
            message: error.to_string(),
        },
    }]
}

fn in_alphabet(value: &str, alphabet: &str) -> bool {
    !value.is_empty() && value.chars().all(|character| alphabet.contains(character))
}

/// The wire contract's decimal (`generated/thalamus-model/src/json.rs`, `decimal_at`):
/// `-?digits(.digits)?`, the whole part without a leading zero. `fixed` and every threshold are
/// read by this one rule, so a file admits no number the wire or `baseline::fold` refuses.
fn is_decimal(text: &str) -> bool {
    let unsigned = text.strip_prefix('-').unwrap_or(text);
    let (whole, fraction) = match unsigned.split_once('.') {
        Some((whole, fraction)) => (whole, Some(fraction)),
        None => (unsigned, None),
    };
    let digits = |part: &str| !part.is_empty() && part.bytes().all(|byte| byte.is_ascii_digit());
    digits(whole) && !(whole.len() > 1 && whole.starts_with('0')) && fraction.is_none_or(digits)
}

/// The components of one admitted duration: weeks, days, hours, minutes, seconds, each as
/// written or absent.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
struct Components {
    weeks: Option<i64>,
    days: Option<i64>,
    hours: Option<i64>,
    minutes: Option<i64>,
    seconds: Option<i64>,
}

impl Components {
    /// The length in seconds, or `None` past `i64`.
    fn total(self) -> Option<i64> {
        [
            (self.weeks, 604_800),
            (self.days, 86_400),
            (self.hours, 3_600),
            (self.minutes, 60),
            (self.seconds, 1),
        ]
        .into_iter()
        .try_fold(0i64, |total, (count, scale)| {
            total.checked_add(count.unwrap_or(0).checked_mul(scale)?)
        })
    }

    /// The canonical RFC 3339 Appendix A `duration` of these components, the same length:
    /// weeks alone stay `P<n>W`; weeks beside any other unit become days (`P1W2D` is `P9D`);
    /// the time part runs from its first unit to its last with no unit skipped (`PT1H1S` is
    /// `PT1H0M1S`); every number is written without leading zeros. `None` past `i64` days.
    fn canonical(self) -> Option<String> {
        let time = [self.hours, self.minutes, self.seconds];
        let has_time = time.iter().any(Option::is_some);
        if let (Some(weeks), None, false) = (self.weeks, self.days, has_time) {
            return Some(format!("P{weeks}W"));
        }
        let mut out = String::from("P");
        let days = match (self.weeks, self.days) {
            (None, None) => None,
            (weeks, days) => Some(
                weeks
                    .unwrap_or(0)
                    .checked_mul(7)?
                    .checked_add(days.unwrap_or(0))?,
            ),
        };
        if let Some(days) = days {
            out.push_str(&format!("{days}D"));
        }
        if has_time {
            out.push('T');
            let first = time.iter().position(Option::is_some)?;
            let last = time.iter().rposition(Option::is_some)?;
            for (index, designator) in ['H', 'M', 'S'].into_iter().enumerate() {
                if (first..=last).contains(&index) {
                    out.push_str(&format!("{}{designator}", time[index].unwrap_or(0)));
                }
            }
        }
        Some(out)
    }
}

/// `text` as a stored `Duration` and its length in seconds.
///
/// Admitted: the short form `<n>s|m|h|d|w` of design § 2.1 (`6h`), and an ISO 8601 duration of
/// whole weeks, days, hours, minutes and seconds, each unit at most once and in that order: the
/// grammar `thalamus_sources::adapter::window_ending` places a window with. Years and months are
/// refused, their length depending on the calendar, and so is a length past `i64` seconds. Zero
/// and placement are [`placeable`]'s.
///
/// Stored: the canonical RFC 3339 Appendix A `duration` ([`Components::canonical`]), the
/// `format: duration` the generated schemas give every `Duration`, of the same length.
fn iso_duration(text: &str) -> Option<(String, i64)> {
    let number = |digits: &str| -> Option<i64> {
        if digits.is_empty() || !digits.bytes().all(|byte| byte.is_ascii_digit()) {
            return None;
        }
        digits.parse().ok()
    };
    let mut parsed = Components::default();
    if let Some(unit) = text.chars().last().filter(char::is_ascii_lowercase) {
        let count = number(&text[..text.len() - 1])?;
        match unit {
            's' => parsed.seconds = Some(count),
            'm' => parsed.minutes = Some(count),
            'h' => parsed.hours = Some(count),
            'd' => parsed.days = Some(count),
            'w' => parsed.weeks = Some(count),
            _ => return None,
        }
    } else {
        let rest = text.strip_prefix('P')?;
        let (date, time) = match rest.split_once('T') {
            Some((date, time)) => (date, Some(time)),
            None => (rest, None),
        };
        let mut read = |part: &str, units: &[u8]| -> Option<usize> {
            let mut count = 0;
            let mut next = 0;
            let mut digits = String::new();
            for byte in part.bytes() {
                if byte.is_ascii_digit() {
                    digits.push(char::from(byte));
                    continue;
                }
                let at = next + units[next..].iter().position(|&unit| unit == byte)?;
                let value = Some(number(&digits)?);
                match units[at] {
                    b'W' => parsed.weeks = value,
                    b'D' => parsed.days = value,
                    b'H' => parsed.hours = value,
                    b'M' => parsed.minutes = value,
                    _ => parsed.seconds = value,
                }
                next = at + 1;
                digits.clear();
                count += 1;
            }
            digits.is_empty().then_some(count)
        };
        let date_count = read(date, b"WD")?;
        let time_count = match time {
            Some(time) => match read(time, b"HMS")? {
                0 => return None,
                count => count,
            },
            None => 0,
        };
        if date_count + time_count == 0 {
            return None;
        }
    }
    let seconds = parsed.total()?;
    Some((parsed.canonical()?, seconds))
}

/// Seconds from 0000-01-01T00:00:00Z to 1970-01-01T00:00:00Z: 719 528 days.
const SECONDS_BEFORE_1970: i64 = 719_528 * 86_400;

/// A length a definition can mean: more than zero, and short enough that a window of it ending
/// at any instant from 1970 on starts no earlier than 0000-01-01T00:00:00Z, the first instant
/// `window_ending` can write (an RFC 3339 year has four digits).
fn placeable(seconds: i64) -> bool {
    seconds > 0 && seconds <= SECONDS_BEFORE_1970
}

/// `<family>.<layer>.<name>`: three non-empty segments of lower-case letters, digits and `_`.
fn is_signal_id(id: &str) -> bool {
    let segments: Vec<&str> = id.split('.').collect();
    segments.len() == 3 && segments.iter().all(|segment| in_alphabet(segment, LOWER))
}

/// The URL schemes a connection id never starts with, compared in any case: the WHATWG URL
/// standard's special schemes, which a parser reads as a URL even without `//`.
const URL_SCHEMES: [&str; 6] = ["http:", "https:", "file:", "ftp:", "ws:", "wss:"];

/// A Connectors connection id: the spec's character set, and no URL or path form: no `:/`
/// anywhere, no leading `/`, and none of [`URL_SCHEMES`] as a prefix in any case.
fn is_connection_id(connection: &str) -> bool {
    let lower = connection.to_ascii_lowercase();
    in_alphabet(connection, CONNECTION)
        && !connection.contains(":/")
        && !connection.starts_with('/')
        && !URL_SCHEMES.iter().any(|scheme| lower.starts_with(scheme))
}

/// A `file:` locator naming a file inside the instance directory: a relative path of non-empty
/// segments, none of them `.` or `..`, with no `\`, no `:` (a scheme or a drive) and no leading
/// `/`. An empty or `.` segment would let the locator name a directory.
fn is_instance_path(locator: &str) -> bool {
    !locator.starts_with('/')
        && !locator.contains(['\\', ':'])
        && locator
            .split('/')
            .all(|segment| !segment.is_empty() && segment != "." && segment != "..")
}

/// The keys of `parents` (key → parent) whose parent chain reaches the key itself, in key order.
fn parent_cycles(parents: &BTreeMap<String, Option<String>>) -> Vec<String> {
    parents
        .keys()
        .filter(|&key| {
            let mut at = parents.get(key).cloned().flatten();
            for _ in 0..parents.len() {
                match at {
                    Some(ref parent) if parent == key => return true,
                    Some(parent) => at = parents.get(&parent).cloned().flatten(),
                    None => return false,
                }
            }
            false
        })
        .cloned()
        .collect()
}

/// A mapping whose keys are distinct: a key given twice is refused, never the last one kept.
fn distinct_keys<'de, D, V>(deserializer: D) -> Result<BTreeMap<String, V>, D::Error>
where
    D: serde::Deserializer<'de>,
    V: Deserialize<'de>,
{
    struct Distinct<V>(std::marker::PhantomData<V>);

    impl<'de, V: Deserialize<'de>> serde::de::Visitor<'de> for Distinct<V> {
        type Value = BTreeMap<String, V>;

        fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str("a mapping whose keys are distinct")
        }

        fn visit_map<A>(self, mut access: A) -> Result<Self::Value, A::Error>
        where
            A: serde::de::MapAccess<'de>,
        {
            let mut map = BTreeMap::new();
            while let Some(key) = access.next_key::<String>()? {
                if map.contains_key(&key) {
                    return Err(serde::de::Error::custom(format!(
                        "key `{key}` is given twice"
                    )));
                }
                let value = access.next_value()?;
                map.insert(key, value);
            }
            Ok(map)
        }
    }

    deserializer.deserialize_map(Distinct(std::marker::PhantomData))
}

// The document shapes of the instance files.

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RegistryDoc {
    registry_id: String,
    title: String,
    #[serde(default, deserialize_with = "distinct_keys")]
    dimensions: BTreeMap<String, DimensionDoc>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct DimensionDoc {
    registry: String,
    #[serde(default)]
    rule: Option<RuleDoc>,
    #[serde(default)]
    parent: Option<String>,
    #[serde(default)]
    otel_attribute: Option<String>,
}

#[derive(Deserialize, Clone, Copy)]
#[serde(rename_all = "snake_case")]
enum RuleDoc {
    SourceOnly,
    Derived,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum EntryDoc {
    Plain(String),
    Entry(ValueEntryDoc),
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ValueEntryDoc {
    value: String,
    #[serde(default)]
    created: Option<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct DefinitionDoc {
    id: String,
    title: String,
    family: String,
    kind: KindDoc,
    #[serde(default)]
    unit: Option<String>,
    #[serde(default)]
    dimensions: Option<Vec<String>>,
    source: SourceDoc,
    extractor: String,
    #[serde(deserialize_with = "distinct_keys")]
    query: BTreeMap<String, String>,
    window: String,
    cadence: String,
    history: HistoryDoc,
    #[serde(default)]
    denominator: Option<DenominatorDoc>,
    evaluation: EvaluationDoc,
    #[serde(default)]
    persistence: Option<PersistenceDoc>,
    #[serde(default)]
    objective: Option<serde_yaml_ng::Value>,
    #[serde(default)]
    owner: Option<String>,
    #[serde(default)]
    links: Option<LinksDoc>,
    lifecycle: LifecycleDoc,
    rev: i64,
    revised_at: String,
    revised_by: String,
    #[serde(default)]
    mute_reason: Option<String>,
    #[serde(default)]
    mute_until: Option<String>,
}

#[derive(Deserialize, Clone, Copy)]
#[serde(rename_all = "snake_case")]
enum KindDoc {
    Gauge,
    CounterIncrease,
    Ratio,
    Instant,
    CountOfCases,
}

#[derive(Deserialize, Clone, Copy)]
#[serde(rename_all = "snake_case")]
enum HistoryDoc {
    Source,
    Store,
}

#[derive(Deserialize, Clone, Copy)]
#[serde(rename_all = "snake_case")]
enum LifecycleDoc {
    Proposed,
    Active,
    Muted,
    Retired,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SourceDoc {
    adapter: String,
    connection: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct DenominatorDoc {
    unit: String,
    min_n: i64,
    since: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct EvaluationDoc {
    rule: String,
    baselines: Vec<BaselineDoc>,
    #[serde(default)]
    warn: Option<ThresholdDoc>,
    #[serde(default)]
    crit: Option<ThresholdDoc>,
    #[serde(default)]
    min_trend_window: Option<String>,
    #[serde(default)]
    fixed: Option<String>,
    #[serde(default)]
    rolling_n: Option<i64>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PersistenceDoc {
    k: i64,
    k_crit: i64,
    m: i64,
}

#[derive(Deserialize, Clone, Copy)]
enum BaselineDoc {
    #[serde(rename = "offset_1d")]
    Offset1d,
    #[serde(rename = "offset_7d")]
    Offset7d,
    #[serde(rename = "pooled")]
    Pooled,
    #[serde(rename = "higher_of")]
    HigherOf,
    #[serde(rename = "lower_of")]
    LowerOf,
    #[serde(rename = "same_slot_median_4w")]
    SameSlotMedian4w,
    #[serde(rename = "rolling_median")]
    RollingMedian,
    #[serde(rename = "fixed")]
    Fixed,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ThresholdDoc {
    #[serde(default)]
    x: Option<String>,
    #[serde(default)]
    abs: Option<String>,
    #[serde(default)]
    min_events: Option<i64>,
}

#[derive(Deserialize, Default)]
#[serde(deny_unknown_fields)]
struct LinksDoc {
    #[serde(default)]
    components: Vec<String>,
    #[serde(default)]
    issues: Vec<String>,
}

/// An instance's dimension registry (design § 2.4): closed, one entry per key.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Registry {
    data: DimensionRegistryData,
    dimensions: BTreeMap<String, DimensionData>,
}

impl Registry {
    /// The registry document `text` of `file`: `registry_id`, `title` and `dimensions`, each key
    /// with `registry` (`file:<locator>` or `open`), and optional `rule`, `parent` and
    /// `otel_attribute`.
    ///
    /// # Errors
    ///
    /// Every [`LoadError`] of the file: its shape, a character set, a `connectors:` registry
    /// (D17), any other registry form, or a parent that is not registered.
    pub fn parse(file: &str, text: &str) -> Result<Self, Vec<LoadError>> {
        let doc: RegistryDoc =
            serde_yaml_ng::from_str(text).map_err(|error| syntax(file, &error))?;
        let mut errors = Errors::new(file);
        errors.alphabet("registry_id", &doc.registry_id, REGISTRY_ID);
        let registry_id = RegistryId(doc.registry_id);
        let mut dimensions = BTreeMap::new();
        for (key, dimension) in &doc.dimensions {
            errors.alphabet("dimension key", key, LOWER);
            let values = if dimension.registry == "open" {
                ValueSource::Open
            } else if let Some(locator) = dimension
                .registry
                .strip_prefix("file:")
                .filter(|locator| !locator.is_empty())
            {
                if !is_instance_path(locator) {
                    errors.push(Rule::LocatorOutsideInstance {
                        key: key.clone(),
                        locator: locator.to_owned(),
                    });
                }
                ValueSource::File(Locator(locator.to_owned()))
            } else if let Some(locator) = dimension
                .registry
                .strip_prefix("connectors:")
                .filter(|locator| !locator.is_empty())
            {
                errors.push(Rule::ConnectorsRegistry {
                    key: key.clone(),
                    locator: locator.to_owned(),
                });
                ValueSource::Connectors(Locator(locator.to_owned()))
            } else {
                errors.push(Rule::RegistryForm {
                    key: key.clone(),
                    value: dimension.registry.clone(),
                });
                ValueSource::Open
            };
            if let Some(parent) = &dimension.parent
                && !doc.dimensions.contains_key(parent)
            {
                errors.push(Rule::UnknownParent {
                    key: key.clone(),
                    parent: parent.clone(),
                });
            }
            let rule = match dimension.rule {
                Some(RuleDoc::SourceOnly) => DimensionRule::SourceOnly,
                Some(RuleDoc::Derived) | None => DimensionRule::Derived,
            };
            dimensions.insert(
                key.clone(),
                DimensionData {
                    key: DimensionKey(key.clone()),
                    registry_id: registry_id.clone(),
                    values,
                    rule,
                    parent: dimension.parent.clone().map(DimensionKey),
                    otel_attribute: dimension.otel_attribute.clone(),
                },
            );
        }
        let parents: BTreeMap<String, Option<String>> = doc
            .dimensions
            .iter()
            .map(|(key, dimension)| (key.clone(), dimension.parent.clone()))
            .collect();
        for key in parent_cycles(&parents) {
            errors.push(Rule::ParentCycle { key });
        }
        errors.finish(Self {
            data: DimensionRegistryData {
                registry_id,
                title: doc.title,
            },
            dimensions,
        })
    }

    /// The registry's id and title.
    #[must_use]
    pub fn data(&self) -> &DimensionRegistryData {
        &self.data
    }

    /// The registered dimension `key`, if any.
    #[must_use]
    pub fn dimension(&self, key: &str) -> Option<&DimensionData> {
        self.dimensions.get(key)
    }

    /// Every registered dimension, in key order.
    pub fn dimensions(&self) -> impl Iterator<Item = &DimensionData> {
        self.dimensions.values()
    }
}

/// The values a `file:` registry lists, each with its optional `created`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct FileValues {
    entries: BTreeMap<String, Option<Timestamp>>,
}

impl FileValues {
    /// The registry file `text` of `file`: a list whose entries are a value, or `value` with an
    /// optional RFC 3339 `created`.
    ///
    /// # Errors
    ///
    /// Every [`LoadError`] of the file: its shape, a `created` that is not RFC 3339, or a value
    /// listed twice.
    pub fn parse(file: &str, text: &str) -> Result<Self, Vec<LoadError>> {
        let doc: Vec<EntryDoc> =
            serde_yaml_ng::from_str(text).map_err(|error| syntax(file, &error))?;
        let mut errors = Errors::new(file);
        let mut entries = BTreeMap::new();
        for entry in doc {
            let (value, created) = match entry {
                EntryDoc::Plain(value) => (value, None),
                EntryDoc::Entry(ValueEntryDoc { value, created }) => (value, created),
            };
            let created = created.map(|created| errors.timestamp("created", &created));
            if entries.insert(value.clone(), created).is_some() {
                errors.push(Rule::DuplicateValue { value });
            }
        }
        errors.finish(Self { entries })
    }

    /// Whether `value` is listed.
    #[must_use]
    pub fn contains(&self, value: &str) -> bool {
        self.entries.contains_key(value)
    }

    /// The `created` of `value`, when it is listed and carries one.
    #[must_use]
    pub fn created(&self, value: &str) -> Option<&Timestamp> {
        self.entries.get(value)?.as_ref()
    }
}

/// One loaded definition: its file, its state and data, and its admitted `denominator.since`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Definition {
    file: String,
    snapshot: SignalDefinitionSnapshot,
    since: Option<Since>,
}

impl Definition {
    /// The definition document `text` of `file` (design § 2.1: `id`, `lifecycle` and the fields
    /// of `thalamus.definition.SignalDefinition`), validated against `registry`.
    ///
    /// # Errors
    ///
    /// Every [`LoadError`] of the file.
    pub fn parse(file: &str, text: &str, registry: &Registry) -> Result<Self, Vec<LoadError>> {
        let doc: DefinitionDoc =
            serde_yaml_ng::from_str(text).map_err(|error| syntax(file, &error))?;
        let mut errors = Errors::new(file);

        if !is_signal_id(&doc.id) {
            errors.push(Rule::SignalIdGrammar { id: doc.id.clone() });
        } else if doc.id.split('.').next() != Some(doc.family.as_str()) {
            errors.push(Rule::IdFamily {
                id: doc.id.clone(),
                family: doc.family.clone(),
            });
        }
        errors.alphabet("family", &doc.family, LOWER);
        errors.alphabet("source.adapter", &doc.source.adapter, LOWER);
        if !is_connection_id(&doc.source.connection) {
            errors.push(Rule::ConnectionNotIdentifier {
                connection: doc.source.connection.clone(),
            });
        }

        let state = match doc.lifecycle {
            LifecycleDoc::Proposed => SignalDefinitionState::Proposed,
            LifecycleDoc::Active => SignalDefinitionState::Active,
            LifecycleDoc::Muted => SignalDefinitionState::Muted,
            LifecycleDoc::Retired => SignalDefinitionState::Retired,
        };
        let sampled = matches!(
            state,
            SignalDefinitionState::Active | SignalDefinitionState::Muted
        );
        let kind = match doc.kind {
            KindDoc::Gauge => SignalKind::Gauge,
            KindDoc::CounterIncrease => SignalKind::CounterIncrease,
            KindDoc::Ratio => SignalKind::Ratio,
            KindDoc::Instant => SignalKind::Instant,
            KindDoc::CountOfCases => SignalKind::CountOfCases,
        };

        if doc.rev < 1 {
            errors.push(Rule::Invariant {
                invariant: REV_INVARIANT,
            });
        }
        if sampled {
            let missing: Vec<&'static str> = [
                ("unit", doc.unit.is_none()),
                ("dimensions", doc.dimensions.is_none()),
                ("denominator", doc.denominator.is_none()),
                ("owner", doc.owner.is_none()),
            ]
            .into_iter()
            .filter_map(|(field, missing)| missing.then_some(field))
            .collect();
            if !missing.is_empty() {
                errors.push(Rule::Incomplete {
                    signal_id: doc.id.clone(),
                    missing,
                });
            }
        }
        let muted = state == SignalDefinitionState::Muted;
        let mute_held = [doc.mute_reason.is_some(), doc.mute_until.is_some()];
        if (muted && mute_held != [true, true]) || (!muted && mute_held != [false, false]) {
            errors.push(Rule::Invariant {
                invariant: MUTE_INVARIANT,
            });
        }

        if let Some(unit) = &doc.unit {
            if let Err(error) = parse_ucum(unit) {
                errors.push(Rule::NotUcum {
                    field: "unit",
                    unit: unit.clone(),
                    error: error.to_string(),
                });
            }
            if kind == SignalKind::Ratio && unit != "1" {
                errors.push(Rule::RatioUnit { unit: unit.clone() });
            }
        }

        let mut seen = BTreeSet::new();
        for key in doc.dimensions.iter().flatten() {
            if !seen.insert(key.as_str()) {
                errors.push(Rule::DuplicateDimension { key: key.clone() });
            }
            if registry.dimension(key).is_none() {
                errors.push(Rule::UnregisteredDimension { key: key.clone() });
            }
        }
        let dimensions: Option<Vec<DimensionKey>> = doc
            .dimensions
            .map(|keys| keys.into_iter().map(DimensionKey).collect());

        let mut since = None;
        let denominator = doc.denominator.map(|denominator| {
            if let Err(error) = parse_ucum(&denominator.unit) {
                errors.push(Rule::NotUcum {
                    field: "denominator.unit",
                    unit: denominator.unit.clone(),
                    error: error.to_string(),
                });
            }
            match parse_since(&denominator.since, dimensions.as_deref(), registry) {
                Ok(admitted) => since = Some(admitted),
                Err(refusal) => errors.push(Rule::Since {
                    value: denominator.since.clone(),
                    refusal,
                }),
            }
            DenominatorSpec {
                unit: Unit(denominator.unit),
                min_n: denominator.min_n,
                since: denominator.since,
            }
        });

        let baselines: Vec<BaselineKind> = doc
            .evaluation
            .baselines
            .iter()
            .map(|baseline| match baseline {
                BaselineDoc::Offset1d => BaselineKind::Offset1d,
                BaselineDoc::Offset7d => BaselineKind::Offset7d,
                BaselineDoc::Pooled => BaselineKind::Pooled,
                BaselineDoc::HigherOf => BaselineKind::HigherOf,
                BaselineDoc::LowerOf => BaselineKind::LowerOf,
                BaselineDoc::SameSlotMedian4w => BaselineKind::SameSlotMedian4w,
                BaselineDoc::RollingMedian => BaselineKind::RollingMedian,
                BaselineDoc::Fixed => BaselineKind::Fixed,
            })
            .collect();
        let instant_not_fixed = kind == SignalKind::Instant
            && baselines.iter().any(|&kind| kind != BaselineKind::Fixed);
        if instant_not_fixed {
            errors.push(Rule::InstantNeedsFixed {
                signal_id: doc.id.clone(),
            });
        }
        let mut threshold = |line: Option<ThresholdDoc>, x: &'static str, abs: &'static str| {
            line.map(|line| Threshold {
                x: errors.decimal(x, line.x),
                abs: errors.decimal(abs, line.abs),
                min_events: line.min_events,
            })
        };
        let warn = threshold(
            doc.evaluation.warn,
            "evaluation.warn.x",
            "evaluation.warn.abs",
        );
        let crit = threshold(
            doc.evaluation.crit,
            "evaluation.crit.x",
            "evaluation.crit.abs",
        );
        let min_trend_window = doc
            .evaluation
            .min_trend_window
            .map(|window| errors.duration("evaluation.min_trend_window", &window));
        let fixed = errors.decimal("evaluation.fixed", doc.evaluation.fixed);
        let rolling_n = doc.evaluation.rolling_n;
        let persistence = doc.persistence.map(|held| Persistence {
            k: held.k,
            k_crit: held.k_crit,
            m: held.m,
        });
        let window = errors.duration("window", &doc.window);
        let cadence = errors.duration("cadence", &doc.cadence);
        let revised_at = errors.timestamp("revised_at", &doc.revised_at);
        let mute_until = doc
            .mute_until
            .map(|until| errors.timestamp("mute_until", &until));
        let objective = match doc.objective.map(|objective| json(&objective)) {
            None => None,
            Some(Ok(objective)) => Some(objective),
            Some(Err(reason)) => {
                errors.push(Rule::Objective { reason });
                None
            }
        };
        let links = doc.links.unwrap_or_default();

        let data = SignalDefinitionData {
            signal_id: SignalId(doc.id),
            title: doc.title,
            family: Family(doc.family),
            kind,
            unit: doc.unit.map(Unit),
            dimensions,
            source: Source {
                adapter: AdapterId(doc.source.adapter),
                connection: ConnectionId(doc.source.connection),
            },
            extractor: ExtractorId(doc.extractor),
            query: doc
                .query
                .into_iter()
                .map(|(name, text)| (name, QueryText(text)))
                .collect(),
            window,
            cadence,
            history: match doc.history {
                HistoryDoc::Source => History::Source,
                HistoryDoc::Store => History::Store,
            },
            denominator,
            evaluation: EvaluationSpec {
                rule: RuleId(doc.evaluation.rule),
                baselines,
                warn,
                crit,
                min_trend_window,
                fixed,
                rolling_n,
            },
            persistence,
            objective,
            owner: doc.owner.map(Owner),
            links: Links {
                components: links.components,
                issues: links.issues,
            },
            rev: doc.rev,
            revised_at,
            revised_by: Author(doc.revised_by),
            mute_reason: doc.mute_reason,
            mute_until,
        };

        for refusal in input_refusals(kind, &data.evaluation, data.persistence.as_ref()) {
            if refusal != InputRefusal::InstantNotFixed {
                errors.push(Rule::Input {
                    signal_id: data.signal_id.0.clone(),
                    refusal,
                });
            }
        }

        if sampled
            && !instant_not_fixed
            && let Err(refusal) = baseline::plan(&data, &probe_sample(&data))
        {
            errors.push(Rule::Baseline { refusal });
        }

        errors.finish(Self {
            file: file.to_owned(),
            snapshot: SignalDefinitionSnapshot { state, data },
            since,
        })
    }

    /// The file the definition was read from.
    #[must_use]
    pub fn file(&self) -> &str {
        &self.file
    }

    /// The definition's fields.
    #[must_use]
    pub fn data(&self) -> &SignalDefinitionData {
        &self.snapshot.data
    }

    /// The definition's lifecycle state, from its `lifecycle` field, the only copy (D19).
    #[must_use]
    pub fn state(&self) -> SignalDefinitionState {
        self.snapshot.state
    }

    /// The definition as the storage port serves it.
    #[must_use]
    pub fn snapshot(&self) -> &SignalDefinitionSnapshot {
        &self.snapshot
    }

    /// The admitted `denominator.since`, when the definition declares a denominator.
    #[must_use]
    pub fn since(&self) -> Option<&Since> {
        self.since.as_ref()
    }
}

/// A sample with a well-formed window, for asking [`baseline::plan`] whether the definition's
/// baseline is one it plans at all. Only the window is read.
fn probe_sample(definition: &SignalDefinitionData) -> SignalSampleData {
    let end = Timestamp("2000-01-15T00:00:00Z".to_owned());
    SignalSampleData {
        sample_id: SampleId("smp_probe".to_owned()),
        signal: definition.signal_id.clone(),
        series: SeriesKey(definition.signal_id.0.clone()),
        dims: BTreeMap::new(),
        ts: end.clone(),
        window: TimeWindow {
            start: Timestamp("2000-01-14T00:00:00Z".to_owned()),
            end,
        },
        reading: Reading::Ok(CompleteReading {
            value: Decimal("0".to_owned()),
        }),
        unit: Unit("1".to_owned()),
        denominator: None,
        provenance: Provenance {
            producer: String::new(),
            version: String::new(),
            definition_rev: definition.rev,
            adapter: definition.source.adapter.clone(),
            connection: definition.source.connection.clone(),
            query_sha256: QueryDigest(String::new()),
            run_id: RunId("run_probe".to_owned()),
        },
    }
}

/// How deep the served `Definitions` view puts an objective: its body is
/// `{"rows":[{…,"objective":…}]}` (the generated `thalamus_service` route), so the objective
/// itself sits at level 3 of the document.
const OBJECTIVE_SERVED_AT: usize = 3;

/// The deepest a value inside an objective sits, counted as `thalamus_model::json::parse` counts
/// levels with the objective itself at 0. The reader refuses a document with a value deeper than
/// 64, and the objective is served [`OBJECTIVE_SERVED_AT`] levels down, so 61 is the most an
/// objective may hold for every body that carries it to read back.
const OBJECTIVE_DEPTH: usize = 64 - OBJECTIVE_SERVED_AT;

/// An objective as the model's JSON value, one `thalamus_model::json::push_value` writes and
/// `thalamus_model::json::parse` reads back, alone and inside the served `Definitions` body:
/// every number a finite JSON number, kept in its YAML rendering, and no value deeper than
/// [`OBJECTIVE_DEPTH`].
fn json(value: &serde_yaml_ng::Value) -> Result<Json, String> {
    json_at(value, 0)
}

fn json_at(value: &serde_yaml_ng::Value, depth: usize) -> Result<Json, String> {
    use serde_yaml_ng::Value as Yaml;
    if depth > OBJECTIVE_DEPTH {
        return Err(format!(
            "is nested deeper than {OBJECTIVE_DEPTH} levels; served {OBJECTIVE_SERVED_AT} levels \
             down, it would pass the 64 the model's JSON reader admits"
        ));
    }
    Ok(match value {
        Yaml::Null => Json::Null,
        Yaml::Bool(value) => Json::Bool(*value),
        Yaml::Number(number) => {
            let spelling = number.to_string();
            if !number.is_finite() || !is_json_number(&spelling) {
                return Err(format!(
                    "holds `{spelling}`, which is not a finite JSON number"
                ));
            }
            Json::Number(spelling)
        }
        Yaml::String(text) => Json::Text(text.clone()),
        Yaml::Sequence(items) => Json::Array(
            items
                .iter()
                .map(|item| json_at(item, depth + 1))
                .collect::<Result<_, _>>()?,
        ),
        Yaml::Mapping(members) => Json::Object(
            members
                .iter()
                .map(|(name, member)| match name {
                    Yaml::String(name) => Ok((name.clone(), json_at(member, depth + 1)?)),
                    _ => Err("has a member whose name is not text".to_owned()),
                })
                .collect::<Result<_, _>>()?,
        ),
        Yaml::Tagged(tagged) => json_at(&tagged.value, depth)?,
    })
}

/// The JSON number grammar: `-?(0|[1-9][0-9]*)(\.[0-9]+)?([eE][+-]?[0-9]+)?`.
fn is_json_number(text: &str) -> bool {
    let rest = text.strip_prefix('-').unwrap_or(text);
    let digits = |part: &str| !part.is_empty() && part.bytes().all(|byte| byte.is_ascii_digit());
    let (mantissa, exponent) = match rest.find(['e', 'E']) {
        Some(at) => (&rest[..at], Some(&rest[at + 1..])),
        None => (rest, None),
    };
    let (whole, fraction) = match mantissa.split_once('.') {
        Some((whole, fraction)) => (whole, Some(fraction)),
        None => (mantissa, None),
    };
    digits(whole)
        && (whole == "0" || !whole.starts_with('0'))
        && fraction.is_none_or(digits)
        && exponent
            .is_none_or(|exponent| digits(exponent.strip_prefix(['+', '-']).unwrap_or(exponent)))
}

/// Who tried to write through a storage port of an [`Instance`], and what was refused.
///
/// The instance's files are the one copy of every definition and of the registry (design § 4,
/// § 11 D19): the ports serve them read-only, record every write as refused, and change nothing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WriteRefused {
    /// The entity the write addressed.
    pub entity: &'static str,
    /// The identity it addressed.
    pub identity: String,
    /// `put` or `delete`.
    pub operation: &'static str,
}

impl fmt::Display for WriteRefused {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} of {} `{}` refused: the instance's files are the one copy (design § 4, § 11 D19); \
             edit the file instead",
            self.operation, self.entity, self.identity
        )
    }
}

impl std::error::Error for WriteRefused {}

/// A loaded instance: its dimension registry and its definitions, served read-only through the
/// generated storage ports.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Instance {
    registry: Registry,
    definitions: BTreeMap<String, Definition>,
    refused: Vec<WriteRefused>,
}

impl Instance {
    /// The instance whose registry document is `registry` and whose definition files are
    /// `definitions`, each a `(file, contents)` pair.
    ///
    /// # Errors
    ///
    /// Every [`LoadError`] of the registry when it does not load; otherwise every [`LoadError`]
    /// of every definition file, and a second file defining an id already defined.
    pub fn load(
        registry: (&str, &str),
        definitions: &[(&str, &str)],
    ) -> Result<Self, Vec<LoadError>> {
        let registry = Registry::parse(registry.0, registry.1)?;
        let mut errors = Vec::new();
        let mut loaded = BTreeMap::new();
        for (file, text) in definitions {
            match Definition::parse(file, text, &registry) {
                Err(refused) => errors.extend(refused),
                Ok(definition) => {
                    let id = definition.data().signal_id.0.clone();
                    match loaded.entry(id) {
                        Entry::Occupied(held) => errors.push(LoadError {
                            file: (*file).to_owned(),
                            rule: Rule::DuplicateDefinition {
                                signal_id: held.key().clone(),
                            },
                        }),
                        Entry::Vacant(slot) => {
                            slot.insert(definition);
                        }
                    }
                }
            }
        }
        if errors.is_empty() {
            Ok(Self {
                registry,
                definitions: loaded,
                refused: Vec::new(),
            })
        } else {
            Err(errors)
        }
    }

    /// The dimension registry.
    #[must_use]
    pub fn registry(&self) -> &Registry {
        &self.registry
    }

    /// The definition with id `signal_id`, if any.
    #[must_use]
    pub fn definition(&self, signal_id: &str) -> Option<&Definition> {
        self.definitions.get(signal_id)
    }

    /// Every definition, in id order.
    pub fn definitions(&self) -> impl Iterator<Item = &Definition> {
        self.definitions.values()
    }

    /// Every write the storage ports refused, in the order they were tried.
    #[must_use]
    pub fn refused_writes(&self) -> &[WriteRefused] {
        &self.refused
    }

    fn refuse(&mut self, entity: &'static str, identity: &str, operation: &'static str) {
        self.refused.push(WriteRefused {
            entity,
            identity: identity.to_owned(),
            operation,
        });
    }
}

impl SignalDefinitionStorage for Instance {
    fn get(&self, identity: &SignalId) -> Option<SignalDefinitionSnapshot> {
        self.definitions
            .get(&identity.0)
            .map(|definition| definition.snapshot.clone())
    }

    fn put(&mut self, snapshot: SignalDefinitionSnapshot) {
        self.refuse(SIGNAL_DEFINITION, &snapshot.data.signal_id.0, "put");
    }

    fn delete(&mut self, identity: &SignalId) {
        self.refuse(SIGNAL_DEFINITION, &identity.0, "delete");
    }

    fn list(&self) -> Vec<SignalDefinitionSnapshot> {
        self.definitions
            .values()
            .map(|definition| definition.snapshot.clone())
            .collect()
    }
}

impl DimensionStorage for Instance {
    fn get(&self, identity: &DimensionKey) -> Option<DimensionSnapshot> {
        self.registry
            .dimension(&identity.0)
            .map(|data| DimensionSnapshot {
                state: DimensionState::Registered,
                data: data.clone(),
            })
    }

    fn put(&mut self, snapshot: DimensionSnapshot) {
        self.refuse(DIMENSION, &snapshot.data.key.0, "put");
    }

    fn delete(&mut self, identity: &DimensionKey) {
        self.refuse(DIMENSION, &identity.0, "delete");
    }

    fn list(&self) -> Vec<DimensionSnapshot> {
        self.registry
            .dimensions()
            .map(|data| DimensionSnapshot {
                state: DimensionState::Registered,
                data: data.clone(),
            })
            .collect()
    }
}

impl DimensionRegistryStorage for Instance {
    fn get(&self, identity: &RegistryId) -> Option<DimensionRegistrySnapshot> {
        (self.registry.data.registry_id == *identity).then(|| DimensionRegistrySnapshot {
            state: DimensionRegistryState::Declared,
            data: self.registry.data.clone(),
        })
    }

    fn put(&mut self, snapshot: DimensionRegistrySnapshot) {
        self.refuse(DIMENSION_REGISTRY, &snapshot.data.registry_id.0, "put");
    }

    fn delete(&mut self, identity: &RegistryId) {
        self.refuse(DIMENSION_REGISTRY, &identity.0, "delete");
    }

    fn list(&self) -> Vec<DimensionRegistrySnapshot> {
        vec![DimensionRegistrySnapshot {
            state: DimensionRegistryState::Declared,
            data: self.registry.data.clone(),
        }]
    }
}

#[cfg(test)]
mod tests {
    use super::{is_decimal, is_json_number, iso_duration, placeable};

    #[test]
    fn durations_are_short_or_iso_8601() {
        for (text, iso, seconds) in [
            ("6h", "PT6H", 21_600),
            ("30s", "PT30S", 30),
            ("5m", "PT5M", 300),
            ("1d", "P1D", 86_400),
            ("2w", "P2W", 1_209_600),
            ("PT6H", "PT6H", 21_600),
            ("P1DT12H", "P1DT12H", 129_600),
            ("P28D", "P28D", 2_419_200),
            ("P1W1DT1H1M1S", "P8DT1H1M1S", 694_861),
            ("PT0S", "PT0S", 0),
            ("P1W2D", "P9D", 777_600),
            ("P1WT1H", "P7DT1H", 608_400),
            ("PT1H1S", "PT1H0M1S", 3_601),
            ("PT1M1S", "PT1M1S", 61),
            ("PT1H1M", "PT1H1M", 3_660),
            ("P1DT1S", "P1DT1S", 86_401),
            ("0006h", "PT6H", 21_600),
            ("P007D", "P7D", 604_800),
            ("P2W", "P2W", 1_209_600),
        ] {
            assert_eq!(
                iso_duration(text),
                Some((iso.to_owned(), seconds)),
                "{text}"
            );
        }
        for text in [
            "",
            "h",
            "6",
            "6x",
            "-6h",
            "P",
            "PT",
            "P1H",
            "PT1D",
            "P1DT",
            "6 h",
            "PT1M1H",
            "P1M",
            "P1Y",
            "P1D1W",
            "99999999999999999999h",
            "P99999999999999999999D",
            "153722867280912931h",
            "P1D1D",
            "PT1S1S",
            "PT1H1H",
        ] {
            assert_eq!(iso_duration(text), None, "{text}");
        }
        // Every canonical form reads back as itself, at the same length.
        for text in ["P1W2D", "PT1H1S", "P1W1DT1H1M1S", "6h", "2w", "P1WT1H"] {
            let (canonical, seconds) = iso_duration(text).expect("admitted");
            assert_eq!(
                iso_duration(&canonical),
                Some((canonical.clone(), seconds)),
                "{text}"
            );
        }
        assert!(!placeable(0));
        assert!(placeable(1));
        assert!(!placeable(i64::MAX));
        // A window ending at 1970-01-01T00:00:00Z starts at 0000-01-01T00:00:00Z at the most.
        assert!(placeable(719_528 * 86_400));
        assert!(!placeable(719_528 * 86_400 + 1));
    }

    #[test]
    fn json_numbers_follow_the_json_grammar() {
        for text in ["0", "-3", "0.995", "1e300", "1.5E-7", "2e+3"] {
            assert!(is_json_number(text), "{text}");
        }
        for text in [
            "", ".5", "01", "1.", "NaN", "inf", "-.inf", "1e", "+1", "0x1F",
        ] {
            assert!(!is_json_number(text), "{text}");
        }
    }

    #[test]
    fn decimals_are_plain() {
        for text in ["0", "2", "-1", "0.02", "10.50"] {
            assert!(is_decimal(text), "{text}");
        }
        for text in ["", ".5", "5.", "1e3", "+1", "0.0.1", "nan"] {
            assert!(!is_decimal(text), "{text}");
        }
    }
}
