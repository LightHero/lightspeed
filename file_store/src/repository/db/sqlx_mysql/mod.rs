use crate::repository::db::DBFileStoreRepositoryManager;
use crate::repository::db::sqlx_mysql::mysql_file_store_binary::MySqlFileStoreBinaryRepository;
use crate::repository::db::sqlx_mysql::mysql_file_store_data::MySqlFileStoreDataRepository;
use c3p0::sqlx::sqlx::*;
use c3p0::sqlx::*;
use lightspeed_core::error::LsError;
use ::sqlx::migrate::Migrator;

pub mod mysql_file_store_binary;
pub mod mysql_file_store_data;

static MIGRATOR: Migrator = migrate!("src_resources/db/mysql/migrations");

#[derive(Clone)]
pub struct MySqlFileStoreRepositoryManager {
    c3p0: SqlxMySqlC3p0Pool,
}

impl MySqlFileStoreRepositoryManager {
    pub fn new(c3p0: SqlxMySqlC3p0Pool) -> Self {
        Self { c3p0 }
    }
}

impl DBFileStoreRepositoryManager for MySqlFileStoreRepositoryManager {
    type Tx<'a> = Transaction<'a, MySql>;
    type C3P0 = SqlxMySqlC3p0Pool;
    type FileStoreBinaryRepo = MySqlFileStoreBinaryRepository;
    type FileStoreDataRepo = MySqlFileStoreDataRepository;

    fn c3p0(&self) -> &Self::C3P0 {
        &self.c3p0
    }

    async fn start(&self) -> Result<(), LsError> {
        MIGRATOR.run(self.c3p0.pool()).await.map_err(|err| LsError::ModuleStartError {
            message: format!("MySqlFileStoreRepositoryManager - db migration failed: {err:?}"),
        })
    }

    fn file_store_binary_repo(&self) -> Self::FileStoreBinaryRepo {
        MySqlFileStoreBinaryRepository::default()
    }

    fn file_store_data_repo(&self) -> Self::FileStoreDataRepo {
        MySqlFileStoreDataRepository::default()
    }
}
