use crate::error::OutboxError;
use crate::model::{OutboxMessageData, OutboxMessageModel, OutboxMessageStatus};
use crate::repository::OutboxRepository;
use c3p0::sqlx::*;
use c3p0::*;
use serde::Serialize;
use serde::de::DeserializeOwned;

#[derive(Clone)]
pub struct SqliteOutboxRepository;

impl Default for SqliteOutboxRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl SqliteOutboxRepository {
    pub fn new() -> Self {
        Self {}
    }
}

impl OutboxRepository for SqliteOutboxRepository {
    type DB = Sqlite;

    /// Fetches an outbox message by id
    async fn fetch_by_id<D: Send + Sync + Unpin + Serialize + DeserializeOwned>(
        &self,
        tx: &mut SqliteConnection,
        id: u64,
    ) -> Result<OutboxMessageModel<D>, OutboxError> {
        Ok(tx.fetch_one_by_id(id).await?)
    }

    /// Fetches all outbox messages and locks them for update.
    /// If the outbox message is already locked by another process, it will be skipped.
    async fn fetch_all_by_type_and_status_for_update<D: Send + Sync + Unpin + Serialize + DeserializeOwned>(
        &self,
        tx: &mut SqliteConnection,
        r#type: &str,
        status: OutboxMessageStatus,
        limit: usize,
    ) -> Result<Vec<OutboxMessageModel<D>>, OutboxError> {
        // No need to use FOR UPDATE in sqlite because each update locks the entire DB by default
        Ok(OutboxMessageModel::query_with(
            r#"
            where data ->> '$.type' = ? AND data ->> '$.status' = ?
            ORDER BY id ASC
            limit ?
        "#,
        )
        .bind(r#type)
        .bind(status.as_ref())
        .bind(limit as i64)
        .fetch_all(tx)
        .await?)
    }

    /// Updates an outbox message
    async fn update<D: Send + Sync + Unpin + Serialize + DeserializeOwned>(
        &self,
        tx: &mut SqliteConnection,
        data: OutboxMessageModel<D>,
    ) -> Result<OutboxMessageModel<D>, OutboxError> {
        Ok(tx.update(data).await?)
    }

    /// Saves a new outbox message
    async fn save<D: Send + Sync + Unpin + Serialize + DeserializeOwned>(
        &self,
        tx: &mut SqliteConnection,
        data: OutboxMessageData<D>,
    ) -> Result<OutboxMessageModel<D>, OutboxError> {
        Ok(tx.save(NewRecord::new(data)).await?)
    }
}
