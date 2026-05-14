use crate::error::LsFileStoreError;
use crate::model::{BinaryContent, FileStoreDataData, FileStoreDataModel};
use c3p0::sqlx::Database;
use c3p0::{sql::OrderBy, *};
use lightspeed_core::error::LsError;

#[cfg(feature = "mysql")]
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
    ) -> impl Future<Output = Result<BinaryContent<'_>, LsFileStoreError>> + Send;

    /// Read the file's bytes through a long-lived connection owned by the
    /// returned `BinaryContent`. Implementations that support true backend
    /// streaming (Postgres Large Objects) pipe bytes through this without
    /// materializing the full payload; backends that don't (MySQL, SQLite)
    /// fall back to a buffered read inside their own transaction.
    fn read_file_streamed(
        &self,
        repository_name: &str,
        file_path: &str,
    ) -> impl Future<Output = Result<BinaryContent<'static>, LsFileStoreError>> + Send;

    fn save_file<'a>(
        &self,
        tx: &mut <Self::DB as Database>::Connection,
        repository_name: &str,
        file_path: &str,
        content: &'a BinaryContent<'a>,
    ) -> impl Future<Output = Result<u64, LsFileStoreError>> + Send;

    fn delete_file(
        &self,
        tx: &mut <Self::DB as Database>::Connection,
        repository_name: &str,
        file_path: &str,
    ) -> impl Future<Output = Result<u64, LsFileStoreError>> + Send;
}

pub trait FileStoreDataRepository: Clone + Send + Sync {
    type DB: Database;

    fn exists_by_repository(
        &self,
        tx: &mut <Self::DB as Database>::Connection,
        repository: &str,
        file_path: &str,
    ) -> impl Future<Output = Result<bool, LsFileStoreError>> + Send;

    fn fetch_one_by_id(
        &self,
        tx: &mut <Self::DB as Database>::Connection,
        id: i64,
    ) -> impl Future<Output = Result<FileStoreDataModel, LsFileStoreError>> + Send;

    fn fetch_one_by_repository(
        &self,
        tx: &mut <Self::DB as Database>::Connection,
        repository: &str,
        file_path: &str,
    ) -> impl Future<Output = Result<FileStoreDataModel, LsFileStoreError>> + Send;

    fn fetch_all_by_repository(
        &self,
        tx: &mut <Self::DB as Database>::Connection,
        repository: &str,
        offset: usize,
        max: usize,
        sort: OrderBy,
    ) -> impl Future<Output = Result<Vec<FileStoreDataModel>, LsFileStoreError>> + Send;

    fn save(
        &self,
        tx: &mut <Self::DB as Database>::Connection,
        model: NewRecord<FileStoreDataData>,
    ) -> impl Future<Output = Result<FileStoreDataModel, LsFileStoreError>> + Send;

    fn delete_by_id(
        &self,
        tx: &mut <Self::DB as Database>::Connection,
        id: i64,
    ) -> impl Future<Output = Result<u64, LsFileStoreError>> + Send;
}
