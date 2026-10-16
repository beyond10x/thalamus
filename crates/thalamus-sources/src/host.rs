//! The local host collector: CPU use, memory use, disk space (used share and free bytes) and CPU
//! temperature of the machine thalamus runs on, read from its kernel interfaces when a definition
//! is sampled.
//!
//! It is the one source that takes no Connectors operation (design § 5.2, § 11 D46): reading this
//! machine's own counters holds no secret and writes nothing. Its source is `{adapter: host,
//! connection: local}`; a definition naming another connection reads `unknown` with
//! [`NOT_LOCAL`] and reads nothing, and `thalamus-core` refuses to propose or revise one
//! (`host-not-local`). It runs only inside a read: no thread, timer or process outlives the call.
//!
//! | extractor | unit | read | dims |
//! |---|---|---|---|
//! | [`CPU_BUSY`] | `1` | the `cpu` line of `/proc/stat`, twice, [`INTERVAL`] apart: 1 − (Δidle + Δiowait) / Δtotal | none |
//! | [`MEMORY_USED`] | `1` | `/proc/meminfo`: (MemTotal − MemAvailable) / MemTotal | none |
//! | [`DISK_USED`] | `1` | `statvfs` of each mount point the definition's `query` lists (every value one mount): (f_blocks − f_bfree) / (f_blocks − f_bfree + f_bavail), as `df` computes it | `mount` |
//! | [`DISK_FREE`] | `By` | `statvfs` of each listed mount point: f_bavail × f_frsize, the bytes an unprivileged writer may still take (`df`'s Avail) | `mount` |
//! | [`CPU_TEMPERATURE`] | `Cel` | the hottest `temp*_input` of a CPU sensor under `/sys/class/hwmon` ([`CPU_SENSORS`]), else of a `/sys/class/thermal` zone of a CPU type ([`CPU_ZONES`]); millidegrees ÷ 1000 | none |
//!
//! Δtotal sums user, nice, system, idle, iowait, irq, softirq and steal; guest and guest_nice are
//! already inside user and nice. A missing file, a missing field, a zero total, a counter of the
//! `cpu` line that went backwards, a busy share outside [0, 1], a mount that cannot be read, a
//! listed path that is no mount point (a plain directory or a file, whose `statvfs` would answer
//! the file system holding it) and a machine with no CPU sensor read `unknown` naming what was
//! wrong, never 0. A mount is spelled once: repeated slashes collapse and a trailing slash is
//! dropped before the listed mounts are deduplicated, and `dims.mount` carries that spelling.
//!
//! A host read measures the machine as it is now, so it can describe only the window ending now:
//! a window end farther than [`MOMENT_TOLERANCE_S`] seconds from the machine's clock reads
//! `unknown` with [`NOT_THIS_MOMENT`]. A disk definition's mounts are checked first, so each
//! listed mount keeps its own series and its own reason.
//!
//! `raw` is the producer's text: the two `cpu` lines, the two meminfo lines, the three `statvfs`
//! counts (`f_bavail` and `f_frsize` for free bytes), the millidegree string. Nothing else is
//! read: no process list, no network, no load average, no other `/proc` or `/sys` path; a listed
//! mount is only `stat`ed, with its parent, to tell a mount point. The reads go through a
//! [`Machine`]; tests hand it a fixture tree and a stopped clock.

use std::collections::{BTreeMap, BTreeSet};
use std::io;
use std::path::{Path, PathBuf};

use thalamus_core::behaviour::definition::{HOST_ADAPTER, LOCAL_CONNECTION};
use thalamus_model::definition::SignalDefinitionData;
use thalamus_model::observation::{CompleteReading, Reading, RunId};
use thalamus_model::primitives::{Decimal, Timestamp};

use crate::adapter::{Adapter, Read, unknown, window_ending};
use crate::connectors::{ConnectorsError, ReadClient};

