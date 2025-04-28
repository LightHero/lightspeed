use crate::repository::db::DBFileStoreRepositoryManager;
use crate::repository::db::sqlx_sqlite::sqlite_file_store_binary::SqliteFileStoreBinaryRepository;
use crate::repository::db::sqlx_sqlite::sqlite_file_store_data::SqliteFileStoreDataRepository;
use c3p0::sqlx::sqlx::{migrate::Migrator, *};
use c3p0::sqlx::*;
use lightspeed_core::error::LsError;

pub mod sqlite_file_store_binary;
pub mod sqlite_file_store_data;

static MIGRATOR: Migrator = migrate!("src_resources/db/sqlx_sqlite/migrations");

#[derive(Clone)]
pub struct SqliteFileStoreRepositoryManager {
    c3p0: SqlxSqliteC3p0Pool,
}

impl SqliteFileStoreRepositoryManager {
    pub fn new(c3p0: SqlxSqliteC3p0Pool) -> Self {
        Self { c3p0 }
    }
}

impl DBFileStoreRepositoryManager for SqliteFileStoreRepositoryManager {
    type Tx<'a> = Transaction<'a, Sqlite>;
    type C3P0 = SqlxSqliteC3p0Pool;
    type FileStoreBinaryRepo = SqliteFileStoreBinaryRepository;
    type FileStoreDataRepo = SqliteFileStoreDataRepository;

    fn c3p0(&self) -> &Self::C3P0 {
        &self.c3p0
    }

    async fn start(&self) -> Result<(), LsError> {
        MIGRATOR.run(self.c3p0.pool()).await.map_err(|err| LsError::ModuleStartError {
            message: format!("SqliteFileStoreRepositoryManager - db migration failed: {err:?}"),
        })
    }

    fn file_store_binary_repo(&self) -> Self::FileStoreBinaryRepo {
        SqliteFileStoreBinaryRepository::default()
    }

    fn file_store_data_repo(&self) -> Self::FileStoreDataRepo {
        SqliteFileStoreDataRepository::default()
    }
}
