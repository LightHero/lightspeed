use crate::config::OutboxConfig;
use crate::model::task::{TaskData, TaskModel, TaskType};
use crate::repository::{OutboxRepositoryManager, TaskRepository};
use c3p0::sqlx::Database;
use c3p0::*;
use lightspeed_core::error::LsError;
use lightspeed_core::utils::*;
use log::*;

#[derive(Clone)]
pub struct LsTaskService<RepoManager: OutboxRepositoryManager> {
    outbox_config: OutboxConfig,
    task_repo: RepoManager::TaskRepo,
}

impl<RepoManager: OutboxRepositoryManager> LsTaskService<RepoManager> {
    pub fn new(auth_config: OutboxConfig, task_repo: RepoManager::TaskRepo) -> Self {
        LsTaskService { outbox_config: auth_config, task_repo: task_repo }
    }

    pub async fn generate_and_save_task_with_conn<S: Into<String>>(
        &self,
        conn: &mut <RepoManager::DB as Database>::Connection,
        username: S,
        task_type: TaskType,
    ) -> Result<TaskModel, LsError> {
        let username = username.into();
        info!("Generate and save task of type [{task_type:?}] for username [{username}]");

        let issued_at = current_epoch_seconds();
        let expire_at_epoch = issued_at + (self.outbox_config.activation_task_validity_minutes * 60);
        let task = NewRecord::new(TaskData {
            task: new_hyphenated_uuid(),
            task_type,
            username,
            expire_at_epoch_seconds: expire_at_epoch,
        });
        self.task_repo.save(conn, task).await
    }

    pub async fn fetch_by_task_with_conn(
        &self,
        conn: &mut <RepoManager::DB as Database>::Connection,
        task: &str,
    ) -> Result<TaskModel, LsError> {
        debug!("Fetch by task [{task}]");
        let task_model = self.task_repo.fetch_by_task(conn, task).await?;

        Ok(task_model)
    }

    pub async fn fetch_all_by_username_with_conn(
        &self,
        conn: &mut <RepoManager::DB as Database>::Connection,
        username: &str,
    ) -> Result<Vec<TaskModel>, LsError> {
        debug!("Fetch by username [{username}]");
        self.task_repo.fetch_by_username(conn, username).await
    }

    pub async fn delete_with_conn(
        &self,
        conn: &mut <RepoManager::DB as Database>::Connection,
        task_model: TaskModel,
    ) -> Result<TaskModel, LsError> {
        debug!("Delete task_model with id [{:?}] and task [{}]", task_model.id, task_model.data.task);
        self.task_repo.delete(conn, task_model).await
    }
}
