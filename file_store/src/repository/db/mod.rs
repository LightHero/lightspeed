use crate::model::{
    BinaryContent, FileStoreDataData, FileStoreDataModel, Repository, RepositoryFile,
};
use c3p0::*;
use lightspeed_core::error::LightSpeedError;

pub mod pg;

#[async_trait::async_trait]
pub trait DBFileStoreRepositoryManager: Clone + Send + Sync {
    type Conn: SqlConnection;
    type C3P0: C3p0Pool<Conn = Self::Conn>;
    type FileStoreBinaryRepo: DBFileStoreBinaryRepository<Conn = Self::Conn>;
    type FileStoreDataRepo: FileStoreDataRepository<Conn = Self::Conn>;

    fn c3p0(&self) -> &Self::C3P0;
    async fn start(&self) -> Result<(), LightSpeedError>;

    fn file_store_binary_repo(&self) -> Self::FileStoreBinaryRepo;
    fn file_store_data_repo(&self) -> Self::FileStoreDataRepo;
}

#[async_trait::async_trait]
pub trait DBFileStoreBinaryRepository: Clone + Send + Sync {
    type Conn: SqlConnection;

    async fn read_file(
        &self,
        conn: &mut Self::Conn,
        repository_name: &str,
        file_path: &str,
    ) -> Result<BinaryContent, LightSpeedError>;

    async fn save_file(
        &self,
        conn: &mut Self::Conn,
        repository_name: &str,
        file_path: &str,
        content: &BinaryContent,
    ) -> Result<u64, LightSpeedError>;

    async fn delete_file(
        &self,
        conn: &mut Self::Conn,
        repository_name: &str,
        file_path: &str,
    ) -> Result<u64, LightSpeedError>;
}

#[async_trait::async_trait]
pub trait FileStoreDataRepository: Clone + Send + Sync {
    type Conn: SqlConnection;

    async fn fetch_one_by_id(
        &self,
        conn: &mut Self::Conn,
        id: IdType,
    ) -> Result<FileStoreDataModel, LightSpeedError>;

    async fn fetch_one_by_repository(
        &self,
        conn: &mut Self::Conn,
        repository: &RepositoryFile,
    ) -> Result<FileStoreDataModel, LightSpeedError>;

    async fn fetch_all_by_repository(
        &self,
        conn: &mut Self::Conn,
        repository: &Repository,
        offset: usize,
        max: usize,
        sort: &OrderBy,
    ) -> Result<Vec<FileStoreDataModel>, LightSpeedError>;

    async fn save(
        &self,
        conn: &mut Self::Conn,
        model: NewModel<FileStoreDataData>,
    ) -> Result<FileStoreDataModel, LightSpeedError>;

    async fn delete_by_id(&self, conn: &mut Self::Conn, id: IdType)
        -> Result<u64, LightSpeedError>;
}
