use crate::repository::pg::pg_content::PgContentRepository;
use crate::repository::pg::pg_project::PgProjectRepository;
use crate::repository::pg::pg_schema::PgSchemaRepository;
use crate::repository::CmsRepositoryManager;
use c3p0::pg::*;
use c3p0::{include_dir::*, *};
use lightspeed_core::error::LightSpeedError;

pub mod pg_content;
pub mod pg_project;
pub mod pg_schema;

const MIGRATIONS: Dir = include_dir!("./src_resources/db/pg/migrations");

#[derive(Clone)]
pub struct PgCmsRepositoryManager {
    c3p0: PgC3p0Pool,
}

impl PgCmsRepositoryManager {
    pub fn new(c3p0: PgC3p0Pool) -> Self {
        Self { c3p0 }
    }
}

impl CmsRepositoryManager for PgCmsRepositoryManager {
    type Conn = PgConnection;
    type C3P0 = PgC3p0Pool;
    type ContentRepo = PgContentRepository;
    type ProjectRepo = PgProjectRepository;
    type SchemaRepo = PgSchemaRepository;

    fn c3p0(&self) -> &PgC3p0Pool {
        &self.c3p0
    }

    fn start(&self) -> Result<(), LightSpeedError> {
        let migrate_table_name = format!("LS_CMS_{}", C3P0_MIGRATE_TABLE_DEFAULT);

        let migrate = C3p0MigrateBuilder::new(self.c3p0().clone())
            .with_table_name(migrate_table_name)
            .with_migrations(from_embed(&MIGRATIONS).map_err(|err| {
                LightSpeedError::ModuleStartError {
                    message: format!(
                        "PgCmsRepositoryManager - failed to read db migrations: {}",
                        err
                    ),
                }
            })?)
            .build();

        migrate
            .migrate()
            .map_err(|err| LightSpeedError::ModuleStartError {
                message: format!("PgCmsRepositoryManager - db migration failed: {}", err),
            })
    }

    fn content_repo(&self, table_name: &str) -> Self::ContentRepo {
        PgContentRepository::new(table_name)
    }

    fn project_repo(&self) -> Self::ProjectRepo {
        PgProjectRepository::default()
    }

    fn schema_repo(&self) -> Self::SchemaRepo {
        PgSchemaRepository::default()
    }
}
