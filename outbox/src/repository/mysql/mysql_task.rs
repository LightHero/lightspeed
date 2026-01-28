use crate::model::task::{TaskData, TaskModel};
use crate::repository::TaskRepository;
use c3p0::sqlx::*;
use c3p0::*;
use lightspeed_core::error::LsError;

#[derive(Clone)]
pub struct MySqlTaskRepository {}

impl Default for MySqlTaskRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl MySqlTaskRepository {
    pub fn new() -> Self {
        Self {}
    }
}

impl TaskRepository for MySqlTaskRepository {
    type DB = MySql;

    async fn fetch_by_token(&self, tx: &mut MySqlConnection, token_string: &str) -> Result<TaskModel, LsError> {
        Ok(TaskModel::query_with(
            r#"
            where data -> '$.token' = ?
            limit 1
        "#,
        )
        .bind(token_string)
        .fetch_one(tx)
        .await?)
    }

    async fn fetch_by_username(&self, tx: &mut MySqlConnection, username: &str) -> Result<Vec<TaskModel>, LsError> {
        Ok(TaskModel::query_with(
            r#"
            where data -> '$.username' = ?
        "#,
        )
        .bind(username)
        .fetch_all(tx)
        .await?)
    }

    async fn save(&self, tx: &mut MySqlConnection, model: NewRecord<TaskData>) -> Result<TaskModel, LsError> {
        Ok(tx.save(model).await?)
    }

    async fn delete(&self, tx: &mut MySqlConnection, model: TaskModel) -> Result<TaskModel, LsError> {
        Ok(tx.delete(model).await?)
    }
}
