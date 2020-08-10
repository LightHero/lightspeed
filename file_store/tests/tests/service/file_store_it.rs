use crate::{data, test};
use c3p0::*;
use lightspeed_core::error::LightSpeedError;
use lightspeed_file_store::model::{BinaryContent, Repository, SaveRepository};
use lightspeed_file_store::repository::db::{
    DBFileStoreBinaryRepository, DBFileStoreRepositoryManager,
};
use std::path::Path;

const SOURCE_FILE: &str = "./Cargo.toml";

#[test]
fn should_save_file_to_db() -> Result<(), LightSpeedError> {
    test(async {
        let data = data(false).await;
        let file_store = &data.0.file_store_service;

        let random: u32 = rand::random();
        let file_name = format!("file_{}", random);
        let binary_content = BinaryContent::FromFs {
            file_path: SOURCE_FILE.to_owned(),
        };
        let content_type = "application/text".to_owned();
        let save_repository = SaveRepository::DB {
            file_path: None,
            repository_name: "MY_REPO".to_owned(),
        };

        let saved = file_store
            .save_file(
                file_name.clone(),
                content_type,
                &binary_content,
                save_repository,
            )
            .await?;

        let loaded = file_store.read_file_data_by_id(saved.id).await?;
        assert_eq!(loaded.data, saved.data);
        match &loaded.data.repository {
            Repository::DB {
                repository_name,
                file_path,
            } => {
                assert_eq!("MY_REPO", repository_name);
                assert_eq!(&file_name, file_path);
            }
            _ => assert!(false),
        }

        match file_store.read_file_content(&loaded.data.repository).await {
            Ok(BinaryContent::InMemory { content }) => {
                let file_content = std::str::from_utf8(&content).unwrap();
                assert_eq!(&std::fs::read_to_string(SOURCE_FILE).unwrap(), file_content);
            }
            _ => assert!(false),
        }

        Ok(())
    })
}

#[test]
fn should_save_file_to_fs() -> Result<(), LightSpeedError> {
    test(async {
        let data = data(false).await;
        let file_store = &data.0.file_store_service;

        let random: u32 = rand::random();
        let file_name = format!("file_{}", random);
        let binary_content = BinaryContent::FromFs {
            file_path: SOURCE_FILE.to_owned(),
        };
        let content_type = "application/text".to_owned();
        let save_repository = SaveRepository::FS {
            file_path: None,
            repository_name: "REPO_ONE".to_owned(),
        };

        let saved = file_store
            .save_file(
                file_name.clone(),
                content_type,
                &binary_content,
                save_repository,
            )
            .await?;

        let loaded = file_store.read_file_data_by_id(saved.id).await?;
        assert_eq!(loaded.data, saved.data);
        match &loaded.data.repository {
            Repository::FS {
                repository_name,
                file_path,
            } => {
                assert_eq!("REPO_ONE", repository_name);
                assert_eq!(&file_name, file_path);
            }
            _ => assert!(false),
        }

        println!("Data: [{:#?}]", loaded.data);

        match &file_store.read_file_content(&loaded.data.repository).await {
            Ok(BinaryContent::FromFs { file_path }) => {
                assert_eq!(&format!("../target/repo_one/{}", file_name), file_path);
                assert_eq!(
                    &std::fs::read_to_string(SOURCE_FILE).unwrap(),
                    &std::fs::read_to_string(file_path).unwrap()
                );
            }
            _ => assert!(false),
        }
        Ok(())
    })
}

