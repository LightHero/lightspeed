use std::{fmt::format, time::Duration};

use c3p0::C3p0Pool;
use lightspeed_core::{
    error::LsError,
    service::random::{self, LsRandomService},
};
use lightspeed_outbox::{
    model::{OutboxMessageData, OutboxMessageStatus},
    repository::{OutboxRepository, OutboxRepositoryManager, postgres::pg_outbox::PgOutboxRepository},
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

#[test]
fn test_repository() -> Result<(), LsError> {
    tokio_test(async {
        let data = data(false).await;
        let outbox_module = &data.0;

        let repo = outbox_module.repo_manager.outbox_repo();
        let c3p0 = outbox_module.repo_manager.c3p0();

        c3p0.transaction(async |tx| {
            // Arrange
            let type_1 = format!("test_type_{}", LsRandomService::random_string(10));

            // Act
            let saved = repo.save(tx, OutboxMessageData::new(&type_1, "test_payload".to_string())).await.unwrap();

            let loaded = repo.fetch_by_id::<String>(tx, saved.id).await.unwrap();

            // Assert
            assert_eq!(saved.id, loaded.id);
            assert_eq!(saved.data, loaded.data);
            assert_eq!(&OutboxMessageStatus::Pending, loaded.data.status());

            Ok(())
        })
        .await
    })
}

/// Tests that entries can be fetched by type
#[test]
fn test_fetch_by_type() -> Result<(), LsError> {
    tokio_test(async {
        let data = data(false).await;
        let outbox_module = &data.0;

        let repo = outbox_module.repo_manager.outbox_repo();
        let c3p0 = outbox_module.repo_manager.c3p0();

        c3p0.transaction(async |tx| {
            // Arrange
            let db_entries = 5;
            let type_1 = format!("test_type_{}", LsRandomService::random_string(10));

            for i in 0..db_entries {
                repo.save(tx, OutboxMessageData::new(&type_1, format!("test_payload_{i}"))).await.unwrap();
            }

            // Act
            let loaded = repo
                .fetch_all_by_type_and_status_for_update::<String>(tx, &type_1, OutboxMessageStatus::Pending, 3)
                .await
                .unwrap();

            // Assert
            assert_eq!(3, loaded.len());

            for i in 0..3 {
                assert_eq!(format!("test_payload_{i}"), loaded[i].data.payload);
            }

            Ok(())
        })
        .await
    })
}

/// Tests that a entries can be fetched by type when multiple types are present.
#[test]
fn test_fetch_by_type_with_multiple() -> Result<(), LsError> {
    tokio_test(async {
        let data = data(false).await;
        let outbox_module = &data.0;

        let repo = outbox_module.repo_manager.outbox_repo();
        let c3p0 = outbox_module.repo_manager.c3p0();

        c3p0.transaction(async |tx| {
            // Arrange
            let db_entries = 5;
            let type_1 = format!("test_type_{}", LsRandomService::random_string(10));
            let type_2 = format!("test_type_{}", LsRandomService::random_string(10));

            for i in 0..db_entries {
                repo.save(tx, OutboxMessageData::new(&type_1, format!("test_payload_{i}"))).await.unwrap();
            }

            for i in 0..db_entries {
                repo.save(tx, OutboxMessageData::new(&type_2, format!("test_payload_{i}"))).await.unwrap();
            }

            // Act
            let loaded_type_1 = repo
                .fetch_all_by_type_and_status_for_update::<String>(tx, &type_1, OutboxMessageStatus::Pending, 10)
                .await
                .unwrap();
            let loaded_type_2 = repo
                .fetch_all_by_type_and_status_for_update::<String>(tx, &type_2, OutboxMessageStatus::Pending, 10)
                .await
                .unwrap();

            // Assert
            assert_eq!(5, loaded_type_1.len());
            for loaded in loaded_type_1 {
                assert_eq!(type_1, loaded.data.r#type);
            }

            assert_eq!(5, loaded_type_2.len());
            for loaded in loaded_type_2 {
                assert_eq!(type_2, loaded.data.r#type);
            }

            Ok(())
        })
        .await
    })
}

/// Tests that a entries can be fetched by type concurrently.
/// Only one reader should be able to fetch entries at a time.
#[test]
fn test_fetch_by_type_concurrently() -> Result<(), LsError> {
    tokio_test(async {
        let data = data(false).await;
        let outbox_module = &data.0;

        let repo = outbox_module.repo_manager.outbox_repo();
        let c3p0 = outbox_module.repo_manager.c3p0();

        // Arrange
        let db_entries = 10;
        let type_1 = format!("test_type_{}", LsRandomService::random_string(10));
        c3p0.transaction::<_, LsError, _>(async |tx| {
            for i in 0..db_entries {
                repo.save(tx, OutboxMessageData::new(&type_1, format!("test_payload_{i}"))).await.unwrap();
            }
            Ok(())
        })
        .await
        .unwrap();

        // Act

        let mut set = JoinSet::new();

        // Warn: this should not be bigger than the number of max connections otherwise the test is not concurrent and it will fail
        for _ in 0..db_entries {
            let repo = repo.clone();
            let c3p0 = c3p0.clone();
            let type_1 = type_1.clone();

            set.spawn(async move {
                let loaded = c3p0
                    .transaction::<_, LsError, _>(async |tx| {
                        let loaded = repo
                            .fetch_all_by_type_and_status_for_update::<String>(
                                tx,
                                &type_1,
                                OutboxMessageStatus::Pending,
                                3,
                            )
                            .await
                            .unwrap();
                        sleep(Duration::from_secs(1)).await;
                        Ok(loaded)
                    })
                    .await
                    .unwrap();

                loaded
            });
        }

        let mut seen = Vec::new();
        while let Some(res) = set.join_next().await {
            let mut entries = res.unwrap();
            seen.append(&mut entries);
        }

        // Assert
        assert_eq!(10, seen.len());

        Ok(())
    })
}

//     // Act

//     let mut set = JoinSet::new();

//     // Warn: this should not be bigger than the number of max connections otherwise the test is not concurrent and it will fail
//     for _ in 0..db_entries*3 {
//         let repo = repo.clone();
//         let pool = pool.clone();
//         set.spawn(async move {
//             let mut tx = pool.begin().await.unwrap();
//             let loaded = repo.fetch_all_by_type_and_status_for_update::<String>(&mut tx, "test_type", OutboxMessageStatus::Pending,3).await.unwrap();
//             sleep(Duration::from_secs(1)).await;
//             tx.commit().await.unwrap();
//             loaded
//         });
//     }

//     let mut seen = Vec::new();
//     while let Some(res) = set.join_next().await {
//         let mut entries= res.unwrap();
//         seen.append(&mut entries);
//     }

//     // Assert
//     assert_eq!(10, seen.len());

// }

// /// Tests that a entries can be fetched by type concurrently.
// /// Only one reader should be able to fetch entries at a time.
// #[tokio::test]
// async fn test_fetch_by_type_concurrently() {
//         // Arrange
//     let (pool, _node) = new_db().await;
//     let repo = PgOutboxRepository::new(&pool).await.unwrap();

//     let mut tx = pool.begin().await.unwrap();

//     let db_entries = 10;

//     for i in 0..db_entries {
//         repo.save(&mut tx, OutboxMessageData::new(
//             "test_type", format!("test_payload_{i}"))).await.unwrap();
//     };
//     tx.commit().await.unwrap();

//     // Act

//     let mut set = JoinSet::new();

//     // Warn: this should not be bigger than the number of max connections otherwise the test is not concurrent and it will fail
//     for _ in 0..db_entries*3 {
//         let repo = repo.clone();
//         let pool = pool.clone();
//         set.spawn(async move {
//             let mut tx = pool.begin().await.unwrap();
//             let loaded = repo.fetch_all_by_type_and_status_for_update::<String>(&mut tx, "test_type", OutboxMessageStatus::Pending,3).await.unwrap();
//             sleep(Duration::from_secs(1)).await;
//             tx.commit().await.unwrap();
//             loaded
//         });
//     }

//     let mut seen = Vec::new();
//     while let Some(res) = set.join_next().await {
//         let mut entries= res.unwrap();
//         seen.append(&mut entries);
//     }

//     // Assert
//     assert_eq!(10, seen.len());

// }
