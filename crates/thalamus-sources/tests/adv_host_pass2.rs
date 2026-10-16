//! Adversarial pass 2 on the local host collector: `host.disk_free` and the fixes of pass 1
//! (mount-point check, moment tolerance, mount spelling).
//!
//! Cases marked "real" read this machine through the collector's own `Rooted` reader at `/`, on
//! the system clock, so `statvfs` and the mount-point check are the real calls. They need Linux
//! with `/proc` and `/sys/fs/cgroup` mounted and `/sys/fs` not a mount point, which the cases
//! assert first.

use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};
use std::io;
use std::path::{Path, PathBuf};

use thalamus_model::definition::{
    AdapterId, Author, ConnectionId, EvaluationSpec, ExtractorId, Family, History, Links, Owner,
    QueryText, RuleId, SignalDefinitionData, SignalId, SignalKind, Source, Unit,
};
use thalamus_model::observation::{Reading, RunId};
use thalamus_model::primitives::{Duration, Timestamp};
use thalamus_sources::adapter::{Adapter, Read};
use thalamus_sources::connectors::Connectors;
use thalamus_sources::host::{Host, MOMENT_TOLERANCE_S, Machine, Rooted, Space};
use thalamus_sources::registry;
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;

/// The in-memory machine's clock, Unix seconds: 2026-10-14T22:57:03Z.
const NOW: i64 = 1_792_018_623;

/// An in-memory machine: a `statvfs` table whose every entry is a mount point, and a clock.
struct Mem {
    spaces: BTreeMap<String, Space>,
    now: i64,
    asked: RefCell<Vec<String>>,
}

impl Mem {
    fn new() -> Self {
        Mem {
            spaces: BTreeMap::new(),
            now: NOW,
            asked: RefCell::new(Vec::new()),
        }
    }
    fn space(mut self, mount: &str, available: u64, fragment: u64) -> Self {
        self.spaces.insert(
            mount.to_string(),
            Space {
                blocks: available.saturating_mul(2).max(1),
                free: available,
                available,
                fragment,
            },
        );
        self
    }
}