/// CPU busy share over [`INTERVAL`], unit `1`.
pub const CPU_BUSY: &str = "host.cpu_busy";
/// Memory in use, unit `1`.
pub const MEMORY_USED: &str = "host.memory_used";
/// Disk space in use per listed mount, unit `1`.
pub const DISK_USED: &str = "host.disk_used";
/// Bytes free to an unprivileged writer per listed mount, unit `By`.
pub const DISK_FREE: &str = "host.disk_free";
/// The hottest CPU sensor, unit `Cel`.
pub const CPU_TEMPERATURE: &str = "host.cpu_temperature";

/// The reason of a host read whose connection is not [`LOCAL_CONNECTION`].
pub const NOT_LOCAL: &str = "host reads only this machine (connection local)";

/// The reason of a host read whose window does not end now.
pub const NOT_THIS_MOMENT: &str = "host reads only this moment";

/// How many seconds a window end may lie from the machine's clock and still be read as now: a
/// pass reads its definitions one after another under one window end.
pub const MOMENT_TOLERANCE_S: i64 = 300;

/// The time between the two reads of `/proc/stat`.
pub const INTERVAL: std::time::Duration = std::time::Duration::from_millis(250);

/// The hwmon chip names that are a CPU sensor.
pub const CPU_SENSORS: [&str; 4] = ["coretemp", "k10temp", "zenpower", "cpu_thermal"];

/// The thermal zone types that are a CPU sensor.
pub const CPU_ZONES: [&str; 2] = ["x86_pkg_temp", "cpu-thermal"];

const STAT: &str = "/proc/stat";
const MEMINFO: &str = "/proc/meminfo";
const HWMON: &str = "/sys/class/hwmon";
const THERMAL: &str = "/sys/class/thermal";

/// The block counts of one `statvfs` answer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Space {
    /// `f_blocks`: the size of the file system in blocks.
    pub blocks: u64,
    /// `f_bfree`: free blocks.
    pub free: u64,
    /// `f_bavail`: free blocks an unprivileged user may take.
    pub available: u64,
    /// `f_frsize`: the size of one block, in bytes, as the counts above count it.
    pub fragment: u64,
}

/// What the collector reads the machine through. Every path is absolute, as on the machine.
pub trait Machine {
    /// The text of the file at `path`.
    fn read(&self, path: &str) -> io::Result<String>;
    /// The names of the entries of the directory at `dir`, sorted.
    fn list(&self, dir: &str) -> io::Result<Vec<String>>;
    /// `statvfs` of `mount`.
    fn space(&self, mount: &str) -> io::Result<Space>;
    /// Whether `path` is a mount point: a directory on another file system than its parent's,
    /// or `/`. A plain directory or a file is not, so `statvfs` of it would answer the file system
    /// that holds it.
    fn mount_point(&self, path: &str) -> io::Result<bool>;
    /// The machine's clock, in whole seconds since the Unix epoch.
    fn now(&self) -> i64;
    /// Wait between the two reads of `/proc/stat`.
    fn pause(&self);
}

/// The machine's files under a root: `/` on the machine itself, a fixture tree in a test.
#[derive(Debug, Clone)]
pub struct Rooted {
    root: PathBuf,
    interval: std::time::Duration,
    clock: Option<i64>,
}

impl Rooted {
    /// The files under `root`, with `interval` between the two reads of `/proc/stat`, on the
    /// system clock.
    pub fn new(root: impl AsRef<Path>, interval: std::time::Duration) -> Self {
        Rooted {
            root: root.as_ref().to_path_buf(),
            interval,
            clock: None,
        }
    }

    /// The same files on a clock stopped at `now`, Unix seconds: a fixture tree read as of the
    /// moment its numbers describe.
    #[must_use]
    pub fn at(self, now: i64) -> Self {
        Rooted {
            clock: Some(now),
            ..self
        }
    }

    fn resolve(&self, path: &str) -> PathBuf {
        self.root.join(path.trim_start_matches('/'))
    }
}

impl Machine for Rooted {
    fn read(&self, path: &str) -> io::Result<String> {
        std::fs::read_to_string(self.resolve(path))
    }

