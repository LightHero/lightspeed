use std::time::Duration;

use c3p0::C3p0Pool;
use lightspeed_core::error::LsError;
use lightspeed_test_utils::tokio_test;
use lightspeed_outbox::{model::{OutboxMessageData, OutboxMessageStatus}, repository::{OutboxRepository, OutboxRepositoryManager, postgres::pg_outbox::PgOutboxRepository}};
use tokio::{task::JoinSet, time::sleep};

use crate::data;

#[test]
fn should_delete_token() -> Result<(), LsError> {
    tokio_test(async {
        let data = data(false).await;
        let outbox_module = &data.0;

        // Arrange
        let repo = outbox_module.repo_manager.outbox_repo();
        let c3p0 = outbox_module.repo_manager.c3p0();

        c3p0.transaction(async |conn| {

            Ok(())
        }).await
    })
}

#[test]
fn test_repository() -> Result<(), LsError> {
    tokio_test(async {
        let data = data(false).await;
        let outbox_module = &data.0;

        // Arrange
        let repo = outbox_module.repo_manager.outbox_repo();
        let c3p0 = outbox_module.repo_manager.c3p0();

        c3p0.transaction(async |tx| {

                // Act
    let saved = repo.save(tx, OutboxMessageData::new(
        "test_type", "test_payload".to_string())).await.unwrap();

    let loaded = repo.fetch_by_id::<String>(tx, saved.id).await.unwrap();

    // Assert
    assert_eq!(saved.id, loaded.id);
    assert_eq!(saved.data, loaded.data);
    assert_eq!(&OutboxMessageStatus::Pending, loaded.data.status());
    
            Ok(())
        }).await
    })
}

// /// Tests that a repository can interact with the database
// #[tokio::test]
// async fn test_repository() {
//     // Arrange
//     let (pool, _node) = new_db().await;
//     let repo = PgOutboxRepository::new(&pool).await.unwrap();

//     let mut tx = pool.begin().await.unwrap();

//     // Act
//     let saved = repo.save(&mut tx, OutboxMessageData::new(
//         "test_type", "test_payload".to_string())).await.unwrap();

//     let loaded = repo.fetch_by_id::<String>(&mut tx, saved.id).await.unwrap();

//     // Assert
//     assert_eq!(saved.id, loaded.id);
//     assert_eq!(saved.data, loaded.data);
//     assert_eq!(&OutboxMessageStatus::Pending, loaded.data.status());
    
//     tx.commit().await.unwrap();

// }

// /// Tests that a entries can be fetched by type.
// #[tokio::test]
// async fn test_fetch_by_type() {
//         // Arrange
//     let (pool, _node) = new_db().await;
//     let repo = PgOutboxRepository::new(&pool).await.unwrap();

//     let mut tx = pool.begin().await.unwrap();

//     let db_entries = 5;

//     for i in 0..db_entries {
//         repo.save(&mut tx, OutboxMessageData::new(
//             "test_type", format!("test_payload_{i}"))).await.unwrap();
//     };

//     // Act
//     let loaded = repo.fetch_all_by_type_and_status_for_update::<String>(&mut tx, "test_type", OutboxMessageStatus::Pending, 3).await.unwrap();

//     // Assert
//     assert_eq!(3, loaded.len());

//     for i in 0..3 {
//         assert_eq!(format!("test_payload_{i}"), loaded[i].data.payload);
//     }

//     tx.commit().await.unwrap();

// }

// /// Tests that a entries can be fetched by type when multiple types are present.
// #[tokio::test]
// async fn test_fetch_by_type_with_multiple() {
//         // Arrange
//     let (pool, _node) = new_db().await;
//     let repo = PgOutboxRepository::new(&pool).await.unwrap();

//     let mut tx = pool.begin().await.unwrap();

//     let db_entries = 5;

//     for i in 0..db_entries {
//         repo.save(&mut tx, OutboxMessageData::new(
//             "test_type_1", format!("test_payload_{i}"))).await.unwrap();
//     };

//     for i in 0..db_entries {
//         repo.save(&mut tx, OutboxMessageData::new(
//             "test_type_2", format!("test_payload_{i}"))).await.unwrap();
//     };

//     // Act
//     let loaded_type_1 = repo.fetch_all_by_type_and_status_for_update::<String>(&mut tx, "test_type_1", OutboxMessageStatus::Pending, 10).await.unwrap();
//     let loaded_type_2 = repo.fetch_all_by_type_and_status_for_update::<String>(&mut tx, "test_type_2", OutboxMessageStatus::Pending, 10).await.unwrap();

//     // Assert
//     assert_eq!(5, loaded_type_1.len());
//     for loaded in loaded_type_1 {
//         assert_eq!(format!("test_type_1"), loaded.data.r#type);
//     }

//     assert_eq!(5, loaded_type_2.len());
//     for loaded in loaded_type_2 {
//         assert_eq!(format!("test_type_2"), loaded.data.r#type);
//     }

//     tx.commit().await.unwrap();

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