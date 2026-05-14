use crate::repository::AuthRepositoryManager;
use c3p0::sqlx::{migrate::Migrator, *};
use c3p0::*;
use lightspeed_core::error::LsError;
use mysql_auth_account::MySqlAuthAccountRepository;
use mysql_token::MySqlTokenRepository;

pub mod mysql_auth_account;
pub mod mysql_token;

static MIGRATOR: Migrator = c3p0::sqlx::migrate!("src_resources/db/mysql/migrations");

#[derive(Clone)]
pub struct MySqlAuthRepositoryManager {
    c3p0: MySqlC3p0Pool,
}

impl MySqlAuthRepositoryManager {
    pub fn new(c3p0: MySqlC3p0Pool) -> MySqlAuthRepositoryManager {
        MySqlAuthRepositoryManager { c3p0 }
    }
}

impl AuthRepositoryManager for MySqlAuthRepositoryManager {
    type DB = MySql;
    type C3P0 = MySqlC3p0Pool;
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
