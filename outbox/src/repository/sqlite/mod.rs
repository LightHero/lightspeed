use crate::repository::OutboxRepositoryManager;
use c3p0::{
    SqliteC3p0Pool,
    sqlx::{migrate::Migrator, *},
};
use lightspeed_core::error::LsError;
use sqlite_outbox::SqliteOutboxRepository;

pub mod sqlite_outbox;

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
    type OutboxRepo = SqliteOutboxRepository;

    fn c3p0(&self) -> &Self::C3P0 {
        &self.c3p0
    }

    async fn start(&self) -> Result<(), LsError> {
        MIGRATOR.run(self.c3p0.pool()).await.map_err(|err| LsError::ModuleStartError {
            message: format!("SqliteAuthRepositoryManager - db migration failed: {err:?}"),
        })
    }

    fn outbox_repo(&self) -> Self::OutboxRepo {
        SqliteOutboxRepository::new()
    }
}
