//! The observation obligations of the synthesis plan (`generated/thalamus-model/PLAN.md`): the
//! command `thalamus.observation.RecordSample` and the view query
//! `thalamus.observation.SeriesSamples`.
//!
//! Each reads and writes only through the generated storage ports: no I/O and no clock of its
//! own. The command selects in the plan's precedence for a command with `when_related:`: the
//! related definition missing, then its state, then the accepting branch.

use std::cmp::Ordering;

use thalamus_model::behaviour::{SignalDefinitionStorage, SignalSampleStorage};
use thalamus_model::definition::SignalDefinitionState;
use thalamus_model::obligation::UnmetObligation;
use thalamus_model::observation::{
    AnySignalSample, RecordSample, RecordSampleOutcome, RecordSampleResponse, SampleId,
    SampleRecorded, SeriesKey, SeriesSamples, SignalNotActive, SignalSample, SignalSampleData,
    UnknownSignal,
};

/// `thalamus.observation.RecordSample`: `unknown-signal`, then `not-active`, then `recorded`
/// under a fresh `smp_` id from the store's [`IdSequence`], which the response returns and the event carries.
///
/// # Errors
///
/// [`UnmetObligation`] naming the identity sequence once it is spent; every input selects a
/// declared outcome.
pub fn record_sample<P>(
    ports: &mut P,
    input: RecordSample,
) -> Result<RecordSampleOutcome, UnmetObligation>
where
    P: SignalDefinitionStorage + SignalSampleStorage + IdSequence + ?Sized,
{
    let Some(definition) = SignalDefinitionStorage::get(ports, &input.signal) else {
        return Ok(RecordSampleOutcome::UnknownSignal {
            error: UnknownSignal {
                signal: input.signal,
            },
        });
    };
    if definition.state != SignalDefinitionState::Active {
        return Ok(RecordSampleOutcome::NotActive {
            error: SignalNotActive {
                signal: input.signal,
            },
        });
    }
    let sample_id = SampleId(mint(ports, "smp_", |ports, id| {
        SignalSampleStorage::get(ports, &SampleId(id.to_owned())).is_some()
    })?);
    let answer = RecordSampleOutcome::Recorded {
        response: RecordSampleResponse {
            sample_id: sample_id.clone(),
        },
        sample_recorded: SampleRecorded {
            sample_id: sample_id.clone(),
            signal: input.signal.clone(),
            series: input.series.clone(),
            ts: input.ts.clone(),
        },
    };
    let data = SignalSampleData {
        sample_id,
        signal: input.signal,
        series: input.series,
        dims: input.dims,
        ts: input.ts,
        window: input.window,
        reading: input.reading,
        unit: input.unit,
        denominator: input.denominator,
        provenance: input.provenance,
    };
    SignalSampleStorage::put(
        ports,
        AnySignalSample::Recorded(SignalSample::new(data)).snapshot(),
    );
    Ok(answer)
}

/// `thalamus.observation.SeriesSamples`: the samples whose `series` is `series`, ordered by `ts`
/// ascending as the view declares (`order_by: [ts asc]`). Two samples at one instant keep the
/// store's order.
///
/// # Errors
///
/// None today. The `Result` is the obligation's contract.
pub fn series_samples<P>(
    ports: &P,
    series: &SeriesKey,
) -> Result<Vec<SeriesSamples>, UnmetObligation>
where
    P: SignalSampleStorage + ?Sized,
{
    let rows: Vec<SeriesSamples> = SignalSampleStorage::list(ports)
        .into_iter()
        .filter(|sample| sample.data.series == *series)
        .map(|sample| SeriesSamples {
            sample_id: sample.data.sample_id,
            ts: sample.data.ts,
            window: sample.data.window,
            reading: sample.data.reading,
            unit: sample.data.unit,
            denominator: sample.data.denominator,
        })
        .collect();
    Ok(in_time_order(rows, |row| row.ts.0.as_str()))
}

/// `rows` in `ts asc` order as [`timestamp_order`] reads it, stable on ties.
///
/// That order is not transitive once a `ts` is no instant (the generated decoder admits any text
/// as a `Timestamp`): an instant and a non-instant compare by bytes, two instants by instant, and
/// three such texts can form a cycle. So the rows are not sorted with it directly. The instants
/// are sorted by instant and the rest by bytes, and the two runs are merged by
/// [`timestamp_order`], which puts every adjacent pair in order whatever the texts are.
fn in_time_order<T>(rows: Vec<T>, ts: impl Fn(&T) -> &str) -> Vec<T> {
    let (mut instants, mut texts): (Vec<T>, Vec<T>) = rows
        .into_iter()
        .partition(|row| Instant::parse(ts(row)).is_some());
    instants.sort_by_key(|row| Instant::parse(ts(row)));
    texts.sort_by(|left, right| ts(left).as_bytes().cmp(ts(right).as_bytes()));
    let mut merged = Vec::with_capacity(instants.len() + texts.len());
    let (mut instants, mut texts) = (
        instants.into_iter().peekable(),
        texts.into_iter().peekable(),
    );
    loop {
        let take_text = match (instants.peek(), texts.peek()) {
            (Some(instant), Some(text)) => timestamp_order(ts(text), ts(instant)).is_lt(),
            (Some(_), None) => false,
            (None, Some(_)) => true,
            (None, None) => return merged,
        };
        let next = if take_text {
            texts.next()
        } else {
            instants.next()
        };
        merged.extend(next);
    }
}

