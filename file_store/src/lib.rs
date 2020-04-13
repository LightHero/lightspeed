use crate::config::FileStoreConfig;
use crate::repository::db::DBFileStoreRepositoryManager;
use crate::service::file_store::FileStoreService;
use lightspeed_core::error::LightSpeedError;
use log::*;

pub mod config;
pub mod dto;
pub mod model;
pub mod repository;
pub mod service;
pub mod web;
pub mod utils;

#[derive(Clone)]
pub struct FileStoreModule<RepoManager: DBFileStoreRepositoryManager> {
    pub config: FileStoreConfig,

    pub repo_manager: RepoManager,
    pub file_store_service: service::file_store::FileStoreService<RepoManager>,
}

impl<RepoManager: DBFileStoreRepositoryManager> FileStoreModule<RepoManager> {
    pub fn new(repo_manager: RepoManager, config: FileStoreConfig) -> Self {
        println!("Creating FileStoreModule");
        info!("Creating FileStoreModule");

        let file_store_service = FileStoreService::new(
            repo_manager.c3p0().clone(),
            repo_manager.file_store_binary_repo(),
        );

        FileStoreModule {
            config,
            repo_manager,
            file_store_service,
        }
    }
}

#[async_trait::async_trait(?Send)]
impl<RepoManager: DBFileStoreRepositoryManager> lightspeed_core::module::Module
    for FileStoreModule<RepoManager>
{
    async fn start(&mut self) -> Result<(), LightSpeedError> {
        info!("Starting FileStoreModule");
        self.repo_manager.start().await?;
        Ok(())
    }
}