impl Machine for Mem {
    fn read(&self, path: &str) -> io::Result<String> {
        self.asked.borrow_mut().push(path.to_string());
        if path == "/proc/meminfo" {
            return Ok("MemTotal: 1000 kB\nMemAvailable: 250 kB\n".to_string());
        }
        Err(io::Error::from(io::ErrorKind::NotFound))
    }
    fn list(&self, dir: &str) -> io::Result<Vec<String>> {
        self.asked.borrow_mut().push(dir.to_string());
        Err(io::Error::from(io::ErrorKind::NotFound))
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
    fn now(&self) -> i64 {
        self.now
    }
    fn pause(&self) {}
}

fn definition(extractor: &str, unit: &str, query: &[(&str, &str)]) -> SignalDefinitionData {
    SignalDefinitionData {
        signal_id: SignalId("platform.host.adv2".into()),
        title: "This machine".into(),
        family: Family("platform".into()),
        kind: SignalKind::Gauge,
        unit: Some(Unit(unit.into())),
        dimensions: Some(vec![]),
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
        &RunId("run_adv2".into()),
    )
}

fn rfc3339(unix: i64) -> String {
    OffsetDateTime::from_unix_timestamp(unix)
        .expect("a valid instant")
        .format(&Rfc3339)
        .expect("formats")
}

fn now_text() -> String {
    rfc3339(OffsetDateTime::now_utc().unix_timestamp())
}

/// This machine, read at `/` on the system clock.
fn real() -> Rooted {
    Rooted::new("/", std::time::Duration::ZERO)
}

fn one(reads: Vec<Read>) -> Read {
    let [read] = <[Read; 1]>::try_from(reads).unwrap_or_else(|reads| {
        panic!("one series expected, got {reads:#?}");
    });
    read
}

fn assert_unknown(read: &Read, context: &str) {
    assert!(
        matches!(read.reading, Reading::Unknown(_)),
        "{context}: expected unknown, got {:?} (dims {:?}, raw {:?})",
        read.reading,
        read.dims,
        read.raw
    );
}

fn ok_text(read: &Read) -> Option<&str> {
    match &read.reading {
        Reading::Ok(complete) => Some(complete.value.0.as_str()),
        _ => None,
    }
}

/// A directory under the test target's temporary directory, made empty.
fn scratch(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("adv_host_pass2")
        .join(name);
    if dir.exists() {
        std::fs::remove_dir_all(&dir).expect("clear the scratch directory");
    }
    std::fs::create_dir_all(&dir).expect("create the scratch directory");
    dir
}

// ---------------------------------------------------------------------------------------------
// Red: findings.
// ---------------------------------------------------------------------------------------------

/// Real. `/proc` is a mount point of a file system with no capacity (f_blocks = 0). `disk_used`
/// reads it unknown ("a zero total"); `disk_free` reads the same statvfs answer as `ok` 0 bytes,
/// which a free-space floor grades CRIT. Unknown is never 0: a file system with no blocks has no
/// free space to report.
#[test]
fn disk_free_of_a_zero_size_file_system_reads_unknown() {
    assert!(
        real().mount_point("/proc").expect("stat /proc"),
        "precondition"
    );
    let end = now_text();
    let used = one(read_at(
        real(),
        &definition("host.disk_used", "1", &[("p", "/proc")]),
        &end,
    ));
    assert_unknown(&used, "disk_used of /proc (control)");
    let free = one(read_at(
        real(),
        &definition("host.disk_free", "By", &[("p", "/proc")]),
        &end,
    ));
    assert_unknown(&free, "disk_free of /proc, a file system of 0 blocks");
}

/// Real. `/sys/fs/cgroup` is a mount point and `/sys/fs` is not. `/sys/fs/cgroup/..` is
/// `/sys/fs`, yet `Rooted::mount_point` compares it with its lexical parent `/sys/fs/cgroup`,
/// finds another device and answers "a mount point". A plain directory under a nested mount (an
/// unmounted disk's directory at `/srv/x` beside a mounted `/srv/x/y`, spelled `/srv/x/y/..`)
/// would read the containing file system as the pass-1 F1 defect did.
#[test]
fn a_dot_dot_path_below_a_mount_is_no_mount_point() {
    let machine = real();
    assert!(
        machine.mount_point("/sys/fs/cgroup").expect("stat"),
        "precondition: /sys/fs/cgroup is a mount point"
    );
    assert!(
        !machine.mount_point("/sys/fs").expect("stat"),
        "precondition: /sys/fs is not a mount point"
    );
    assert!(
        !machine.mount_point("/sys/fs/cgroup/..").expect("stat"),
        "/sys/fs/cgroup/.. is /sys/fs, which is no mount point"
    );
}

/// A symlink `link -> /proc/self`: `/proc/self` is a directory of the proc file system and no
/// mount point. `mount_point` follows the link for the path and not for its parent, sees two
/// devices and answers "a mount point"; `disk_free` then reads `ok` 0 bytes under `mount=/link`.
/// The same holds for `/backup -> /mnt/disk/backup` on a real disk: any symlink into another file
/// system is read as a mount of it.
#[test]
fn a_symlink_into_another_file_system_is_no_mount_point() {
    let dir = scratch("symlink");
    std::os::unix::fs::symlink("/proc/self", dir.join("link")).expect("make the symlink");
    let machine = Rooted::new(&dir, std::time::Duration::ZERO);
    assert!(
        !machine.mount_point("/link").expect("stat /link"),
        "/link is a symlink to /proc/self, which is no mount point"
    );
    let read = one(read_at(
        Rooted::new(&dir, std::time::Duration::ZERO),
        &definition("host.disk_free", "By", &[("l", "/link")]),
        &now_text(),
    ));
    assert_unknown(&read, "disk_free of a symlink");
}

/// Real. `/proc/.` is `/proc`; normalisation collapses `//` and drops a trailing `/` but keeps `.`,
/// and `Path::parent` of `/proc/.` is `/`, so both spellings pass the mount check and are read as
/// two series of one file system.
#[test]
fn one_mount_spelled_with_a_dot_is_one_series() {
    let reads = read_at(
        real(),
        &definition("host.disk_used", "1", &[("a", "/proc"), ("b", "/proc/.")]),
        &now_text(),
    );
    let mounts: BTreeSet<String> = reads.iter().map(|r| r.dims["mount"].clone()).collect();
    assert_eq!(mounts.len(), 1, "one mount, one series: {mounts:?}");
}

// ---------------------------------------------------------------------------------------------
// Guards that held (green).
// ---------------------------------------------------------------------------------------------

#[test]
fn disk_free_is_avail_times_fragment_and_its_defects_read_unknown() {
    let machine = Mem::new()
        .space("/", 3, 4096)
        .space("/zero", 5, 0)
        .space("/huge", u64::MAX / 2, 4096)
        .space("/full", 0, 4096);
    let reads = read_at(
        machine,
        &definition(
            "host.disk_free",
            "By",
            &[
                ("a", "/"),
                ("b", "/zero"),
                ("c", "/huge"),
                ("d", "/gone"),
                ("e", "relative"),
                ("f", ""),
                ("g", "/full"),
            ],
        ),
        &rfc3339(NOW),
    );
    let by: BTreeMap<String, Read> = reads
        .into_iter()
        .map(|r| (r.dims["mount"].clone(), r))
        .collect();
    assert_eq!(ok_text(&by["/"]), Some("12288"));
    assert_eq!(by["/"].raw.as_deref(), Some("f_bavail=3 f_frsize=4096"));
    for bad in ["/zero", "/huge", "/gone", "relative", ""] {
        assert_unknown(&by[bad], bad);
        assert_eq!(by[bad].raw, None, "{bad}");
    }
    // A full disk on a real file system is a measured 0, not an unknown.
    assert_eq!(ok_text(&by["/full"]), Some("0"));
}

#[test]
fn mount_spellings_collapse() {
    let machine = Mem::new().space("/", 1, 4096).space("/data/x", 1, 4096);
    let reads = read_at(
        machine,
        &definition(
            "host.disk_free",
            "By",
            &[
                ("a", "/"),
                ("b", "//"),
                ("c", "/data//x"),
                ("d", "/data/x/"),
            ],
        ),
        &rfc3339(NOW),
    );
    let mounts: Vec<String> = reads.iter().map(|r| r.dims["mount"].clone()).collect();
    assert_eq!(mounts, ["/", "/data/x"]);
}

#[test]
fn moment_tolerance_is_inclusive_and_symmetric() {
    let t = MOMENT_TOLERANCE_S;
    for (offset, ok) in [
        (0, true),
        (t, true),
        (-t, true),
        (t + 1, false),
        (-t - 1, false),
    ] {
        let read = one(read_at(
            Mem::new(),
            &definition("host.memory_used", "1", &[]),
            &rfc3339(NOW + offset),
        ));
        assert_eq!(
            ok_text(&read).is_some(),
            ok,
            "end at now{offset:+}s: {:?}",
            read.reading
        );
    }
    // Per mount for a disk: each listed mount keeps its own series and its own reason.
    let reads = read_at(
        Mem::new().space("/", 1, 4096),
        &definition("host.disk_free", "By", &[("a", "/"), ("b", "/gone")]),
        &rfc3339(NOW + 10 * t),
    );
    assert_eq!(reads.len(), 2);
    for read in &reads {
        assert_unknown(read, "a window end far from now");
    }
}
