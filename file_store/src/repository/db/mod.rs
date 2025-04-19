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

    async fn read_file(
        &self,
        tx: &mut Self::Tx<'_>,
        repository_name: &str,
        file_path: &str,
    ) -> Result<BinaryContent<'_>, LsError>;

    async fn save_file<'a>(
        &self,
        tx: &mut Self::Tx<'_>,
        repository_name: &str,
        file_path: &str,
        content: &'a BinaryContent<'a>,
    ) -> Result<u64, LsError>;

    async fn delete_file(&self, tx: &mut Self::Tx<'_>, repository_name: &str, file_path: &str) -> Result<u64, LsError>;
}

pub trait FileStoreDataRepository: Clone + Send + Sync {
    type Tx<'a>: Send + Sync;

    async fn exists_by_repository(&self, tx: &mut Self::Tx<'_>, repository: &RepositoryFile) -> Result<bool, LsError>;

    async fn fetch_one_by_id(&self, tx: &mut Self::Tx<'_>, id: u64) -> Result<FileStoreDataModel, LsError>;

    async fn fetch_one_by_repository(
        &self,
        tx: &mut Self::Tx<'_>,
        repository: &RepositoryFile,
    ) -> Result<FileStoreDataModel, LsError>;

    async fn fetch_all_by_repository(
        &self,
        tx: &mut Self::Tx<'_>,
        repository: &Repository,
        offset: usize,
        max: usize,
        sort: &OrderBy,
    ) -> Result<Vec<FileStoreDataModel>, LsError>;

    async fn save(&self, tx: &mut Self::Tx<'_>, model: NewModel<FileStoreDataData>) -> Result<FileStoreDataModel, LsError>;

    async fn delete_by_id(&self, tx: &mut Self::Tx<'_>, id: u64) -> Result<u64, LsError>;
}
