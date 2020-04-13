use crate::repository::filesystem::fs_file_store_binary::FsFileStoreBinaryRepository;
use crate::repository::db::{DBFileStoreBinaryRepository};
use c3p0::*;
use lightspeed_core::error::LightSpeedError;

pub mod filesystem;
pub mod db;

#[derive(Clone)]
pub enum FileStoreRepoManager<
DBFileStoreRepositoryManager: crate::repository::db::DBFileStoreRepositoryManager
> {
    DB(DBFileStoreRepositoryManager),
    FS(FsFileStoreBinaryRepository)
}

impl <DBFileStoreRepositoryManager: crate::repository::db::DBFileStoreRepositoryManager> FileStoreRepoManager<DBFileStoreRepositoryManager> {

    pub async fn start(&self) -> Result<(), LightSpeedError> {
        match self {
            FileStoreRepoManager::DB(repo_manager) => repo_manager.start().await,
            FileStoreRepoManager::FS(..) => Ok(())
        }
    }

}
