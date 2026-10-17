//! One append-only Eventlog stream per instance, on the provider a [`Locator`] names: SQLite for a
//! local instance, PostgreSQL for a shared one (design § 4, § 5.1, § 11 D24).
//!
//! Every record an instance keeps is one event on that stream, in the order it was appended. The
//! event body is the record itself, in its wire form (`thalamus.sample/1`, `thalamus.evaluation/1`,
//! design § 2.2 and § 2.3), never wrapped in a CloudEvents envelope (§ 6.2).
//!
//! Every record read from the stream is checked before it is answered: its schema is one this
//! build knows, it decodes, it encodes back to exactly the body stored, and that body still hashes
//! to the request hash Eventlog recorded with it. A record that fails any of these refuses the
//! open (or the read) rather than vanishing from an answer or coming back altered.
//!
//! A handle stays current with the stream. An append that meets another writer's append (a
//! version conflict) takes that writer's records in and retries, up to [`CATCH_UP_ATTEMPTS`]
//! times; an append under an id another writer stored with another body is refused
//! [`ID_TAKEN`]; and an evaluation of a sample the handle does not hold reads the stream first. A
//! read that answers from what the stream holds (`sample`, `evaluation`, the ranges, the
//! generated ports) also reads the records appended since the handle last moved.
//!
//! Identities (`smp_`, `evl_` and twelve digits) are minted unique across writers (D24): each
//! number is reserved on the instance's own id stream, beside the instance stream, by an append
//! at the version the handle last read, before it is answered. A writer that meets another's
//! reservation reads the id stream again and tries the next number. The instance stream holds
//! only the § 2.2 and § 2.3 records, and the open never reads the id stream.
//!
//! Eventlog's ports are asynchronous; the store drives them on its own single-threaded runtime,
//! so every call here blocks and none may be made from inside another Tokio runtime. The runtime
//! is shut down in the background when the store is dropped, so a store may be dropped anywhere.
//!
//! Numbers: a `Decimal` is a decimal string (D11), so a record carries no JSON number but an
//! integer. Eventlog keeps `serde_json/arbitrary_precision` off because it changes the request hash
//! of a body holding a fraction; this workspace has it on (`thalamus-spec-types`, feature
//! `exact-numbers`). Every body is checked to hold integers only before it is appended, so the
//! stored hashes are the same with the feature on or off.

use std::collections::{BTreeMap, HashMap};
use std::fmt;
use std::future::Future;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::Arc;
use std::time::{Duration, Instant};

use eventlog_core::{
    CommandMeta, EventLogError, EventStore, Expected, Guard, MAX_READ_LIMIT, NewEvent,
    RecordedEvent, StreamId, TenantId,
};
use eventlog_postgres::{PoolOptions, PostgresConfig, PostgresEventStore};
use eventlog_sqlite::SqliteEventStore;
use serde_json::Value;
use thalamus_core::behaviour::observation::IdSequence;
use thalamus_model::evaluation::SignalEvaluationData;
use thalamus_model::json;
use thalamus_model::observation::SignalSampleData;
use time::OffsetDateTime;
use tokio::runtime::Runtime;

/// The wire schema of a stored sample (design § 2.2).
pub const SAMPLE_SCHEMA: &str = "thalamus.sample/1";
/// The wire schema of a stored evaluation (design § 2.3).
pub const EVALUATION_SCHEMA: &str = "thalamus.evaluation/1";

/// The refusal code of a record whose identity is already stored with another body.
pub const ID_TAKEN: &str = "thalamus.record.id-taken";

/// A locator that is neither `sqlite:<path>` nor a `postgres://` or `postgresql://` URL.
pub const LOCATOR_UNKNOWN: &str = "thalamus.store.locator-unknown";
/// A SQLite locator that names no database file: `:memory:`, an empty path, a `file:` URI or a
/// directory. Such a store would not keep what it records.
pub const SQLITE_NOT_A_FILE: &str = "thalamus.store.sqlite-not-a-file";
/// A PostgreSQL URL that carries a password. Thalamus reads no credential: the database
/// authenticates the local connection itself (peer or trust).
pub const POSTGRES_PASSWORD: &str = "thalamus.store.postgres-password";
/// A PostgreSQL URL that asks for TLS: an `sslmode` other than `disable`, or a direct TLS
/// negotiation. TLS (Eventlog's `verified` connection) is not wired yet, and the connection
/// thalamus makes is plaintext, so the request is refused rather than silently not honoured.
pub const POSTGRES_TLS: &str = "thalamus.store.postgres-tls";
/// A locator whose database directory cannot be created.
pub const LOCATOR_UNUSABLE: &str = "thalamus.store.locator-unusable";

/// How many times an append that meets another writer's append is retried after taking that
/// writer's records in, and how many numbers a mint tries, before the store answers
/// [`StoreError::Contended`].
pub const CATCH_UP_ATTEMPTS: u32 = 64;

