//! Pluggable storage backend for distributed schedules.

use std::future::Future;
use std::time::{Duration, SystemTime};

use crate::error::SchedulerError;

mod memory;

#[cfg(feature = "c3p0")]
pub mod sql;
#[cfg(feature = "c3p0")]
pub use sql::{ScheduleData, ScheduleDataCodec, ScheduleModel, ScheduleSqlBackend, SqlScheduleRepository};

#[cfg(feature = "mysql")]
pub mod mysql;
#[cfg(feature = "mysql")]
pub use mysql::{MySqlScheduleBackend, MySqlScheduleRepository};

#[cfg(feature = "postgres")]
pub mod postgres;
#[cfg(feature = "postgres")]
pub use postgres::{PgScheduleBackend, PgScheduleRepository};

#[cfg(feature = "sqlite")]
pub mod sqlite;
#[cfg(feature = "sqlite")]
pub use sqlite::{SqliteScheduleBackend, SqliteScheduleRepository};

pub use memory::MemoryScheduleRepository;

/// A persisted schedule row.
///
/// The row only holds coordination state (when the schedule should next fire
/// and when it last fired) and its identity `(group, name)`. The schedule
/// *definition* — cron expression, interval, etc. — lives in the running
/// process and is not persisted, so a redeploy with a different schedule
/// simply takes effect from the next firing onward.
#[derive(Debug, Clone)]
pub struct ScheduleRow {
    pub group: String,
    pub name: String,
    pub next_run_at: SystemTime,
    pub last_run_at: Option<SystemTime>,
}

/// Pluggable storage for distributed schedules.
///
/// The contract is intentionally minimal: each implementation must provide
/// "claim exactly one due row, advance it, and don't let two processes pick
/// up the same row at the same time".
///
/// Each task is identified by a `(group, name)` pair. `group` lets one
/// database hold schedules for multiple applications or business domains
/// without name collisions and supports indexed group-scoped queries.
pub trait ScheduleRepository: Send + Sync + Clone + 'static {
    /// Transaction handle scoped to a single tick.
    type Tx: Send;

    fn begin(&self) -> impl Future<Output = Result<Self::Tx, SchedulerError>> + Send;

    fn commit(&self, tx: Self::Tx) -> impl Future<Output = Result<(), SchedulerError>> + Send;

    fn rollback(&self, tx: Self::Tx) -> impl Future<Output = Result<(), SchedulerError>> + Send;

    /// Inserts the schedule row if `(group, name)` doesn't exist yet. Safe to
    /// call on every process startup — only one INSERT wins; concurrent
    /// callers no-op. The schedule *definition* itself is held by the caller
    /// (in-memory) and is intentionally not persisted; what *is* persisted
    /// is a `schedule_fingerprint` — an opaque string the caller computes
    /// from the definition (see [`Schedule::fingerprint`](crate::Schedule::fingerprint)).
    ///
    /// **Change detection.** If the row already exists and the stored
    /// fingerprint differs from `schedule_fingerprint`, the schedule
    /// definition has changed across the redeploy: the backend updates the
    /// row's `next_run_at` to the supplied value, replaces the fingerprint,
    /// and emits a warning log. This is the only path through which an
    /// existing row's `next_run_at` is rewritten without going through
    /// [`advance`](Self::advance).
    fn register(
        &self,
        group: &str,
        name: &str,
        next_run_at: SystemTime,
        schedule_fingerprint: &str,
    ) -> impl Future<Output = Result<(), SchedulerError>> + Send;

    /// Within `tx`, claims the schedule `(group, name)` if its `next_run_at`
    /// is less than or equal to the **repository's** current time (Postgres
    /// `NOW()` for the SQL backend; process clock for the memory backend) and
    /// no other transaction currently holds it. Returns the row when claimed;
    /// returns `None` when the schedule isn't due or another worker already
    /// holds the lock.
    ///
    /// Using the repository's own clock means clock skew between scheduler
    /// processes doesn't cause early or duplicate firings.
    fn try_claim_due(
        &self,
        tx: &mut Self::Tx,
        group: &str,
        name: &str,
    ) -> impl Future<Output = Result<Option<ScheduleRow>, SchedulerError>> + Send;

    /// Within `tx`, updates `next_run_at` and `last_run_at` for `(group, name)`.
    fn advance(
        &self,
        tx: &mut Self::Tx,
        group: &str,
        name: &str,
        next_run_at: SystemTime,
        last_run_at: SystemTime,
    ) -> impl Future<Output = Result<(), SchedulerError>> + Send;

    /// Duration from the **repository's** current time until the earliest
    /// `next_run_at` across the rows identified by the given `(group, name)`
    /// pairs. `Some(Duration::ZERO)` means at least one row is already due;
    /// `None` means no matching rows exist.
    ///
    /// The scheduler's poll loop uses this to size its sleep so that timing
    /// decisions are anchored to the same clock the claim query uses,
    /// neutralising clock skew between scheduler processes.
    fn time_until_next_due(
        &self,
        keys: &[(&str, &str)],
    ) -> impl Future<Output = Result<Option<Duration>, SchedulerError>> + Send;
}