    fn list(&self, dir: &str) -> io::Result<Vec<String>> {
        let mut names = std::fs::read_dir(self.resolve(dir))?
            .map(|entry| entry.map(|e| e.file_name().to_string_lossy().into_owned()))
            .collect::<io::Result<Vec<_>>>()?;
        names.sort();
        Ok(names)
    }

    fn space(&self, mount: &str) -> io::Result<Space> {
        let held = rustix::fs::statvfs(self.resolve(mount)).map_err(io::Error::from)?;
        Ok(Space {
            blocks: held.f_blocks,
            free: held.f_bfree,
            available: held.f_bavail,
            fragment: held.f_frsize,
        })
    }

    fn mount_point(&self, path: &str) -> io::Result<bool> {
        use std::os::unix::fs::MetadataExt;
        let at = self.resolve(path);
        let held = std::fs::metadata(&at)?;
        if !held.is_dir() {
            return Ok(false);
        }
        if path == "/" {
            return Ok(true);
        }
        let parent = at.parent().map_or_else(|| at.clone(), Path::to_path_buf);
        Ok(std::fs::metadata(parent)?.dev() != held.dev())
    }

    fn now(&self) -> i64 {
        self.clock.unwrap_or_else(|| {
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_or(0, |since| {
                    i64::try_from(since.as_secs()).unwrap_or(i64::MAX)
                })
        })
    }

    fn pause(&self) {
        std::thread::sleep(self.interval);
    }
}

/// The host adapter on this machine: its files under `/`.
pub fn adapter() -> Option<Box<dyn Adapter>> {
    Some(Box::new(Host::new(Rooted::new("/", INTERVAL))))
}

/// The host collector reading `machine`.
#[derive(Debug, Clone)]
pub struct Host<M> {
    machine: M,
}

impl<M: Machine> Host<M> {
    /// The collector reading `machine`.
    pub fn new(machine: M) -> Self {
        Host { machine }
    }
}

impl<M: Machine> Adapter for Host<M> {
    fn read_operations(&self) -> &[&str] {
        &[]
    }

    fn read(
        &self,
        _client: &ReadClient<'_>,
        definition: &SignalDefinitionData,
        end: &Timestamp,
        _run_id: &RunId,
    ) -> Result<Vec<Read>, ConnectorsError> {
        let nothing = |why: String| Ok(vec![unknown(definition, end, why)]);
        if definition.source.connection.0 != LOCAL_CONNECTION {
            return nothing(NOT_LOCAL.to_string());
        }
        if definition.source.adapter.0 != HOST_ADAPTER {
            return nothing(format!(
                "the host collector reads source adapter {HOST_ADAPTER}, and the definition names \
                 {}",
                definition.source.adapter.0
            ));
        }
        let unit = match definition.extractor.0.as_str() {
            CPU_BUSY | MEMORY_USED | DISK_USED => "1",
            DISK_FREE => "By",
            CPU_TEMPERATURE => "Cel",
            other => {
                return nothing(format!(
                    "the host collector has no extractor {other}; it reads {CPU_BUSY}, \
                     {MEMORY_USED}, {DISK_USED}, {DISK_FREE} and {CPU_TEMPERATURE}"
                ));
            }
        };
        if let Some(declared) = &definition.unit
            && declared.0 != unit
        {
            return nothing(format!(
                "{} has unit {unit}, and the definition declares unit {}",
                definition.extractor.0, declared.0
            ));
        }
        if let Err(why) = window_ending(&definition.window, end) {
            return nothing(why);
        }
        let moment = this_moment(&self.machine, end);
        let one = |answer: fn(&M) -> Result<Known, String>| {
            vec![read_of(
                definition,
                end,
                moment.clone().and_then(|()| answer(&self.machine)),
            )]
        };
        Ok(match definition.extractor.0.as_str() {
            CPU_BUSY => one(cpu_busy),
            MEMORY_USED => one(memory_used),
            CPU_TEMPERATURE => one(cpu_temperature),
            DISK_FREE => disks(&self.machine, definition, end, &moment, space_free),
            _ => disks(&self.machine, definition, end, &moment, space_used),
        })
    }
}