/// How long an open keeps retrying while another opener holds the schema, journal or projection
/// lock.
const OPEN_BUDGET: Duration = Duration::from_secs(30);
/// The operation Eventlog's PostgreSQL provider names when a statement or a lock wait outlives
/// its timeout (SQLSTATE 57014 or 55P03). Only that deadline is retried at open: a server that
/// does not answer the connection fails at once.
const LOCK_DEADLINE: &str = "database statement or lock";
/// The message SQLite gives `SQLITE_BUSY`, a lock another connection holds, which waiting can
/// clear. `SQLITE_LOCKED` ("database table is locked", "database schema is locked") is not
/// matched: on a private connection waiting cannot clear it.
/// Switching a new database file to WAL answers `SQLITE_BUSY` at once, without the busy handler,
/// while another connection is switching it, so two processes opening a new store together meet
/// it. Only this is retried at open, within [`OPEN_BUDGET`]; any other backend failure fails
/// at once.
const SQLITE_BUSY_MESSAGE: &str = "database is locked";
/// The pause before an SQLite open that met another opener's lock is tried again.
const SQLITE_OPEN_PAUSE: Duration = Duration::from_millis(10);
/// How long an append, and an id reservation, that finds the SQLite write lock held keeps trying
/// again. The driver's own busy handler waits 5 s and then answers `database is locked`, which a
/// writer starved under load can meet; the append was refused before it wrote anything, so it is
/// safe to repeat.
const LOCK_BUDGET: Duration = Duration::from_secs(30);

/// Whether `error` is SQLite answering that another connection holds a lock.
fn is_sqlite_locked(error: &EventLogError) -> bool {
    matches!(error, EventLogError::Backend(message) if message == SQLITE_BUSY_MESSAGE)
}

/// The table prefix in either database.
const TABLE_PREFIX: &str = "thalamus";
/// The stream type of an instance's one stream.
const STREAM_TYPE: &str = "thalamus.instance";
/// The stream type of an instance's id stream: one reservation per number answered.
const ID_STREAM_TYPE: &str = "thalamus.ids";
/// The event name and schema of a reservation on the id stream.
const RESERVATION_EVENT: &str = "thalamus.id-reservation";
const RESERVATION_SCHEMA: &str = "thalamus.id-reservation/1";
/// Who every append is for and who makes it: the instance's own store, an opaque id.
const SUBJECT: &str = "thalamus";
const ACTOR: &str = "thalamus-store";

/// Where an instance's store lives and which provider keeps it: `sqlite:<path>` or a PostgreSQL
/// URL (`postgres://` or `postgresql://`, no password).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Locator {
    /// The SQLite provider, in this database file.
    Sqlite(PathBuf),
    /// The PostgreSQL provider, at this URL.
    Postgres(String),
}

impl Locator {
    /// Whether the locator names an SQLite database file that is not there yet: a path
    /// [`EventlogStore::open`] would accept and create, with nothing at it (not even a dangling
    /// link). A read command answers such a store as empty without opening it, so reading creates
    /// nothing. Every other locator, and a path that cannot be inspected, answers `false`: it is
    /// opened, and refused there if it must be.
    pub fn names_a_missing_sqlite_file(&self) -> bool {
        let Self::Sqlite(path) = self else {
            return false;
        };
        sqlite_file(path).is_ok()
            && matches!(
                path.symlink_metadata(),
                Err(error) if error.kind() == std::io::ErrorKind::NotFound
            )
    }
}

impl FromStr for Locator {
    type Err = StoreError;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        if let Some(path) = text.strip_prefix("sqlite:")
            && !path.is_empty()
        {
            return Ok(Self::Sqlite(PathBuf::from(path)));
        }
        if text.starts_with("postgres://") || text.starts_with("postgresql://") {
            admitted_postgres(text)?;
            return Ok(Self::Postgres(text.to_owned()));
        }
        Err(StoreError::locator(
            LOCATOR_UNKNOWN,
            format!("`{text}` is not `sqlite:<path>` or a `postgres://` URL"),
        ))
    }
}

impl fmt::Display for Locator {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Sqlite(path) => write!(formatter, "sqlite:{}", path.display()),
            Self::Postgres(url) => formatter.write_str(url),
        }
    }
}

/// Refuses a PostgreSQL URL that asks for TLS, that carries a password, or that is no URL at all.
fn admitted_postgres(url: &str) -> Result<(), StoreError> {
    no_tls(url)?;
    no_password(url)
}

/// Refuses a PostgreSQL URL that asks for TLS: a `sslmode` parameter whose value is not
/// `disable`, or a `sslnegotiation` parameter whose value is not `postgres`. Every `?`- or
/// `&`-separated part of the URL is read, its key and value percent-decoded as the driver decodes
/// them, so a request for TLS is found wherever the driver would find it. The parameter's value
/// is not repeated in the refusal.
fn no_tls(url: &str) -> Result<(), StoreError> {
    let after_scheme = url.split_once("://").map_or(url, |(_, rest)| rest);
    for part in after_scheme.split(['?', '&']).skip(1) {
        let (key, value) = part.split_once('=').unwrap_or((part, ""));
        let (key, value) = (percent_decoded(key), percent_decoded(value));
        let plaintext = match key.as_str() {
            "sslmode" => value == "disable",
            "sslnegotiation" => value == "postgres",
            _ => true,
        };
        if !plaintext {
            return Err(StoreError::locator(
                POSTGRES_TLS,
                format!(
                    "the URL asks for TLS (`{key}`), and TLS, Eventlog's `verified` connection, \
                     is not wired yet: thalamus connects in plaintext to a loopback or \
                     Unix-socket server only (design § 11 D24); leave `sslmode` out or set it \
                     to `disable`"
                ),
            ));
        }
    }
    Ok(())
}

