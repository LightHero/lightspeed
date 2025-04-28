use crate::repository::AuthRepositoryManager;
use ::sqlx::{migrate::Migrator, *};
use c3p0::sqlx::*;
use lightspeed_core::error::LsError;
use sqlite_auth_account::SqliteAuthAccountRepository;
use sqlite_token::SqliteTokenRepository;

pub mod sqlite_auth_account;
pub mod sqlite_token;

static MIGRATOR: Migrator = ::sqlx::migrate!("src_resources/db/sqlx_sqlite/migrations");

#[derive(Clone)]
pub struct SqliteAuthRepositoryManager {
    c3p0: SqlxSqliteC3p0Pool,
}

impl SqliteAuthRepositoryManager {
    pub fn new(c3p0: SqlxSqliteC3p0Pool) -> SqliteAuthRepositoryManager {
        SqliteAuthRepositoryManager { c3p0 }
    }
}

impl AuthRepositoryManager for SqliteAuthRepositoryManager {
    type Tx<'a> = Transaction<'a, Sqlite>;
    type C3P0 = SqlxSqliteC3p0Pool;
    type AuthAccountRepo = SqliteAuthAccountRepository;
    type TokenRepo = SqliteTokenRepository;

    fn c3p0(&self) -> &Self::C3P0 {
        &self.c3p0
    }

    async fn start(&self) -> Result<(), LsError> {
        MIGRATOR.run(self.c3p0.pool()).await.map_err(|err| LsError::ModuleStartError {
            message: format!("SqliteAuthRepositoryManager - db migration failed: {err:?}"),
        })
    }

    fn auth_account_repo(&self) -> Self::AuthAccountRepo {
        SqliteAuthAccountRepository::new()
    }

    fn token_repo(&self) -> Self::TokenRepo {
        SqliteTokenRepository::new()
    }
}