/// Two `Timestamp` wire texts in order as the ESS reference runner ranks them (`compare_nodes`):
/// two instants by the instant they name, so `2026-01-05T10:00:01+02:00` is before
/// `2026-01-05T09:00:03Z`, and any other pair by its UTF-8 bytes.
pub(crate) fn timestamp_order(left: &str, right: &str) -> Ordering {
    match (Instant::parse(left), Instant::parse(right)) {
        (Some(left), Some(right)) => left.cmp(&right),
        _ => left.as_bytes().cmp(right.as_bytes()),
    }
}

/// An instant an RFC 3339 `date-time` names, read with the grammar of the ESS reference
/// (`Rfc3339Instant::parse_rfc3339`) and nothing wider: a `T` or `t` separator, seconds `00` to
/// `59`, one to nine fraction digits, and `Z`, `z` or a `±HH:MM` offset. No zone database and no
/// clock. The one instant reader of this crate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Instant {
    seconds: i64,
    nanos: u32,
}

impl Instant {
    /// The instant `text` names, or `None` where it is not an RFC 3339 `date-time`.
    pub(crate) fn parse(text: &str) -> Option<Self> {
        let bytes = text.as_bytes();
        let digits = |from: usize, to: usize| -> Option<u32> {
            let slice = bytes.get(from..to)?;
            if slice.is_empty() || !slice.iter().all(u8::is_ascii_digit) {
                return None;
            }
            slice.iter().try_fold(0u32, |total, digit| {
                Some(total * 10 + u32::from(digit - b'0'))
            })
        };
        let at = |index: usize, expected: &[u8]| {
            bytes.get(index).is_some_and(|byte| expected.contains(byte))
        };
        if !(at(4, b"-") && at(7, b"-") && at(10, b"Tt") && at(13, b":") && at(16, b":")) {
            return None;
        }
        let (year, month, day) = (digits(0, 4)?, digits(5, 7)?, digits(8, 10)?);
        let leap =
            (year.is_multiple_of(4) && !year.is_multiple_of(100)) || year.is_multiple_of(400);
        let length = match month {
            1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
            4 | 6 | 9 | 11 => 30,
            2 if leap => 29,
            2 => 28,
            _ => return None,
        };
        let (hour, minute, second) = (digits(11, 13)?, digits(14, 16)?, digits(17, 19)?);
        if day < 1 || day > length || hour > 23 || minute > 59 || second > 59 {
            return None;
        }
        let mut position = 19;
        let mut nanos = 0;
        if at(position, b".") {
            let start = position + 1;
            let mut end = start;
            while bytes.get(end).is_some_and(u8::is_ascii_digit) {
                end += 1;
            }
            let width = u32::try_from(end - start).ok()?;
            if width == 0 || width > 9 {
                return None;
            }
            nanos = digits(start, end)? * 10u32.pow(9 - width);
            position = end;
        }
        let offset = match bytes.get(position..)? {
            b"Z" | b"z" => 0,
            [sign @ (b'+' | b'-'), _, _, b':', _, _] => {
                let (hours, minutes) = (
                    digits(position + 1, position + 3)?,
                    digits(position + 4, position + 6)?,
                );
                if hours > 23 || minutes > 59 {
                    return None;
                }
                let magnitude = i64::from(hours * 3600 + minutes * 60);
                if *sign == b'-' { -magnitude } else { magnitude }
            }
            _ => return None,
        };
        let days = days_from_civil(i64::from(year), i64::from(month), i64::from(day));
        Some(Self {
            seconds: days * 86_400 + i64::from(hour * 3600 + minute * 60 + second) - offset,
            nanos,
        })
    }

    /// The whole seconds from 1970-01-01T00:00:00Z, the fraction dropped.
    pub(crate) fn seconds(self) -> i64 {
        self.seconds
    }
}

/// Days from 1970-01-01 to the civil date.
pub(crate) fn days_from_civil(year: i64, month: i64, day: i64) -> i64 {
    let year = if month <= 2 { year - 1 } else { year };
    let era = year.div_euclid(400);
    let of_era = year - era * 400;
    let of_year = (153 * (if month > 2 { month - 3 } else { month + 9 }) + 2) / 5 + day - 1;
    let of_cycle = of_era * 365 + of_era / 4 - of_era / 100 + of_year;
    era * 146_097 + of_cycle - 719_468
}

