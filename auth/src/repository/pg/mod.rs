use crate::repository::pg::pg_auth_account::PgAuthAccountRepository;
use crate::repository::pg::pg_token::PgTokenRepository;
use crate::repository::AuthRepositoryManager;
use c3p0::pg::*;
use c3p0::*;
use include_dir::*;
use lightspeed_core::error::LightSpeedError;
use std::convert::TryInto;

pub mod pg_auth_account;
pub mod pg_token;

const MIGRATIONS: Dir = include_dir!("./src_resources/db/pg/migrations");

#[derive(Clone)]
pub struct PgAuthRepositoryManager {
    c3p0: C3p0PoolPg,
}

impl PgAuthRepositoryManager {
    pub fn new(c3p0: C3p0PoolPg) -> Self {
        Self { c3p0 }
    }
}

impl AuthRepositoryManager for PgAuthRepositoryManager {
    type CONN = PgConnection;
    type C3P0 = C3p0PoolPg;
    type AUTH_ACCOUNT_REPO = PgAuthAccountRepository;
    type TOKEN_REPO = PgTokenRepository;

    fn c3p0(&self) -> &C3p0PoolPg {
        &self.c3p0
    }

    fn start(&self) -> Result<(), LightSpeedError> {
        let migrate_table_name = format!("AUTH_{}", C3P0_MIGRATE_TABLE_DEFAULT);
        let migrations: Migrations =
            (&MIGRATIONS)
                .try_into()
                .map_err(|err| LightSpeedError::ModuleStartError {
                    message: format!("PgAuthRepositoryManager - failed to read db migrations: {}", err),
                })?;

        let migrate = C3p0MigrateBuilder::new(self.c3p0().clone())
            .with_table_name(migrate_table_name)
            .with_migrations(migrations)
            .build();

        migrate
            .migrate()
            .map_err(|err| LightSpeedError::ModuleStartError {
                message: format!("PgAuthRepositoryManager - db migration failed: {}", err),
            })
    }

    fn auth_account_repo(&self) -> Self::AUTH_ACCOUNT_REPO {
        PgAuthAccountRepository::default()
    }

    fn token_repo(&self) -> Self::TOKEN_REPO {
        PgTokenRepository::default()
    }
}
