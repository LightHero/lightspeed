use crate::error::OutboxError;
use crate::model::{OutboxMessageData, OutboxMessageModel, OutboxMessageStatus};
use crate::repository::OutboxRepository;
use c3p0::sqlx::*;
use c3p0::*;
use serde::Serialize;
use serde::de::DeserializeOwned;

#[derive(Clone)]
pub struct MySqlOutboxRepository;

impl Default for MySqlOutboxRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl MySqlOutboxRepository {
    pub fn new() -> Self {
        Self {}
    }
}

impl OutboxRepository for MySqlOutboxRepository {
    type DB = MySql;

    /// Fetches an outbox message by id
    async fn fetch_by_id<D: Send + Sync + Unpin + Serialize + DeserializeOwned>(
        &self,
        tx: &mut MySqlConnection,
        id: u64,
    ) -> Result<OutboxMessageModel<D>, OutboxError> {
        Ok(tx.fetch_one_by_id(id).await?)
    }

    /// Fetches all outbox messages and locks them for update.
    /// If the outbox message is already locked by another process, it will be skipped.
    async fn fetch_all_by_type_and_status_for_update<D: Send + Sync + Unpin + Serialize + DeserializeOwned>(
        &self,
        tx: &mut MySqlConnection,
        r#type: &str,
        status: OutboxMessageStatus,
        limit: usize,
    ) -> Result<Vec<OutboxMessageModel<D>>, OutboxError> {
        // Not using FOR UPDATE SKIP LOCKED because it randomly hangs in MySQL!!
        // In fact MySQL does not lock only the selected rows, but the whole table or a subset of it causing a deadlock
        // https://dev.mysql.com/doc/refman/8.0/en/innodb-locking-reads.html
        //
        // Due do this the outbox will still work correctly, but in case of concurrent access,
        // instead of silently completing the request, one of the process will return a C3p0 optimistic lock error
        //
        let result = OutboxMessageModel::query_with(
            r#"
            where data -> '$.type' = ? AND data -> '$.status' = ?
            ORDER BY id ASC
            LIMIT ?
        "#,
        )
        .bind(r#type)
        .bind(status.as_ref())
        .bind(limit as i64)
        .fetch_all(tx)
        .await?;

        Ok(result)
    }

    /// Updates an outbox message
    async fn update<D: Send + Sync + Unpin + Serialize + DeserializeOwned>(
        &self,
        tx: &mut MySqlConnection,
        data: OutboxMessageModel<D>,
    ) -> Result<OutboxMessageModel<D>, OutboxError> {
        Ok(tx.update(data).await?)
    }

    /// Saves a new outbox message
    async fn save<D: Send + Sync + Unpin + Serialize + DeserializeOwned>(
        &self,
        tx: &mut MySqlConnection,
        data: OutboxMessageData<D>,
    ) -> Result<OutboxMessageModel<D>, OutboxError> {
        Ok(tx.save(NewRecord::new(data)).await?)
    }
}
