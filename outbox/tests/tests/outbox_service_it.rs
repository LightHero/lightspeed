use std::time::Duration;

use c3p0::C3p0Pool;
use lightspeed_core::{error::LsError, service::random::LsRandomService};
use lightspeed_outbox::{
    error::OutboxError,
    model::{OutboxMessage, OutboxMessageData, OutboxMessageStatus},
    repository::{OutboxRepository, OutboxRepositoryManager},
};
use lightspeed_test_utils::tokio_test;
use serde_json::Value;
use tokio::{task::JoinSet, time::sleep};

use crate::{RepoManager, data};

#[test]
fn should_delete_token() -> Result<(), LsError> {
    tokio_test(async {
        let data = data(false).await;
        let outbox_module = &data.0;

        let repo = outbox_module.repo_manager.outbox_repo();
        let c3p0 = outbox_module.repo_manager.c3p0();

        c3p0.transaction(async |conn| {
            // Arrange

            Ok(())
        })
        .await
    })
}

#[test]
fn test_outbox_callback_success() -> Result<(), LsError> {
    tokio_test(async {
        let data = data(false).await;
        let outbox_module = &data.0;

        let repo_manager = &outbox_module.repo_manager;
        let c3p0 = outbox_module.repo_manager.c3p0();
        let service = &outbox_module.outbox_service.clone();

        // Arrange
        let (response_sender, mut response_receiver) = tokio::sync::mpsc::channel(100);
        let type_1 = format!("test_type_{}", LsRandomService::random_string(10));

        let (outbox_sender, outbox_receiver) = {
            let repo_manager = repo_manager.clone();
            service.channel::<String, _>(&type_1, move |id, data| {
                let data = data.clone();
                let response_sender = response_sender.clone();
                let repo_manager = repo_manager.clone();
                Box::pin(async move {
                    // During the callback the status should be processing
                    assert_message_status(&repo_manager, id, OutboxMessageStatus::Processing).await;
                    response_sender.send(data).await.unwrap();
                    Ok(())
                })
            })
        };

        // Act
        let message = c3p0
            .transaction::<_, OutboxError, _>(async |tx| {
                let message = OutboxMessage::new("test_payload".to_string(), 0);
                outbox_sender.send(tx, message).await
            })
            .await
            .unwrap();

        // Before the callback is called the status should be pending
        assert_message_status(&repo_manager, message.id, OutboxMessageStatus::Pending).await;

        outbox_receiver.poll(1).await.unwrap();

        // Assert
        let data = response_receiver.recv().await.unwrap();
        assert_eq!(data, "test_payload".to_string());

        // After the callback is called the status should be failed
        assert_message_status(&repo_manager, message.id, OutboxMessageStatus::Processed).await;

        Ok(())
    })
}

#[test]
fn test_outbox_callback_failure() -> Result<(), LsError> {
    tokio_test(async {
        let data = data(false).await;
        let outbox_module = &data.0;

        let repo_manager = &outbox_module.repo_manager;
        let c3p0 = outbox_module.repo_manager.c3p0();
        let service = &outbox_module.outbox_service.clone();

        // Arrange
        let (response_sender, mut response_receiver) = tokio::sync::mpsc::channel(100);
        let type_1 = format!("test_type_{}", LsRandomService::random_string(10));

        let (outbox_sender, outbox_receiver) = {
            let repo_manager = repo_manager.clone();
            service.channel::<String, _>(&type_1, move |id, data| {
                let data = data.clone();
                let response_sender = response_sender.clone();
                let repo_manager = repo_manager.clone();
                Box::pin(async move {
                    // During the callback the status should be processing
                    assert_message_status(&repo_manager, id, OutboxMessageStatus::Processing).await;
                    response_sender.send(data).await.unwrap();
                    Err("Error".into())
                })
            })
        };

        // Act
        let message = c3p0
            .transaction::<_, OutboxError, _>(async |tx| {
                let message = OutboxMessage::new("test_payload".to_string(), 0);
                outbox_sender.send(tx, message).await
            })
            .await
            .unwrap();

        // Before the callback is called the status should be pending
        assert_message_status(&repo_manager, message.id, OutboxMessageStatus::Pending).await;

        outbox_receiver.poll(1).await.unwrap();

        // Assert
        let data = response_receiver.recv().await.unwrap();
        assert_eq!(data, "test_payload".to_string());

        // After the callback is called the status should be failed
        assert_message_status(&repo_manager, message.id, OutboxMessageStatus::Failed).await;

        Ok(())
    })
}

async fn assert_message_status(repo_manager: &RepoManager, id: u64, status: OutboxMessageStatus) {
    repo_manager
        .c3p0()
        .transaction::<_, OutboxError, _>(async |tx| {
            let message = repo_manager.outbox_repo().fetch_by_id::<Value>(tx, id).await.unwrap();
            assert_eq!(message.data.status(), &status);
            Ok(())
        })
        .await
        .unwrap();
}