/// `text` with every `%` and two hex digits replaced by the byte they name.
fn percent_decoded(text: &str) -> String {
    let bytes = text.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut at = 0;
    while at < bytes.len() {
        let hex = bytes
            .get(at + 1..at + 3)
            .filter(|pair| pair.iter().all(u8::is_ascii_hexdigit))
            .and_then(|pair| std::str::from_utf8(pair).ok())
            .and_then(|pair| u8::from_str_radix(pair, 16).ok());
        match (bytes[at], hex) {
            (b'%', Some(byte)) => {
                decoded.push(byte);
                at += 3;
            }
            (byte, _) => {
                decoded.push(byte);
                at += 1;
            }
        }
    }
    String::from_utf8_lossy(&decoded).into_owned()
}

/// Refuses a PostgreSQL URL that carries a password, or that is no URL at all.
fn no_password(url: &str) -> Result<(), StoreError> {
    let config = tokio_postgres::Config::from_str(url).map_err(|_| {
        StoreError::locator(
            LOCATOR_UNKNOWN,
            "not a PostgreSQL connection URL".to_owned(),
        )
    })?;
    if config.get_password().is_some() {
        return Err(StoreError::locator(
            POSTGRES_PASSWORD,
            "the URL carries a password; thalamus reads no credential, so the database must \
             authenticate the local connection itself (peer or trust)"
                .to_owned(),
        ));
    }
    Ok(())
}

/// The database file a SQLite locator names, or its refusal.
fn sqlite_file(path: &Path) -> Result<&str, StoreError> {
    let not_a_file = |why: &str| {
        Err(StoreError::locator(
            SQLITE_NOT_A_FILE,
            format!("sqlite:{} is not a database file: {why}", path.display()),
        ))
    };
    let Some(text) = path.to_str() else {
        return not_a_file("the path is not UTF-8");
    };
    if text.is_empty() {
        return not_a_file("the path is empty");
    }
    if text.starts_with(':') {
        return not_a_file("SQLite reads it as an in-memory database");
    }
    if text.starts_with("file:") {
        return not_a_file("SQLite reads it as a URI, which may name an in-memory database");
    }
    if text.ends_with('/') || path.is_dir() {
        return not_a_file("it is a directory");
    }
    Ok(text)
}

/// Why the store refused or failed.
#[derive(Debug)]
pub enum StoreError {
    /// The locator names no store this build opens: [`LOCATOR_UNKNOWN`], [`SQLITE_NOT_A_FILE`],
    /// [`POSTGRES_PASSWORD`], [`POSTGRES_TLS`] or [`LOCATOR_UNUSABLE`].
    Locator {
        /// The refusal code.
        code: &'static str,
        /// What was refused.
        detail: String,
    },
    /// The store refused the append with a stable code: [`ID_TAKEN`],
    /// [`crate::evaluations::DUPLICATE_EVALUATION`] or [`crate::evaluations::UNKNOWN_SAMPLE`].
    /// Nothing was written.
    Refused {
        /// The refusal code.
        code: String,
    },
    /// Another writer appended first on every one of `attempts` tries ([`CATCH_UP_ATTEMPTS`]).
    /// Nothing was written; the call may be made again.
    Contended {
        /// How many times the append or the reservation was tried.
        attempts: u32,
    },
    /// A record that cannot be written as its wire form and read back, or a stored record that
    /// does not decode, does not encode back to its stored body, or no longer hashes to the
    /// request Eventlog recorded with it.
    Record(String),
    /// The runtime that drives Eventlog could not be built.
    Runtime(String),
    /// Eventlog failed.
    Eventlog(EventLogError),
}

impl StoreError {
    fn locator(code: &'static str, detail: String) -> Self {
        Self::Locator { code, detail }
    }

    /// Whether another writer appended to the stream since this handle last read it.
    pub(crate) fn is_conflict(&self) -> bool {
        matches!(self, Self::Eventlog(EventLogError::Conflict { .. }))
    }

    /// Whether SQLite answered that another connection holds a lock.
    fn is_locked(&self) -> bool {
        matches!(self, Self::Eventlog(error) if is_sqlite_locked(error))
    }

    /// Whether the id was already stored on the stream with another body.
    fn is_id_taken_elsewhere(&self) -> bool {
        matches!(
            self,
            Self::Eventlog(EventLogError::IdempotencyMismatch { .. })
        )
    }
}

impl fmt::Display for StoreError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Locator { code, detail } => write!(formatter, "{code}: {detail}"),
            Self::Record(message) | Self::Runtime(message) => formatter.write_str(message),
            Self::Refused { code } => write!(formatter, "refused: {code}"),
            Self::Contended { attempts } => write!(
                formatter,
                "another writer appended to the stream first on each of {attempts} attempts; \
                 nothing was written"
            ),
            Self::Eventlog(error) => write!(formatter, "eventlog: {error}"),
        }
    }
}

