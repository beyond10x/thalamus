//! The append-only sample stream, evaluations and rollups behind one trait.
//!
//! The Eventlog store ([`eventlog::EventlogStore`]) keeps an instance's records on Eventlog's SQLite
//! provider (a local instance) or its PostgreSQL provider (a shared one); there is no File
//! provider (design § 11 D24).
//!
//! It drives Eventlog's asynchronous ports on a current-thread Tokio runtime of its own: every
//! call blocks, and none may be made from inside another Tokio runtime (an async caller moves the
//! store onto a blocking thread). The runtime is shut down in the background on drop, so the store
//! may be dropped anywhere, also inside asynchronous code.
//!
//! A handle stays current with its stream: appends take in what other writers appended, and the
//! reads that answer from the stream also read what was appended since the handle last moved. Such
//! a read panics when the stream cannot be read, rather than answering from a stale handle; the
//! slices (`samples`, `evaluations`, `records`) are the stream as of the handle's last call that
//! took `&mut self`.
//!
//! Tampering: Eventlog's SQLite provider does not itself detect a value altered in the database
//! file. The store checks every record it reads against the request hash Eventlog recorded with
//! it, which refuses a body altered on its own, but not one altered together with that hash.

pub mod definition_events;
pub mod evaluations;
pub mod eventlog;
pub mod range;
pub mod rollups;
pub mod samples;
pub mod transitions;
