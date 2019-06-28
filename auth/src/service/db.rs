use c3p0::prelude::*;
use include_dir::*;
use ls_core::error::LightSpeedError;
use c3p0_migrate::{C3p0MigrateBuilder, C3P0_MIGRATE_TABLE_DEFAULT};
use std::convert::TryInto;
use c3p0_migrate::migration::Migrations;

const MIGRATIONS: Dir = include_dir!("./src_resources/db/migrations");

#[derive(Clone)]
pub struct AuthDbService {
    c3p0: C3p0
}

impl AuthDbService {

    pub fn new(c3p0: C3p0) -> Self {
        AuthDbService {
            c3p0
        }
    }

    pub fn start(&self) -> Result<(), LightSpeedError> {

        let migrate_table_name = format!("AUTH_{}", C3P0_MIGRATE_TABLE_DEFAULT);
        let migrations: Migrations = (&MIGRATIONS).try_into().map_err(|err|
            LightSpeedError::ModuleStartError {
                message: format!("AuthDbService failed to start: {}", err)
            }
        )?;

        let migrate = C3p0MigrateBuilder::new()
            .with_table_name(migrate_table_name)
            .with_migrations(migrations)
            .build();

        migrate.migrate(&self.c3p0).map_err(|err|
            LightSpeedError::ModuleStartError {
                message: format!("AuthDbService failed to start: {}", err)
            }
        )

    }

}