impl std::error::Error for StoreError {}

impl From<EventLogError> for StoreError {
    fn from(error: EventLogError) -> Self {
        match error {
            EventLogError::GuardRefused { code } => Self::Refused { code },
            other => Self::Eventlog(other),
        }
    }
}

/// One stored record: its place on the stream and its wire body.
#[derive(Debug, Clone, PartialEq)]
pub struct Record {
    version: u64,
    body: Value,
}

impl Record {
    /// The record's version on the stream, from 1.
    pub fn version(&self) -> u64 {
        self.version
    }

    /// The record as stored: the wire form, with its `schema` member.
    pub fn body(&self) -> &Value {
        &self.body
    }

    /// The record's `schema`, such as [`SAMPLE_SCHEMA`].
    pub fn schema(&self) -> Option<&str> {
        self.body.get("schema").and_then(Value::as_str)
    }
}

/// The typed row a record holds.
pub(crate) enum Row {
    Sample(SignalSampleData),
    Evaluation(SignalEvaluationData),
    Unmuted(crate::definition_events::UnmutedRecord),
    Transition(thalamus_model::transition::TransitionData),
}

/// The rows appended since the handle last moved, read for a `&self` answer.
#[derive(Default)]
pub(crate) struct Tail {
    pub(crate) samples: Vec<SignalSampleData>,
    pub(crate) evaluations: Vec<SignalEvaluationData>,
    pub(crate) unmuted: Vec<crate::definition_events::UnmutedRecord>,
    pub(crate) transitions: Vec<thalamus_model::transition::TransitionData>,
}

/// The store's own runtime, shut down in the background when dropped: a blocking shutdown would
/// panic when the store is dropped inside asynchronous code.
struct OwnedRuntime(Option<Runtime>);

impl OwnedRuntime {
    fn new() -> Result<Self, StoreError> {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map(|runtime| Self(Some(runtime)))
            .map_err(|error| StoreError::Runtime(format!("tokio runtime: {error}")))
    }

    fn block_on<F: Future>(&self, future: F) -> F::Output {
        match &self.0 {
            Some(runtime) => runtime.block_on(future),
            None => unreachable!("the runtime is held until the store is dropped"),
        }
    }
}

impl Drop for OwnedRuntime {
    fn drop(&mut self) {
        if let Some(runtime) = self.0.take() {
            runtime.shutdown_background();
        }
    }
}

/// The Eventlog store `locator` names, with its tables created when absent and the inline
/// projections registered: the part of an open that takes the database's locks.
fn connect(runtime: &OwnedRuntime, locator: &Locator) -> Result<Arc<dyn EventStore>, StoreError> {
    let store: Arc<dyn EventStore> = match locator {
        Locator::Sqlite(path) => {
            let file = sqlite_file(path)?;
            if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
                std::fs::create_dir_all(parent).map_err(|error| {
                    StoreError::locator(LOCATOR_UNUSABLE, format!("{}: {error}", parent.display()))
                })?;
            }
            Arc::new(runtime.block_on(SqliteEventStore::open(file, TABLE_PREFIX))?)
        }
        Locator::Postgres(url) => {
            admitted_postgres(url)?;
            let config = PostgresConfig::isolated(url, TABLE_PREFIX)?;
            Arc::new(runtime.block_on(PostgresEventStore::local(config, PoolOptions::default()))?)
        }
    };
    for projector in crate::evaluations::projectors() {
        runtime.block_on(store.register_inline(projector))?;
    }
    Ok(store)
}

/// An instance's records on one Eventlog stream, with every sample and evaluation decoded.
pub struct EventlogStore {
    store: Arc<dyn EventStore>,
    runtime: OwnedRuntime,
    stream: StreamId,
    head: u64,
    records: Vec<Record>,
    pub(crate) samples: Vec<SignalSampleData>,
    pub(crate) sample_index: HashMap<String, usize>,
    /// The positions in `samples` of each series' samples, by series key, in append order: the
    /// index a series range read goes through, so it never scans other series.
    pub(crate) series_index: HashMap<String, Vec<usize>>,
    pub(crate) evaluations: Vec<SignalEvaluationData>,
    pub(crate) evaluation_index: HashMap<String, usize>,
    /// The last number reserved or stored per id prefix (`smp_`, `evl_`).
    sequences: BTreeMap<String, u64>,
    /// The instance's id stream, and the version of the last reservation this handle read on it.
    ids: StreamId,
    ids_head: u64,
}

