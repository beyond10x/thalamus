//! Adversarial pass 1 on the local host collector (`host.` extractors).
//!
//! Most cases read an in-memory machine (`Mem`), which logs every path asked for. The
//! `statvfs`-on-a-non-mount cases read `tests/fixtures/host/adv/plain` through the collector's own
//! `Rooted` reader, so `statvfs` is the real call.

use std::cell::{Cell, RefCell};
use std::collections::BTreeMap;
use std::io;
use std::path::Path;

use thalamus_core::behaviour::definition::input_refusals;
use thalamus_model::definition::{
    AdapterId, Author, ConnectionId, EvaluationSpec, ExtractorId, Family, History, Links, Owner,
    QueryText, RuleId, SignalDefinitionData, SignalId, SignalKind, Source, Unit,
};
use thalamus_model::observation::{Reading, RunId};
use thalamus_model::primitives::{Duration, Timestamp};
use thalamus_sources::adapter::{Adapter, Read, query_digest};
use thalamus_sources::connectors::Connectors;
use thalamus_sources::host::{Host, Machine, Rooted, Space};
use thalamus_sources::registry;

const END: &str = "2026-10-14T22:57:03Z";

/// An in-memory machine: files by absolute path, the successive answers of `/proc/stat`, and a
/// `statvfs` table. Every path asked for is logged.
#[derive(Default)]
struct Mem {
    files: BTreeMap<String, String>,
    stats: Vec<String>,
    stat_n: Cell<usize>,
    spaces: BTreeMap<String, Space>,
    asked: RefCell<Vec<String>>,
}

impl Mem {
    fn file(mut self, path: &str, text: &str) -> Self {
        self.files.insert(path.to_string(), text.to_string());
        self
    }
    fn stat(mut self, text: &str) -> Self {
        self.stats.push(text.to_string());
        self
    }
    fn space(mut self, mount: &str, blocks: u64, free: u64, available: u64) -> Self {
        self.spaces.insert(
            mount.to_string(),
            Space {
                blocks,
                free,
                available,
                fragment: 4096,
            },
        );
        self
    }
}

impl Machine for Mem {
    fn read(&self, path: &str) -> io::Result<String> {
        self.asked.borrow_mut().push(path.to_string());
        if path == "/proc/stat" && !self.stats.is_empty() {
            let n = self.stat_n.get();
            self.stat_n.set(n + 1);
            return self
                .stats
                .get(n)
                .cloned()
                .ok_or_else(|| io::Error::from(io::ErrorKind::NotFound));
        }
        self.files
            .get(path)
            .cloned()
            .ok_or_else(|| io::Error::from(io::ErrorKind::NotFound))
    }

    fn list(&self, dir: &str) -> io::Result<Vec<String>> {
        self.asked.borrow_mut().push(dir.to_string());
        let prefix = format!("{dir}/");
        let mut names: Vec<String> = self
            .files
            .keys()
            .filter_map(|k| k.strip_prefix(&prefix))
            .map(|rest| rest.split('/').next().unwrap_or_default().to_string())
            .collect();
        names.sort();
        names.dedup();
        if names.is_empty() {
            return Err(io::Error::from(io::ErrorKind::NotFound));
        }
        Ok(names)
    }

    fn space(&self, mount: &str) -> io::Result<Space> {
        self.asked.borrow_mut().push(format!("statvfs {mount}"));
        self.spaces
            .get(mount)
            .copied()
            .ok_or_else(|| io::Error::from(io::ErrorKind::NotFound))
    }

    fn mount_point(&self, mount: &str) -> io::Result<bool> {
        Ok(self.spaces.contains_key(mount))
    }

    /// The in-memory machine is read at [`END`].
    fn now(&self) -> i64 {
        1_792_018_623
    }

    fn pause(&self) {}
}

