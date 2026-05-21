//! Shared SQL [`ScheduleRepository`] machinery.
//!
//! The three SQL backends (Postgres, MySQL, SQLite) only differ in dialect:
//! JSON access syntax, placeholder style, the SQL fragment that returns the
//! database's current millisecond timestamp, and the SQL fragment used to
//! "claim" a row (row-level locking on Postgres/MySQL; plain SELECT on
//! SQLite, which has no `FOR UPDATE`). Everything else — the
//! register/advance/time-until flow, the `c3p0` record layout, the codec, the
//! transaction handling — is the same. This module encodes that split: each
//! backend implements [`ScheduleSqlBackend`] with only its dialect-specific
//! queries; the generic [`SqlScheduleRepository`] wraps any backend and
//! implements the full [`ScheduleRepository`] contract on top of it.

use std::future::Future;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use c3p0::sqlx::{Database, Transaction};
use c3p0::*;
use serde::{Deserialize, Serialize};

use crate::error::SchedulerError;
use crate::repository::{ScheduleRepository, ScheduleRow};

/// Convenience alias for `Record<ScheduleData>` — the c3p0 record carrying
/// the scheduler payload.
pub type ScheduleModel = Record<ScheduleData>;

/// JSON payload of a schedule row. The same shape is used by every SQL
/// backend; only the SQL dialect that reads/writes it differs.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ScheduleData {
    pub group_name: String,
    pub name: String,
    pub next_run_at_millis: i64,
    pub last_run_at_millis: Option<i64>,
    /// Opaque identifier of the in-memory schedule definition that produced
    /// this row's `next_run_at_millis`. When a redeploy registers the same
    /// `(group, name)` with a different definition, the executor uses this
    /// to detect the change and re-anchor `next_run_at_millis`.
    ///
    /// Defaults to `""` so rows written by older versions (without this
    /// field) deserialize cleanly; the first re-register after upgrade will
    /// then populate it (and log a warning — see
    /// [`SqlScheduleRepository::register`]).
    #[serde(default)]
    pub schedule_fingerprint: String,
}

impl DataType for ScheduleData {
    const TABLE_NAME: &'static str = "LS_SCHEDULE";
    type CODEC = ScheduleDataCodec;
}

/// Versioned wire form of [`ScheduleData`]. Same shape as
/// `lightspeed_account_management`'s `*DataToken` enums — gives future
/// schema migrations a `V2`/`V3` arm to land in.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "_codec_tag")]
pub enum ScheduleDataCodec {
    V1(ScheduleData),
}

impl Codec<ScheduleData> for ScheduleDataCodec {
    fn encode(data: ScheduleData) -> Self {
        ScheduleDataCodec::V1(data)
    }

    fn decode(data: Self) -> ScheduleData {
        match data {
            ScheduleDataCodec::V1(data) => data,
        }
    }
}

/// Dialect-specific operations a SQL backend must provide so the generic
/// [`SqlScheduleRepository`] can drive it.
///
/// Implementors only translate three things into their own dialect:
/// 1. how to find a schedule row by `(group, name)` (used by register and
///    advance);
/// 2. how to atomically *claim* a due schedule row — `FOR UPDATE SKIP LOCKED`
///    against the DB clock on Postgres/MySQL, plain SELECT against the DB
///    clock on SQLite (which has no row locking);
/// 3. how to compute the milliseconds-until-next-due across a set of
///    `(group, name)` pairs, again against the DB clock.
pub trait ScheduleSqlBackend: Clone + Send + Sync + 'static {
    type DB: Database;
    type C3p0Pool: C3p0Pool<DB = Self::DB> + Clone + Send + Sync + 'static;

    fn c3p0(&self) -> &Self::C3p0Pool;

    /// Underlying sqlx pool. Needed for `pool.begin()` and for the
    /// `time_until_next_due` query that runs outside a tx.
    fn pool(&self) -> &c3p0::sqlx::Pool<Self::DB>;

    /// Looks up the schedule row for `(group, name)` without locking it.
    /// `None` when no such row exists.
    fn fetch_record(
        tx: &mut <Self::DB as Database>::Connection,
        group: &str,
        name: &str,
    ) -> impl Future<Output = Result<Option<ScheduleModel>, C3p0Error>> + Send;

    /// Within `tx`, attempts to claim the schedule row for `(group, name)`:
    /// returns it if `next_run_at_millis` is `<=` the **database's** current
    /// time *and* no other transaction is currently holding it. Returns
    /// `None` otherwise.
    ///
    /// The "DB clock" anchor neutralises clock skew between scheduler
    /// processes.
    fn try_claim_record(
        tx: &mut <Self::DB as Database>::Connection,
        group: &str,
        name: &str,
    ) -> impl Future<Output = Result<Option<ScheduleModel>, C3p0Error>> + Send;

    /// Duration (in milliseconds) from the **database's** current time until
    /// the earliest `next_run_at_millis` across the rows identified by the
    /// given `(group, name)` pairs. Negative values (a row is already
    /// overdue) are clamped to zero by [`SqlScheduleRepository`]; `None`
    /// means no matching rows exist.
    fn time_until_next_due_millis(
        pool: &c3p0::sqlx::Pool<Self::DB>,
        keys: &[(&str, &str)],
    ) -> impl Future<Output = Result<Option<i64>, C3p0Error>> + Send;
}

/// Generic [`ScheduleRepository`] over any [`ScheduleSqlBackend`].
///
/// Carries the cross-backend logic (register flow, advance flow,
/// `Duration` conversion, transaction lifecycle); the dialect-specific bits
/// are delegated to the [`ScheduleSqlBackend`] trait.
#[derive(Clone)]
pub struct SqlScheduleRepository<B: ScheduleSqlBackend> {
    backend: B,
}

