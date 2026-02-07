use std::pin::Pin;
use std::sync::Arc;

use crate::config::OutboxConfig;
use crate::error::OutboxError;
use crate::model::{OutboxMessage, OutboxMessageData, OutboxMessageModel, OutboxMessageStatus};
use crate::repository::{OutboxRepository, OutboxRepositoryManager};
use c3p0::sqlx::Database;
use c3p0::*;
use serde::Serialize;
use serde::de::DeserializeOwned;

#[derive(Clone)]
pub struct LsOutboxService<RepoManager: OutboxRepositoryManager> {
    outbox_config: OutboxConfig,
    repo: RepoManager::OutboxRepo,
    c3p0: RepoManager::C3P0,
}

impl<RepoManager: OutboxRepositoryManager> LsOutboxService<RepoManager> {
    pub fn new(auth_config: OutboxConfig, outbox_repo: &RepoManager) -> Self {
        LsOutboxService { outbox_config: auth_config, repo: outbox_repo.outbox_repo(), c3p0: outbox_repo.c3p0().clone() }
    }


    /// Saves a new outbox message
    pub async fn save<D: Send + Sync + Unpin + Serialize + DeserializeOwned>(&self, tx: &mut <RepoManager::DB as Database>::Connection, data: OutboxMessageData<D>) -> Result<OutboxMessageModel<D>, OutboxError> {
        self.repo.save(tx, data).await
    }

    /// Creates a new outbox channel
    pub fn channel<D: Send + Sync + Unpin + Serialize + DeserializeOwned, F: 'static
            + Send
            + Sync
            + Fn(u64, &D) -> Pin<Box<dyn Future<Output = Result<(), Box<dyn std::error::Error + Send + Sync>>> + Send>>>(&self, r#type: &str, callback: F) -> (OutboxSender<RepoManager, D>, OutboxReceiver<RepoManager, D>) {
        let receiver = OutboxReceiver {
            callback: Arc::new(callback),
            r#type: r#type.to_string(),
            c3p0: self.c3p0.clone(),
            repo: self.repo.clone()
            
            };
        let sender = OutboxSender::new(self.repo.clone(), r#type.to_string());
        (sender, receiver)
    }

}

/// A types sender for sending outbox messages
#[derive(Clone)]
pub struct OutboxSender<RepoManager: OutboxRepositoryManager, D> {
    repo: RepoManager::OutboxRepo,
    r#type: String,
    phantom_d: std::marker::PhantomData<D>
}

impl <RepoManager: OutboxRepositoryManager, D: Send + Sync + Unpin + Serialize + DeserializeOwned> OutboxSender<RepoManager, D> {
    /// Creates a new OutboxSender
    pub fn new(repo: RepoManager::OutboxRepo, r#type: String) -> Self {
        Self {
            repo,
            r#type,
            phantom_d: std::marker::PhantomData
        }
    }

    /// Sends a new outbox message
    pub async fn send(&self, tx: &mut <RepoManager::DB as Database>::Connection, message: OutboxMessage<D>) -> Result<OutboxMessageModel<D>, OutboxError> {
        let message = message.to_data(self.r#type.clone());
        self.repo.save(tx, message).await
    }
}

pub type ReceiveCallback<D> = dyn 'static
    + Send
    + Sync
    + Fn(u64, &D) -> Pin<Box<dyn Future<Output = Result<(), Box<dyn std::error::Error + Send + Sync>>> + Send>>;

/// A service for managing the outbox
#[derive(Clone)]
pub struct OutboxReceiver<RepoManager: OutboxRepositoryManager, D> {
    callback: Arc<ReceiveCallback<D>>,
    r#type: String,
    repo: RepoManager::OutboxRepo,
    c3p0: RepoManager::C3P0,
}

impl <RepoManager: OutboxRepositoryManager, D: Send + Sync + Unpin + Serialize + DeserializeOwned> OutboxReceiver<RepoManager, D> {
    
    /// Polls the outbox for pending messages and calls the callback function
    pub async fn poll(&self, max_messages: usize) -> Result<(), OutboxError> {
        // fetch all pending messages and set them to processing
        let mut messages = {
            self.c3p0.transaction::<_, OutboxError, _>(async |tx| {
                let messages = self.repo.fetch_all_by_type_and_status_for_update::<D>(tx, &self.r#type, OutboxMessageStatus::Pending, max_messages).await?;
                let mut processing_messages = vec![];
                for mut message in messages.into_iter() {
                    message.data.status = OutboxMessageStatus::Processing;
                    processing_messages.push(self.repo.update(tx, message).await?);
                }
                Ok(processing_messages)
            }).await?
        };

        // call the callback for each message
        for message in messages.iter_mut() {
            if let Ok(()) = (self.callback)(message.id, &message.data.payload).await {
                message.data.status = OutboxMessageStatus::Processed;
            } else {
                message.data.status = OutboxMessageStatus::Failed;
            }
        }

        // update the messages
            self.c3p0.transaction::<_, OutboxError, _>(async |tx| {
                for message in messages {
                    self.repo.update(tx, message).await?;
                }
                Ok(())
            }).await?;

        Ok(())
    }
}