/// A value read and the producer text it was read from.
struct Known {
    value: Value,
    raw: String,
}

/// A value as measured: a share or a temperature, or an exact count of bytes.
enum Value {
    Real(f64),
    Count(u64),
}

/// The read of `answer`: `ok` with its value and raw text, or `unknown` with the reason.
fn read_of(
    definition: &SignalDefinitionData,
    end: &Timestamp,
    answer: Result<Known, String>,
) -> Read {
    match answer {
        Ok(Known {
            value: Value::Count(n),
            raw,
        }) => ok(definition, end, n.to_string(), raw),
        Ok(Known {
            value: Value::Real(n),
            raw,
        }) if n.is_finite() => ok(definition, end, render(n), raw),
        Ok(known) => unknown(
            definition,
            end,
            format!("{} is not a finite number", known.raw),
        ),
        Err(why) => unknown(definition, end, why),
    }
}

fn ok(definition: &SignalDefinitionData, end: &Timestamp, value: String, raw: String) -> Read {
    let mut read = unknown(definition, end, String::new());
    read.reading = Reading::Ok(CompleteReading {
        value: Decimal(value),
    });
    read.raw = Some(raw);
    read
}

/// `Ok` when `end` is within [`MOMENT_TOLERANCE_S`] seconds of the machine's clock; otherwise
/// the reason.
fn this_moment(machine: &impl Machine, end: &Timestamp) -> Result<(), String> {
    let end_s = thalamus_core::behaviour::observation::Instant::parse(&end.0)
        .ok_or_else(|| format!("window end {} is not an RFC 3339 instant", end.0))?
        .seconds();
    let now = machine.now();
    if end_s.abs_diff(now) > MOMENT_TOLERANCE_S.unsigned_abs() {
        return Err(format!(
            "{NOT_THIS_MOMENT}: the window ends at {}, {}s from the machine's clock, more than \
             the {MOMENT_TOLERANCE_S}s a read of now may lie from it",
            end.0,
            end_s.saturating_sub(now)
        ));
    }
    Ok(())
}

fn render(n: f64) -> String {
    if n == 0.0 {
        "0".to_string()
    } else {
        format!("{n}")
    }
}

fn file(machine: &impl Machine, path: &str) -> Result<String, String> {
    machine
        .read(path)
        .map_err(|e| format!("{path} could not be read: {e}"))
}

/// The `cpu` line of `/proc/stat`, as written, and its first eight counters (user, nice, system,
/// idle, iowait, irq, softirq, steal, as far as the line has them).
fn cpu_line(machine: &impl Machine) -> Result<(String, Vec<u64>), String> {
    let text = file(machine, STAT)?;
    let line = text
        .lines()
        .find(|line| line.split_whitespace().next() == Some("cpu"))
        .ok_or_else(|| format!("{STAT} has no cpu line"))?;
    let fields = line
        .split_whitespace()
        .skip(1)
        .take(8)
        .map(|field| field.parse::<u64>())
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("the cpu line of {STAT} holds a field that is no count: {e}"))?;
    if fields.len() < 5 {
        return Err(format!(
            "the cpu line of {STAT} has {} fields; idle and iowait need 5",
            fields.len()
        ));
    }
    Ok((line.to_string(), fields))
}

fn total(fields: &[u64]) -> Result<u64, String> {
    fields
        .iter()
        .try_fold(0u64, |sum, n| sum.checked_add(*n))
        .ok_or_else(|| format!("the cpu line of {STAT} sums past a 64-bit count"))
}