/// Where a behaviour takes the number of the next identity it issues: one never-decreasing
/// sequence per prefix, belonging to the store the behaviour writes to, so that every writer to
/// one store draws from the same sequence.
///
/// The specification leaves the identity of a recorded sample, evaluation and assertion to the
/// implementation, and its generated context port mints none. An identity is never issued twice
/// by one store, also after the row that carried it is deleted, because an evaluation or a
/// transition still references it (design § 2.3, D13). The in-memory harness holds one sequence
/// per process for every in-memory store (`crate::conformance`); a durable store mints a ULID
/// instead (design § 2.2).
pub trait IdSequence {
    /// The next number of the sequence of `prefix`, one higher than the last it answered, or
    /// `None` once the sequence is spent.
    fn next_number(&mut self, prefix: &'static str) -> Option<u64>;
}

/// The widest number an identity carries: twelve digits.
const LAST_NUMBER: u64 = 999_999_999_999;

/// The next identity of `prefix` the store does not hold: `prefix` and the next number of its
/// sequence in twelve digits (`smp_000000000001`), skipping any `taken` already claims.
///
/// Fixed width makes the identities of one sequence sort as text in the order they were issued,
/// as a ULID does, so a store keyed by identity keeps rows in the order they were recorded.
///
/// # Errors
///
/// [`UnmetObligation`] naming the identity sequence once it is spent, or once it passes twelve
/// digits.
pub(crate) fn mint<P>(
    ports: &mut P,
    prefix: &'static str,
    taken: impl Fn(&P, &str) -> bool,
) -> Result<String, UnmetObligation>
where
    P: IdSequence + ?Sized,
{
    loop {
        let Some(number) = ports
            .next_number(prefix)
            .filter(|number| *number <= LAST_NUMBER)
        else {
            return Err(UnmetObligation {
                capability: "identity sequence",
                source: prefix,
            });
        };
        let id = format!("{prefix}{number:012}");
        if !taken(ports, &id) {
            return Ok(id);
        }
    }
}

#[cfg(test)]
mod tests {
    use std::cmp::Ordering;

    use super::{Instant, in_time_order, timestamp_order};

    #[test]
    fn the_instant_reader_admits_the_reference_grammar_and_nothing_wider() {
        let at = |text: &str| Instant::parse(text).map(Instant::seconds);
        assert_eq!(at("2026-01-01T00:00:00Z"), Some(1_767_225_600));
        assert_eq!(
            at("2026-01-01t01:00:00.123456789+01:00"),
            Some(1_767_225_600)
        );
        assert_eq!(at("2000-02-29T00:00:00z"), Some(951_782_400));
        for refused in [
            "2026-01-05 10:00:01Z",
            "2026-01-05T10:00:00.0000000001+02:00",
            "2026-01-05T10:00:60Z",
            "2026-01-05T10:00:00.Z",
            "2026-01-04T00:00:00",
            "2026-02-29T00:00:00Z",
            "2026-01-05T10:00:00+24:00",
            "yesterday",
        ] {
            assert_eq!(at(refused), None, "{refused}");
        }
    }

    #[test]
    fn two_instants_order_by_instant_and_any_other_pair_by_bytes() {
        assert_eq!(
            timestamp_order("2026-01-05T10:00:01+02:00", "2026-01-05T09:00:03Z"),
            Ordering::Less
        );
        assert_eq!(
            timestamp_order("2026-01-05T09:00:02.5Z", "2026-01-05T09:00:02.25Z"),
            Ordering::Greater
        );
        assert_eq!(
            timestamp_order("2026-01-05T09:00:03Z", "2026-01-05 10:00:01Z"),
            Ordering::Greater
        );
        assert_eq!(
            timestamp_order("2026-01-04T00:00:00", "2026-01-05T00:00:00Z"),
            Ordering::Less
        );
    }

    /// Three texts whose pairwise order is a cycle (`A < B` by instant, `B < C` and `C < A` by
    /// bytes) still come out with every adjacent pair in order, and nothing panics.
    #[test]
    fn rows_come_out_with_every_adjacent_pair_in_order() {
        let cases: [&[&str]; 3] = [
            &[
                "2026-01-05T09:30:00",
                "2026-01-05T09:00:00Z",
                "2026-01-05T10:00:00+02:00",
            ],
            &[
                "2026-01-05T00:00:00Z",
                "2026-01-04T00:00:00",
                "2026-01-05T10:00:02.25+01:00",
                "2026-01-05T09:00:02.5Z",
                "2026-01-05 10:00:01Z",
                "2026-01-05T09:00:03Z",
            ],
            &[],
        ];
        for rows in cases {
            let ordered = in_time_order(rows.to_vec(), |row| *row);
            assert_eq!(ordered.len(), rows.len());
            for pair in ordered.windows(2) {
                assert_ne!(
                    timestamp_order(pair[0], pair[1]),
                    Ordering::Greater,
                    "{ordered:?}"
                );
            }
        }
        let ties = in_time_order(
            vec![
                ("2026-01-05T09:00:00Z", 1),
                ("2026-01-05T10:00:00+01:00", 2),
            ],
            |row| row.0,
        );
        assert_eq!(
            ties.iter().map(|row| row.1).collect::<Vec<_>>(),
            [1, 2],
            "equal instants keep the store's order"
        );
    }
}
