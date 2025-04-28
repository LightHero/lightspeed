use crate::repository::postgres::pg_auth_account::PostgresAuthAccountRepository;
use crate::repository::postgres::pg_token::PostgresTokenRepository;
use c3p0::postgres::*;
use c3p0::*;
use lightspeed_core::error::LsError;

use super::AuthRepositoryManager;

pub mod pg_auth_account;
pub mod pg_token;

const MIGRATIONS: include_dir::Dir =
    include_dir::include_dir!("$CARGO_MANIFEST_DIR/src_resources/db/postgres/migrations");

#[derive(Clone)]
pub struct PostgresAuthRepositoryManager {
    c3p0: PgC3p0Pool,
}

impl PostgresAuthRepositoryManager {
    pub fn new(c3p0: PgC3p0Pool) -> Self {
        Self { c3p0 }
    }
}

impl AuthRepositoryManager for PostgresAuthRepositoryManager {
    type Tx<'a> = PgTx<'a>;
    type C3P0 = PgC3p0Pool;
    type AuthAccountRepo = PostgresAuthAccountRepository;
    type TokenRepo = PostgresTokenRepository;

    fn c3p0(&self) -> &PgC3p0Pool {
        &self.c3p0
    }

    async fn start(&self) -> Result<(), LsError> {
        let migrate_table_name = format!("LS_AUTH_{}", C3P0_MIGRATE_TABLE_DEFAULT);

        let migrate = C3p0MigrateBuilder::new(self.c3p0().clone())
            .with_table_name(migrate_table_name)
            .with_migrations(from_embed(&MIGRATIONS).map_err(|err| LsError::ModuleStartError {
                message: format!("PostgresAuthRepositoryManager - failed to read db migrations: {:?}", err),
            })?)
            .build();

        migrate.migrate().await.map_err(|err| LsError::ModuleStartError {
            message: format!("PostgresAuthRepositoryManager - db migration failed: {:?}", err),
        })?;

        Ok(())
    }

    fn auth_account_repo(&self) -> PostgresAuthAccountRepository {
        PostgresAuthAccountRepository::default()
    }

    fn token_repo(&self) -> PostgresTokenRepository {
        PostgresTokenRepository::default()
    }
}
