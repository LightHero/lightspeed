use crate::repository::AMRepositoryManager;
use c3p0::{
    SqliteC3p0Pool,
    sqlx::{migrate::Migrator, *},
};
use lightspeed_core::error::LsError;
use sqlite_account::SqliteAccountRepository;
use sqlite_token::SqliteTokenRepository;

pub mod sqlite_account;
pub mod sqlite_token;

static MIGRATOR: Migrator = ::sqlx::migrate!("src_resources/db/sqlite/migrations");

#[derive(Clone)]
pub struct SqliteAMRepositoryManager {
    c3p0: SqliteC3p0Pool,
}

impl SqliteAMRepositoryManager {
    pub fn new(c3p0: SqliteC3p0Pool) -> SqliteAMRepositoryManager {
        SqliteAMRepositoryManager { c3p0 }
    }
}

impl AMRepositoryManager for SqliteAMRepositoryManager {
    type DB = Sqlite;
    type C3P0 = SqliteC3p0Pool;
    type AccountRepo = SqliteAccountRepository;
    type TokenRepo = SqliteTokenRepository;

    fn c3p0(&self) -> &Self::C3P0 {
        &self.c3p0
    }

    async fn start(&self) -> Result<(), LsError> {
        MIGRATOR.run(self.c3p0.pool()).await.map_err(|err| LsError::ModuleStartError {
            message: format!("SqliteAuthRepositoryManager - db migration failed: {err:?}"),
        })
    }

    fn account_repo(&self) -> Self::AccountRepo {
        SqliteAccountRepository::new()
    }

    fn token_repo(&self) -> Self::TokenRepo {
        SqliteTokenRepository::new()
    }
}