impl EventlogStore {
    /// Opens the stream of `instance` in the store `locator` names, creating the store when it
    /// does not exist, and reads and checks every record on it.
    ///
    /// SQLite: the database file and its directory are created. PostgreSQL: the URL names a
    /// loopback or Unix-socket server (Eventlog's isolated configuration, plaintext), no password
    /// and no request for TLS; the tables are created when absent.
    ///
    /// Every open takes the database's schema and projection locks for a moment. An open that
    /// waits past the lock timeout (PostgreSQL), or finds the database locked (SQLite: another
    /// process setting up the same new file), while other processes open the same database is
    /// tried again, for up to 30 seconds.
    ///
    /// # Errors
    ///
    /// [`StoreError::Locator`] for a locator naming no usable store; [`StoreError::Record`] for a
    /// stored record that fails its checks; the Eventlog failure, including a PostgreSQL host
    /// that is not local.
    pub fn open(locator: &Locator, instance: &str) -> Result<Self, StoreError> {
        let runtime = OwnedRuntime::new()?;
        let started = Instant::now();
        let store = loop {
            match connect(&runtime, locator) {
                Err(StoreError::Eventlog(EventLogError::Deadline { operation }))
                    if operation == LOCK_DEADLINE && started.elapsed() < OPEN_BUDGET => {}
                Err(StoreError::Eventlog(EventLogError::Backend(message)))
                    if matches!(locator, Locator::Sqlite(_))
                        && message == SQLITE_BUSY_MESSAGE
                        && started.elapsed() < OPEN_BUDGET =>
                {
                    std::thread::sleep(SQLITE_OPEN_PAUSE);
                }
                answer => break answer?,
            }
        };
        let tenant = TenantId::new(instance)?;
        let stream = StreamId::new(tenant.clone(), STREAM_TYPE, instance)?;
        let ids = StreamId::new(tenant, ID_STREAM_TYPE, instance)?;
        let mut this = Self {
            store,
            runtime,
            stream,
            head: 0,
            records: Vec::new(),
            samples: Vec::new(),
            sample_index: HashMap::new(),
            series_index: HashMap::new(),
            evaluations: Vec::new(),
            evaluation_index: HashMap::new(),
            sequences: BTreeMap::new(),
            ids,
            ids_head: 0,
        };
        this.refresh()?;
        Ok(this)
    }

    /// The version of the last record this handle has taken in, 0 for an empty stream.
    pub fn head(&self) -> u64 {
        self.head
    }

    /// Every record this handle has taken in, in the order it was appended: the stream as of the
    /// handle's last call that took `&mut self` (open, an append, [`Self::refresh`]).
    pub fn records(&self) -> &[Record] {
        &self.records
    }

    /// Takes in every record appended to the stream since this handle last read it.
    ///
    /// # Errors
    ///
    /// [`StoreError::Record`] for a record that fails its checks, or the Eventlog failure.
    pub fn refresh(&mut self) -> Result<(), StoreError> {
        for (record, row) in self.read_after(self.head)? {
            self.keep(record, row);
        }
        Ok(())
    }

    /// The rows appended since this handle last moved, for an answer that must not be stale.
    ///
    /// # Panics
    ///
    /// When the stream cannot be read or a record on it fails its checks: a read that answered
    /// from the handle alone could say a stored sample does not exist.
    pub(crate) fn tail(&self) -> Tail {
        let rows = self
            .read_after(self.head)
            .unwrap_or_else(|error| panic!("the instance stream cannot be read: {error}"));
        let mut tail = Tail::default();
        for (_, row) in rows {
            match row {
                Row::Sample(sample) => tail.samples.push(sample),
                Row::Evaluation(evaluation) => tail.evaluations.push(evaluation),
                Row::Unmuted(unmuted) => tail.unmuted.push(unmuted),
                Row::Transition(transition) => tail.transitions.push(transition),
            }
        }
        tail
    }

    /// The tenant the stream belongs to, for a guard to read its projection rows under.
    pub(crate) fn tenant(&self) -> &TenantId {
        self.stream.tenant()
    }

    /// Every record after `after`, read and checked.
    fn read_after(&self, mut after: u64) -> Result<Vec<(Record, Row)>, StoreError> {
        let mut rows = Vec::new();
        loop {
            let slice = self.runtime.block_on(self.store.read_stream(
                &self.stream,
                after,
                MAX_READ_LIMIT,
            ))?;
            for event in slice.events {
                after = event.version;
                rows.push(self.checked(event, false)?);
            }
            if slice.end_of_stream {
                return Ok(rows);
            }
        }
    }

    /// One recorded event as its record and typed row, after every check the module names. An
    /// event this handle has just appended (`appended_here`) skips the request-hash lookup: the
    /// handle computed that hash from the same body a moment ago, and on PostgreSQL every read
    /// waits on the publication gate every writer shares.
    fn checked(
        &self,
        event: RecordedEvent,
        appended_here: bool,
    ) -> Result<(Record, Row), StoreError> {
        let record = Record {
            version: event.version,
            body: event.data,
        };
        let refuse = |why: String| {
            Err(StoreError::Record(format!(
                "record {} on the stream {why}",
                record.version
            )))
        };
        let (row, again) = match record.schema() {
            Some(SAMPLE_SCHEMA) => {
                let sample = crate::samples::decode(&record.body)?;
                let again = crate::samples::encode(&sample)?;
                (Row::Sample(sample), again)
            }
            Some(EVALUATION_SCHEMA) => {
                let evaluation = crate::evaluations::decode(&record.body)?;
                let again = crate::evaluations::encode(&evaluation)?;
                (Row::Evaluation(evaluation), again)
            }
            Some(crate::definition_events::DEFINITION_EVENT_SCHEMA) => {
                let unmuted = crate::definition_events::decode(&record.body)?;
                let again = crate::definition_events::encode(&unmuted)?;
                (Row::Unmuted(unmuted), again)
            }
            Some(crate::transitions::TRANSITION_SCHEMA) => {
                let transition = crate::transitions::decode(&record.body)?;
                let again = crate::transitions::encode(&transition)?;
                (Row::Transition(transition), again)
            }
            other => return refuse(format!("has the unknown schema {other:?}")),
        };
        if again != record.body {
            return refuse(format!(
                "does not encode back to its stored body: stored {}, its wire form {again}",
                record.body
            ));
        }
        if appended_here {
            return Ok((record, row));
        }
        let hash = eventlog_core::request_hash(&record.body)?;
        match self.runtime.block_on(self.store.recorded_command(
            &self.stream,
            &event.request_id,
            &hash,
        )) {
            Err(EventLogError::IdempotencyMismatch { .. }) => {
                return refuse(
                    "no longer hashes to the request Eventlog recorded with it: it was altered \
                     after it was written"
                        .to_owned(),
                );
            }
            Err(other) => return Err(other.into()),
            Ok(_) => {}
        }
        Ok((record, row))
    }

