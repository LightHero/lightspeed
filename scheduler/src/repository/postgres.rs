//! Postgres-backed [`ScheduleRepository`].
//!
//! Schedule rows are stored in the standard c3p0 table layout
//! (`id / version / create_time / update_time / data`); the scheduler-specific
//! fields live in `data` (a JSONB column) via the [`ScheduleData`]
//! [`DataType`] impl. The `(data->>'group_name', data->>'name')` tuple is
//! enforced unique by an index in the migration.
//!
//! Distributed-safety relies on `FOR UPDATE SKIP LOCKED`: when one process
//! claims a schedule row inside a transaction, concurrent claimers see the
//! row as locked and skip it.

use std::time::{Duration, SystemTime, UNIX_EPOCH};

use c3p0::sqlx::{AssertSqlSafe, Postgres, Row, Transaction, migrate::Migrator, query};
use c3p0::*;
use serde::{Deserialize, Serialize};

use crate::error::SchedulerError;
use crate::repository::{ScheduleRepository, ScheduleRow};

static MIGRATOR: Migrator = c3p0::sqlx::migrate!("src/resources/postgres/migrations");

/// Convenience alias for `Record<ScheduleData>` — the c3p0 record carrying
/// the scheduler payload.
pub type ScheduleModel = Record<ScheduleData>;

/// JSONB payload of a schedule row.
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
    /// then populate it (and log a warning — see `register`).
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

/// Postgres-backed [`ScheduleRepository`] implementation. Wraps a
/// [`PgC3p0Pool`] so the rest of the workspace can hand the same pool to
/// other lightspeed modules.
#[derive(Clone)]
pub struct PgScheduleRepository {
    c3p0: PgC3p0Pool,
}

impl PgScheduleRepository {
    /// Wraps `c3p0` and applies the bundled migrations. This is the only
    /// constructor — the repository is never observable in a pre-migration
    /// state. Safe to call from every process on startup: sqlx's migration
    /// tracker table makes the bundled migrations idempotent.
    pub async fn init(c3p0: PgC3p0Pool) -> Result<Self, SchedulerError> {
        MIGRATOR.run(c3p0.pool()).await?;
        Ok(Self { c3p0 })
    }

    pub fn c3p0(&self) -> &PgC3p0Pool {
        &self.c3p0
    }
}

impl ScheduleRepository for PgScheduleRepository {
    type Tx = Transaction<'static, Postgres>;

    async fn begin(&self) -> Result<Self::Tx, SchedulerError> {
        Ok(self.c3p0.pool().begin().await?)
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
        // Run the existence check + insert/update atomically through c3p0's
        // closure-based transaction API: returning `Ok` commits, returning
        // `Err` rolls back (which is what we need on a unique-index race —
        // a poisoned tx can't be committed, only rolled back).
        let group_owned = group.to_string();
        let name_owned = name.to_string();
        let fingerprint_owned = schedule_fingerprint.to_string();
        let result: Result<(), C3p0Error> = self
            .c3p0
            .transaction(async move |tx: &mut c3p0::sqlx::PgConnection| -> Result<(), C3p0Error> {
                let existing: Option<ScheduleModel> = ScheduleModel::query_with_tail(
                    "WHERE data->>'group_name' = $1 AND data->>'name' = $2 LIMIT 1",
                )
                .bind(&group_owned)
                .bind(&name_owned)
                .fetch_optional(&mut *tx)
                .await?;
                if let Some(mut record) = existing {
                    if record.data.schedule_fingerprint != fingerprint_owned {
                        log::warn!(
                            target: "lightspeed_scheduler",
                            "schedule definition changed; re-anchoring next_run_at. \
                             group: {group_owned}, name: {name_owned}, \
                             old_fingerprint: {old:?}, new_fingerprint: {new:?}",
                            old = record.data.schedule_fingerprint,
                            new = fingerprint_owned,
                        );
                        record.data.next_run_at_millis = to_millis(next_run_at);
                        record.data.schedule_fingerprint = fingerprint_owned.clone();
                        tx.update(record).await?;
                    }
                    return Ok(());
                }
                let new = NewRecord::new(ScheduleData {
                    group_name: group_owned.clone(),
                    name: name_owned.clone(),
                    next_run_at_millis: to_millis(next_run_at),
                    last_run_at_millis: None,
                    schedule_fingerprint: fingerprint_owned.clone(),
                });
                tx.save(new).await?;
                Ok(())
            })
            .await;
        match result {
            Ok(()) => Ok(()),
            // Concurrent register won the race; row exists now — done.
            Err(C3p0Error::SqlxError(e)) if is_unique_violation(&e) => Ok(()),
            Err(e) => Err(e.into()),
        }
    }

