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
    phantom_id: std::marker::PhantomData<Id>,
    c3p0: SqlxPgC3p0Pool,
}

impl <Id: IdType> PgAuthRepositoryManager<Id> {
    pub fn new(c3p0: SqlxPgC3p0Pool) -> Self {
        Self { 
            phantom_id: std::marker::PhantomData,
            c3p0
        }
    }
}

#[async_trait::async_trait]
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
        PgAuthAccountRepository::default()
    }

    fn token_repo(&self) -> Self::TokenRepo {
        PgTokenRepository::default()
    }
}
