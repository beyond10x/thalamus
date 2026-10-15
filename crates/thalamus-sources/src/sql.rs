//! SQL reads: the Connectors `sql` adapter's `query.read`, turned into typed reads.
//!
//! Connectors runs each query in a read-only transaction (PostgreSQL) or session (MySQL), returns
//! at most [`ROW_LIMIT`] rows, writes every non-NULL cell as the database's own lossless text and
//! NULL as JSON null, and says whether it cut the result (`truncated`). Thalamus sends only the
//! query text and the row limit.
//!
//! Three extractors:
//!
//! - `sql.value` runs the query `value`. Each row is one read. Its value is the cell of the column
//!   named `value`, or else of the one column whose name is not a declared dimension. A `dims` key
//!   is set only from a column the row returns, named for a dimension the definition declares,
//!   and holding a non-NULL cell (L9). A series the query returns twice or more reads `unknown`
//!   once: no one of its rows is its value.
//! - `sql.ratio` runs `numerator`, then `denominator`, pairs their rows by `dims`, and reads their
//!   quotient, unit `1`. The denominator count, when it is a whole number, is the read's
//!   population. The two queries must return the same dimension columns: a NULL cell and an
//!   absent column both set no key, so otherwise each series of the shared columns reads
//!   `unknown`. A series the numerator does not return reads numerator 0 (a grouped count has no
//!   row for a group with no case), but only when every numerator series has a denominator twin;
//!   otherwise the pass is one `unknown` read naming the unmatched series (design § 11 D20). A
//!   numerator that answers no rows at all infers no 0: every denominator series reads `unknown`. A
//!   series the numerator returns twice, or whose denominator is 0, reads `unknown`.
//! - `sql.count` runs the query `value` and counts its rows per series, a series being the
//!   declared dimension columns the query returns. A cut answer of one series holds at least the
//!   rows it returned: the read is `partial` with bound `lower`.
//!
//! A cut answer of many series is one `unknown` read naming the cut: the series past it are not
//! read, and the ones before it are not passed off as the whole answer. Two result columns of one
//! name, when that name is `value` or a declared dimension, are ambiguous: one `unknown` read.
//!
//! A NULL cell, and a cell whose text is not a decimal number, read `unknown` with a reason, never
//! 0. Every value read from a cell keeps the cell's text as `raw`; a NULL cell has none, and a
//! ratio, read from two cells, has none. Numbers are parsed from their text exactly and never pass
//! through a binary float. A quotient that does not terminate keeps [`RATIO_SCALE`] fractional
//! digits, the last rounded half away from zero, trailing zeros dropped.

use std::collections::BTreeMap;

use serde_json::{Value, json};
use thalamus_model::definition::{SignalDefinitionData, Unit};
use thalamus_model::observation::{
    Bound, CompleteReading, PartialReading, Reading, RunId, TimeWindow, UnreadReading,
};
use thalamus_model::primitives::{Decimal, Timestamp};

use crate::adapter::{Adapter, Population, Read, query_digest, unknown, window_ending};
use crate::connectors::{ConnectorsError, ReadClient};

/// The Connectors operation every SQL read runs.
pub const QUERY_READ: &str = "query.read";

/// The rows asked for: Connectors' own ceiling for `query.read`.
pub const ROW_LIMIT: u16 = 1000;

/// The fractional digits a quotient that does not terminate keeps.
pub const RATIO_SCALE: usize = 18;

/// The longest exponent a cell's number may carry. A longer one is no number a reading holds.
const EXPONENT_LIMIT: i64 = 4096;

/// The SQL adapter.
pub fn adapter() -> Option<Box<dyn Adapter>> {
    Some(Box::new(Sql))
}

/// Reads `sql.value`, `sql.ratio` and `sql.count` through `query.read`.
#[derive(Debug, Clone, Copy, Default)]
pub struct Sql;

