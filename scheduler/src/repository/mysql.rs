//! MySQL-backed [`ScheduleRepository`].
//!
//! Mirrors the Postgres backend: schedule rows live in the standard c3p0
//! table layout (`id / version / create_time / update_time / data`) with the
//! scheduler payload in `data` (a JSON column). The `(group_name, name)`
//! tuple is enforced unique by a multi-column index on the JSON-extracted
//! virtual columns.
//!
//! Distributed-safety relies on InnoDB's `FOR UPDATE SKIP LOCKED`
//! (MySQL 8.0+). The due predicate uses `CURRENT_TIMESTAMP(3)` so clock skew
//! between scheduler processes cannot cause early or duplicate firings.

use c3p0::sqlx::{AssertSqlSafe, MySql, MySqlConnection, Row, migrate::Migrator, query};
use c3p0::*;

use crate::error::SchedulerError;
use crate::repository::sql::{ScheduleData, ScheduleModel, ScheduleSqlBackend, SqlScheduleRepository};

static MIGRATOR: Migrator = c3p0::sqlx::migrate!("src/resources/mysql/migrations");

/// MySQL [`ScheduleSqlBackend`]. Holds a [`MySqlC3p0Pool`] so the rest of the
/// workspace can hand the same pool to other lightspeed modules.
#[derive(Clone)]
pub struct MySqlScheduleBackend {
    c3p0: MySqlC3p0Pool,
}

impl MySqlScheduleBackend {
    pub fn new(c3p0: MySqlC3p0Pool) -> Self {
        Self { c3p0 }
    }
}

impl ScheduleSqlBackend for MySqlScheduleBackend {
    type DB = MySql;
    type C3p0Pool = MySqlC3p0Pool;

    fn c3p0(&self) -> &Self::C3p0Pool {
        &self.c3p0
    }

    fn pool(&self) -> &c3p0::sqlx::Pool<MySql> {
        self.c3p0.pool()
    }

    async fn fetch_record(
        tx: &mut MySqlConnection,
        group: &str,
        name: &str,
    ) -> Result<Option<ScheduleModel>, C3p0Error> {
        Ok(ScheduleModel::query_with_tail(
            "WHERE JSON_VALUE(data, '$.group_name' RETURNING CHAR(255)) = ? \
               AND JSON_VALUE(data, '$.name' RETURNING CHAR(255)) = ? \
             LIMIT 1",
        )
        .bind(group)
        .bind(name)
        .fetch_optional(tx)
        .await?)
    }

    async fn try_claim_record(
        tx: &mut MySqlConnection,
        group: &str,
        name: &str,
    ) -> Result<Option<ScheduleModel>, C3p0Error> {
        Ok(ScheduleModel::query_with_tail(
            "WHERE JSON_VALUE(data, '$.group_name' RETURNING CHAR(255)) = ? \
               AND JSON_VALUE(data, '$.name' RETURNING CHAR(255)) = ? \
               AND JSON_VALUE(data, '$.next_run_at_millis' RETURNING SIGNED) \
                   <= CAST(UNIX_TIMESTAMP(CURRENT_TIMESTAMP(3)) * 1000 AS SIGNED) \
             LIMIT 1 \
             FOR UPDATE SKIP LOCKED",
        )
        .bind(group)
        .bind(name)
        .fetch_optional(tx)
        .await?)
    }

    async fn advance_claimed(
        tx: &mut MySqlConnection,
        group: &str,
        name: &str,
        next_run_at_millis: i64,
        last_run_at_millis: i64,
    ) -> Result<(), C3p0Error> {
        // Single round-trip: `JSON_SET` rewrites both fields in one call,
        // version is bumped, `update_time` is refreshed. The tx already
        // holds the row lock from `try_claim_record`, so no version re-check
        // is needed.
        const SQL: &str = "UPDATE LS_SCHEDULE \
             SET version = version + 1, \
                 update_time = CURRENT_TIMESTAMP(3), \
                 data = JSON_SET(data, \
                     '$.next_run_at_millis', ?, \
                     '$.last_run_at_millis', ?) \
             WHERE JSON_VALUE(data, '$.group_name' RETURNING CHAR(255)) = ? \
               AND JSON_VALUE(data, '$.name' RETURNING CHAR(255)) = ?";
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
        pool: &c3p0::sqlx::Pool<MySql>,
        keys: &[(&str, &str)],
    ) -> Result<Option<i64>, C3p0Error> {
        // MySQL has no UNNEST, so the (group, name) list expands to a row-value
        // IN list with one `(?, ?)` pair per key. The delta is computed on the
        // DB side so the "now" used here is the same `CURRENT_TIMESTAMP(3)`
        // the claim query uses.
        let placeholders = vec!["(?, ?)"; keys.len()].join(", ");
        let sql = format!(
            "SELECT MIN(JSON_VALUE(data, '$.next_run_at_millis' RETURNING SIGNED)) \
                    - CAST(UNIX_TIMESTAMP(CURRENT_TIMESTAMP(3)) * 1000 AS SIGNED) \
                    AS millis_until \
             FROM {} \
             WHERE ( \
                 JSON_VALUE(data, '$.group_name' RETURNING CHAR(255)), \
                 JSON_VALUE(data, '$.name' RETURNING CHAR(255)) \
             ) IN ({})",
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

/// MySQL-backed [`crate::ScheduleRepository`]. Wraps a [`MySqlScheduleBackend`]
/// in the generic [`SqlScheduleRepository`].
pub type MySqlScheduleRepository = SqlScheduleRepository<MySqlScheduleBackend>;

impl MySqlScheduleRepository {
    /// Applies the bundled migrations and returns a ready-to-use repository.
    /// This is the only constructor — the repository is never observable in
    /// a pre-migration state. Safe to call from every process on startup:
    /// sqlx's migration tracker table makes the bundled migrations
    /// idempotent.
    pub async fn init(c3p0: MySqlC3p0Pool) -> Result<Self, SchedulerError> {
        MIGRATOR.run(c3p0.pool()).await?;
        Ok(SqlScheduleRepository::new(MySqlScheduleBackend::new(c3p0)))
    }
}
