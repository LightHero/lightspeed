use crate::repository::AuthRepositoryManager;
use ::sqlx::{migrate::Migrator, *};
use c3p0::sqlx::*;
use lightspeed_core::error::LsError;
use mysql_auth_account::MySqlAuthAccountRepository;
use mysql_token::MySqlTokenRepository;

pub mod mysql_auth_account;
pub mod mysql_token;

static MIGRATOR: Migrator = ::sqlx::migrate!("src_resources/db/sqlx_mysql/migrations");

#[derive(Clone)]
pub struct MySqlAuthRepositoryManager {
    c3p0: SqlxMySqlC3p0Pool,
}

impl MySqlAuthRepositoryManager {
    pub fn new(c3p0: SqlxMySqlC3p0Pool) -> MySqlAuthRepositoryManager {
        MySqlAuthRepositoryManager { c3p0 }
    }
}

impl AuthRepositoryManager for MySqlAuthRepositoryManager {
    type Tx<'a> = Transaction<'a, MySql>;
    type C3P0 = SqlxMySqlC3p0Pool;
    type AuthAccountRepo = MySqlAuthAccountRepository;
    type TokenRepo = MySqlTokenRepository;

    fn c3p0(&self) -> &Self::C3P0 {
        &self.c3p0
    }

    async fn start(&self) -> Result<(), LsError> {
        MIGRATOR.run(self.c3p0.pool()).await.map_err(|err| LsError::ModuleStartError {
            message: format!("MySqlAuthRepositoryManager - db migration failed: {err:?}"),
        })
    }

    fn auth_account_repo(&self) -> Self::AuthAccountRepo {
        MySqlAuthAccountRepository::new()
    }

    fn token_repo(&self) -> Self::TokenRepo {
        MySqlTokenRepository::new()
    }
}