impl Adapter for Sql {
    fn read_operations(&self) -> &[&str] {
        &[QUERY_READ]
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
            Err(_) => {
                return Ok(vec![unknown(
                    definition,
                    end,
                    format!("{extractor}: no window"),
                )]);
            }
        };
        let reads = Reads {
            definition,
            window,
            extractor,
        };
        match extractor.strip_prefix("sql.") {
            Some("value") => value(client, &reads),
            Some("ratio") => ratio(client, &reads),
            Some("count") => count(client, &reads),
            _ => {
                Ok(vec![
                    reads.unknown(
                        BTreeMap::new(),
                        "no such sql extractor; sql reads sql.value, sql.ratio and sql.count"
                            .to_string(),
                    ),
                ])
            }
        }
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
        raw: Option<String>,
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
            raw,
            retention_start: None,
        }
    }

    /// An `unknown` read saying `reason`, prefixed with the extractor.
    fn unknown(&self, dims: BTreeMap<String, String>, reason: String) -> Read {
        let reason = format!("{}: {reason}", self.extractor);
        self.read(dims, Reading::Unknown(UnreadReading { reason }), None, None)
    }

    /// The query text named `name`, or the `unknown` read saying it is missing.
    fn query(&self, name: &str) -> Result<&str, Box<Read>> {
        self.definition
            .query
            .get(name)
            .map(|text| text.0.as_str())
            .ok_or_else(|| {
                Box::new(self.unknown(
                    BTreeMap::new(),
                    format!("the definition has no query `{name}`"),
                ))
            })
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

    /// Runs `query` through `query.read`.
    fn run(&self, client: &ReadClient<'_>, query: &str) -> Result<Answer, ConnectorsError> {
        let input = json!({"query": query, "limit": ROW_LIMIT});
        let document = client.invoke(&self.definition.source, QUERY_READ, &input)?;
        Answer::parse(&document).map_err(|detail| ConnectorsError::Unavailable {
            call: format!("{}.{QUERY_READ}", self.definition.source.adapter.0),
            detail: format!("answered a result that is not a query.read result: {detail}"),
        })
    }
}

/// A `query.read` result: column names, rows of cells, and whether Connectors cut it.
struct Answer {
    columns: Vec<String>,
    rows: Vec<Vec<Option<String>>>,
    truncated: bool,
}

impl Answer {
    fn parse(document: &Value) -> Result<Answer, String> {
        let columns = document["columns"]
            .as_array()
            .ok_or("no `columns` list")?
            .iter()
            .map(|column| column["name"].as_str().map(str::to_string))
            .collect::<Option<Vec<_>>>()
            .ok_or("a column without a `name`")?;
        let truncated = document["truncated"]
            .as_bool()
            .ok_or("no `truncated` flag")?;
        let rows = document["rows"]
            .as_array()
            .ok_or("no `rows` list")?
            .iter()
            .map(|row| {
                let cells = row.as_array().ok_or("a row that is not a list")?;
                if cells.len() != columns.len() {
                    return Err(format!(
                        "a row of {} cells under {} columns",
                        cells.len(),
                        columns.len()
                    ));
                }
                cells
                    .iter()
                    .map(|cell| match cell {
                        Value::Null => Ok(None),
                        Value::String(text) => Ok(Some(text.clone())),
                        other => Err(format!("a cell that is neither text nor null: {other}")),
                    })
                    .collect()
            })
            .collect::<Result<Vec<_>, String>>()?;
        Ok(Answer {
            columns,
            rows,
            truncated,
        })
    }

    /// The column holding the value: the one named `value`, else the one column not named for a
    /// declared dimension. Two columns of the chosen name leave the value ambiguous.
    fn value_column(&self, declared: &[&str]) -> Result<usize, String> {
        match self.positions("value").as_slice() {
            [at] => return Ok(*at),
            [] => {}
            more => {
                return Err(format!(
                    "the query returned {} columns named `value`; which one is the value is \
                     ambiguous",
                    more.len()
                ));
            }
        }
        let candidates: Vec<usize> = (0..self.columns.len())
            .filter(|&at| !declared.contains(&self.columns[at].as_str()))
            .collect();
        match candidates.as_slice() {
            [at] => Ok(*at),
            _ => Err(format!(
                "the query returned columns {:?}; one must be named `value`, or exactly one must \
                 not be a declared dimension",
                self.columns
            )),
        }
    }