    async fn try_claim_due(
        &self,
        tx: &mut Self::Tx,
        group: &str,
        name: &str,
    ) -> Result<Option<ScheduleRow>, SchedulerError> {
        // `FOR UPDATE SKIP LOCKED` makes the DB the single authority on who
        // owns each row this tick. `NOW()` (DB clock) is the due predicate
        // so clock skew between scheduler processes can't cause early or
        // duplicate firings.
        let record: Option<ScheduleModel> = ScheduleModel::query_with_tail(
            "WHERE data->>'group_name' = $1 \
               AND data->>'name' = $2 \
               AND (data->>'next_run_at_millis')::bigint <= (EXTRACT(EPOCH FROM NOW()) * 1000)::BIGINT \
             LIMIT 1 \
             FOR UPDATE SKIP LOCKED",
        )
        .bind(group)
        .bind(name)
        .fetch_optional(&mut **tx)
        .await?;

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
        // The tx already holds `FOR UPDATE` on this row (from `try_claim_due`),
        // so the refetch is contention-free. Going through `tx.update` gives
        // us version bumping, `update_time` bookkeeping, and JSONB encoding
        // for free.
        let mut record: ScheduleModel = ScheduleModel::query_with_tail(
            "WHERE data->>'group_name' = $1 AND data->>'name' = $2 LIMIT 1",
        )
        .bind(group)
        .bind(name)
        .fetch_one(&mut **tx)
        .await?;
        record.data.next_run_at_millis = to_millis(next_run_at);
        record.data.last_run_at_millis = Some(to_millis(last_run_at));
        (&mut **tx).update(record).await?;
        Ok(())
    }

    async fn time_until_next_due(
        &self,
        keys: &[(&str, &str)],
    ) -> Result<Option<Duration>, SchedulerError> {
        if keys.is_empty() {
            return Ok(None);
        }
        // Two parallel arrays let us bind any number of (group, name) pairs
        // through one prepared statement; UNNEST zips them back into rows.
        let groups: Vec<String> = keys.iter().map(|(g, _)| (*g).to_string()).collect();
        let names: Vec<String> = keys.iter().map(|(_, n)| (*n).to_string()).collect();
        // Compute the delta on the DB side so the "now" used here is the
        // same NOW() used by `try_claim_due`'s predicate. A negative result
        // (row is already overdue) is clamped to 0 below.
        let sql = format!(
            "SELECT MIN((data->>'next_run_at_millis')::bigint) \
                    - (EXTRACT(EPOCH FROM NOW()) * 1000)::BIGINT AS millis_until \
             FROM {} \
             WHERE (data->>'group_name', data->>'name') IN ( \
                 SELECT g, n FROM UNNEST($1::TEXT[], $2::TEXT[]) AS t(g, n) \
             )",
            ScheduleData::TABLE_NAME,
        );
        let row = query(AssertSqlSafe(sql))
            .bind(&groups)
            .bind(&names)
            .fetch_one(self.c3p0.pool())
            .await?;
        let millis: Option<i64> = row.try_get("millis_until")?;
        Ok(millis.map(|m| Duration::from_millis(m.max(0) as u64)))
    }
}

fn is_unique_violation(e: &c3p0::sqlx::Error) -> bool {
    if let c3p0::sqlx::Error::Database(db_err) = e {
        return db_err.is_unique_violation();
    }
    false
}

fn to_millis(t: SystemTime) -> i64 {
    t.duration_since(UNIX_EPOCH)
        .expect("system time before unix epoch")
        .as_millis() as i64
}

fn from_millis(ms: i64) -> SystemTime {
    UNIX_EPOCH + Duration::from_millis(ms as u64)
}
