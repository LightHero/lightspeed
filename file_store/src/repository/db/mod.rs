use crate::model::{BinaryContent, FileStoreDataData, FileStoreDataModel, Repository, RepositoryFile};
use c3p0::*;
use lightspeed_core::error::LsError;

#[cfg(feature = "postgres")]
pub mod pg;

pub trait DBFileStoreRepositoryManager: Clone + Send + Sync {
    type Tx<'a>: Send + Sync;
    type C3P0: for<'a> C3p0Pool<Tx<'a> = Self::Tx<'a>>;
    type FileStoreBinaryRepo: for<'a> DBFileStoreBinaryRepository<Tx<'a> = Self::Tx<'a>>;
    type FileStoreDataRepo: for<'a> FileStoreDataRepository<Tx<'a> = Self::Tx<'a>>;

    fn c3p0(&self) -> &Self::C3P0;
    fn start(&self) -> impl Future<Output = Result<(), LsError>> + Send;

    fn file_store_binary_repo(&self) -> Self::FileStoreBinaryRepo;
    fn file_store_data_repo(&self) -> Self::FileStoreDataRepo;
}

pub trait DBFileStoreBinaryRepository: Clone + Send + Sync {
    type Tx<'a>: Send + Sync;

    fn read_file(
        &self,
        tx: &mut Self::Tx<'_>,
        repository_name: &str,
        file_path: &str,
    ) -> impl Future<Output = Result<BinaryContent<'_>, LsError>> + Send;

    fn save_file<'a>(
        &self,
        tx: &mut Self::Tx<'_>,
        repository_name: &str,
        file_path: &str,
        content: &'a BinaryContent<'a>,
    ) -> impl Future<Output = Result<u64, LsError>> + Send;

    fn delete_file(
        &self,
        tx: &mut Self::Tx<'_>,
        repository_name: &str,
        file_path: &str,
    ) -> impl Future<Output = Result<u64, LsError>> + Send;
}

pub trait FileStoreDataRepository: Clone + Send + Sync {
    type Tx<'a>: Send + Sync;

    fn exists_by_repository(
        &self,
        tx: &mut Self::Tx<'_>,
        repository: &RepositoryFile,
    ) -> impl Future<Output = Result<bool, LsError>> + Send;

    fn fetch_one_by_id(
        &self,
        tx: &mut Self::Tx<'_>,
        id: u64,
    ) -> impl Future<Output = Result<FileStoreDataModel, LsError>> + Send;

    fn fetch_one_by_repository(
        &self,
        tx: &mut Self::Tx<'_>,
        repository: &RepositoryFile,
    ) -> impl Future<Output = Result<FileStoreDataModel, LsError>> + Send;

    fn fetch_all_by_repository(
        &self,
        tx: &mut Self::Tx<'_>,
        repository: &Repository,
        offset: usize,
        max: usize,
        sort: &OrderBy,
    ) -> impl Future<Output = Result<Vec<FileStoreDataModel>, LsError>> + Send;

    fn save(
        &self,
        tx: &mut Self::Tx<'_>,
        model: NewModel<FileStoreDataData>,
    ) -> impl Future<Output = Result<FileStoreDataModel, LsError>> + Send;

    fn delete_by_id(&self, tx: &mut Self::Tx<'_>, id: u64) -> impl Future<Output = Result<u64, LsError>> + Send;
}
