use crate::repository::filesystem::fs_file_store_binary::FsFileStoreBinaryRepository;
use lightspeed_core::error::LightSpeedError;
use std::str::FromStr;
use crate::config::FileStoreConfig;
use log::*;

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

    pub fn new(config: FileStoreConfig, db_repo_manager: Option<DBFileStoreRepositoryManager>) -> Result<Self, LightSpeedError> {
        match config.file_store_type {
            FileStoreType::DB => {
                info!("FileStoreRepoManager - Build DB FileStoreRepoManager");
                let repo_manager = db_repo_manager.ok_or_else(|| LightSpeedError::ConfigurationError {
                    message: "FileStoreRepoManager - A DBFileStoreRepositoryManager should be provided if FileStoreType is DB".to_owned()
                })?;
                Ok(FileStoreRepoManager::DB(repo_manager))
            },
            FileStoreType::FS => {
                let base_folder = config.file_store_fs_base_folder.ok_or_else(|| LightSpeedError::ConfigurationError {
                    message: "FileStoreRepoManager - A base_folder should be provided if FileStoreType is FS".to_owned()
                })?;
                info!("FileStoreRepoManager - Build FileSystem FileStoreRepoManager with base_folder [{}]", base_folder);
                Ok(FileStoreRepoManager::FS(FsFileStoreBinaryRepository::new(base_folder)))
            }
        }
    }

    pub async fn start(&self) -> Result<(), LightSpeedError> {
        match self {
            FileStoreRepoManager::DB(repo_manager) => repo_manager.start().await,
            FileStoreRepoManager::FS(..) => Ok(())
        }
    }

}


#[derive(Debug, PartialEq, Copy, Clone)]
pub enum FileStoreType {
    DB,
    FS,
}

impl FromStr for FileStoreType {
    type Err = LightSpeedError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "db" => Ok(FileStoreType::DB),
            "fs" => Ok(FileStoreType::FS),
            _ => Err(LightSpeedError::ConfigurationError {
                message: format!("Unknown FileStoreType [{}]", s),
            }),
        }
    }
}
