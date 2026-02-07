use crate::config::OutboxConfig;
use crate::repository::{OutboxRepository, OutboxRepositoryManager};
use c3p0::sqlx::Database;
use c3p0::*;
use lightspeed_core::error::LsError;
use lightspeed_core::utils::*;
use log::*;

#[derive(Clone)]
pub struct LsOutboxService<RepoManager: OutboxRepositoryManager> {
    outbox_config: OutboxConfig,
    task_repo: RepoManager::OutboxRepo,
}

impl<RepoManager: OutboxRepositoryManager> LsOutboxService<RepoManager> {
    pub fn new(auth_config: OutboxConfig, task_repo: RepoManager::OutboxRepo) -> Self {
        LsOutboxService { outbox_config: auth_config, task_repo: task_repo }
    }
}
