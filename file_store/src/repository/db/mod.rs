use crate::model::{BinaryContent, FileStoreDataData, FileStoreDataModel};
use c3p0::{sql::OrderBy, *};
use lightspeed_core::error::LsError;
use ::sqlx::Database;

#[cfg(feature = "mysql_unsupported")]
pub mod mysql;

#[cfg(feature = "postgres")]
pub mod postgres;

#[cfg(feature = "sqlite")]
pub mod sqlite;

pub trait DBFileStoreRepositoryManager: Clone + Send + Sync {
    type DB: Database;
    type C3P0: C3p0Pool<DB = Self::DB>;
    type FileStoreBinaryRepo: DBFileStoreBinaryRepository<DB = Self::DB>;
    type FileStoreDataRepo: FileStoreDataRepository<DB = Self::DB>;

    fn c3p0(&self) -> &Self::C3P0;
    fn start(&self) -> impl Future<Output = Result<(), LsError>> + Send;

    fn file_store_binary_repo(&self) -> Self::FileStoreBinaryRepo;
    fn file_store_data_repo(&self) -> Self::FileStoreDataRepo;
}

pub trait DBFileStoreBinaryRepository: Clone + Send + Sync {
    type DB: Database;

    fn read_file(
        &self,
        tx: &mut <Self::DB as Database>::Connection,
        repository_name: &str,
        file_path: &str,
    ) -> impl Future<Output = Result<BinaryContent<'_>, LsError>> + Send;

    fn save_file<'a>(
        &self,
        tx: &mut <Self::DB as Database>::Connection,
        repository_name: &str,
        file_path: &str,
        content: &'a BinaryContent<'a>,
    ) -> impl Future<Output = Result<u64, LsError>> + Send;

    fn delete_file(
        &self,
        tx: &mut <Self::DB as Database>::Connection,
        repository_name: &str,
        file_path: &str,
    ) -> impl Future<Output = Result<u64, LsError>> + Send;
}

pub trait FileStoreDataRepository: Clone + Send + Sync {
    type DB: Database;

    fn exists_by_repository(
        &self,
        tx: &mut <Self::DB as Database>::Connection,
        repository: &str,
        file_path: &str,
    ) -> impl Future<Output = Result<bool, LsError>> + Send;

    fn fetch_one_by_id(
        &self,
        tx: &mut <Self::DB as Database>::Connection,
        id: u64,
    ) -> impl Future<Output = Result<FileStoreDataModel, LsError>> + Send;

    fn fetch_one_by_repository(
        &self,
        tx: &mut <Self::DB as Database>::Connection,
        repository: &str,
        file_path: &str,
    ) -> impl Future<Output = Result<FileStoreDataModel, LsError>> + Send;

    fn fetch_all_by_repository(
        &self,
        tx: &mut <Self::DB as Database>::Connection,
        repository: &str,
        offset: usize,
        max: usize,
        sort: OrderBy,
    ) -> impl Future<Output = Result<Vec<FileStoreDataModel>, LsError>> + Send;

    fn save(
        &self,
        tx: &mut <Self::DB as Database>::Connection,
        model: NewRecord<FileStoreDataData>,
    ) -> impl Future<Output = Result<FileStoreDataModel, LsError>> + Send;

    fn delete_by_id(&self, tx: &mut <Self::DB as Database>::Connection, id: u64) -> impl Future<Output = Result<u64, LsError>> + Send;
}