fn definition(extractor: &str, unit: &str, query: &[(&str, &str)]) -> SignalDefinitionData {
    SignalDefinitionData {
        signal_id: SignalId("platform.host.adv".into()),
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

fn read_at<M: Machine>(machine: M, definition: &SignalDefinitionData, end: &str) -> Vec<Read> {
    let host = Host::new(machine);
    registry::read_with(
        Some(&host as &dyn Adapter),
        &Connectors::new("/nonexistent/connectors"),
        definition,
        &Timestamp(end.into()),
        &RunId("run_adv".into()),
    )
}

fn read<M: Machine>(machine: M, definition: &SignalDefinitionData) -> Vec<Read> {
    read_at(machine, definition, END)
}

fn one(reads: Vec<Read>) -> Read {
    let [read] = <[Read; 1]>::try_from(reads).unwrap_or_else(|reads| {
        panic!("one series expected, got {reads:#?}");
    });
    read
}

fn ok_value(reading: &Reading) -> Option<f64> {
    match reading {
        Reading::Ok(complete) => Some(complete.value.0.parse::<f64>().expect("a decimal")),
        _ => None,
    }
}

fn assert_unknown(read: &Read, context: &str) {
    assert!(
        matches!(read.reading, Reading::Unknown(_)),
        "{context}: expected unknown, got {:?} (raw {:?})",
        read.reading,
        read.raw
    );
    assert_eq!(read.raw, None, "{context}: an unknown read carries no raw");
}

fn plain_root() -> Rooted {
    Rooted::new(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/host/adv/plain"),
        std::time::Duration::ZERO,
    )
}

// ---------------------------------------------------------------------------------------------
// Red: findings.
// ---------------------------------------------------------------------------------------------

/// A non-idle counter (user) going backwards while idle advances: Δtotal = 50, Δidle = 100. The
/// collector checks only total, idle and iowait for regression, so it computes busy =
/// 1 − 100/50 = −1 and reports it `ok`. A busy share outside [0, 1] is no known answer.
#[test]
fn cpu_counter_regression_never_reads_a_busy_share_outside_zero_to_one() {
    let machine = Mem::default()
        .stat("cpu  1000 0 500 8000 500 0 0 0 0 0\n")
        .stat("cpu  950 0 500 8100 500 0 0 0 0 0\n");
    let read = one(read(machine, &definition("host.cpu_busy", "1", &[])));
    if let Some(v) = ok_value(&read.reading) {
        assert!(
            (0.0..=1.0).contains(&v),
            "busy share {v} read ok from a regressed user counter; expected unknown"
        );
    }
}

/// `/data` is a plain directory on the root file system, not a mount point (an unmounted disk's
/// mount directory looks exactly like this). `statvfs` of it answers the root file system, and the
/// collector records the root's usage under `mount=/data`. The story: "a mount that does not
/// exist ... reads unknown".
#[test]
fn disk_used_of_a_directory_that_is_no_mount_point_reads_unknown() {
    let read = one(read(
        plain_root(),
        &definition("host.disk_used", "1", &[("data", "/data")]),
    ));
    assert_eq!(read.dims.get("mount").map(String::as_str), Some("/data"));
    assert_unknown(&read, "/data is a directory, not a mount point");
}

/// `statvfs` of a regular file answers the file system holding it; a file is no mount point.
#[test]
fn disk_used_of_a_regular_file_reads_unknown() {
    let read = one(read(
        plain_root(),
        &definition("host.disk_used", "1", &[("f", "/file.txt")]),
    ));
    assert_unknown(&read, "/file.txt is a regular file");
}

/// The collector reads the machine as it is now, and stamps it with the window ending at `end`.
/// `thalamus sample --at 2001-01-01T00:00:00Z` (`crates/thalamus-cli/src/cmd/sample.rs`) records
/// today's memory use as the reading of a window 25 years ago. A host read cannot read the past;
/// a window it did not observe is unknown.
#[test]
fn a_window_end_far_from_now_reads_unknown_not_this_moment() {
    let machine = Mem::default().file(
        "/proc/meminfo",
        "MemTotal:       16000000 kB\nMemAvailable:    4000000 kB\n",
    );
    let read = one(read_at(
        machine,
        &definition("host.memory_used", "1", &[]),
        "2001-01-01T00:00:00Z",
    ));
    assert_unknown(&read, "a host read stamped 2001");
}

/// `spec/domains/definition.yaml` (ConnectionId naming): "The id local is reserved for the host
/// adapter". Nothing refuses another adapter on connection `local`.
#[test]
fn local_connection_is_reserved_for_the_host_adapter() {
    let def = definition("prometheus.query", "1", &[]);
    let source = Source {
        adapter: AdapterId("prometheus".into()),
        connection: ConnectionId("local".into()),
    };
    let refusals = input_refusals(def.kind, &source, &def.evaluation, None);
    assert!(
        !refusals.is_empty(),
        "a prometheus source on the reserved connection local is accepted"
    );
}

/// `/` and `//` are one path and one mount; the collector records two series of one file system.
#[test]
fn one_mount_spelled_twice_is_one_series() {
    let machine = Mem::default()
        .space("/", 1000, 300, 175)
        .space("//", 1000, 300, 175);
    let reads = read(
        machine,
        &definition("host.disk_used", "1", &[("a", "/"), ("b", "//")]),
    );
    let mounts: Vec<_> = reads.iter().map(|r| r.dims["mount"].clone()).collect();
    assert_eq!(mounts, ["/"], "one mount, one series");
}

// ---------------------------------------------------------------------------------------------
// Guards that held (green): kept so a regression turns them red.
// ---------------------------------------------------------------------------------------------

#[test]
fn zero_cpu_total_delta_reads_unknown() {
    let line = "cpu  1000 0 500 8000 500 0 0 0 0 0\n";
    let read = one(read(
        Mem::default().stat(line).stat(line),
        &definition("host.cpu_busy", "1", &[]),
    ));
    assert_unknown(&read, "zero total");
}

#[test]
fn cpu_total_going_backwards_reads_unknown() {
    let read = one(read(
        Mem::default()
            .stat("cpu  1000 0 500 8000 500 0 0 0 0 0\n")
            .stat("cpu  10 0 5 80 5 0 0 0 0 0\n"),
        &definition("host.cpu_busy", "1", &[]),
    ));
    assert_unknown(&read, "counter wrap");
}

#[test]
fn malformed_cpu_lines_read_unknown() {
    for bad in [
        "cpu  1000 0 x 8000 500\n",
        "cpu  1000 0 500\n",
        "cpu0 1 2 3 4 5\n",
        "",
        "cpu  -1 0 500 8000 500\n",
    ] {
        let read = one(read(
            Mem::default()
                .stat(bad)
                .stat("cpu  2000 0 500 9000 500 0 0 0\n"),
            &definition("host.cpu_busy", "1", &[]),
        ));
        assert_unknown(&read, &format!("cpu line {bad:?}"));
    }
}

#[test]
fn meminfo_defects_read_unknown() {
    for text in [
        "MemTotal: 1000 kB\nMemAvailable: 2000 kB\n",
        "MemTotal: 0 kB\nMemAvailable: 0 kB\n",
        "MemTotal: abc kB\nMemAvailable: 10 kB\n",
        "MemTotal: 1000 kB\nMemAvailable:\n",
        "MemAvailable: 10 kB\n",
        "",
    ] {
        let read = one(read(
            Mem::default().file("/proc/meminfo", text),
            &definition("host.memory_used", "1", &[]),
        ));
        assert_unknown(&read, &format!("meminfo {text:?}"));
    }
}

#[test]
fn disk_query_defects_read_unknown() {
    let machine = || {
        Mem::default()
            .space("/", 1000, 300, 175)
            .space("rel", 1, 0, 1)
    };
    let empty = one(read(machine(), &definition("host.disk_used", "1", &[])));
    assert_unknown(&empty, "empty query");
    let relative = one(read(
        machine(),
        &definition("host.disk_used", "1", &[("r", "rel")]),
    ));
    assert_unknown(&relative, "relative mount");
    let machine = Mem::default().space("/z", 0, 0, 0).space("/w", 10, 20, 5);
    for each in read(
        machine,
        &definition("host.disk_used", "1", &[("z", "/z"), ("w", "/w")]),
    ) {
        assert_unknown(&each, "zero or inconsistent statvfs");
    }
}

#[test]
fn non_cpu_hwmon_is_never_chosen_and_the_hottest_cpu_input_wins() {
    let machine = Mem::default()
        .file("/sys/class/hwmon/hwmon0/name", "amdgpu\n")
        .file("/sys/class/hwmon/hwmon0/temp1_input", "95000\n")
        .file("/sys/class/hwmon/hwmon1/name", "nvme\n")
        .file("/sys/class/hwmon/hwmon1/temp1_input", "90000\n")
        .file("/sys/class/hwmon/hwmon2/name", "k10temp\n")
        .file("/sys/class/hwmon/hwmon2/temp1_input", "-5000\n")
        .file("/sys/class/hwmon/hwmon2/temp2_input", "\n")
        .file("/sys/class/hwmon/hwmon2/temp3_input", "4x\n")
        .file("/sys/class/hwmon/hwmon2/temp4_input", "-2500\n")
        .file("/sys/class/hwmon/hwmon2/temp4_crit", "99000\n");
    let read = one(read(
        machine,
        &definition("host.cpu_temperature", "Cel", &[]),
    ));
    assert_eq!(ok_value(&read.reading), Some(-2.5));
    assert_eq!(read.raw.as_deref(), Some("-2500"));
}

#[test]
fn only_unparseable_cpu_inputs_and_no_zone_read_unknown() {
    let machine = Mem::default()
        .file("/sys/class/hwmon/hwmon0/name", "coretemp\n")
        .file("/sys/class/hwmon/hwmon0/temp1_input", "\n")
        .file("/sys/class/hwmon/hwmon0/temp2_input", "hot\n");
    let read = one(read(
        machine,
        &definition("host.cpu_temperature", "Cel", &[]),
    ));
    assert_unknown(&read, "no parseable CPU input");
}

#[test]
fn every_path_read_is_one_of_the_four_allowed() {
    let allowed = |p: &str| {
        p == "/proc/stat"
            || p == "/proc/meminfo"
            || p.starts_with("statvfs ")
            || p.starts_with("/sys/class/hwmon")
            || p.starts_with("/sys/class/thermal")
    };
    // A Machine over a borrowed Mem, so the log survives the read.
    struct ByRef<'a>(&'a Mem);
    impl Machine for ByRef<'_> {
        fn read(&self, p: &str) -> io::Result<String> {
            self.0.read(p)
        }
        fn list(&self, d: &str) -> io::Result<Vec<String>> {
            self.0.list(d)
        }
        fn space(&self, m: &str) -> io::Result<Space> {
            self.0.space(m)
        }
        fn mount_point(&self, m: &str) -> io::Result<bool> {
            self.0.mount_point(m)
        }
        fn now(&self) -> i64 {
            self.0.now()
        }
        fn pause(&self) {
            self.0.pause()
        }
    }
    for (extractor, unit) in [
        ("host.cpu_busy", "1"),
        ("host.memory_used", "1"),
        ("host.disk_used", "1"),
        ("host.cpu_temperature", "Cel"),
        ("host.load_average", "1"),
    ] {
        let mem = Mem::default()
            .file("/sys/class/hwmon/hwmon0/name", "acpitz\n")
            .file("/sys/class/thermal/thermal_zone0/type", "acpitz\n")
            .file("/sys/class/thermal/cooling_device0/type", "fan\n");
        let _ = read(ByRef(&mem), &definition(extractor, unit, &[("r", "/")]));
        let asked = mem.asked.borrow().clone();
        assert!(
            asked.iter().all(|p| allowed(p)),
            "{extractor} read outside the four: {asked:?}"
        );
        if extractor == "host.load_average" {
            assert!(asked.is_empty(), "an unknown extractor read {asked:?}");
        }
    }
}

#[test]
fn reads_are_deterministic_and_carry_the_query_digest() {
    let mk = || {
        Mem::default()
            .space("/", 1000, 300, 175)
            .space("/data", 2000, 1500, 1500)
    };
    let def = definition(
        "host.disk_used",
        "1",
        &[("b", "/data"), ("a", "/"), ("c", "/")],
    );
    let first = read(mk(), &def);
    let second = read(mk(), &def);
    assert_eq!(first, second);
    assert_eq!(first.len(), 2, "duplicate mount values are one series");
    for r in &first {
        assert_eq!(r.query_sha256, query_digest(&def.query));
    }
}
