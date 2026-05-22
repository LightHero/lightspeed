//! Postgres-backed [`ScheduleRepository`].
//!
//! Schedule rows are stored in the standard c3p0 table layout
//! (`id / version / create_time / update_time / data`); the scheduler-specific
//! fields live in `data` (a JSONB column). The `(data->>'group_name',
//! data->>'name')` tuple is enforced unique by an index in the migration.
//!
//! Distributed-safety relies on `FOR UPDATE SKIP LOCKED`: when one process
//! claims a schedule row inside a transaction, concurrent claimers see the
//! row as locked and skip it. The due predicate uses Postgres' `NOW()` so
//! clock skew between scheduler processes cannot cause early or duplicate
//! firings.

use c3p0::sqlx::{AssertSqlSafe, PgConnection, Postgres, Row, migrate::Migrator, query};
use c3p0::*;

use crate::error::SchedulerError;
use crate::repository::sql::{ScheduleData, ScheduleModel, ScheduleSqlBackend, SqlScheduleRepository};

static MIGRATOR: Migrator = c3p0::sqlx::migrate!("src/resources/postgres/migrations");

/// Postgres [`ScheduleSqlBackend`]. Holds a [`PgC3p0Pool`] so the rest of the
/// workspace can hand the same pool to other lightspeed modules.
#[derive(Clone)]
pub struct PgScheduleBackend {
    c3p0: PgC3p0Pool,
}

impl PgScheduleBackend {
    pub fn new(c3p0: PgC3p0Pool) -> Self {
        Self { c3p0 }
    }
}

impl ScheduleSqlBackend for PgScheduleBackend {
    type DB = Postgres;
    type C3p0Pool = PgC3p0Pool;

    fn c3p0(&self) -> &Self::C3p0Pool {
        &self.c3p0
    }

    fn pool(&self) -> &c3p0::sqlx::Pool<Postgres> {
        self.c3p0.pool()
    }

    async fn fetch_record(tx: &mut PgConnection, group: &str, name: &str) -> Result<Option<ScheduleModel>, C3p0Error> {
        Ok(ScheduleModel::query_with_tail("WHERE data->>'group_name' = $1 AND data->>'name' = $2 LIMIT 1")
            .bind(group)
            .bind(name)
            .fetch_optional(tx)
            .await?)
    }

    async fn try_claim_record(
        tx: &mut PgConnection,
        group: &str,
        name: &str,
    ) -> Result<Option<ScheduleModel>, C3p0Error> {
        Ok(ScheduleModel::query_with_tail(
            "WHERE data->>'group_name' = $1 \
               AND data->>'name' = $2 \
               AND (data->>'next_run_at_millis')::bigint <= (EXTRACT(EPOCH FROM NOW()) * 1000)::BIGINT \
             LIMIT 1 \
             FOR UPDATE SKIP LOCKED",
        )
        .bind(group)
        .bind(name)
        .fetch_optional(tx)
        .await?)
    }

    async fn advance_claimed(
        tx: &mut PgConnection,
        group: &str,
        name: &str,
        next_run_at_millis: i64,
        last_run_at_millis: i64,
    ) -> Result<(), C3p0Error> {
        // Single round-trip: merge the two updated fields into `data` via
        // the `||` (jsonb concat) operator, bump `version`, refresh
        // `update_time`. No SELECT and no version re-check because the tx
        // already holds the row lock from `try_claim_record`.
        const SQL: &str = "UPDATE LS_SCHEDULE \
             SET version = version + 1, \
                 update_time = CURRENT_TIMESTAMP, \
                 data = data || jsonb_build_object( \
                     'next_run_at_millis', $1::bigint, \
                     'last_run_at_millis', $2::bigint \
                 ) \
             WHERE data->>'group_name' = $3 AND data->>'name' = $4";
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
        pool: &c3p0::sqlx::Pool<Postgres>,
        keys: &[(&str, &str)],
    ) -> Result<Option<i64>, C3p0Error> {
        // Two parallel arrays let us bind any number of (group, name) pairs
        // through one prepared statement; UNNEST zips them back into rows.
        // Computing the delta on the DB side means the "now" used here is
        // the same NOW() used by `try_claim_due`'s predicate.
        //
        // The arrays bind as `Vec<&str>` rather than `Vec<String>` — sqlx
        // encodes `TEXT[]` directly from `&str` slices, so the only
        // allocation is the two `Vec`s themselves (no per-element String
        // clones).
        let groups: Vec<&str> = keys.iter().map(|(g, _)| *g).collect();
        let names: Vec<&str> = keys.iter().map(|(_, n)| *n).collect();
        // The query is fully static — `TABLE_NAME` is a `const`, so the SQL
        // string lives in `.rodata` instead of being `format!`-built on
        // every poll.
        const SQL: &str = "SELECT MIN((data->>'next_run_at_millis')::bigint) \
                    - (EXTRACT(EPOCH FROM NOW()) * 1000)::BIGINT AS millis_until \
             FROM LS_SCHEDULE \
             WHERE (data->>'group_name', data->>'name') IN ( \
                 SELECT g, n FROM UNNEST($1::TEXT[], $2::TEXT[]) AS t(g, n) \
             )";
        // Compile-time assert that the const above and `ScheduleData::TABLE_NAME`
        // can't drift apart silently.
        const _: () = assert!(matches_table_name(SQL));

        let row = query(AssertSqlSafe(SQL)).bind(&groups).bind(&names).fetch_one(pool).await?;
        Ok(row.try_get("millis_until")?)
    }
}

/// `true` iff `sql` contains the literal token `LS_SCHEDULE` (the value of
/// [`ScheduleData::TABLE_NAME`]). Used as a `const` assertion so the
/// hand-inlined table name in [`PgScheduleBackend::time_until_next_due_millis`]
/// can't silently drift from the trait's `TABLE_NAME` constant.
const fn matches_table_name(sql: &str) -> bool {
    let needle = ScheduleData::TABLE_NAME.as_bytes();
    let haystack = sql.as_bytes();
    let mut i = 0;
    while i + needle.len() <= haystack.len() {
        let mut j = 0;
        while j < needle.len() && haystack[i + j] == needle[j] {
            j += 1;
        }
        if j == needle.len() {
            return true;
        }
        i += 1;
    }
    false
}

/// Postgres-backed [`crate::ScheduleRepository`]. Wraps a [`PgScheduleBackend`]
/// in the generic [`SqlScheduleRepository`].
pub type PgScheduleRepository = SqlScheduleRepository<PgScheduleBackend>;

impl PgScheduleRepository {
    /// Applies the bundled migrations and returns a ready-to-use repository.
    /// This is the only constructor — the repository is never observable in a
    /// pre-migration state. Safe to call from every process on startup: sqlx's
    /// migration tracker table makes the bundled migrations idempotent.
    pub async fn init(c3p0: PgC3p0Pool) -> Result<Self, SchedulerError> {
        MIGRATOR.run(c3p0.pool()).await?;
        Ok(SqlScheduleRepository::new(PgScheduleBackend::new(c3p0)))
    }
}
