use lightspeed_core::error::LightSpeedError;
use log::*;
use crate::config::FileStoreConfig;
use crate::repository::FileStoreRepositoryManager;
use crate::service::file_store::FileStoreService;

pub mod config;
pub mod dto;
pub mod model;
pub mod repository;
pub mod service;

#[derive(Clone)]
pub struct FileStoreModule<RepoManager: FileStoreRepositoryManager> {
    pub config: FileStoreConfig,

    pub repo_manager: RepoManager,
    pub file_store_service: service::file_store::FileStoreService<RepoManager>,

}

impl<RepoManager: FileStoreRepositoryManager> FileStoreModule<RepoManager> {
    pub fn new(repo_manager: RepoManager, config: FileStoreConfig) -> Self {
        println!("Creating FileStoreModule");
        info!("Creating FileStoreModule");

        let file_store_service = FileStoreService::new(
            repo_manager.c3p0().clone(),
            repo_manager.file_store_repo(),
        );

        FileStoreModule {
            config,
            repo_manager,
            file_store_service
        }
    }
}

#[async_trait::async_trait(?Send)]
impl<RepoManager: FileStoreRepositoryManager> lightspeed_core::module::Module
    for FileStoreModule<RepoManager>
{
    async fn start(&mut self) -> Result<(), LightSpeedError> {
        info!("Starting FileStoreModule");
        self.repo_manager.start().await?;
        Ok(())
    }
}
