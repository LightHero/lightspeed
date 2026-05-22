//! SQLite-backed [`ScheduleRepository`].
//!
//! Mirrors the Postgres/MySQL backends: schedule rows live in the standard
//! c3p0 table layout (`id / version / create_time / update_time / data`) with
//! the scheduler payload in `data` (a JSON column). The `(group_name, name)`
//! tuple is enforced unique by a multi-column index on JSON-extracted
//! expressions.
//!
//! Unlike Postgres/MySQL, SQLite has no `FOR UPDATE SKIP LOCKED`. Multi-process
//! distribution is therefore not supported in the same sense: SQLite is
//! intended for single-process scheduling. The shared integration suite's
//! concurrent-claim test (`for_update_skip_locked_serialises_concurrent_claims`)
//! is consequently skipped on the sqlite backend. The due predicate still uses
//! the **DB clock** (`strftime('%s','now')`) so a schedule that anchors itself
//! to a future time is honoured even if the writing process's wall clock
//! drifts.

use c3p0::sqlx::{AssertSqlSafe, Row, Sqlite, SqliteConnection, migrate::Migrator, query};
use c3p0::*;

use crate::error::SchedulerError;
use crate::repository::sql::{
    ScheduleData, ScheduleModel, ScheduleSqlBackend, SqlScheduleRepository,
};

static MIGRATOR: Migrator = c3p0::sqlx::migrate!("src/resources/sqlite/migrations");

/// SQLite [`ScheduleSqlBackend`]. Holds a [`SqliteC3p0Pool`] so the rest of
/// the workspace can hand the same pool to other lightspeed modules.
#[derive(Clone)]
pub struct SqliteScheduleBackend {
    c3p0: SqliteC3p0Pool,
}

impl SqliteScheduleBackend {
    pub fn new(c3p0: SqliteC3p0Pool) -> Self {
        Self { c3p0 }
    }
}

/// SQL fragment that returns SQLite's current time in milliseconds since the
/// Unix epoch as an integer. `unixepoch('now', 'subsec')` (SQLite 3.42+)
/// returns floating-point seconds with sub-second resolution; the trailing
/// `* 1000` and CAST yields the millisecond integer that matches
/// `next_run_at_millis`. Plain second-precision `strftime('%s','now')` is not
/// enough — a job registered "due now" (with the writer's ms-precision clock)
/// would otherwise look not-yet-due to the DB clock for up to a second.
const NOW_MILLIS_EXPR: &str = "CAST(unixepoch('now', 'subsec') * 1000 AS INTEGER)";

impl ScheduleSqlBackend for SqliteScheduleBackend {
    type DB = Sqlite;
    type C3p0Pool = SqliteC3p0Pool;

    fn c3p0(&self) -> &Self::C3p0Pool {
        &self.c3p0
    }

    fn pool(&self) -> &c3p0::sqlx::Pool<Sqlite> {
        self.c3p0.pool()
    }

    async fn fetch_record(
        tx: &mut SqliteConnection,
        group: &str,
        name: &str,
    ) -> Result<Option<ScheduleModel>, C3p0Error> {
        Ok(ScheduleModel::query_with_tail(
            "WHERE data->>'$.group_name' = ? AND data->>'$.name' = ? LIMIT 1",
        )
        .bind(group)
        .bind(name)
        .fetch_optional(tx)
        .await?)
    }

    async fn try_claim_record(
        tx: &mut SqliteConnection,
        group: &str,
        name: &str,
    ) -> Result<Option<ScheduleModel>, C3p0Error> {
        // SQLite has no `FOR UPDATE`, so this is a plain SELECT against the
        // DB clock. Single-process scheduling is the supported model; with a
        // single-writer pool the `try_claim_due → advance → commit` flow is
        // already serialised by the connection.
        let sql = format!(
            "WHERE data->>'$.group_name' = ? \
               AND data->>'$.name' = ? \
               AND CAST(data->>'$.next_run_at_millis' AS INTEGER) <= {NOW_MILLIS_EXPR} \
             LIMIT 1",
        );
        Ok(ScheduleModel::query_with_tail(&sql)
            .bind(group)
            .bind(name)
            .fetch_optional(tx)
            .await?)
    }

    async fn advance_claimed(
        tx: &mut SqliteConnection,
        group: &str,
        name: &str,
        next_run_at_millis: i64,
        last_run_at_millis: i64,
    ) -> Result<(), C3p0Error> {
        // Single round-trip: `json_set` rewrites both fields in one call,
        // version is bumped, `update_time` is refreshed (same ISO-8601
        // format c3p0's sqlite update uses). The tx already holds the row
        // claim from `try_claim_record` and SQLite serialises writes anyway,
        // so no version re-check is needed.
        const SQL: &str = "UPDATE LS_SCHEDULE \
             SET version = version + 1, \
                 update_time = strftime('%Y-%m-%dT%H:%M:%fZ', 'now'), \
                 data = json_set(data, \
                     '$.next_run_at_millis', ?, \
                     '$.last_run_at_millis', ?) \
             WHERE data->>'$.group_name' = ? \
               AND data->>'$.name' = ?";
        query(AssertSqlSafe(SQL))
            .bind(next_run_at_millis)
            .bind(last_run_at_millis)
            .bind(group)
            .bind(name)
            .execute(tx)
            .await?;
        Ok(())
    }

    async fn time_until_next_due_millis(
        pool: &c3p0::sqlx::Pool<Sqlite>,
        keys: &[(&str, &str)],
    ) -> Result<Option<i64>, C3p0Error> {
        // SQLite supports row-value IN since 3.15, so the (group, name) list
        // expands to one `(?, ?)` pair per key. The delta is computed on the
        // DB side so the "now" used here is the same `strftime` expression
        // the claim query uses.
        let placeholders = vec!["(?, ?)"; keys.len()].join(", ");
        let sql = format!(
            "SELECT MIN(CAST(data->>'$.next_run_at_millis' AS INTEGER)) - {NOW_MILLIS_EXPR} \
                    AS millis_until \
             FROM {} \
             WHERE (data->>'$.group_name', data->>'$.name') IN ({})",
            ScheduleData::TABLE_NAME,
            placeholders,
        );
        let mut q = query(AssertSqlSafe(sql));
        for (g, n) in keys {
            q = q.bind(*g).bind(*n);
        }
        let row = q.fetch_one(pool).await?;
        Ok(row.try_get("millis_until")?)
    }
}

/// SQLite-backed [`crate::ScheduleRepository`]. Wraps a
/// [`SqliteScheduleBackend`] in the generic [`SqlScheduleRepository`].
pub type SqliteScheduleRepository = SqlScheduleRepository<SqliteScheduleBackend>;

impl SqliteScheduleRepository {
    /// Applies the bundled migrations and returns a ready-to-use repository.
    /// This is the only constructor — the repository is never observable in
    /// a pre-migration state. Safe to call from every process on startup:
    /// sqlx's migration tracker table makes the bundled migrations
    /// idempotent.
    pub async fn init(c3p0: SqliteC3p0Pool) -> Result<Self, SchedulerError> {
        MIGRATOR.run(c3p0.pool()).await?;
        Ok(SqlScheduleRepository::new(SqliteScheduleBackend::new(c3p0)))
    }
}