fn cpu_busy(machine: &impl Machine) -> Result<Known, String> {
    const NAMES: [&str; 8] = [
        "user", "nice", "system", "idle", "iowait", "irq", "softirq", "steal",
    ];
    let (before, first) = cpu_line(machine)?;
    machine.pause();
    let (after, second) = cpu_line(machine)?;
    if first.len() != second.len() {
        return Err(format!(
            "the cpu line of {STAT} has {} counters on the first read and {} on the second",
            first.len(),
            second.len()
        ));
    }
    let deltas = first
        .iter()
        .zip(&second)
        .zip(NAMES)
        .map(|((a, b), name)| {
            b.checked_sub(*a).ok_or_else(|| {
                format!("the {name} counter of {STAT} went backwards between the two reads")
            })
        })
        .collect::<Result<Vec<u64>, String>>()?;
    let elapsed = total(&deltas)?;
    let waiting = deltas[3] + deltas[4];
    if elapsed == 0 {
        return Err(format!(
            "no CPU time passed between the two reads of {STAT} (a zero total)"
        ));
    }
    let busy = 1.0 - waiting as f64 / elapsed as f64;
    if !(0.0..=1.0).contains(&busy) {
        return Err(format!(
            "the cpu counters of {STAT} give a busy share of {busy}, outside 0 to 1"
        ));
    }
    Ok(Known {
        value: Value::Real(busy),
        raw: format!("{before}\n{after}"),
    })
}

fn memory_used(machine: &impl Machine) -> Result<Known, String> {
    let text = file(machine, MEMINFO)?;
    let field = |name: &str| -> Result<(&str, u64), String> {
        let line = text
            .lines()
            .find(|line| line.split(':').next() == Some(name))
            .ok_or_else(|| format!("{MEMINFO} has no {name}"))?;
        let value = line
            .split_once(':')
            .and_then(|(_, rest)| rest.split_whitespace().next())
            .and_then(|n| n.parse::<u64>().ok())
            .ok_or_else(|| format!("{name} of {MEMINFO} is no count: {line:?}"))?;
        Ok((line.trim_end(), value))
    };
    let (total_line, total) = field("MemTotal")?;
    let (available_line, available) = field("MemAvailable")?;
    if total == 0 {
        return Err(format!("MemTotal of {MEMINFO} is 0 (a zero total)"));
    }
    let used = total
        .checked_sub(available)
        .ok_or_else(|| format!("MemAvailable of {MEMINFO} exceeds MemTotal"))?;
    Ok(Known {
        value: Value::Real(used as f64 / total as f64),
        raw: format!("{total_line}\n{available_line}"),
    })
}

/// One read per mount the definition's `query` lists, each with `dims.mount` its normalised
/// spelling: the mount is checked ([`mount_point`]), then the moment, then `measure` reads it.
fn disks<M: Machine>(
    machine: &M,
    definition: &SignalDefinitionData,
    end: &Timestamp,
    moment: &Result<(), String>,
    measure: fn(&M, &str) -> Result<Known, String>,
) -> Vec<Read> {
    let mounts: BTreeSet<String> = definition.query.values().map(|q| normalise(&q.0)).collect();
    if mounts.is_empty() {
        return vec![unknown(
            definition,
            end,
            format!("{} lists no mount point in query", definition.extractor.0),
        )];
    }
    mounts
        .into_iter()
        .map(|mount| {
            let answer = mount_point(machine, &mount)
                .and_then(|()| moment.clone())
                .and_then(|()| measure(machine, &mount));
            let mut read = read_of(definition, end, answer);
            read.dims = BTreeMap::from([("mount".to_string(), mount)]);
            read
        })
        .collect()
}

/// One spelling of a mount: repeated slashes collapse into one and a trailing slash is dropped,
/// so `//` is `/` and `/data/` is `/data`.
fn normalise(mount: &str) -> String {
    let mut spelled = String::with_capacity(mount.len());
    for c in mount.chars() {
        if !(c == '/' && spelled.ends_with('/')) {
            spelled.push(c);
        }
    }
    if spelled.len() > 1 && spelled.ends_with('/') {
        spelled.pop();
    }
    spelled
}