    /// Where the columns named `name` are.
    fn positions(&self, name: &str) -> Vec<usize> {
        (0..self.columns.len())
            .filter(|&at| self.columns[at] == name)
            .collect()
    }

    /// The dimension columns: each declared key the query returns, with the column it is in. A key
    /// two columns carry is ambiguous.
    fn dimension_columns<'a>(&self, declared: &[&'a str]) -> Result<Vec<(&'a str, usize)>, String> {
        let mut columns = Vec::new();
        for key in declared {
            match self.positions(key).as_slice() {
                [] => {}
                [at] => columns.push((*key, *at)),
                more => {
                    return Err(format!(
                        "the query returned {} columns named `{key}`; which one sets the dimension \
                         is ambiguous",
                        more.len()
                    ));
                }
            }
        }
        Ok(columns)
    }

    /// Why the `name` answer reads nothing when Connectors cut it; `None` when it is whole.
    fn cut(&self, name: &str) -> Option<String> {
        self.truncated.then(|| {
            format!(
                "the {name} answer was cut at {ROW_LIMIT} rows; series past the cut are not read"
            )
        })
    }
}

/// The dims of `row`: each dimension column holding a non-NULL cell.
fn dims_of(row: &[Option<String>], columns: &[(&str, usize)]) -> BTreeMap<String, String> {
    columns
        .iter()
        .filter_map(|(key, at)| row[*at].clone().map(|value| (key.to_string(), value)))
        .collect()
}

/// The decimal in a cell, or why there is none.
fn number(cell: Option<&str>, column: &str) -> Result<Dec, String> {
    let text = cell.ok_or_else(|| format!("NULL in column `{column}`; unknown, not 0"))?;
    Dec::parse(text).ok_or_else(|| format!("{text:?} in column `{column}` is not a number"))
}

/// One `unknown` read of the whole definition, saying `why`.
fn one_unknown(reads: &Reads<'_>, why: String) -> Vec<Read> {
    vec![reads.unknown(BTreeMap::new(), why)]
}

fn value(client: &ReadClient<'_>, reads: &Reads<'_>) -> Result<Vec<Read>, ConnectorsError> {
    let query = match reads.query("value") {
        Ok(query) => query,
        Err(read) => return Ok(vec![*read]),
    };
    let answer = reads.run(client, query)?;
    if let Some(why) = answer.cut("value") {
        return Ok(one_unknown(reads, why));
    }
    let declared = reads.declared();
    let columns = answer
        .value_column(&declared)
        .and_then(|at| Ok((at, answer.dimension_columns(&declared)?)));
    let (at, dimensions) = match columns {
        Ok(columns) => columns,
        Err(why) => return Ok(one_unknown(reads, why)),
    };
    if answer.rows.is_empty() {
        return Ok(one_unknown(reads, "the query returned no rows".to_string()));
    }
    let column = answer.columns[at].as_str();
    let mut rows_of: BTreeMap<BTreeMap<String, String>, usize> = BTreeMap::new();
    for row in &answer.rows {
        *rows_of.entry(dims_of(row, &dimensions)).or_default() += 1;
    }
    Ok(answer
        .rows
        .iter()
        .filter_map(|row| {
            let dims = dims_of(row, &dimensions);
            match rows_of.get_mut(&dims) {
                Some(n) if *n > 1 => {
                    // Reported once, at the series' first row; neither row is its value.
                    *n = 0;
                    let why = "the query returned this series twice or more".to_string();
                    return Some(reads.unknown(dims, why));
                }
                Some(0) => return None,
                _ => {}
            }
            let raw = row[at].clone();
            Some(match number(raw.as_deref(), column) {
                Ok(dec) => reads.read(
                    dims,
                    Reading::Ok(CompleteReading {
                        value: Decimal(dec.text),
                    }),
                    raw,
                    None,
                ),
                Err(why) => {
                    let mut read = reads.unknown(dims, why);
                    read.raw = raw;
                    read
                }
            })
        })
        .collect())
}

