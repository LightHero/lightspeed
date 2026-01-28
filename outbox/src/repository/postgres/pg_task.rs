use crate::model::task::{TaskData, TaskModel};
use crate::repository::TaskRepository;
use c3p0::sqlx::*;
use c3p0::*;
use lightspeed_core::error::LsError;

#[derive(Clone)]
pub struct PgTaskRepository;

impl Default for PgTaskRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl PgTaskRepository {
    pub fn new() -> Self {
        Self {}
    }
}

impl TaskRepository for PgTaskRepository {
    type DB = Postgres;

    async fn fetch_by_token(&self, tx: &mut PgConnection, token_string: &str) -> Result<TaskModel, LsError> {
        Ok(TaskModel::query_with(
            r#"
            where data ->> 'token' = $1
            limit 1
        "#,
        )
        .bind(token_string)
        .fetch_one(tx)
        .await?)
    }

    async fn fetch_by_username(&self, tx: &mut PgConnection, username: &str) -> Result<Vec<TaskModel>, LsError> {
        Ok(TaskModel::query_with(
            r#"
            where data ->> 'username' = $1
        "#,
        )
        .bind(username)
        .fetch_all(tx)
        .await?)
    }

    async fn save(&self, tx: &mut PgConnection, model: NewRecord<TaskData>) -> Result<TaskModel, LsError> {
        Ok(tx.save(model).await?)
    }

    async fn delete(&self, tx: &mut PgConnection, model: TaskModel) -> Result<TaskModel, LsError> {
        Ok(tx.delete(model).await?)
    }
}