/// `Ok` when `mount` is an absolute path the machine answers is a mount point.
fn mount_point(machine: &impl Machine, mount: &str) -> Result<(), String> {
    if !mount.starts_with('/') {
        return Err(format!("mount {mount:?} is not an absolute path"));
    }
    match machine.mount_point(mount) {
        Ok(true) => Ok(()),
        Ok(false) => Err(format!(
            "{mount} is no mount point (a plain directory or a file): its statvfs would answer \
             the file system that holds it"
        )),
        Err(e) => Err(format!("mount {mount} could not be read: {e}")),
    }
}

fn statvfs(machine: &impl Machine, mount: &str) -> Result<Space, String> {
    machine
        .space(mount)
        .map_err(|e| format!("statvfs of mount {mount} failed: {e}"))
}

fn space_free(machine: &impl Machine, mount: &str) -> Result<Known, String> {
    let space = statvfs(machine, mount)?;
    if space.fragment == 0 {
        return Err(format!(
            "statvfs of mount {mount} reports a block size of 0"
        ));
    }
    let bytes = space
        .available
        .checked_mul(space.fragment)
        .ok_or_else(|| format!("the free bytes of mount {mount} exceed a 64-bit count"))?;
    Ok(Known {
        value: Value::Count(bytes),
        raw: format!("f_bavail={} f_frsize={}", space.available, space.fragment),
    })
}

fn space_used(machine: &impl Machine, mount: &str) -> Result<Known, String> {
    let space = statvfs(machine, mount)?;
    let used = space
        .blocks
        .checked_sub(space.free)
        .ok_or_else(|| format!("statvfs of mount {mount} counts more free blocks than blocks"))?;
    let usable = used
        .checked_add(space.available)
        .filter(|n| *n > 0)
        .ok_or_else(|| format!("statvfs of mount {mount} counts no usable block (a zero total)"))?;
    Ok(Known {
        value: Value::Real(used as f64 / usable as f64),
        raw: format!(
            "f_blocks={} f_bfree={} f_bavail={}",
            space.blocks, space.free, space.available
        ),
    })
}

fn cpu_temperature(machine: &impl Machine) -> Result<Known, String> {
    let hottest = |readings: Vec<String>| -> Option<Known> {
        readings
            .into_iter()
            .filter_map(|text| {
                let raw = text.trim().to_string();
                raw.parse::<i64>().ok().map(|milli| (milli, raw))
            })
            .max_by_key(|(milli, _)| *milli)
            .map(|(milli, raw)| Known {
                value: Value::Real(milli as f64 / 1000.0),
                raw,
            })
    };
    let names = |dir: &str| machine.list(dir).unwrap_or_default();
    let mut sensors = Vec::new();
    for chip in names(HWMON) {
        let dir = format!("{HWMON}/{chip}");
        let is_cpu = machine
            .read(&format!("{dir}/name"))
            .is_ok_and(|name| CPU_SENSORS.contains(&name.trim()));
        if !is_cpu {
            continue;
        }
        for entry in names(&dir) {
            if entry.starts_with("temp")
                && entry.ends_with("_input")
                && let Ok(text) = machine.read(&format!("{dir}/{entry}"))
            {
                sensors.push(text);
            }
        }
    }
    if let Some(known) = hottest(sensors) {
        return Ok(known);
    }
    let mut zones = Vec::new();
    for zone in names(THERMAL) {
        if !zone.starts_with("thermal_zone") {
            continue;
        }
        let dir = format!("{THERMAL}/{zone}");
        let is_cpu = machine
            .read(&format!("{dir}/type"))
            .is_ok_and(|kind| CPU_ZONES.contains(&kind.trim()));
        if is_cpu && let Ok(text) = machine.read(&format!("{dir}/temp")) {
            zones.push(text);
        }
    }
    hottest(zones).ok_or_else(|| {
        format!(
            "no CPU temperature sensor: no {HWMON} chip named {} with a readable temp*_input, and \
             no {THERMAL} zone of type {} with a readable temp",
            CPU_SENSORS.join(", "),
            CPU_ZONES.join(" or ")
        )
    })
}
