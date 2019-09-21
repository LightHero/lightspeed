use crate::repository::CmsRepositoryManager;
use c3p0::pg::*;
use c3p0::*;
use include_dir::*;
use lightspeed_core::error::LightSpeedError;
use std::convert::TryInto;
use crate::repository::pg::pg_project::PgProjectRepository;
use crate::repository::pg::pg_schema::PgSchemaRepository;
use crate::repository::pg::pg_schema_content_mapping::PgSchemaContentMappingRepository;

pub mod pg_project;
pub mod pg_schema;
pub mod pg_schema_content_mapping;

const MIGRATIONS: Dir = include_dir!("./src_resources/db/pg/migrations");

#[derive(Clone)]
pub struct PgCmsRepositoryManager {
    c3p0: C3p0PoolPg,
}

impl PgCmsRepositoryManager {
    pub fn new(c3p0: C3p0PoolPg) -> Self {
        Self { c3p0 }
    }
}


impl CmsRepositoryManager for PgCmsRepositoryManager {
    type CONN = PgConnection;
    type C3P0 = C3p0PoolPg;
    type PROJECT_REPO = PgProjectRepository;
    type SCHEMA_REPO = PgSchemaRepository;
    type SCHEMA_CONTENT_MAPPING_REPO = PgSchemaContentMappingRepository;

    fn c3p0(&self) -> &C3p0PoolPg {
        &self.c3p0
    }

    fn start(&self) -> Result<(), LightSpeedError> {
        let migrate_table_name = format!("CMS_{}", C3P0_MIGRATE_TABLE_DEFAULT);
        let migrations: Migrations =
            (&MIGRATIONS)
                .try_into()
                .map_err(|err| LightSpeedError::ModuleStartError {
                    message: format!("CmsRepositoryManager failed to start: {}", err),
                })?;

        let migrate = C3p0MigrateBuilder::new(self.c3p0().clone())
            .with_table_name(migrate_table_name)
            .with_migrations(migrations)
            .build();

        migrate
            .migrate()
            .map_err(|err| LightSpeedError::ModuleStartError {
                message: format!("CmsRepositoryManager failed to start: {}", err),
            })
    }

    fn project_repo(&self) -> Self::PROJECT_REPO {
        PgProjectRepository::default()
    }

    fn schema_repo(&self) -> Self::SCHEMA_REPO {
        PgSchemaRepository::default()
    }

    fn schema_content_repo(&self) -> Self::SCHEMA_CONTENT_MAPPING_REPO {
        PgSchemaContentMappingRepository::default()
    }
}
