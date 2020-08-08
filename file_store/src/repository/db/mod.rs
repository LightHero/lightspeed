use c3p0::*;
use lightspeed_core::error::LightSpeedError;
use crate::model::{BinaryContent, FileStoreDataModel, FileStoreDataData};

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
        id: IdType,
    ) -> Result<BinaryContent, LightSpeedError>;

    async fn save_file(
        &self,
        conn: &mut Self::Conn,
        content: &BinaryContent,
    ) -> Result<IdType, LightSpeedError>;

    async fn delete_file(
        &self,
        conn: &mut Self::Conn,
        id: IdType,
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

    async fn save(
        &self,
        conn: &mut Self::Conn,
        model: NewModel<FileStoreDataData>,
    ) -> Result<FileStoreDataModel, LightSpeedError>;

    async fn delete_by_id(
        &self,
        conn: &mut Self::Conn,
        id: IdType,
    ) -> Result<u64, LightSpeedError>;

}
