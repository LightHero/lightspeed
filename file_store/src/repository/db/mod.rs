use crate::dto::FileData;
use c3p0::*;
use lightspeed_core::error::LightSpeedError;

pub mod pg;

#[async_trait::async_trait]
pub trait DBFileStoreRepositoryManager: Clone + Send + Sync {
    type Conn: SqlConnection;
    type C3P0: C3p0Pool<Conn = Self::Conn>;
    type FileStoreBinaryRepo: DBFileStoreBinaryRepository<Conn = Self::Conn>;

    fn c3p0(&self) -> &Self::C3P0;
    async fn start(&self) -> Result<(), LightSpeedError>;

    fn file_store_binary_repo(&self) -> &Self::FileStoreBinaryRepo;
}

#[async_trait::async_trait]
pub trait DBFileStoreBinaryRepository: Clone + Send + Sync {
    type Conn: SqlConnection;

    async fn read_file(
        &self,
        conn: &mut Self::Conn,
        file_name: &str,
    ) -> Result<FileData, LightSpeedError>;

    async fn save_file(
        &self,
        conn: &mut Self::Conn,
        source_path: &str,
        file_name: &str,
    ) -> Result<(), LightSpeedError>;

    async fn delete_by_filename(
        &self,
        conn: &mut Self::Conn,
        file_name: &str,
    ) -> Result<u64, LightSpeedError>;
}
