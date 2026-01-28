use crate::repository::OutboxRepositoryManager;
use c3p0::{
    SqliteC3p0Pool,
    sqlx::{migrate::Migrator, *},
};
use lightspeed_core::error::LsError;
use sqlite_task::SqliteTaskRepository;

pub mod sqlite_task;

static MIGRATOR: Migrator = ::sqlx::migrate!("src_resources/db/sqlite/migrations");

#[derive(Clone)]
pub struct SqliteOutboxRepositoryManager {
    c3p0: SqliteC3p0Pool,
}

impl SqliteOutboxRepositoryManager {
    pub fn new(c3p0: SqliteC3p0Pool) -> SqliteOutboxRepositoryManager {
        SqliteOutboxRepositoryManager { c3p0 }
    }
}

impl OutboxRepositoryManager for SqliteOutboxRepositoryManager {
    type DB = Sqlite;
    type C3P0 = SqliteC3p0Pool;
    type TaskRepo = SqliteTaskRepository;

    fn c3p0(&self) -> &Self::C3P0 {
        &self.c3p0
    }

    async fn start(&self) -> Result<(), LsError> {
        MIGRATOR.run(self.c3p0.pool()).await.map_err(|err| LsError::ModuleStartError {
            message: format!("SqliteAuthRepositoryManager - db migration failed: {err:?}"),
        })
    }

    fn task_repo(&self) -> Self::TaskRepo {
        SqliteTaskRepository::new()
    }
}
