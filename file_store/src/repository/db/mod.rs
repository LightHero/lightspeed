use crate::model::{BinaryContent, FileStoreDataData, FileStoreDataModel, Repository, RepositoryFile};
use c3p0::*;
use lightspeed_core::error::LsError;

pub mod pg;

#[async_trait::async_trait]
pub trait DBFileStoreRepositoryManager: Clone + Send + Sync {
    type Tx: SqlTx;
    type C3P0: C3p0Pool<Tx = Self::Tx>;
    type FileStoreBinaryRepo: DBFileStoreBinaryRepository<Tx = Self::Tx>;
    type FileStoreDataRepo: FileStoreDataRepository<Tx = Self::Tx>;

    fn c3p0(&self) -> &Self::C3P0;
    async fn start(&self) -> Result<(), LsError>;

    fn file_store_binary_repo(&self) -> Self::FileStoreBinaryRepo;
    fn file_store_data_repo(&self) -> Self::FileStoreDataRepo;
}

#[async_trait::async_trait]
pub trait DBFileStoreBinaryRepository: Clone + Send + Sync {
    type Tx: SqlTx;

    async fn read_file<'a>(
        &self,
        tx: &mut Self::Tx,
        repository_name: &str,
        file_path: &str,
    ) -> Result<BinaryContent<'a>, LsError>;

    async fn save_file<'a>(
        &self,
        tx: &mut Self::Tx,
        repository_name: &str,
        file_path: &str,
        content: &'a BinaryContent<'a>,
    ) -> Result<u64, LsError>;

    async fn delete_file(
        &self,
        tx: &mut Self::Tx,
        repository_name: &str,
        file_path: &str,
    ) -> Result<u64, LsError>;
}

#[async_trait::async_trait]
pub trait FileStoreDataRepository: Clone + Send + Sync {
    type Tx: SqlTx;

    async fn exists_by_repository(
        &self,
        tx: &mut Self::Tx,
        repository: &RepositoryFile,
    ) -> Result<bool, LsError>;

    async fn fetch_one_by_id(&self, tx: &mut Self::Tx, id: IdType) -> Result<FileStoreDataModel, LsError>;

    async fn fetch_one_by_repository(
        &self,
        tx: &mut Self::Tx,
        repository: &RepositoryFile,
    ) -> Result<FileStoreDataModel, LsError>;

    async fn fetch_all_by_repository(
        &self,
        tx: &mut Self::Tx,
        repository: &Repository,
        offset: usize,
        max: usize,
        sort: &OrderBy,
    ) -> Result<Vec<FileStoreDataModel>, LsError>;

    async fn save(
        &self,
        tx: &mut Self::Tx,
        model: NewModel<FileStoreDataData>,
    ) -> Result<FileStoreDataModel, LsError>;

    async fn delete_by_id(&self, tx: &mut Self::Tx, id: IdType) -> Result<u64, LsError>;
}