fn count(client: &ReadClient<'_>, reads: &Reads<'_>) -> Result<Vec<Read>, ConnectorsError> {
    let query = match reads.query("value") {
        Ok(query) => query,
        Err(read) => return Ok(vec![*read]),
    };
    let answer = reads.run(client, query)?;
    let dimensions = match answer.dimension_columns(&reads.declared()) {
        Ok(dimensions) => dimensions,
        Err(why) => return Ok(one_unknown(reads, why)),
    };
    if dimensions.is_empty() {
        // One series: a cut answer holds at least the rows it returned.
        let value = Decimal(answer.rows.len().to_string());
        let reading = if answer.truncated {
            Reading::Partial(PartialReading {
                value,
                bound: Bound::Lower,
            })
        } else {
            Reading::Ok(CompleteReading { value })
        };
        return Ok(vec![reads.read(BTreeMap::new(), reading, None, None)]);
    }
    if let Some(why) = answer.cut("value") {
        return Ok(one_unknown(reads, why));
    }
    if answer.rows.is_empty() {
        return Ok(one_unknown(
            reads,
            "the query returned no rows, so no series to count".to_string(),
        ));
    }
    let mut counts: BTreeMap<BTreeMap<String, String>, usize> = BTreeMap::new();
    for row in &answer.rows {
        *counts.entry(dims_of(row, &dimensions)).or_default() += 1;
    }
    Ok(counts
        .into_iter()
        .map(|(dims, n)| {
            let reading = Reading::Ok(CompleteReading {
                value: Decimal(n.to_string()),
            });
            reads.read(dims, reading, None, None)
        })
        .collect())
}

/// One side of a ratio: the cell of each series, keyed by its dims.
type Side = BTreeMap<BTreeMap<String, String>, Result<Option<String>, String>>;

fn side(answer: &Answer, at: usize, dimensions: &[(&str, usize)], name: &str) -> Side {
    let mut cells = Side::new();
    for row in &answer.rows {
        let dims = dims_of(row, dimensions);
        let cell = if cells.contains_key(&dims) {
            Err(format!("the {name} query returned this series twice"))
        } else {
            Ok(row[at].clone())
        };
        cells.insert(dims, cell);
    }
    cells
}

/// Each declared dimension key an answer returns, with the column it is in.
type DimensionColumns<'a> = Vec<(&'a str, usize)>;

/// The value column and dimension columns of one side of a ratio; the reason names the side.
fn ratio_columns<'a>(
    answer: &Answer,
    declared: &[&'a str],
    name: &str,
) -> Result<(usize, DimensionColumns<'a>), String> {
    answer
        .value_column(declared)
        .and_then(|at| Ok((at, answer.dimension_columns(declared)?)))
        .map_err(|why| format!("the {name} query: {why}"))
}

