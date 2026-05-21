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
use crate::repository::sql::{
    ScheduleData, ScheduleModel, ScheduleSqlBackend, SqlScheduleRepository,
};

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

    async fn fetch_record(
        tx: &mut PgConnection,
        group: &str,
        name: &str,
    ) -> Result<Option<ScheduleModel>, C3p0Error> {
        Ok(ScheduleModel::query_with_tail(
            "WHERE data->>'group_name' = $1 AND data->>'name' = $2 LIMIT 1",
        )
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

    async fn time_until_next_due_millis(
        pool: &c3p0::sqlx::Pool<Postgres>,
        keys: &[(&str, &str)],
    ) -> Result<Option<i64>, C3p0Error> {
        // Two parallel arrays let us bind any number of (group, name) pairs
        // through one prepared statement; UNNEST zips them back into rows.
        // Computing the delta on the DB side means the "now" used here is
        // the same NOW() used by `try_claim_due`'s predicate.
        let groups: Vec<String> = keys.iter().map(|(g, _)| (*g).to_string()).collect();
        let names: Vec<String> = keys.iter().map(|(_, n)| (*n).to_string()).collect();
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
            .fetch_one(pool)
            .await?;
        Ok(row.try_get("millis_until")?)
    }
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
