use crate::model::task::{TaskData, TaskModel};
use crate::repository::TaskRepository;
use c3p0::sqlx::*;
use c3p0::*;
use lightspeed_core::error::LsError;

#[derive(Clone)]
pub struct SqliteTaskRepository {}

impl Default for SqliteTaskRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl SqliteTaskRepository {
    pub fn new() -> Self {
        Self {}
    }
}

impl TaskRepository for SqliteTaskRepository {
    type DB = Sqlite;

    async fn fetch_by_token(&self, tx: &mut SqliteConnection, token_string: &str) -> Result<TaskModel, LsError> {
        Ok(TaskModel::query_with(
            r#"
            where data ->> '$.token' = ?
            limit 1
        "#,
        )
        .bind(token_string)
        .fetch_one(tx)
        .await?)
    }

    async fn fetch_by_username(&self, tx: &mut SqliteConnection, username: &str) -> Result<Vec<TaskModel>, LsError> {
        Ok(TaskModel::query_with(
            r#"
            where data ->> '$.username' = ?
        "#,
        )
        .bind(username)
        .fetch_all(tx)
        .await?)
    }

    async fn save(&self, tx: &mut SqliteConnection, model: NewRecord<TaskData>) -> Result<TaskModel, LsError> {
        Ok(tx.save(model).await?)
    }

    async fn delete(&self, tx: &mut SqliteConnection, model: TaskModel) -> Result<TaskModel, LsError> {
        Ok(tx.delete(model).await?)
    }
}
