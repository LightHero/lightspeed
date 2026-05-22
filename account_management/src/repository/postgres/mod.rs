use crate::repository::AMRepositoryManager;
use crate::repository::postgres::pg_account::PgAccountRepository;
use crate::repository::postgres::pg_token::PgTokenRepository;
use c3p0::sqlx::{migrate::Migrator, *};
use c3p0::*;
use lightspeed_core::error::LsError;

pub mod pg_account;
pub mod pg_token;

static MIGRATOR: Migrator = c3p0::sqlx::migrate!("src_resources/db/postgres/migrations");

#[derive(Clone)]
pub struct PgAMRepositoryManager {
    c3p0: PgC3p0Pool,
}

impl PgAMRepositoryManager {
    pub fn new(c3p0: PgC3p0Pool) -> PgAMRepositoryManager {
        PgAMRepositoryManager { c3p0 }
    }
}

impl AMRepositoryManager for PgAMRepositoryManager {
    type DB = Postgres;
    type C3P0 = PgC3p0Pool;
    type AccountRepo = PgAccountRepository;
    type TokenRepo = PgTokenRepository;

    fn c3p0(&self) -> &Self::C3P0 {
        &self.c3p0
    }

    async fn start(&self) -> Result<(), LsError> {
        MIGRATOR.run(self.c3p0.pool()).await.map_err(|err| LsError::ModuleStartError {
            message: format!("PgAuthRepositoryManager - db migration failed: {err:?}"),
        })
    }

    fn account_repo(&self) -> Self::AccountRepo {
        PgAccountRepository::new()
    }

    fn token_repo(&self) -> Self::TokenRepo {
        PgTokenRepository::new()
    }
}
