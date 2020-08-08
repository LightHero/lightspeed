use crate::config::FileStoreConfig;
use crate::repository::db::DBFileStoreRepositoryManager;
use crate::service::file_store::FileStoreService;
use lightspeed_core::error::LightSpeedError;
use log::*;
use std::sync::Arc;

pub mod config;
pub mod dto;
pub mod model;
pub mod repository;
pub mod service;
pub mod utils;
pub mod web;

#[derive(Clone)]
pub struct FileStoreModule<RepoManager: DBFileStoreRepositoryManager> {
    pub config: FileStoreConfig,

    pub repo_manager: RepoManager,
    pub file_store_service: Arc<service::file_store::FileStoreService<RepoManager>>,
}

impl<RepoManager: DBFileStoreRepositoryManager> FileStoreModule<RepoManager> {
    pub fn new(
        repo_manager: RepoManager,
        config: FileStoreConfig,
    ) -> Result<Self, LightSpeedError> {
        println!("Creating FileStoreModule");
        info!("Creating FileStoreModule");
        /*
               let file_store_repo_manager =
                   FileStoreRepoManager::new(config.clone(), &repo_manager)?;


        */
        let file_store_service = Arc::new(FileStoreService::new(&repo_manager, config.fs_repo_base_folders.clone()));

        Ok(FileStoreModule {
            config,
            repo_manager,
            file_store_service,
        })
    }
}

#[async_trait::async_trait]
impl<RepoManager: DBFileStoreRepositoryManager> lightspeed_core::module::Module
    for FileStoreModule<RepoManager>
{
    async fn start(&mut self) -> Result<(), LightSpeedError> {
        info!("Starting FileStoreModule");
        self.repo_manager.start().await?;
        Ok(())
    }
}
