use crate::repository::CmsRepositoryManager;
use crate::repository::postgres::pg_content::PostgresContentRepository;
use crate::repository::postgres::pg_project::PostgresProjectRepository;
use crate::repository::postgres::pg_schema::PostgresSchemaRepository;
use ::sqlx::Postgres;
use c3p0::postgres::*;
use c3p0::sqlx::{migrate::Migrator, *};
use lightspeed_core::error::LsError;

pub mod pg_content;
pub mod pg_project;
pub mod pg_schema;

static MIGRATOR: Migrator = c3p0::sqlx::migrate!("src_resources/db/postgres/migrations");

#[derive(Clone)]
pub struct PostgresCmsRepositoryManager {
    c3p0: PgC3p0Pool,
}

impl PostgresCmsRepositoryManager {
    pub fn new(c3p0: PgC3p0Pool) -> Self {
        Self { c3p0 }
    }
}

impl CmsRepositoryManager for PostgresCmsRepositoryManager {
    type DB = Postgres;
    type C3P0 = PgC3p0Pool;
    type ContentRepo = PostgresContentRepository;
    type ProjectRepo = PostgresProjectRepository;
    type SchemaRepo = PostgresSchemaRepository;

    fn c3p0(&self) -> &Self::C3P0 {
        &self.c3p0
    }

    async fn start(&self) -> Result<(), LsError> {
        MIGRATOR.run(self.c3p0.pool()).await.map_err(|err| LsError::ModuleStartError {
            message: format!("PostgresCmsRepositoryManager - db migration failed: {err:?}"),
        })
    }

    fn content_repo(&self) -> Self::ContentRepo {
        PostgresContentRepository::new()
    }

    fn project_repo(&self) -> Self::ProjectRepo {
        PostgresProjectRepository::default()
    }

    fn schema_repo(&self) -> Self::SchemaRepo {
        PostgresSchemaRepository::default()
    }
}