    /// Appends one record body at the stream's head, under `guard` when there is one, and takes in
    /// what the stream recorded. A version conflict is answered as the error, for the caller to
    /// refresh and retry.
    ///
    /// # Errors
    ///
    /// [`StoreError::Record`] for a body holding a number that is not an integer, a
    /// [`StoreError::Refused`] carrying the guard's code, or the Eventlog failure.
    pub(crate) fn append(
        &mut self,
        event: &str,
        id: &str,
        body: Value,
        guard: Option<Arc<dyn Guard>>,
    ) -> Result<(), StoreError> {
        integers_only(&body, "")?;
        let meta = command_meta(id, &body)?;
        let events = [NewEvent::new(event, 1, body)?];
        let expected = match self.head {
            0 => Expected::NoStream,
            head => Expected::Exact(head),
        };
        let appended = self.runtime.block_on(async {
            match guard {
                Some(guard) => {
                    self.store
                        .append_guarded(&self.stream, expected, &events, &meta, guard)
                        .await
                }
                None => {
                    self.store
                        .append(&self.stream, expected, &events, &meta)
                        .await
                }
            }
        })?;
        if appended.deduplicated {
            return self.refresh();
        }
        for event in appended.events {
            let (record, row) = self.checked(event, true)?;
            self.keep(record, row);
        }
        Ok(())
    }

    /// Runs `attempt` (an append through [`Self::append`]) until it is not met by another
    /// writer's append. A version conflict takes the other writer's records in and tries again,
    /// up to [`CATCH_UP_ATTEMPTS`] times in all; an id the stream holds with another body (Eventlog
    /// checks the idempotency key before the version, so a stale handle meets it first) takes the
    /// stream in and is refused [`ID_TAKEN`].
    ///
    /// # Errors
    ///
    /// [`StoreError::Contended`] after [`CATCH_UP_ATTEMPTS`] conflicts, [`StoreError::Refused`]
    /// with [`ID_TAKEN`], or whatever `attempt` or the catch-up answers.
    pub(crate) fn append_caught_up(
        &mut self,
        mut attempt: impl FnMut(&mut Self) -> Result<(), StoreError>,
    ) -> Result<(), StoreError> {
        let started = Instant::now();
        let mut attempts = 0;
        while attempts < CATCH_UP_ATTEMPTS {
            match attempt(self) {
                Err(error) if error.is_locked() && started.elapsed() < LOCK_BUDGET => {
                    std::thread::sleep(SQLITE_OPEN_PAUSE);
                    continue;
                }
                Err(error) if error.is_conflict() => self.refresh()?,
                Err(error) if error.is_id_taken_elsewhere() => {
                    self.refresh()?;
                    return Err(StoreError::Refused {
                        code: ID_TAKEN.to_owned(),
                    });
                }
                answer => return answer,
            }
            attempts += 1;
        }
        Err(StoreError::Contended {
            attempts: CATCH_UP_ATTEMPTS,
        })
    }

    /// Reserves the next number of `prefix` on the id stream and answers it: the number after
    /// the highest the instance stream and the id stream hold, appended as a reservation at the
    /// id stream's version this handle last read. Another writer's reservation in between (a
    /// version conflict, or the same reservation already recorded) is read and the next number
    /// tried, up to [`CATCH_UP_ATTEMPTS`] times. An append that finds the SQLite write lock held is
    /// repeated for up to [`LOCK_BUDGET`], and is no attempt. `None` once the sequence is spent.
    fn reserve(&mut self, prefix: &str) -> Result<Option<u64>, StoreError> {
        self.refresh()?;
        let started = Instant::now();
        let mut attempts = 0;
        while attempts < CATCH_UP_ATTEMPTS {
            self.read_reservations()?;
            let last = self.sequences.get(prefix).copied().unwrap_or(0);
            let Some(number) = last.checked_add(1) else {
                return Ok(None);
            };
            let key = format!("{prefix}{number:012}");
            let body = serde_json::json!({
                "schema": RESERVATION_SCHEMA,
                "prefix": prefix,
                "number": number,
            });
            let meta = command_meta(&key, &body)?;
            let events = [NewEvent::new(RESERVATION_EVENT, 1, body)?];
            let expected = match self.ids_head {
                0 => Expected::NoStream,
                head => Expected::Exact(head),
            };
            match self
                .runtime
                .block_on(self.store.append(&self.ids, expected, &events, &meta))
            {
                Ok(appended) if !appended.deduplicated => {
                    for event in appended.events {
                        self.take_reservation(&event)?;
                    }
                    return Ok(Some(number));
                }
                Err(locked) if is_sqlite_locked(&locked) && started.elapsed() < LOCK_BUDGET => {
                    std::thread::sleep(SQLITE_OPEN_PAUSE);
                    continue;
                }
                Ok(_) | Err(EventLogError::Conflict { .. }) => {}
                Err(other) => return Err(other.into()),
            }
            attempts += 1;
        }
        Err(StoreError::Contended {
            attempts: CATCH_UP_ATTEMPTS,
        })
    }

