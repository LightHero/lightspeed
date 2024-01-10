use std::sync::Arc;

use crate::repository::pg::pg_auth_account::PgAuthAccountRepository;
use crate::repository::pg::pg_token::PgTokenRepository;
use crate::repository::AuthRepositoryManager;
use ::sqlx::{migrate::Migrator, *};
use c3p0::{sqlx::*, IdType};
use lightspeed_core::error::LsError;

pub mod pg_auth_account;
pub mod pg_token;

static MIGRATOR: Migrator = ::sqlx::migrate!("src_resources/db/pg/migrations");

#[derive(Clone)]
pub struct PgAuthRepositoryManager<Id: IdType> {
    id_generator: Arc<dyn PostgresIdGenerator<Id>>,
    c3p0: SqlxPgC3p0Pool,
}

impl <Id: IdType> PgAuthRepositoryManager<Id> {
    pub fn new(c3p0: SqlxPgC3p0Pool) -> PgAuthRepositoryManager<u64> {
        PgAuthRepositoryManager { 
            id_generator: Arc::new(PostgresAutogeneratedIdGenerator{}),
            c3p0
        }
    }

    pub fn new_with_id_generator(c3p0: SqlxPgC3p0Pool, id_generator: Arc<dyn PostgresIdGenerator<Id>>) -> Self {
        Self { 
            id_generator,
            c3p0
        }
    }
}

impl <Id: IdType> AuthRepositoryManager<Id> for PgAuthRepositoryManager<Id> {
    type Tx = PgTx;
    type C3P0 = SqlxPgC3p0Pool;
    type AuthAccountRepo = PgAuthAccountRepository<Id>;
    type TokenRepo = PgTokenRepository<Id>;

    fn c3p0(&self) -> &Self::C3P0 {
        &self.c3p0
    }

    async fn start(&self) -> Result<(), LsError> {
        MIGRATOR.run(self.c3p0.pool()).await.map_err(|err| LsError::ModuleStartError {
            message: format!("PgAuthRepositoryManager - db migration failed: {err:?}"),
        })
    }

    fn auth_account_repo(&self) -> Self::AuthAccountRepo {
        PgAuthAccountRepository::new(self.id_generator.clone())
    }

    fn token_repo(&self) -> Self::TokenRepo {
        PgTokenRepository::new(self.id_generator.clone())
    }
}
