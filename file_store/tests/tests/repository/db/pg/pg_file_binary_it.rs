use crate::{data, test};
use c3p0::*;
use lightspeed_core::error::LightSpeedError;
use lightspeed_file_store::model::BinaryContent;
use lightspeed_file_store::repository::db::{DBFileStoreBinaryRepository, DBFileStoreRepositoryManager};
use std::borrow::Cow;

const SOURCE_FILE: &str = "./Cargo.toml";

#[test]
fn should_save_file_from_fs() -> Result<(), LightSpeedError> {
    test(async {
        let data = data(false).await;
        let repo_manager = &data.0.repo_manager;
        let file_store = repo_manager.file_store_binary_repo();
        let binary_content = BinaryContent::FromFs { file_path: SOURCE_FILE.to_owned().into() };
        let repository_name = &format!("repository_{}", rand::random::<u32>());
        let file_path = &format!("file_path_{}", rand::random::<u32>());

        repo_manager
            .c3p0()
            .transaction(|mut conn| async move {
                file_store.save_file(&mut conn, repository_name, file_path, &binary_content).await?;

                match file_store.read_file(&mut conn, repository_name, file_path).await {
                    Ok(BinaryContent::InMemory { content }) => {
                        let file_content = std::str::from_utf8(&content).unwrap();
                        assert_eq!(&std::fs::read_to_string(SOURCE_FILE).unwrap(), file_content);
                    }
                    _ => panic!(),
                }

                Ok(())
            })
            .await
    })
}

#[test]
fn should_save_file_from_memory() -> Result<(), LightSpeedError> {
    test(async {
        let data = data(false).await;
        let repo_manager = &data.0.repo_manager;
        let file_store = repo_manager.file_store_binary_repo();
        let binary_content = BinaryContent::InMemory { content: Cow::Borrowed("Hello world!".as_bytes()) };
        let repository_name = &format!("repository_{}", rand::random::<u32>());
        let file_path = &format!("file_path_{}", rand::random::<u32>());

        repo_manager
            .c3p0()
            .transaction(|mut conn| async move {
                file_store.save_file(&mut conn, repository_name, file_path, &binary_content).await?;

                match file_store.read_file(&mut conn, repository_name, file_path).await {
                    Ok(BinaryContent::InMemory { content }) => {
                        assert_eq!("Hello world!", String::from_utf8(content.into_owned()).unwrap());
                    }
                    _ => panic!(),
                }

                Ok(())
            })
            .await
    })
}

#[test]
fn save_file_should_fail_if_file_exists_in_same_repository() -> Result<(), LightSpeedError> {
    test(async {
        let data = data(false).await;
        let repo_manager = &data.0.repo_manager;
        let file_store = repo_manager.file_store_binary_repo();
        let binary_content = BinaryContent::FromFs { file_path: SOURCE_FILE.to_owned().into() };
        let repository_name = &format!("repository_{}", rand::random::<u32>());
        let file_path = &format!("file_path_{}", rand::random::<u32>());

        repo_manager
            .c3p0()
            .transaction(|mut conn| async move {
                file_store.save_file(&mut conn, repository_name, file_path, &binary_content).await?;
                assert!(file_store.save_file(&mut conn, repository_name, file_path, &binary_content).await.is_err());
                Ok(())
            })
            .await
    })
}

#[test]
fn save_file_not_should_fail_if_file_exists_in_different_repository() -> Result<(), LightSpeedError> {
    test(async {
        let data = data(false).await;
        let repo_manager = &data.0.repo_manager;
        let file_store = repo_manager.file_store_binary_repo();
        let binary_content = BinaryContent::FromFs { file_path: SOURCE_FILE.to_owned().into() };
        let repository_name_1 = &format!("repository_{}", rand::random::<u32>());
        let repository_name_2 = &format!("repository_{}", rand::random::<u32>());
        let file_path = &format!("file_path_{}", rand::random::<u32>());

        repo_manager
            .c3p0()
            .transaction(|mut conn| async move {
                file_store.save_file(&mut conn, repository_name_1, file_path, &binary_content).await?;
                assert!(file_store.save_file(&mut conn, repository_name_2, file_path, &binary_content).await.is_ok());
                Ok(())
            })
            .await
    })
}
