use crate::repository::OutboxRepositoryManager;
use crate::repository::postgres::pg_outbox::PgOutboxRepository;
use c3p0::sqlx::{migrate::Migrator, *};
use c3p0::*;
use lightspeed_core::error::LsError;

pub mod pg_outbox;

static MIGRATOR: Migrator = c3p0::sqlx::migrate!("src_resources/db/postgres/migrations");

#[derive(Clone)]
pub struct PgOutboxRepositoryManager {
    c3p0: PgC3p0Pool,
}

impl PgOutboxRepositoryManager {
    pub fn new(c3p0: PgC3p0Pool) -> PgOutboxRepositoryManager {
        PgOutboxRepositoryManager { c3p0 }
    }
}

impl OutboxRepositoryManager for PgOutboxRepositoryManager {
    type DB = Postgres;
    type C3P0 = PgC3p0Pool;
    type OutboxRepo = PgOutboxRepository;

    fn c3p0(&self) -> &Self::C3P0 {
        &self.c3p0
    }

    async fn start(&self) -> Result<(), LsError> {
        MIGRATOR.run(self.c3p0.pool()).await.map_err(|err| LsError::ModuleStartError {
            message: format!("PgAuthRepositoryManager - db migration failed: {err:?}"),
        })
    }

    fn outbox_repo(&self) -> Self::OutboxRepo {
        PgOutboxRepository::new()
    }
}