#[test]
fn should_save_file_to_db_with_specific_repo() -> Result<(), LightSpeedError> {
    test(async {
        let data = data(false).await;
        let file_store = &data.0.file_store_service;

        let random: u32 = rand::random();
        let binary_content = BinaryContent::FromFs {
            file_path: SOURCE_FILE.to_owned(),
        };
        let content_type = "application/text".to_owned();
        let file_name_1 = format!("file_2_{}", random);
        let file_name_2 = format!("file_1_{}", random);
        let save_repository_1 = SaveRepository::DB {
            file_path: None,
            repository_name: "REPO_ONE".to_owned(),
        };

        let save_repository_2 = SaveRepository::DB {
            file_path: None,
            repository_name: "REPO_TWO".to_owned(),
        };

        let saved_1 = file_store
            .save_file(
                file_name_1.clone(),
                content_type.clone(),
                &binary_content,
                save_repository_1,
            )
            .await?;
        let saved_2 = file_store
            .save_file(
                file_name_2.clone(),
                content_type,
                &binary_content,
                save_repository_2,
            )
            .await?;

        match file_store.read_file_content(&saved_1.data.repository).await {
            Ok(BinaryContent::InMemory { content }) => {
                let file_content = std::str::from_utf8(&content).unwrap();
                assert_eq!(&std::fs::read_to_string(SOURCE_FILE).unwrap(), file_content);
            }
            _ => assert!(false),
        }

        match file_store.read_file_content(&saved_2.data.repository).await {
            Ok(BinaryContent::InMemory { content }) => {
                let file_content = std::str::from_utf8(&content).unwrap();
                assert_eq!(&std::fs::read_to_string(SOURCE_FILE).unwrap(), file_content);
            }
            _ => assert!(false),
        }

        Ok(())
    })
}

#[test]
fn should_save_file_to_fs_with_specific_repo() -> Result<(), LightSpeedError> {
    test(async {
        let data = data(false).await;
        let file_store = &data.0.file_store_service;

        let random: u32 = rand::random();
        let binary_content = BinaryContent::FromFs {
            file_path: SOURCE_FILE.to_owned(),
        };
        let content_type = "application/text".to_owned();
        let file_name_1 = format!("file_2_{}", random);
        let file_name_2 = format!("file_1_{}", random);
        let save_repository_1 = SaveRepository::FS {
            file_path: None,
            repository_name: "REPO_ONE".to_owned(),
        };

        let save_repository_2 = SaveRepository::FS {
            file_path: None,
            repository_name: "REPO_TWO".to_owned(),
        };

        let save_1 = file_store
            .save_file(
                file_name_1.clone(),
                content_type.clone(),
                &binary_content,
                save_repository_1,
            )
            .await?;
        let save_2 = file_store
            .save_file(
                file_name_2.clone(),
                content_type,
                &binary_content,
                save_repository_2,
            )
            .await?;

        match file_store.read_file_content(&save_1.data.repository).await {
            Ok(BinaryContent::FromFs { file_path }) => {
                assert_eq!(format!("../target/repo_one/{}", file_name_1), file_path);
            }
            _ => assert!(false),
        }

        match file_store.read_file_content(&save_2.data.repository).await {
            Ok(BinaryContent::FromFs { file_path }) => {
                assert_eq!(format!("../target/repo_two/{}", file_name_2), file_path);
            }
            _ => assert!(false),
        }

        Ok(())
    })
}

#[test]
fn save_should_fails_if_fs_repo_does_not_exist() -> Result<(), LightSpeedError> {
    test(async {
        let data = data(false).await;
        let file_store = &data.0.file_store_service;

        let random: u32 = rand::random();
        let binary_content = BinaryContent::FromFs {
            file_path: SOURCE_FILE.to_owned(),
        };
        let content_type = "application/text".to_owned();
        let file_name_1 = format!("file_2_{}", random);
        let save_repository_1 = SaveRepository::FS {
            file_path: None,
            repository_name: "REPO_NOT_EXISTING".to_owned(),
        };

        assert!(file_store
            .save_file(
                file_name_1.clone(),
                content_type.clone(),
                &binary_content,
                save_repository_1
            )
            .await
            .is_err());

        Ok(())
    })
}

