use crate::repository::pg::pg_content::PgContentRepository;
use crate::repository::pg::pg_project::PgProjectRepository;
use crate::repository::pg::pg_schema::PgSchemaRepository;
use crate::repository::CmsRepositoryManager;
use ::sqlx::{migrate::Migrator, *};
use c3p0::sqlx::*;
use lightspeed_core::error::LsError;

pub mod pg_content;
pub mod pg_project;
pub mod pg_schema;

static MIGRATOR: Migrator = migrate!("src_resources/db/pg/migrations");

#[derive(Clone)]
pub struct PgCmsRepositoryManager {
    c3p0: SqlxPgC3p0Pool,
}

impl PgCmsRepositoryManager {
    pub fn new(c3p0: SqlxPgC3p0Pool) -> Self {
        Self { c3p0 }
    }
}

#[async_trait::async_trait]
impl CmsRepositoryManager for PgCmsRepositoryManager {
    type Tx = PgTx;
    type C3P0 = SqlxPgC3p0Pool;
    type ContentRepo = PgContentRepository;
    type ProjectRepo = PgProjectRepository;
    type SchemaRepo = PgSchemaRepository;

    fn c3p0(&self) -> &Self::C3P0 {
        &self.c3p0
    }

    async fn start(&self) -> Result<(), LsError> {
        MIGRATOR.run(self.c3p0.pool()).await.map_err(|err| LsError::ModuleStartError {
            message: format!("PgCmsRepositoryManager - db migration failed: {err:?}"),
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
