use c3p0::*;
use include_dir::*;
use lightspeed_core::error::LightSpeedError;
use std::convert::TryInto;

const MIGRATIONS: Dir = include_dir!("./src_resources/db/migrations");

#[derive(Clone)]
pub struct AuthDbRepository {
    c3p0: C3p0Pool<PgPoolManager>,
}

impl AuthDbRepository {
    pub fn new(c3p0: C3p0Pool<PgPoolManager>) -> Self {
        AuthDbRepository { c3p0 }
    }

    pub fn start(&self) -> Result<(), LightSpeedError> {
        let migrate_table_name = format!("AUTH_{}", migrate::C3P0_MIGRATE_TABLE_DEFAULT);
        let migrations: migrate::Migrations =
            (&MIGRATIONS)
                .try_into()
                .map_err(|err| LightSpeedError::ModuleStartError {
                    message: format!("AuthDbService failed to start: {}", err),
                })?;

        let migrate = migrate::C3p0MigrateBuilder::new(self.c3p0.clone())
            .with_table_name(migrate_table_name)
            .with_migrations(migrations)
            .build();

        migrate
            .migrate()
            .map_err(|err| LightSpeedError::ModuleStartError {
                message: format!("AuthDbService failed to start: {}", err),
            })
    }
}