fn ratio(client: &ReadClient<'_>, reads: &Reads<'_>) -> Result<Vec<Read>, ConnectorsError> {
    if let Some(Unit(unit)) = &reads.definition.unit
        && unit != "1"
    {
        return Ok(one_unknown(
            reads,
            format!("a ratio has unit 1; the definition declares unit {unit:?}"),
        ));
    }
    let (numerator, denominator) = match (reads.query("numerator"), reads.query("denominator")) {
        (Ok(n), Ok(d)) => (n, d),
        (Err(read), _) | (_, Err(read)) => return Ok(vec![*read]),
    };
    let numerator = reads.run(client, numerator)?;
    let denominator = reads.run(client, denominator)?;
    if let Some(why) = numerator
        .cut("numerator")
        .or_else(|| denominator.cut("denominator"))
    {
        return Ok(one_unknown(reads, why));
    }
    let declared = reads.declared();
    let (n_columns, d_columns) = match (
        ratio_columns(&numerator, &declared, "numerator"),
        ratio_columns(&denominator, &declared, "denominator"),
    ) {
        (Ok(n), Ok(d)) => (n, d),
        (Err(why), _) | (_, Err(why)) => return Ok(one_unknown(reads, why)),
    };
    let keys = |columns: &[(&str, usize)]| -> Vec<String> {
        columns.iter().map(|(key, _)| key.to_string()).collect()
    };
    if keys(&n_columns.1) != keys(&d_columns.1) {
        return Ok(unpaired(
            reads,
            (&numerator, &n_columns.1),
            (&denominator, &d_columns.1),
        ));
    }
    let numerator = side(&numerator, n_columns.0, &n_columns.1, "numerator");
    let denominator = side(&denominator, d_columns.0, &d_columns.1, "denominator");
    if numerator.is_empty() && denominator.is_empty() {
        return Ok(one_unknown(
            reads,
            "the queries returned no rows".to_string(),
        ));
    }
    // Design § 11 D20, extended: a numerator answer with no rows over a non-empty denominator
    // looks the same as a renamed column or a broken filter, so no series is inferred 0.
    if numerator.is_empty() {
        return Ok(denominator
            .keys()
            .map(|dims| {
                let why = "the numerator answered no rows; no series is inferred 0".to_string();
                reads.unknown(dims.clone(), why)
            })
            .collect());
    }
    // Design § 11 D20: a numerator series without a denominator twin makes the pairing itself
    // doubtful, so the whole pass reads one unknown naming each such series.
    let unmatched: Vec<&BTreeMap<String, String>> = numerator
        .keys()
        .filter(|dims| !denominator.contains_key(*dims))
        .collect();
    if !unmatched.is_empty() {
        return Ok(one_unknown(
            reads,
            format!(
                "the numerator returned series {unmatched:?} the denominator did not; no series \
                 is read"
            ),
        ));
    }
    let unit = reads
        .definition
        .denominator
        .as_ref()
        .map_or_else(|| Unit("1".to_string()), |spec| spec.unit.clone());
    Ok(denominator
        .iter()
        .map(|(dims, denominator)| {
            // Both answers are whole and every numerator series has its twin: a grouped count
            // returns no row for a group with no case, so a series the numerator lacks has
            // numerator 0 (J1).
            let numerator = match numerator.get(dims) {
                None => Ok(Dec::zero()),
                Some(Err(why)) => Err(why.clone()),
                Some(Ok(cell)) => number(cell.as_deref(), "numerator"),
            };
            let denominator = match denominator {
                Err(why) => Err(why.clone()),
                Ok(cell) => number(cell.as_deref(), "denominator"),
            };
            let quotient = numerator.and_then(|n| {
                let d = denominator?;
                Ok((n.quotient(&d)?, d.count()))
            });
            match quotient {
                Ok((value, n)) => reads.read(
                    dims.clone(),
                    Reading::Ok(CompleteReading {
                        value: Decimal(value),
                    }),
                    None,
                    n.map(|n| Population {
                        n,
                        unit: unit.clone(),
                    }),
                ),
                Err(why) => reads.unknown(dims.clone(), why),
            }
        })
        .collect())
}

