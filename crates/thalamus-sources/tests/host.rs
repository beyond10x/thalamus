//! The local host collector (`host.` extractors), read through the extractor registry over the
//! synthetic machine trees of `tests/fixtures/host/`.
//!
//! Each test reads a fixture root in place of `/` through [`Rooted`], the collector's own file
//! reader. The test machine wraps it for the two things a file tree cannot hold: the second read of
//! `/proc/stat` answers `proc/stat.after`, and `statvfs` answers the fixture's `statvfs` table. It
//! logs every path the collector asked for, so a test can tell that nothing was read.

use std::cell::{Cell, RefCell};
use std::collections::BTreeMap;
use std::io;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use thalamus_model::definition::{
    AdapterId, Author, ConnectionId, EvaluationSpec, ExtractorId, Family, History, Links, Owner,
    QueryText, RuleId, SignalDefinitionData, SignalId, SignalKind, Source, Unit,
};
use thalamus_model::observation::{CompleteReading, Reading, RunId};
use thalamus_model::primitives::{Decimal, Duration, Timestamp};
use thalamus_sources::adapter::{Adapter, Read, query_digest, window_ending};
use thalamus_sources::connectors::Connectors;
use thalamus_sources::host::{Host, Machine, NOT_LOCAL, Rooted, Space};
use thalamus_sources::registry::{self, SourceKind};

const END: &str = "2026-10-14T22:57:03Z";

/// A fixture root, read as the machine.
struct Fixture {
    files: Rooted,
    root: PathBuf,
    stat_reads: Cell<usize>,
    asked: Rc<RefCell<Vec<String>>>,
}

impl Fixture {
    fn new(case: &str) -> (Fixture, Rc<RefCell<Vec<String>>>) {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/host")
            .join(case);
        let asked = Rc::new(RefCell::new(Vec::new()));
        let fixture = Fixture {
            files: Rooted::new(&root, std::time::Duration::ZERO),
            root,
            stat_reads: Cell::new(0),
            asked: Rc::clone(&asked),
        };
        (fixture, asked)
    }
}

impl Machine for Fixture {
    fn read(&self, path: &str) -> io::Result<String> {
        self.asked.borrow_mut().push(path.to_string());
        if path == "/proc/stat" {
            let n = self.stat_reads.get();
            self.stat_reads.set(n + 1);
            if n > 0 {
                return self.files.read("/proc/stat.after");
            }
        }
        self.files.read(path)
    }

    fn list(&self, dir: &str) -> io::Result<Vec<String>> {
        self.asked.borrow_mut().push(dir.to_string());
        self.files.list(dir)
    }

    fn space(&self, mount: &str) -> io::Result<Space> {
        self.asked.borrow_mut().push(format!("statvfs {mount}"));
        let table = std::fs::read_to_string(self.root.join("statvfs"))?;
        table
            .lines()
            .map(|line| line.split_whitespace().collect::<Vec<_>>())
            .find(|fields| fields.first() == Some(&mount))
            .map(|fields| {
                let n = |i: usize| fields[i].parse::<u64>().expect("a block count");
                Space {
                    blocks: n(1),
                    free: n(2),
                    available: n(3),
                }
            })
            .ok_or_else(|| io::Error::from(io::ErrorKind::NotFound))
    }

    fn pause(&self) {}
}

fn definition(extractor: &str, unit: &str, query: &[(&str, &str)]) -> SignalDefinitionData {
    SignalDefinitionData {
        signal_id: SignalId("platform.host.probe".into()),
        title: "This machine".into(),
        family: Family("platform".into()),
        kind: SignalKind::Gauge,
        unit: Some(Unit(unit.into())),
        dimensions: Some(Vec::new()),
        source: Source {
            adapter: AdapterId("host".into()),
            connection: ConnectionId("local".into()),
        },
        extractor: ExtractorId(extractor.into()),
        query: query
            .iter()
            .map(|(k, v)| (k.to_string(), QueryText(v.to_string())))
            .collect(),
        window: Duration("PT5M".into()),
        cadence: Duration("PT5M".into()),
        history: History::Store,
        denominator: None,
        evaluation: EvaluationSpec {
            rule: RuleId("threshold".into()),
            baselines: Vec::new(),
            warn: None,
            crit: None,
            min_trend_window: None,
            fixed: None,
            rolling_n: None,
            mapping: None,
        },
        persistence: None,
        objective: None,
        owner: Some(Owner("team:platform".into())),
        links: Links {
            components: Vec::new(),
            issues: Vec::new(),
        },
        rev: 1,
        revised_at: Timestamp("2026-10-01T00:00:00Z".into()),
        revised_by: Author("tester".into()),
        mute_reason: None,
        mute_until: None,
    }
}