#[test]
fn should_save_file_to_db_with_relative_folder() -> Result<(), LightSpeedError> {
    test(async {
        let data = data(false).await;
        let file_store = &data.0.file_store_service;

        let random: u32 = rand::random();
        let file_name = format!("relative/folder/file_{}", random);
        let binary_content = BinaryContent::FromFs {
            file_path: SOURCE_FILE.to_owned(),
        };
        let content_type = "application/text".to_owned();
        let save_repository = SaveRepository::DB {
            file_path: None,
            repository_name: "REPO_ONE".to_owned(),
        };

        let saved = file_store
            .save_file(
                file_name.clone(),
                content_type,
                &binary_content,
                save_repository,
            )
            .await?;

        let loaded = file_store.read_file_data_by_id(saved.id).await?;
        assert_eq!(loaded.data, saved.data);
        match &loaded.data.repository {
            Repository::DB {
                repository_name,
                file_path,
            } => {
                assert_eq!("REPO_ONE", repository_name);
                assert_eq!(&file_name, file_path);
            }
            _ => assert!(false),
        }

        match file_store.read_file_content(&saved.data.repository).await {
            Ok(BinaryContent::InMemory { content }) => {
                let file_content = std::str::from_utf8(&content).unwrap();
                assert_eq!(&std::fs::read_to_string(SOURCE_FILE).unwrap(), file_content);
            }
            _ => assert!(false),
        }

        Ok(())
    })
}

#[test]
fn should_save_file_to_fs_with_relative_folder() -> Result<(), LightSpeedError> {
    test(async {
        let data = data(false).await;
        let file_store = &data.0.file_store_service;

        let random: u32 = rand::random();
        let file_name = format!("relative/folder/file_{}", random);
        let binary_content = BinaryContent::FromFs {
            file_path: SOURCE_FILE.to_owned(),
        };
        let content_type = "application/text".to_owned();
        let save_repository = SaveRepository::FS {
            file_path: None,
            repository_name: "REPO_ONE".to_owned(),
        };

        let saved = file_store
            .save_file(
                file_name.clone(),
                content_type,
                &binary_content,
                save_repository,
            )
            .await?;

        let loaded = file_store.read_file_data_by_id(saved.id).await?;
        assert_eq!(loaded.data, saved.data);
        match &loaded.data.repository {
            Repository::FS {
                repository_name,
                file_path,
            } => {
                assert_eq!("REPO_ONE", repository_name);
                assert_eq!(&file_name, file_path);
            }
            _ => assert!(false),
        }

        match file_store.read_file_content(&saved.data.repository).await {
            Ok(BinaryContent::FromFs { file_path }) => {
                assert_eq!(format!("../target/repo_one/{}", file_name), file_path);
                assert_eq!(
                    &std::fs::read_to_string(SOURCE_FILE).unwrap(),
                    &std::fs::read_to_string(file_path).unwrap()
                );
            }
            _ => assert!(false),
        }

        Ok(())
    })
}

#[test]
fn should_save_file_to_db_with_relative_folder_in_repository() -> Result<(), LightSpeedError> {
    test(async {
        let data = data(false).await;
        let file_store = &data.0.file_store_service;

        let random: u32 = rand::random();
        let file_name = format!("folder/file_{}", random);
        let binary_content = BinaryContent::FromFs {
            file_path: SOURCE_FILE.to_owned(),
        };
        let content_type = "application/text".to_owned();
        let save_repository = SaveRepository::DB {
            file_path: Some("relative".to_owned()),
            repository_name: "REPO_ONE".to_owned(),
        };

        let saved = file_store
            .save_file(
                file_name.clone(),
                content_type,
                &binary_content,
                save_repository,
            )
            .await?;

        let loaded = file_store.read_file_data_by_id(saved.id).await?;
        assert_eq!(loaded.data, saved.data);
        match &loaded.data.repository {
            Repository::DB {
                repository_name,
                file_path,
            } => {
                assert_eq!("REPO_ONE", repository_name);
                assert_eq!(&format!("relative/{}", file_name), file_path);
            }
            _ => assert!(false),
        }

        match file_store.read_file_content(&saved.data.repository).await {
            Ok(BinaryContent::InMemory { content }) => {
                let file_content = std::str::from_utf8(&content).unwrap();
                assert_eq!(&std::fs::read_to_string(SOURCE_FILE).unwrap(), file_content);
            }
            _ => assert!(false),
        }

        Ok(())
    })
}

