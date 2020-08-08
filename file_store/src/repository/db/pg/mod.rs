use crate::repository::db::pg::pg_file_store_binary::PgFileStoreBinaryRepository;
use crate::repository::db::DBFileStoreRepositoryManager;
use c3p0::postgres::*;
use c3p0::*;
use lightspeed_core::error::LightSpeedError;
use crate::repository::db::pg::pg_file_store_data::PgFileStoreDataRepository;

pub mod pg_file_store_binary;
pub mod pg_file_store_data;

const MIGRATIONS: include_dir::Dir = include_dir::include_dir!("./src_resources/db/pg/migrations");

#[derive(Clone)]
pub struct PgFileStoreRepositoryManager {
    c3p0: PgC3p0Pool,
}

impl PgFileStoreRepositoryManager {
    pub fn new(c3p0: PgC3p0Pool) -> Self {
        Self {
            c3p0,
        }
    }
}

#[async_trait::async_trait]
impl DBFileStoreRepositoryManager for PgFileStoreRepositoryManager {
    type Conn = PgConnection;
    type C3P0 = PgC3p0Pool;
    type FileStoreBinaryRepo = PgFileStoreBinaryRepository;
    type FileStoreDataRepo = PgFileStoreDataRepository;

    fn c3p0(&self) -> &PgC3p0Pool {
        &self.c3p0
    }

    async fn start(&self) -> Result<(), LightSpeedError> {
        let migrate_table_name = format!("LS_FILE_STORE_{}", C3P0_MIGRATE_TABLE_DEFAULT);

        let migrate = C3p0MigrateBuilder::new(self.c3p0().clone())
            .with_table_name(migrate_table_name)
            .with_migrations(from_embed(&MIGRATIONS).map_err(|err| {
                LightSpeedError::ModuleStartError {
                    message: format!(
                        "PgFileStoreRepositoryManager - failed to read db migrations: {}",
                        err
                    ),
                }
            })?)
            .build();

        migrate
            .migrate()
            .await
            .map_err(|err| LightSpeedError::ModuleStartError {
                message: format!(
                    "PgFileStoreRepositoryManager - db migration failed: {}",
                    err
                ),
            })
    }

    fn file_store_binary_repo(&self) -> Self::FileStoreBinaryRepo {
        PgFileStoreBinaryRepository::default()
    }

    fn file_store_data_repo(&self) -> Self::FileStoreDataRepo {
        PgFileStoreDataRepository::default()
    }
}
