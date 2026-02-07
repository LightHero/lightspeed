use std::time::Duration;

use c3p0::C3p0Pool;
use lightspeed_core::{
    error::LsError,
    service::random::LsRandomService,
};
use lightspeed_outbox::{
    model::{OutboxMessageData, OutboxMessageStatus},
    repository::{OutboxRepository, OutboxRepositoryManager},
};
use lightspeed_test_utils::tokio_test;
use tokio::{task::JoinSet, time::sleep};

use crate::data;

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
