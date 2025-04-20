use crate::repository::db::DBFileStoreRepositoryManager;
use crate::repository::db::pg::pg_file_store_binary::PgFileStoreBinaryRepository;
use crate::repository::db::pg::pg_file_store_data::PgFileStoreDataRepository;
use ::sqlx::{migrate::Migrator, *};
use c3p0::sqlx::*;
use lightspeed_core::error::LsError;

pub mod pg_file_store_binary;
pub mod pg_file_store_data;

static MIGRATOR: Migrator = migrate!("src_resources/db/pg/migrations");

#[derive(Clone)]
pub struct PgFileStoreRepositoryManager {
    c3p0: SqlxPgC3p0Pool,
}

impl PgFileStoreRepositoryManager {
    pub fn new(c3p0: SqlxPgC3p0Pool) -> Self {
        Self { c3p0 }
    }
}

impl DBFileStoreRepositoryManager for PgFileStoreRepositoryManager {
    type Tx<'a> = Transaction<'a, Postgres>;
    type C3P0 = SqlxPgC3p0Pool;
    type FileStoreBinaryRepo = PgFileStoreBinaryRepository;
    type FileStoreDataRepo = PgFileStoreDataRepository;

    fn c3p0(&self) -> &Self::C3P0 {
        &self.c3p0
    }

    async fn start(&self) -> Result<(), LsError> {
        MIGRATOR.run(self.c3p0.pool()).await.map_err(|err| LsError::ModuleStartError {
            message: format!("PgFileStoreRepositoryManager - db migration failed: {err:?}"),
        })
    }

    fn file_store_binary_repo(&self) -> Self::FileStoreBinaryRepo {
        PgFileStoreBinaryRepository::default()
    }

    fn file_store_data_repo(&self) -> Self::FileStoreDataRepo {
        PgFileStoreDataRepository::default()
    }
}