impl<B: ScheduleSqlBackend> SqlScheduleRepository<B> {
    pub fn new(backend: B) -> Self {
        Self { backend }
    }

    pub fn c3p0(&self) -> &B::C3p0Pool {
        self.backend.c3p0()
    }

    pub fn backend(&self) -> &B {
        &self.backend
    }
}

impl<B> ScheduleRepository for SqlScheduleRepository<B>
where
    B: ScheduleSqlBackend,
    Record<ScheduleData>: DbOps<B::DB, ScheduleData>,
    NewRecord<ScheduleData>: DbSave<B::DB, ScheduleData>,
{
    type Tx = Transaction<'static, B::DB>;

    async fn begin(&self) -> Result<Self::Tx, SchedulerError> {
        Ok(self.backend.pool().begin().await?)
    }

    async fn commit(&self, tx: Self::Tx) -> Result<(), SchedulerError> {
        tx.commit().await?;
        Ok(())
    }

    async fn rollback(&self, tx: Self::Tx) -> Result<(), SchedulerError> {
        tx.rollback().await?;
        Ok(())
    }

    async fn register(
        &self,
        group: &str,
        name: &str,
        next_run_at: SystemTime,
        schedule_fingerprint: &str,
    ) -> Result<(), SchedulerError> {
        // Existence check + insert/update run inside one transaction so
        // concurrent registers either both succeed idempotently or one
        // surfaces a unique-violation we treat as a "row exists now" no-op.
        let mut tx = self.backend.pool().begin().await.map_err(C3p0Error::from)?;
        let outcome: Result<(), C3p0Error> = async {
            let existing = B::fetch_record(&mut *tx, group, name).await?;
            if let Some(mut record) = existing {
                if record.data.schedule_fingerprint != schedule_fingerprint {
                    log::warn!(
                        target: "lightspeed_scheduler",
                        "schedule definition changed; re-anchoring next_run_at. \
                         group: {group}, name: {name}, \
                         old_fingerprint: {old:?}, new_fingerprint: {new:?}",
                        old = record.data.schedule_fingerprint,
                        new = schedule_fingerprint,
                    );
                    record.data.next_run_at_millis = to_millis(next_run_at);
                    record.data.schedule_fingerprint = schedule_fingerprint.to_string();
                    record.update(&mut *tx).await?;
                }
                return Ok(());
            }
            let new = NewRecord::new(ScheduleData {
                group_name: group.to_string(),
                name: name.to_string(),
                next_run_at_millis: to_millis(next_run_at),
                last_run_at_millis: None,
                schedule_fingerprint: schedule_fingerprint.to_string(),
            });
            new.save(&mut *tx).await?;
            Ok(())
        }
        .await;

        match outcome {
            Ok(()) => {
                tx.commit().await.map_err(C3p0Error::from)?;
                Ok(())
            }
            // Concurrent register won the race — the row exists now, so the
            // post-condition still holds. The tx is poisoned by the failed
            // INSERT and can only be rolled back.
            Err(C3p0Error::SqlxError(e)) if is_unique_violation(&e) => {
                let _ = tx.rollback().await;
                Ok(())
            }
            Err(e) => {
                let _ = tx.rollback().await;
                Err(e.into())
            }
        }
    }

    async fn try_claim_due(
        &self,
        tx: &mut Self::Tx,
        group: &str,
        name: &str,
    ) -> Result<Option<ScheduleRow>, SchedulerError> {
        let record = B::try_claim_record(&mut **tx, group, name).await?;
        Ok(record.map(|r| ScheduleRow {
            group: r.data.group_name,
            name: r.data.name,
            next_run_at: from_millis(r.data.next_run_at_millis),
            last_run_at: r.data.last_run_at_millis.map(from_millis),
        }))
    }

    async fn advance(
        &self,
        tx: &mut Self::Tx,
        group: &str,
        name: &str,
        next_run_at: SystemTime,
        last_run_at: SystemTime,
    ) -> Result<(), SchedulerError> {
        // The tx already holds the claim lock (from try_claim_due), so the
        // refetch is contention-free. Going through c3p0's update gives us
        // version bumping, `update_time` bookkeeping, and JSON encoding for
        // free.
        let mut record = B::fetch_record(&mut **tx, group, name).await?.ok_or_else(|| {
            C3p0Error::Other {
                cause: format!("schedule row not found for advance: {group}/{name}"),
            }
        })?;
        record.data.next_run_at_millis = to_millis(next_run_at);
        record.data.last_run_at_millis = Some(to_millis(last_run_at));
        record.update(&mut **tx).await?;
        Ok(())
    }

    async fn time_until_next_due(
        &self,
        keys: &[(&str, &str)],
    ) -> Result<Option<Duration>, SchedulerError> {
        if keys.is_empty() {
            return Ok(None);
        }
        let millis = B::time_until_next_due_millis(self.backend.pool(), keys).await?;
        Ok(millis.map(|m| Duration::from_millis(m.max(0) as u64)))
    }
}

/// True for any sqlx error that came from a database unique-index violation,
/// across all three SQL backends. `sqlx::DatabaseError::is_unique_violation`
/// already normalises the per-engine error codes for us.
pub(crate) fn is_unique_violation(e: &c3p0::sqlx::Error) -> bool {
    if let c3p0::sqlx::Error::Database(db_err) = e {
        return db_err.is_unique_violation();
    }
    false
}

pub(crate) fn to_millis(t: SystemTime) -> i64 {
    t.duration_since(UNIX_EPOCH)
        .expect("system time before unix epoch")
        .as_millis() as i64
}

pub(crate) fn from_millis(ms: i64) -> SystemTime {
    UNIX_EPOCH + Duration::from_millis(ms as u64)
}