#[test]
fn should_save_file_to_fs_with_relative_folder_in_repository() -> Result<(), LightSpeedError> {
    test(async {
        let data = data(false).await;
        let file_store = &data.0.file_store_service;

        let random: u32 = rand::random();
        let file_name = format!("folder/file_{}", random);
        let binary_content = BinaryContent::FromFs {
            file_path: SOURCE_FILE.to_owned(),
        };
        let content_type = "application/text".to_owned();
        let save_repository = SaveRepository::FS {
            file_path: Some("relative".to_owned()),
            repository_name: "REPO_ONE".to_owned(),
        };

        let saved = file_store
            .save_file(
                file_name.clone(),
                content_type,
                &binary_content,
                save_repository,
            )
            .await?;

        let loaded = file_store.read_file_data_by_id(saved.id).await?;
        assert_eq!(loaded.data, saved.data);
        match &loaded.data.repository {
            Repository::FS {
                repository_name,
                file_path,
            } => {
                assert_eq!("REPO_ONE", repository_name);
                assert_eq!(&format!("relative/{}", file_name), file_path);
            }
            _ => assert!(false),
        }

        match file_store.read_file_content(&saved.data.repository).await {
            Ok(BinaryContent::FromFs { file_path }) => {
                assert_eq!(
                    format!("../target/repo_one/relative/{}", file_name),
                    file_path
                );
                assert_eq!(
                    &std::fs::read_to_string(SOURCE_FILE).unwrap(),
                    &std::fs::read_to_string(file_path).unwrap()
                );
            }
            _ => assert!(false),
        }

        Ok(())
    })
}

#[test]
fn should_delete_file_from_db() -> Result<(), LightSpeedError> {
    test(async {
        let data = data(false).await;
        let file_store = &data.0.file_store_service;

        let db_file_binary_repo = &data.0.repo_manager.file_store_binary_repo();

        let random: u32 = rand::random();
        let file_name = format!("file_{}", random);
        let binary_content = BinaryContent::FromFs {
            file_path: SOURCE_FILE.to_owned(),
        };
        let content_type = "application/text".to_owned();
        let save_repository = SaveRepository::DB {
            file_path: Some("relative".to_owned()),
            repository_name: "REPO_ONE".to_owned(),
        };

        let saved = file_store
            .save_file(file_name, content_type, &binary_content, save_repository)
            .await?;

        let (repository_name, file_path) = match &saved.data.repository {
            Repository::DB {
                repository_name,
                file_path,
            } => (repository_name.as_str(), file_path.as_str()),
            _ => {
                assert!(false);
                ("", "")
            }
        };

        data.0
            .repo_manager
            .c3p0()
            .transaction::<_, LightSpeedError, _, _>(|mut conn| async move {
                assert!(db_file_binary_repo
                    .read_file(&mut conn, repository_name, file_path)
                    .await
                    .is_ok());
                Ok(())
            })
            .await
            .unwrap();

        assert_eq!(1, file_store.delete_file_by_id(saved.id).await?);
        assert!(file_store.read_file_data_by_id(saved.id).await.is_err());

        data.0
            .repo_manager
            .c3p0()
            .transaction::<_, LightSpeedError, _, _>(|mut conn| async move {
                assert!(db_file_binary_repo
                    .read_file(&mut conn, repository_name, file_path)
                    .await
                    .is_err());
                Ok(())
            })
            .await
            .unwrap();

        Ok(())
    })
}

#[test]
fn should_delete_file_from_fs() -> Result<(), LightSpeedError> {
    test(async {
        let data = data(false).await;
        let file_store = &data.0.file_store_service;

        let random: u32 = rand::random();
        let file_name = format!("relative/folder/file_{}", random);
        let binary_content = BinaryContent::FromFs {
            file_path: SOURCE_FILE.to_owned(),
        };
        let content_type = "application/text".to_owned();
        let save_repository = SaveRepository::FS {
            file_path: None,
            repository_name: "REPO_ONE".to_owned(),
        };

        let saved = file_store
            .save_file(
                file_name.clone(),
                content_type,
                &binary_content,
                save_repository,
            )
            .await?;

        let file_path = match &saved.data.repository {
            Repository::FS { file_path, .. } => format!("../target/repo_one/{}", file_path),
            _ => {
                assert!(false);
                "".to_owned()
            }
        };

        println!("file_path: {}", file_path);

        assert!(Path::new(&file_path).exists());

        assert_eq!(1, file_store.delete_file_by_id(saved.id).await?);
        assert!(file_store.read_file_data_by_id(saved.id).await.is_err());

        assert!(!Path::new(&file_path).exists());

        Ok(())
    })
}