/// The reads of a ratio whose two queries return different dimension columns: a NULL cell and a
/// column the query does not return both set no key, so the two answers' series cannot be told
/// apart. Each series of the columns both return reads `unknown`; with none, one read does.
fn unpaired(
    reads: &Reads<'_>,
    numerator: (&Answer, &[(&str, usize)]),
    denominator: (&Answer, &[(&str, usize)]),
) -> Vec<Read> {
    let names = |columns: &[(&str, usize)]| -> Vec<String> {
        columns.iter().map(|(key, _)| key.to_string()).collect()
    };
    let why = format!(
        "the numerator returns dimension columns {:?} and the denominator {:?}; their series \
         cannot be paired",
        names(numerator.1),
        names(denominator.1)
    );
    let shared = |columns: &[(&str, usize)], other: &[(&str, usize)]| -> Vec<(String, usize)> {
        columns
            .iter()
            .filter(|(key, _)| other.iter().any(|(k, _)| k == key))
            .map(|(key, at)| (key.to_string(), *at))
            .collect()
    };
    let mut series = std::collections::BTreeSet::new();
    for (answer, columns) in [
        (numerator.0, shared(numerator.1, denominator.1)),
        (denominator.0, shared(denominator.1, numerator.1)),
    ] {
        let columns: Vec<(&str, usize)> = columns.iter().map(|(k, at)| (k.as_str(), *at)).collect();
        for row in &answer.rows {
            series.insert(dims_of(row, &columns));
        }
    }
    if series.is_empty() {
        return one_unknown(reads, why);
    }
    series
        .into_iter()
        .map(|dims| reads.unknown(dims, why.clone()))
        .collect()
}

/// An exact decimal read from a cell: its text in the wire grammar, and its sign, digits (no
/// leading zero, empty for zero) and how many of them are fractional.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Dec {
    text: String,
    negative: bool,
    digits: Vec<u8>,
    scale: usize,
}

impl Dec {
    /// The number a cell's text spells: an optional `-`, digits, an optional `.` and digits, and an
    /// optional exponent (`e` or `E`, an optional sign, digits) as a binary float's text carries.
    /// Text already in the wire grammar is kept as it is, trailing zeros included; an exponent is
    /// written out. `None` for anything else: `NaN`, `Infinity`, the empty text, units, words.
    fn parse(text: &str) -> Option<Dec> {
        let (negative, unsigned) = match text.strip_prefix('-') {
            Some(rest) => (true, rest),
            None => (false, text),
        };
        let (mantissa, exponent) = match unsigned.find(['e', 'E']) {
            Some(at) => (&unsigned[..at], Some(&unsigned[at + 1..])),
            None => (unsigned, None),
        };
        let (whole, fraction) = match mantissa.split_once('.') {
            Some((whole, fraction)) => (whole, fraction),
            None => (mantissa, ""),
        };
        let all_digits = |part: &str| part.bytes().all(|b| b.is_ascii_digit());
        if whole.is_empty()
            || !all_digits(whole)
            || !all_digits(fraction)
            || (mantissa.contains('.') && fraction.is_empty())
        {
            return None;
        }
        let exponent: i64 = match exponent {
            None => 0,
            Some(e) => {
                let digits = e.strip_prefix(['+', '-']).unwrap_or(e);
                if digits.is_empty() || !all_digits(digits) || digits.len() > 6 {
                    return None;
                }
                e.parse().ok()?
            }
        };
        if exponent.abs() > EXPONENT_LIMIT {
            return None;
        }
        let mut digits: Vec<u8> = whole
            .bytes()
            .chain(fraction.bytes())
            .map(|b| b - b'0')
            .collect();
        let mut scale = i64::try_from(fraction.len()).ok()? - exponent;
        if scale < 0 {
            digits.extend(std::iter::repeat_n(0, usize::try_from(-scale).ok()?));
            scale = 0;
        }
        let scale = usize::try_from(scale).ok()?;
        let digits = trimmed(digits);
        let in_grammar = exponent == 0
            && !(whole.len() > 1 && whole.starts_with('0'))
            && !(negative && digits.is_empty());
        let text = if in_grammar {
            text.to_string()
        } else {
            render(negative, &digits, scale, false)
        };
        Some(Dec {
            text,
            negative: negative && !digits.is_empty(),
            digits,
            scale,
        })
    }