fn end() -> Timestamp {
    Timestamp(END.into())
}

/// The reads of `definition` on the machine of fixture `case`, through the registry's read path.
fn read(case: &str, definition: &SignalDefinitionData) -> (Vec<Read>, Vec<String>) {
    let (fixture, asked) = Fixture::new(case);
    let host = Host::new(fixture);
    let reads = registry::read_with(
        Some(&host as &dyn Adapter),
        &Connectors::new("/nonexistent/connectors"),
        definition,
        &end(),
        &RunId("run_test".into()),
    );
    let asked = asked.borrow().clone();
    (reads, asked)
}

fn one(reads: Vec<Read>) -> Read {
    let [read] = <[Read; 1]>::try_from(reads).unwrap_or_else(|reads| {
        panic!("one series expected, got {reads:#?}");
    });
    read
}

fn value(v: &str) -> Reading {
    Reading::Ok(CompleteReading {
        value: Decimal(v.into()),
    })
}

fn reason(reading: &Reading) -> &str {
    match reading {
        Reading::Unknown(unread) => &unread.reason,
        other => panic!("expected an unknown reading, got {other:?}"),
    }
}

/// The provenance every host read carries.
fn assert_provenance(read: &Read, definition: &SignalDefinitionData) {
    assert_eq!(read.adapter, AdapterId("host".into()));
    assert_eq!(read.connection, definition.source.connection);
    assert_eq!(read.query_sha256, query_digest(&definition.query));
    assert_eq!(
        read.window,
        window_ending(&Duration("PT5M".into()), &end()).expect("the window is placeable")
    );
    assert_eq!(read.denominator, None);
    assert_eq!(read.retention_start, None);
}

#[test]
fn cpu_busy_is_one_minus_idle_share_of_the_delta() {
    // before: total 10000, idle 8000, iowait 500; after: total 11600, idle 8600, iowait 600.
    // busy = 1 - (600 + 100) / 1600 = 0.5625. guest and guest_nice are inside user and nice.
    let definition = definition("host.cpu_busy", "1", &[]);
    let (reads, asked) = read("busy", &definition);
    let read = one(reads);
    assert_eq!(read.reading, value("0.5625"));
    assert_eq!(read.dims, BTreeMap::new());
    assert_eq!(
        read.raw.as_deref(),
        Some("cpu  1000 0 500 8000 500 0 0 0 0 0\ncpu  1600 0 700 8600 600 0 100 0 0 0")
    );
    assert_provenance(&read, &definition);
    assert_eq!(asked, ["/proc/stat", "/proc/stat"]);
}

#[test]
fn memory_used_is_total_minus_available_over_total() {
    // (16000000 - 4000000) / 16000000 = 0.75
    let definition = definition("host.memory_used", "1", &[]);
    let (reads, asked) = read("busy", &definition);
    let read = one(reads);
    assert_eq!(read.reading, value("0.75"));
    assert_eq!(read.dims, BTreeMap::new());
    assert_eq!(
        read.raw.as_deref(),
        Some("MemTotal:       16000000 kB\nMemAvailable:    4000000 kB")
    );
    assert_provenance(&read, &definition);
    assert_eq!(asked, ["/proc/meminfo"]);
}

#[test]
fn disk_used_matches_df_per_listed_mount() {
    // df: used = f_blocks - f_bfree; use% = used / (used + f_bavail).
    // /: 700 / (700 + 175) = 0.8; /data: 500 / (500 + 1500) = 0.25.
    let definition = definition("host.disk_used", "1", &[("root", "/"), ("data", "/data")]);
    let (reads, asked) = read("busy", &definition);
    let by_mount: BTreeMap<String, Read> = reads
        .into_iter()
        .map(|read| (read.dims["mount"].clone(), read))
        .collect();
    assert_eq!(
        by_mount.keys().collect::<Vec<_>>(),
        ["/", "/data"],
        "one series per listed mount"
    );
    assert_eq!(by_mount["/"].reading, value("0.8"));
    assert_eq!(
        by_mount["/"].raw.as_deref(),
        Some("f_blocks=1000 f_bfree=300 f_bavail=175")
    );
    assert_eq!(by_mount["/data"].reading, value("0.25"));
    for read in by_mount.values() {
        assert_eq!(read.dims.len(), 1, "mount is the only dimension");
        assert_provenance(read, &definition);
    }
    let mut asked = asked;
    asked.sort();
    assert_eq!(asked, ["statvfs /", "statvfs /data"]);
}