    /// Takes in every reservation on the id stream after the last one this handle read.
    fn read_reservations(&mut self) -> Result<(), StoreError> {
        loop {
            let slice = self.runtime.block_on(self.store.read_stream(
                &self.ids,
                self.ids_head,
                MAX_READ_LIMIT,
            ))?;
            for event in &slice.events {
                self.take_reservation(event)?;
            }
            if slice.end_of_stream {
                return Ok(());
            }
        }
    }

    /// Moves the sequence of a reservation's prefix to at least its number.
    fn take_reservation(&mut self, event: &RecordedEvent) -> Result<(), StoreError> {
        let body = &event.data;
        let prefix = body.get("prefix").and_then(Value::as_str);
        let number = body.get("number").and_then(Value::as_u64);
        let (Some(RESERVATION_SCHEMA), Some(prefix), Some(number)) =
            (body.get("schema").and_then(Value::as_str), prefix, number)
        else {
            return Err(StoreError::Record(format!(
                "reservation {} on the id stream is not a {RESERVATION_SCHEMA} record",
                event.version
            )));
        };
        let last = self.sequences.entry(prefix.to_owned()).or_insert(0);
        *last = (*last).max(number);
        self.ids_head = event.version;
        Ok(())
    }

    /// Takes one checked record and its row into the handle.
    fn keep(&mut self, record: Record, row: Row) {
        match row {
            Row::Sample(sample) => {
                self.resume_after(&sample.sample_id.0);
                self.sample_index
                    .insert(sample.sample_id.0.clone(), self.samples.len());
                self.series_index
                    .entry(sample.series.0.clone())
                    .or_default()
                    .push(self.samples.len());
                self.samples.push(sample);
            }
            Row::Evaluation(evaluation) => {
                self.resume_after(&evaluation.evaluation_id.0);
                self.evaluation_index
                    .insert(evaluation.evaluation_id.0.clone(), self.evaluations.len());
                self.evaluations.push(evaluation);
            }
            Row::Unmuted(_) => {}
            Row::Transition(transition) => self.resume_after(&transition.transition_id.0),
        }
        self.head = record.version;
        self.records.push(record);
    }

    /// Moves the sequence of `id`'s prefix to at least `id`'s number. An id that is not a prefix
    /// ending in `_` followed by digits takes no number from the sequence and moves nothing.
    fn resume_after(&mut self, id: &str) {
        let Some(split) = id.find('_').map(|at| at + 1) else {
            return;
        };
        let (prefix, digits) = id.split_at(split);
        if digits.is_empty() || !digits.bytes().all(|byte| byte.is_ascii_digit()) {
            return;
        }
        if let Ok(number) = digits.parse::<u64>() {
            let last = self.sequences.entry(prefix.to_owned()).or_insert(0);
            *last = (*last).max(number);
        }
    }
}

/// Identities are store-local (design § 2.2, D24): `smp_`, `evl_` and a never-decreasing sequence
/// per prefix. Each number is reserved on the instance's id stream before it is answered, after
/// the highest number of its prefix the instance stream and the id stream hold, so no two
/// handles, processes or reopened stores answer one number, stored or not.
///
/// # Panics
///
/// When a stream cannot be read or written, or another writer reserved first on each of
/// [`CATCH_UP_ATTEMPTS`] tries: a number answered without its reservation could be one another
/// writer answers too.
impl IdSequence for EventlogStore {
    fn next_number(&mut self, prefix: &'static str) -> Option<u64> {
        self.reserve(prefix).unwrap_or_else(|error| {
            panic!("the next {prefix} id cannot be reserved on the instance's id stream: {error}")
        })
    }
}

/// The command an append under `key` is recorded with: the key is the record's identity, and the
/// request hash is the body's.
fn command_meta(key: &str, body: &Value) -> Result<CommandMeta, StoreError> {
    Ok(CommandMeta {
        idempotency_key: key.to_owned(),
        request_hash: eventlog_core::request_hash(body)?,
        subject: SUBJECT.to_owned(),
        actor: ACTOR.to_owned(),
        request_id: key.to_owned(),
        trace_id: key.to_owned(),
        causation_id: None,
        causation_depth: 0,
        occurred_at: OffsetDateTime::now_utc(),
        claim: None,
    })
}

