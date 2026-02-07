use std::future::Future;

use crate::error::OutboxError;
use crate::model::{OutboxMessageData, OutboxMessageModel, OutboxMessageStatus};
use c3p0::sqlx::Database;
use c3p0::*;
use lightspeed_core::error::LsError;
use serde::Serialize;
use serde::de::DeserializeOwned;

#[cfg(feature = "mysql")]
pub mod mysql;

#[cfg(feature = "postgres")]
pub mod postgres;

#[cfg(feature = "sqlite")]
pub mod sqlite;

pub trait OutboxRepositoryManager: Clone + Send + Sync {
    type DB: Database;
    type C3P0: C3p0Pool<DB = Self::DB>;
    type OutboxRepo: for<'a> OutboxRepository<DB = Self::DB>;

    fn c3p0(&self) -> &Self::C3P0;
    fn start(&self) -> impl Future<Output = Result<(), LsError>> + Send;
    fn outbox_repo(&self) -> Self::OutboxRepo;
}

pub trait OutboxRepository: Clone + Send + Sync {
    type DB: Database;

    fn fetch_by_id<D: Send + Sync + Unpin + Serialize + DeserializeOwned>(
        &self,
        tx: &mut <Self::DB as Database>::Connection,
        id: u64,
    ) -> impl Future<Output = Result<OutboxMessageModel<D>, OutboxError>> + Send;

    /// Fetches all outbox messages and locks them for update.
    /// If the outbox message is already locked by another process, it will be skipped.
    fn fetch_all_by_type_and_status_for_update<D: Send + Sync + Unpin + Serialize + DeserializeOwned>(
        &self,
        tx: &mut <Self::DB as Database>::Connection,
        r#type: &str,
        status: OutboxMessageStatus,
        limit: usize,
    ) -> impl Future<Output = Result<Vec<OutboxMessageModel<D>>, OutboxError>> + Send;

    /// Updates an outbox message
    fn update<D: Send + Sync + Unpin + Serialize + DeserializeOwned>(
        &self,
        tx: &mut <Self::DB as Database>::Connection,
        data: OutboxMessageModel<D>,
    ) -> impl Future<Output = Result<OutboxMessageModel<D>, OutboxError>> + Send;

    /// Saves a new outbox message
    fn save<D: Send + Sync + Unpin + Serialize + DeserializeOwned>(
        &self,
        tx: &mut <Self::DB as Database>::Connection,
        data: OutboxMessageData<D>,
    ) -> impl Future<Output = Result<OutboxMessageModel<D>, OutboxError>> + Send;
}