    /// Zero.
    fn zero() -> Dec {
        Dec {
            text: "0".to_string(),
            negative: false,
            digits: Vec::new(),
            scale: 0,
        }
    }

    /// This value as a non-negative whole number that fits an `i64`, if it is one.
    fn count(&self) -> Option<i64> {
        if self.negative {
            return None;
        }
        let split = self.digits.len().saturating_sub(self.scale);
        let (whole, fraction) = self.digits.split_at(split);
        if fraction.iter().any(|&d| d != 0) {
            return None;
        }
        whole
            .iter()
            .try_fold(0_i64, |n, &d| n.checked_mul(10)?.checked_add(i64::from(d)))
    }

    /// `self / divisor` in the wire grammar, [`RATIO_SCALE`] fractional digits deep, the last
    /// rounded half away from zero, trailing zeros dropped.
    fn quotient(&self, divisor: &Dec) -> Result<String, String> {
        if divisor.digits.is_empty() {
            return Err("the denominator is 0; the ratio is undefined".to_string());
        }
        // self = A·10^-sa and divisor = B·10^-sb, B without trailing zeros, so the ratio is
        // (A / B)·10^(sb - sa). Its digits at RATIO_SCALE, plus one deciding the rounding, are
        // floor(A·10^p / B) with p = RATIO_SCALE + sb - sa + 1. For p < 0 that is
        // floor(floor(A / 10^-p) / B): A loses its last -p digits.
        let zeros = divisor.digits.iter().rev().take_while(|&&d| d == 0).count();
        let significant = &divisor.digits[..divisor.digits.len() - zeros];
        // Below 10^37, `remainder * 10 + 9` stays below 10^38, inside a u128.
        if significant.len() > 37 {
            return Err(format!(
                "the denominator {} has more significant digits than an exact division takes",
                divisor.text
            ));
        }
        let by = significant
            .iter()
            .fold(0_u128, |n, &d| n * 10 + u128::from(d));
        let width = |n: usize| i128::try_from(n).unwrap_or(i128::MAX);
        let p = width(RATIO_SCALE) + width(divisor.scale) - width(zeros) - width(self.scale) + 1;
        let mut dividend = self.digits.clone();
        let shift = usize::try_from(p.unsigned_abs()).unwrap_or(usize::MAX);
        if p >= 0 {
            dividend.extend(std::iter::repeat_n(0, shift));
        } else {
            dividend.truncate(dividend.len().saturating_sub(shift));
        }
        let mut remainder: u128 = 0;
        let mut step = |digit: u8| {
            let current = remainder * 10 + u128::from(digit);
            remainder = current % by;
            u8::try_from(current / by).unwrap_or_default()
        };
        let mut quotient: Vec<u8> = dividend.iter().map(|&d| step(d)).collect();
        if quotient.pop().unwrap_or(0) >= 5 {
            round_up(&mut quotient);
        }
        Ok(render(
            self.negative != divisor.negative,
            &trimmed(quotient),
            RATIO_SCALE,
            true,
        ))
    }
}

/// Drops leading zeros; zero is the empty vector.
fn trimmed(mut digits: Vec<u8>) -> Vec<u8> {
    let leading = digits.iter().take_while(|&&d| d == 0).count();
    digits.drain(..leading);
    digits
}

fn round_up(digits: &mut Vec<u8>) {
    for digit in digits.iter_mut().rev() {
        if *digit == 9 {
            *digit = 0;
        } else {
            *digit += 1;
            return;
        }
    }
    digits.insert(0, 1);
}

