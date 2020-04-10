use c3p0::*;
use lightspeed_core::error::LightSpeedError;

pub mod pg;

#[async_trait::async_trait(?Send)]
pub trait FileStoreRepositoryManager: Clone + Send + Sync {
    type Conn: SqlConnectionAsync;
    type C3P0: C3p0PoolAsync<CONN = Self::Conn>;
    type FileStoreRepo: FileStoreRepository<Conn = Self::Conn>;

    fn c3p0(&self) -> &Self::C3P0;
    async fn start(&self) -> Result<(), LightSpeedError>;

    fn file_store_repo(&self) -> Self::FileStoreRepo;
}

#[async_trait::async_trait]
pub trait FileStoreRepository: Clone + Send + Sync {
    type Conn: SqlConnectionAsync;

}
