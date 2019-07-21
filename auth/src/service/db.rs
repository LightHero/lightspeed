use c3p0::*;
use include_dir::*;
use ls_core::error::LightSpeedError;
use std::convert::TryInto;

const MIGRATIONS: Dir = include_dir!("./src_resources/db/migrations");

#[derive(Clone)]
pub struct AuthDbService {
    c3p0: C3p0Builder
}

impl AuthDbService {

    pub fn new(c3p0: C3p0Builder) -> Self {
        AuthDbService {
            c3p0
        }
    }

    pub fn start(&self) -> Result<(), LightSpeedError> {

        let migrate_table_name = format!("AUTH_{}", migrate::C3P0_MIGRATE_TABLE_DEFAULT);
        let migrations: migrate::Migrations = (&MIGRATIONS).try_into().map_err(|err|
            LightSpeedError::ModuleStartError {
                message: format!("AuthDbService failed to start: {}", err)
            }
        )?;

        let migrate = self.c3p0.migrate()
            .with_table_name(migrate_table_name)
            .with_migrations(migrations)
            .build();

        migrate.migrate().map_err(|err|
            LightSpeedError::ModuleStartError {
                message: format!("AuthDbService failed to start: {}", err)
            }
        )

    }

}