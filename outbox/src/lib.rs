use crate::config::OutboxConfig;
use crate::repository::OutboxRepositoryManager;
use lightspeed_core::error::LsError;
use log::*;
use std::sync::Arc;

pub mod config;
pub mod dto;
pub mod model;
pub mod repository;
pub mod service;

#[derive(Clone)]
pub struct LsOutboxModule<RepoManager: OutboxRepositoryManager> {
    pub outbox_config: OutboxConfig,

    pub repo_manager: RepoManager,

    pub task_service: Arc<service::task::LsTaskService<RepoManager>>,
}

impl<RepoManager: OutboxRepositoryManager> LsOutboxModule<RepoManager> {
    pub fn new(repo_manager: RepoManager, outbox_config: OutboxConfig) -> Self {
        println!("Creating LsOutboxModule");
        info!("Creating LsOutboxModule");

        let task_service =
            Arc::new(service::task::LsTaskService::new(outbox_config.clone(), repo_manager.task_repo()));

        LsOutboxModule { outbox_config, repo_manager, task_service }
    }
}

impl<RepoManager: OutboxRepositoryManager> lightspeed_core::module::LsModule for LsOutboxModule<RepoManager> {
    async fn start(&mut self) -> Result<(), LsError> {
        info!("Starting LsOutboxModule");
        self.repo_manager.start().await?;
        Ok(())
    }
}