/// A magnitude with `scale` fractional digits in the wire grammar: no leading zero and never
/// `-0`; trailing fractional zeros dropped when `trim` says so, kept otherwise.
fn render(negative: bool, digits: &[u8], scale: usize, trim: bool) -> String {
    let mut padded = vec![0; (scale + 1).saturating_sub(digits.len())];
    padded.extend_from_slice(digits);
    let (whole, fraction) = padded.split_at(padded.len() - scale);
    let text = |part: &[u8]| -> String { part.iter().map(|d| char::from(b'0' + d)).collect() };
    let whole = text(whole);
    let whole = match whole.trim_start_matches('0') {
        "" => "0",
        rest => rest,
    };
    let fraction = text(fraction);
    let fraction = if trim {
        fraction.trim_end_matches('0')
    } else {
        fraction.as_str()
    };
    let zero = whole == "0" && fraction.bytes().all(|b| b == b'0');
    let sign = if negative && !zero { "-" } else { "" };
    if fraction.is_empty() {
        format!("{sign}{whole}")
    } else {
        format!("{sign}{whole}.{fraction}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn text(cell: &str) -> Option<String> {
        Dec::parse(cell).map(|d| d.text)
    }

    #[test]
    fn cell_text_parses_exactly() {
        for (cell, wire) in [
            ("0", "0"),
            ("-0", "0"),
            ("-0.00", "0.00"),
            ("15", "15"),
            ("0.0250", "0.0250"),
            ("007", "7"),
            ("1e-4", "0.0001"),
            ("1E+3", "1000"),
            ("-1.5e+3", "-1500"),
            ("1.50e-1", "0.150"),
            ("1e+300", &format!("1{}", "0".repeat(300))),
            ("-0e5", "0"),
        ] {
            assert_eq!(text(cell).as_deref(), Some(wire), "{cell}");
        }
        for cell in [
            "",
            "-",
            ".5",
            "5.",
            "+5",
            "NaN",
            "Infinity",
            "-Infinity",
            "1e",
            "1e+",
            "1e99999",
            "0x10",
            "1,5",
            " 1",
            "1 ",
            "12 ms",
            "green",
            "1.2.3",
            "--1",
        ] {
            assert_eq!(text(cell), None, "{cell:?}");
        }
    }

    #[test]
    fn quotient_is_exact_to_its_scale() {
        let q = |a: &str, b: &str| {
            Dec::parse(a)
                .expect("a")
                .quotient(&Dec::parse(b).expect("b"))
        };
        assert_eq!(q("15", "556").as_deref(), Ok("0.02697841726618705"));
        assert_eq!(q("1", "4").as_deref(), Ok("0.25"));
        assert_eq!(q("2", "3").as_deref(), Ok("0.666666666666666667"));
        assert_eq!(q("-1", "3").as_deref(), Ok("-0.333333333333333333"));
        assert_eq!(q("0", "7").as_deref(), Ok("0"));
        assert_eq!(q("0.5", "0.25").as_deref(), Ok("2"));
        assert_eq!(q("1e-20", "1").as_deref(), Ok("0"));
        assert_eq!(q("1e-40", "1").as_deref(), Ok("0"));
        assert_eq!(q("1", "1e+50").as_deref(), Ok("0"));
        assert_eq!(q("1e+60", "1e+50").as_deref(), Ok("10000000000"));
        assert_eq!(q("1", "2e+18").as_deref(), Ok("0.000000000000000001"));
        assert_eq!(q("1", "4e+18").as_deref(), Ok("0"));
        assert_eq!(q("15", "556.000").as_deref(), Ok("0.02697841726618705"));
        assert_eq!(
            q("1", "0.0000000000000000000000000000000000000000001").as_deref(),
            Ok("10000000000000000000000000000000000000000000")
        );
        assert!(q("1", "1234567890123456789012345678901234567891").is_err());
        assert!(q("1", "0").is_err());
        assert!(q("1", "0.000").is_err());
    }

    #[test]
    fn count_is_a_whole_non_negative_number() {
        let n = |a: &str| Dec::parse(a).expect("a").count();
        assert_eq!(n("556"), Some(556));
        assert_eq!(n("556.000"), Some(556));
        assert_eq!(n("0"), Some(0));
        assert_eq!(n("556.5"), None);
        assert_eq!(n("-3"), None);
        assert_eq!(n("99999999999999999999"), None);
    }
}
