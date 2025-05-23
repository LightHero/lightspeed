use crate::repository::pg::pg_content::PgContentRepository;
use crate::repository::pg::pg_project::PgProjectRepository;
use crate::repository::pg::pg_schema::PgSchemaRepository;
use crate::repository::CmsRepositoryManager;
use c3p0::postgres::*;
use c3p0::*;
use lightspeed_core::error::LsError;

pub mod pg_content;
pub mod pg_project;
pub mod pg_schema;

const MIGRATIONS: include_dir::Dir =
    include_dir::include_dir!("$CARGO_MANIFEST_DIR/src_resources/db/postgres/migrations");

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
    type Tx<'a> = PgTx<'a>;
    type C3P0 = PgC3p0Pool;
    type ContentRepo = PgContentRepository;
    type ProjectRepo = PgProjectRepository;
    type SchemaRepo = PgSchemaRepository;

    fn c3p0(&self) -> &Self::C3P0 {
        &self.c3p0
    }


    async fn start(&self) -> Result<(), LsError> {
        let migrate_table_name = format!("LS_CMS_{}", C3P0_MIGRATE_TABLE_DEFAULT);

        let migrate = C3p0MigrateBuilder::new(self.c3p0().clone())
            .with_table_name(migrate_table_name)
            .with_migrations(from_embed(&MIGRATIONS).map_err(|err| LsError::ModuleStartError {
                message: format!("PostgresCmsRepositoryManager - failed to read db migrations: {:?}", err),
            })?)
            .build();

        migrate.migrate().await.map_err(|err| LsError::ModuleStartError {
            message: format!("PostgresCmsRepositoryManager - db migration failed: {:?}", err),
        })?;

        Ok(())
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