/// Refuses a body holding a JSON number that is not an integer: a `Decimal` is a string (D11),
/// and a fraction would hash differently with `serde_json/arbitrary_precision` on.
fn integers_only(value: &Value, at: &str) -> Result<(), StoreError> {
    match value {
        Value::Number(number) if !(number.is_i64() || number.is_u64()) => Err(StoreError::Record(
            format!("{at} is the number {number}, not an integer: a Decimal is a string"),
        )),
        Value::Array(items) => items
            .iter()
            .enumerate()
            .try_for_each(|(index, item)| integers_only(item, &format!("{at}/{index}"))),
        Value::Object(members) => members
            .iter()
            .try_for_each(|(name, member)| integers_only(member, &format!("{at}/{name}"))),
        _ => Ok(()),
    }
}

/// A part of a record, written by the generated wire encoder, as a JSON value.
pub(crate) fn part<T>(value: &T, encode: fn(&T, &mut String)) -> Result<Value, StoreError> {
    let mut text = String::new();
    encode(value, &mut text);
    serde_json::from_str(&text)
        .map_err(|error| StoreError::Record(format!("the wire encoder wrote no JSON: {error}")))
}

/// A part of a record, read by the generated wire decoder.
pub(crate) fn unpart<T>(
    body: &Value,
    name: &str,
    decode: fn(&json::Value, &str) -> Result<T, json::DecodeError>,
) -> Result<T, StoreError> {
    let member = body
        .get(name)
        .ok_or_else(|| StoreError::Record(format!("the record has no `{name}`")))?;
    let parsed = json::parse(&member.to_string())
        .map_err(|error| StoreError::Record(format!("`{name}`: {error}")))?;
    decode(&parsed, name).map_err(|error| StoreError::Record(error.to_string()))
}

/// An optional part: absent when the member is absent.
pub(crate) fn unpart_optional<T>(
    body: &Value,
    name: &str,
    decode: fn(&json::Value, &str) -> Result<T, json::DecodeError>,
) -> Result<Option<T>, StoreError> {
    match body.get(name) {
        None => Ok(None),
        Some(_) => unpart(body, name, decode).map(Some),
    }
}

/// A text member of a record.
pub(crate) fn text(body: &Value, name: &str) -> Result<String, StoreError> {
    body.get(name)
        .and_then(Value::as_str)
        .map(str::to_owned)
        .ok_or_else(|| StoreError::Record(format!("the record has no text `{name}`")))
}

/// A record's optional text member `name`: absent, or a string.
pub(crate) fn optional_text(body: &Value, name: &str) -> Result<Option<String>, StoreError> {
    match body.get(name) {
        None => Ok(None),
        Some(Value::String(value)) => Ok(Some(value.clone())),
        Some(_) => Err(StoreError::Record(format!(
            "the record's `{name}` is not text"
        ))),
    }
}

/// A record's `dims`: a map of text.
pub(crate) fn dims(
    body: &Value,
) -> Result<BTreeMap<String, thalamus_model::dimension::DimensionValue>, StoreError> {
    let Some(Value::Object(members)) = body.get("dims") else {
        return Err(StoreError::Record(
            "the record has no `dims` map".to_owned(),
        ));
    };
    members
        .iter()
        .map(|(key, value)| {
            value
                .as_str()
                .map(|text| {
                    (
                        key.clone(),
                        thalamus_model::dimension::DimensionValue(text.to_owned()),
                    )
                })
                .ok_or_else(|| StoreError::Record(format!("`dims/{key}` is not text")))
        })
        .collect()
}

/// `dims` as a JSON map of text.
pub(crate) fn dims_value(
    dims: &BTreeMap<String, thalamus_model::dimension::DimensionValue>,
) -> Value {
    Value::Object(
        dims.iter()
            .map(|(key, value)| (key.clone(), Value::String(value.0.clone())))
            .collect(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Only SQLite's busy lock is retried; a table or schema lock is not.
    #[test]
    fn only_a_busy_sqlite_lock_is_retried() {
        let backend = |m: &str| EventLogError::Backend(m.to_string());
        assert!(is_sqlite_locked(&backend("database is locked")));
        assert!(!is_sqlite_locked(&backend("database table is locked")));
        assert!(!is_sqlite_locked(&backend("database schema is locked")));
    }

    /// An append that meets another writer's append on every attempt is retried up to its bound,
    /// then answered with an error naming the attempts, never looped forever or dropped silently.
    #[test]
    fn catch_up_gives_up_after_its_bound_naming_the_attempts() {
        let root = tempfile::tempdir().expect("tempdir");
        let mut store = EventlogStore::open(&Locator::Sqlite(root.path().join("t.sqlite")), "unit")
            .expect("opens");
        let mut calls = 0;
        let answer = store.append_caught_up(|_| {
            calls += 1;
            Err(StoreError::Eventlog(EventLogError::Conflict {
                expected: 1,
                actual: 2,
            }))
        });
        assert_eq!(calls, 64);
        let error = answer.expect_err("gives up");
        assert!(error.to_string().contains("64 attempts"), "{error}");
    }
}
