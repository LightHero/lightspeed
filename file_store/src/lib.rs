use crate::config::FileStoreConfig;
use crate::repository::db::DBFileStoreRepositoryManager;
use crate::service::file_store::LsFileStoreService;
use lightspeed_core::error::LsError;
use log::*;
use std::sync::Arc;

pub mod config;
pub mod dto;
pub mod model;
pub mod repository;
pub mod service;
pub mod web;

#[derive(Clone)]
pub struct LsFileStoreModule<RepoManager: DBFileStoreRepositoryManager> {
    pub repo_manager: RepoManager,
    pub file_store_service: Arc<service::file_store::LsFileStoreService<RepoManager>>,
}

impl<RepoManager: DBFileStoreRepositoryManager> LsFileStoreModule<RepoManager> {
    pub fn new(repo_manager: RepoManager, config: FileStoreConfig) -> Result<Self, LsError> {
        println!("Creating LsFileStoreModule");
        info!("Creating LsFileStoreModule");
        let file_store_service = Arc::new(LsFileStoreService::new(&repo_manager, config.repositories));

        Ok(LsFileStoreModule { repo_manager, file_store_service })
    }
}

impl<RepoManager: DBFileStoreRepositoryManager> lightspeed_core::module::LsModule for LsFileStoreModule<RepoManager> {
    async fn start(&mut self) -> Result<(), LsError> {
        info!("Starting LsFileStoreModule");
        self.repo_manager.start().await?;
        Ok(())
    }
}
