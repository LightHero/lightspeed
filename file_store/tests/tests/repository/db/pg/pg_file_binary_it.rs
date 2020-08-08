use crate::{data, test};
use c3p0::*;
use lightspeed_core::error::LightSpeedError;
use lightspeed_file_store::repository::db::{
    DBFileStoreBinaryRepository, DBFileStoreRepositoryManager,
};
use lightspeed_file_store::model::BinaryContent;

const SOURCE_FILE: &str = "./Cargo.toml";

#[test]
fn should_save_file_from_fs() -> Result<(), LightSpeedError> {
    test(async {
        let data = data(false).await;
        let repo_manager = &data.0.repo_manager;
        let file_store = repo_manager.file_store_binary_repo();
        let binary_content = BinaryContent::FromFs {file_path: SOURCE_FILE.to_owned()};

        repo_manager
            .c3p0()
            .transaction(|mut conn| async move {
                let id = file_store
                    .save_file(&mut conn, &binary_content)
                    .await?;

                match file_store.read_file(&mut conn, id).await {
                    Ok(BinaryContent::InMemory { content }) => {
                        let file_content = std::str::from_utf8(&content).unwrap();
                        assert_eq!(&std::fs::read_to_string(SOURCE_FILE).unwrap(), file_content);
                    }
                    _ => assert!(false),
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
        let binary_content = BinaryContent::InMemory {
            content: "Hello world!".to_owned().into_bytes()
        };

        repo_manager
            .c3p0()
            .transaction(|mut conn| async move {
                let id = file_store
                    .save_file(&mut conn, &binary_content)
                    .await?;

                match file_store.read_file(&mut conn, id).await {
                    Ok(BinaryContent::InMemory { content }) => {
                        assert_eq!("Hello world!", String::from_utf8(content).unwrap());
                    }
                    _ => assert!(false),
                }

                Ok(())
            })
            .await
    })
}

#[test]
fn save_file_not_should_fail_if_file_exists() -> Result<(), LightSpeedError> {
    test(async {
        let data = data(false).await;
        let repo_manager = &data.0.repo_manager;
        let file_store = repo_manager.file_store_binary_repo();
        let binary_content = BinaryContent::FromFs {file_path: SOURCE_FILE.to_owned()};

        repo_manager
            .c3p0()
            .transaction(|mut conn| async move {
                file_store
                    .save_file(&mut conn, &binary_content)
                    .await?;
                assert!(file_store
                    .save_file(&mut conn, &binary_content)
                    .await
                    .is_ok());
                Ok(())
            })
            .await
    })
}