#[test]
fn cpu_temperature_is_hottest_cpu_sensor_in_celsius() {
    // coretemp reads 45000, 61500 and 52000 millidegrees: 61.5 Cel. The nvme sensor at 70000 and
    // the x86_pkg_temp zone at 99000 are not read: a CPU hwmon sensor is preferred to a zone.
    let definition = definition("host.cpu_temperature", "Cel", &[]);
    let (reads, asked) = read("busy", &definition);
    let read = one(reads);
    assert_eq!(read.reading, value("61.5"));
    assert_eq!(read.raw.as_deref(), Some("61500"));
    assert_eq!(read.dims, BTreeMap::new());
    assert_provenance(&read, &definition);
    assert!(
        asked
            .iter()
            .all(|path| path.starts_with("/sys/class/hwmon")),
        "a CPU hwmon sensor was found, so no thermal zone is read: {asked:?}"
    );
    assert!(
        !asked.iter().any(|path| path.ends_with("_crit")),
        "only temp*_input is read: {asked:?}"
    );
}

#[test]
fn cpu_temperature_falls_back_to_a_cpu_thermal_zone() {
    // No CPU hwmon; zone acpitz (80000) is no CPU zone, zone cpu-thermal reads 48250: 48.25 Cel.
    let definition = definition("host.cpu_temperature", "Cel", &[]);
    let (reads, _) = read("thermal_zone", &definition);
    let read = one(reads);
    assert_eq!(read.reading, value("48.25"));
    assert_eq!(read.raw.as_deref(), Some("48250"));
}

#[test]
fn no_cpu_sensor_reads_unknown_never_zero() {
    let definition = definition("host.cpu_temperature", "Cel", &[]);
    let (reads, _) = read("no_sensor", &definition);
    let read = one(reads);
    let why = reason(&read.reading);
    assert!(why.contains("no CPU temperature sensor"), "{why}");
    assert_eq!(read.raw, None);
    assert_provenance(&read, &definition);
}

#[test]
fn missing_meminfo_field_reads_unknown() {
    let definition = definition("host.memory_used", "1", &[]);
    let (reads, _) = read("meminfo_missing_field", &definition);
    let no_field = one(reads);
    let why = reason(&no_field.reading);
    assert!(why.contains("MemAvailable"), "{why}");
    assert_eq!(no_field.raw, None);
    assert_provenance(&no_field, &definition);

    // A machine with no /proc/meminfo at all reads unknown too, naming the file.
    let (reads, _) = read("thermal_zone", &definition);
    let no_file = one(reads);
    let why = reason(&no_file.reading);
    assert!(why.contains("/proc/meminfo"), "{why}");
}

#[test]
fn unknown_mount_reads_unknown() {
    let definition = definition(
        "host.disk_used",
        "1",
        &[("root", "/"), ("gone", "/not-mounted")],
    );
    let (reads, _) = read("busy", &definition);
    let by_mount: BTreeMap<String, Read> = reads
        .into_iter()
        .map(|read| (read.dims["mount"].clone(), read))
        .collect();
    assert_eq!(by_mount["/"].reading, value("0.8"));
    let gone = &by_mount["/not-mounted"];
    let why = reason(&gone.reading);
    assert!(why.contains("/not-mounted"), "{why}");
    assert_eq!(gone.raw, None);
    assert_provenance(gone, &definition);

    // A disk definition listing no mount reads one unknown, never an empty pass.
    let definition = definition_without_query("host.disk_used");
    let (reads, _) = read("busy", &definition);
    let why = reason(&one(reads).reading).to_string();
    assert!(why.contains("mount"), "{why}");
}

fn definition_without_query(extractor: &str) -> SignalDefinitionData {
    definition(extractor, "1", &[])
}

#[test]
fn connection_other_than_local_reads_unknown() {
    for extractor in [
        "host.cpu_busy",
        "host.memory_used",
        "host.disk_used",
        "host.cpu_temperature",
    ] {
        let unit = if extractor == "host.cpu_temperature" {
            "Cel"
        } else {
            "1"
        };
        let mut definition = definition(extractor, unit, &[("root", "/")]);
        definition.source.connection = ConnectionId("forge-main".into());
        let (reads, asked) = read("busy", &definition);
        let read = one(reads);
        assert_eq!(reason(&read.reading), NOT_LOCAL);
        assert_eq!(read.raw, None);
        assert_eq!(read.connection, ConnectionId("forge-main".into()));
        assert!(asked.is_empty(), "{extractor} read this machine: {asked:?}");
    }
    assert_eq!(NOT_LOCAL, "host reads only this machine (connection local)");
}

#[test]
fn host_prefix_chooses_the_host_adapter_with_no_connectors_operation() {
    assert_eq!(
        registry::choose(&ExtractorId("host.cpu_busy".into())),
        Some(SourceKind::Host)
    );
    assert_eq!(SourceKind::Host.prefix(), "host.");
    let adapter = SourceKind::Host.adapter().expect("the host adapter exists");
    assert!(adapter.read_operations().is_empty());
}
