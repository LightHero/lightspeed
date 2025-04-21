use crate::data;
use c3p0::*;
use lightspeed_core::error::LsError;
use lightspeed_file_store::model::BinaryContent;
use lightspeed_file_store::repository::db::{DBFileStoreBinaryRepository, DBFileStoreRepositoryManager};
use opendal::Operator;
use opendal::services::Fs;
use std::path::{Path, PathBuf};
use test_utils::tokio_test;

const SOURCE_FILE: &str = "./Cargo.toml";

#[test]
fn should_save_file_to_db() -> Result<(), LsError> {
    tokio_test(async {
        let data = data(false).await;
        let file_store = &data.0.file_store_service;

        let random: u32 = rand::random();
        let file_name = format!("file_{random}");
        let operator = Operator::new(Fs::default().root("./")).unwrap().finish().into();
        let binary_content = BinaryContent::OpenDal { operator, path: SOURCE_FILE.to_owned() };
        let content_type = "application/text".to_owned();
        let save_repository = "DB_ONE";

        let saved = file_store
            .save_file(save_repository.to_owned(), file_name.clone(), file_name.clone(), content_type, &binary_content)
            .await?;

        let loaded = file_store.read_file_data_by_id(saved.id).await?;
        assert_eq!(loaded.data, saved.data);
        assert_eq!("DB_ONE", loaded.data.repository);
        assert_eq!(&file_name, &loaded.data.file_path);

        match file_store.read_file_content(&loaded.data.repository, &loaded.data.file_path).await {
            Ok(BinaryContent::InMemory { content }) => {
                let file_content = std::str::from_utf8(&content).unwrap();
                assert_eq!(&std::fs::read_to_string(SOURCE_FILE).unwrap(), file_content);
            }
            _ => panic!(),
        }

        Ok(())
    })
}

#[test]
fn should_save_file_to_fs() -> Result<(), LsError> {
    tokio_test(async {
        let data = data(false).await;
        let file_store = &data.0.file_store_service;

        let random: u32 = rand::random();
        let file_name = format!("file_{random}");
        let operator = Operator::new(Fs::default().root("./")).unwrap().finish().into();
        let binary_content = BinaryContent::OpenDal { operator, path: SOURCE_FILE.to_owned() };
        let content_type = "application/text".to_owned();
        let save_repository = "FS_TWO";

        let saved = file_store
            .save_file(save_repository.to_owned(), file_name.clone(), file_name.clone(), content_type, &binary_content)
            .await?;

        let loaded = file_store.read_file_data_by_id(saved.id).await?;
        assert_eq!(loaded.data, saved.data);
        assert_eq!(loaded.data, saved.data);
        assert_eq!("FS_TWO", loaded.data.repository);
        assert_eq!(&file_name, &loaded.data.file_path);

        println!("Data: [{:#?}]", loaded.data);

        let read_content = file_store
            .read_file_content(&loaded.data.repository, &loaded.data.file_path)
            .await
            .unwrap()
            .read()
            .await
            .unwrap();
        assert_eq!(read_content.as_ref(), &std::fs::read(SOURCE_FILE).unwrap());

        Ok(())
    })
}

#[test]
fn should_save_file_to_db_with_specific_repo() -> Result<(), LsError> {
    tokio_test(async {
        let data = data(false).await;
        let file_store = &data.0.file_store_service;

        let random: u32 = rand::random();
        let operator = Operator::new(Fs::default().root("./")).unwrap().finish().into();
        let binary_content = BinaryContent::OpenDal { operator, path: SOURCE_FILE.to_owned() };
        let content_type = "application/text".to_owned();
        let file_name_1 = format!("file_2_{random}");
        let file_name_2 = format!("file_1_{random}");

        let save_repository_1 = "DB_ONE";

        let save_repository_2 = "DB_TWO";

        let saved_1 = file_store
            .save_file(
                save_repository_1.to_owned(),
                file_name_1.clone(),
                file_name_1.clone(),
                content_type.clone(),
                &binary_content,
            )
            .await?;
        let saved_2 = file_store
            .save_file(
                save_repository_2.to_owned(),
                file_name_2.clone(),
                file_name_2.clone(),
                content_type,
                &binary_content,
            )
            .await?;

        match file_store.read_file_content(&saved_1.data.repository, &saved_1.data.file_path).await {
            Ok(BinaryContent::InMemory { content }) => {
                let file_content = std::str::from_utf8(&content).unwrap();
                assert_eq!(&std::fs::read_to_string(SOURCE_FILE).unwrap(), file_content);
            }
            _ => panic!(),
        }

        match file_store.read_file_content(&saved_2.data.repository, &saved_2.data.file_path).await {
            Ok(BinaryContent::InMemory { content }) => {
                let file_content = std::str::from_utf8(&content).unwrap();
                assert_eq!(&std::fs::read_to_string(SOURCE_FILE).unwrap(), file_content);
            }
            _ => panic!(),
        }

        println!("{:#?}", serde_json::to_string(&saved_1).unwrap());

        let loaded_data_by_repository_1 =
            file_store.read_file_data_by_repository(&saved_1.data.repository, &saved_1.data.file_path).await.unwrap();
        assert_eq!(saved_1, loaded_data_by_repository_1);

        let loaded_data_by_repository_2 =
            file_store.read_file_data_by_repository(&saved_2.data.repository, &saved_2.data.file_path).await.unwrap();
        assert_eq!(saved_2, loaded_data_by_repository_2);

        Ok(())
    })
}

#[test]
fn should_save_file_to_fs_with_specific_repo() -> Result<(), LsError> {
    tokio_test(async {
        let data = data(false).await;
        let file_store = &data.0.file_store_service;

        let random: u32 = rand::random();
        let operator = Operator::new(Fs::default().root("./")).unwrap().finish().into();
        let binary_content = BinaryContent::OpenDal { operator, path: SOURCE_FILE.to_owned() };
        let content_type = "application/text".to_owned();
        let file_name_1 = format!("file_2_{random}");
        let file_name_2 = format!("file_1_{random}");
        let save_repository_1 = "FS_ONE";

        let save_repository_2 = "FS_TWO";

        let save_1 = file_store
            .save_file(
                save_repository_1.to_owned(),
                file_name_1.clone(),
                file_name_1.clone(),
                content_type.clone(),
                &binary_content,
            )
            .await?;
        let save_2 = file_store
            .save_file(
                save_repository_2.to_owned(),
                file_name_2.clone(),
                file_name_2.clone(),
                content_type,
                &binary_content,
            )
            .await?;

        match file_store.read_file_content(&save_1.data.repository, &save_1.data.file_path).await {
            Ok(BinaryContent::OpenDal { operator: _, path }) => {
                assert_eq!(file_name_1, path);
                assert!(PathBuf::from(format!("../target/repo_one/{file_name_1}")).exists());
            }
            _ => panic!(),
        }

        match file_store.read_file_content(&save_2.data.repository, &save_2.data.file_path).await {
            Ok(BinaryContent::OpenDal { operator: _, path }) => {
                assert_eq!(file_name_2, path);
                assert!(PathBuf::from(format!("../target/repo_two/{file_name_2}")).exists());
            }
            _ => panic!(),
        }

        let loaded_data_by_repository_1 =
            file_store.read_file_data_by_repository(&save_1.data.repository, &save_1.data.file_path).await.unwrap();
        assert_eq!(save_1, loaded_data_by_repository_1);

        let loaded_data_by_repository_2 =
            file_store.read_file_data_by_repository(&save_2.data.repository, &save_2.data.file_path).await.unwrap();
        assert_eq!(save_2, loaded_data_by_repository_2);

        Ok(())
    })
}

#[test]
fn save_should_fails_if_fs_repo_does_not_exist() -> Result<(), LsError> {
    tokio_test(async {
        let data = data(false).await;
        let file_store = &data.0.file_store_service;

        let random: u32 = rand::random();
        let operator = Operator::new(Fs::default().root("./")).unwrap().finish().into();
        let binary_content = BinaryContent::OpenDal { operator, path: SOURCE_FILE.to_owned() };
        let content_type = "application/text".to_owned();
        let file_name_1 = format!("file_2_{random}");
        let save_repository_1 = "REPO_NOT_EXISTING";

        assert!(
            file_store
                .save_file(
                    save_repository_1.to_owned(),
                    file_name_1.clone(),
                    file_name_1.clone(),
                    content_type.clone(),
                    &binary_content
                )
                .await
                .is_err()
        );

        Ok(())
    })
}

#[test]
fn should_save_file_to_db_with_relative_folder() -> Result<(), LsError> {
    tokio_test(async {
        let data = data(false).await;
        let file_store = &data.0.file_store_service;

        let random: u32 = rand::random();
        let file_name = format!("relative/folder/file_{random}");
        let operator = Operator::new(Fs::default().root("./")).unwrap().finish().into();
        let binary_content = BinaryContent::OpenDal { operator, path: SOURCE_FILE.to_owned() };
        let content_type = "application/text".to_owned();
        let save_repository = "DB_ONE";

        let saved = file_store
            .save_file(save_repository.to_owned(), file_name.clone(), file_name.clone(), content_type, &binary_content)
            .await?;

        let loaded = file_store.read_file_data_by_id(saved.id).await?;
        assert_eq!(loaded.data, saved.data);
        assert_eq!("DB_ONE", &saved.data.repository);
        assert_eq!(&file_name, &saved.data.file_path);

        match file_store.read_file_content(&saved.data.repository, &saved.data.file_path).await {
            Ok(BinaryContent::InMemory { content }) => {
                let file_content = std::str::from_utf8(&content).unwrap();
                assert_eq!(&std::fs::read_to_string(SOURCE_FILE).unwrap(), file_content);
            }
            _ => panic!(),
        }

        Ok(())
    })
}

#[test]
fn should_save_file_to_fs_with_relative_folder() -> Result<(), LsError> {
    tokio_test(async {
        let data = data(false).await;
        let file_store = &data.0.file_store_service;

        let random: u32 = rand::random();
        let file_name = format!("relative/folder/file_{random}");
        let operator = Operator::new(Fs::default().root("./")).unwrap().finish().into();
        let binary_content = BinaryContent::OpenDal { operator, path: SOURCE_FILE.to_owned() };
        let content_type = "application/text".to_owned();
        let save_repository = "FS_ONE";

        let saved = file_store
            .save_file(save_repository.to_owned(), file_name.clone(), file_name.clone(), content_type, &binary_content)
            .await?;

        let loaded = file_store.read_file_data_by_id(saved.id).await?;
        assert_eq!(loaded.data, saved.data);
        assert_eq!("FS_ONE", loaded.data.repository);
        assert_eq!(&file_name, &loaded.data.file_path);

        match file_store.read_file_content(&saved.data.repository, &loaded.data.file_path).await {
            Ok(BinaryContent::OpenDal { operator, path }) => {
                let dest: Vec<u8> = operator.read(&path).await.unwrap().to_vec();
                assert_eq!(&std::fs::read(SOURCE_FILE).unwrap(), &dest);
            }
            _ => panic!(),
        }

        Ok(())
    })
}

#[test]
fn should_save_file_to_db_with_relative_folder_in_repository() -> Result<(), LsError> {
    tokio_test(async {
        let data = data(false).await;
        let file_store = &data.0.file_store_service;

        let random: u32 = rand::random();
        let file_name = format!("folder/file_{random}");
        let operator = Operator::new(Fs::default().root("./")).unwrap().finish().into();
        let binary_content = BinaryContent::OpenDal { operator, path: SOURCE_FILE.to_owned() };
        let content_type = "application/text".to_owned();
        let save_repository = "DB_ONE";
        let file_path = format!("relative/{file_name}");

        let saved = file_store
            .save_file(save_repository.to_owned(), file_path.clone(), file_name.clone(), content_type, &binary_content)
            .await?;

        let loaded = file_store.read_file_data_by_id(saved.id).await?;
        assert_eq!(loaded.data, saved.data);
        assert_eq!("DB_ONE", loaded.data.repository);
        assert_eq!(file_path, loaded.data.file_path);

        match file_store.read_file_content(&saved.data.repository, &loaded.data.file_path).await {
            Ok(BinaryContent::InMemory { content }) => {
                let file_content = std::str::from_utf8(&content).unwrap();
                assert_eq!(&std::fs::read_to_string(SOURCE_FILE).unwrap(), file_content);
            }
            _ => panic!(),
        }

        Ok(())
    })
}

#[test]
fn should_save_file_to_fs_with_relative_folder_in_repository() -> Result<(), LsError> {
    tokio_test(async {
        let data = data(false).await;
        let file_store = &data.0.file_store_service;

        let random: u32 = rand::random();
        let file_name = format!("folder/file_{random}");
        let operator = Operator::new(Fs::default().root("./")).unwrap().finish().into();
        let binary_content = BinaryContent::OpenDal { operator, path: SOURCE_FILE.to_owned() };
        let content_type = "application/text".to_owned();
        let save_repository = "FS_ONE";
        let file_path = format!("relative/{file_name}");

        let saved = file_store
            .save_file(save_repository.to_owned(), file_path.clone(), file_name.clone(), content_type, &binary_content)
            .await?;

        let loaded = file_store.read_file_data_by_id(saved.id).await?;
        assert_eq!(loaded.data, saved.data);
        assert_eq!("FS_ONE", loaded.data.repository);
        assert_eq!(file_path, loaded.data.file_path);

        match file_store.read_file_content(&saved.data.repository, &loaded.data.file_path).await {
            Ok(BinaryContent::OpenDal { operator, path }) => {
                let dest: Vec<u8> = operator.read(&path).await.unwrap().to_vec();
                assert_eq!(&std::fs::read(SOURCE_FILE).unwrap(), &dest);
            }
            _ => panic!(),
        }

        Ok(())
    })
}

#[test]
fn should_delete_file_from_db() -> Result<(), LsError> {
    tokio_test(async {
        let data = data(false).await;
        let file_store = &data.0.file_store_service;

        let db_file_binary_repo = &data.0.repo_manager.file_store_binary_repo();

        let random: u32 = rand::random();
        let file_name = format!("file_{random}");
        let operator = Operator::new(Fs::default().root("./")).unwrap().finish().into();
        let binary_content = BinaryContent::OpenDal { operator, path: SOURCE_FILE.to_owned() };
        let content_type = "application/text".to_owned();
        let save_repository = "DB_ONE";
        let file_path = format!("relative/{file_name}");

        let saved = file_store
            .save_file(save_repository.to_owned(), file_path.clone(), file_name, content_type, &binary_content)
            .await?;

        data.0
            .repo_manager
            .c3p0()
            .transaction::<_, LsError, _>(async |conn| {
                assert!(db_file_binary_repo.read_file(conn, &save_repository, &file_path).await.is_ok());
                Ok(())
            })
            .await
            .unwrap();

        file_store.delete_file_by_id(saved.id).await?;
        assert!(file_store.read_file_data_by_id(saved.id).await.is_err());

        data.0
            .repo_manager
            .c3p0()
            .transaction::<_, LsError, _>(async |conn| {
                assert!(db_file_binary_repo.read_file(conn, &save_repository, &file_path).await.is_err());
                Ok(())
            })
            .await
            .unwrap();

        Ok(())
    })
}

#[test]
fn should_delete_file_from_fs() -> Result<(), LsError> {
    tokio_test(async {
        let data = data(false).await;
        let file_store = &data.0.file_store_service;

        let random: u32 = rand::random();
        let file_name = format!("file_{random}");
        let operator = Operator::new(Fs::default().root("./")).unwrap().finish().into();
        let binary_content = BinaryContent::OpenDal { operator, path: SOURCE_FILE.to_owned() };
        let content_type = "application/text".to_owned();
        let save_repository = "FS_ONE";
        let file_path = format!("relative/{file_name}");

        let saved = file_store
            .save_file(save_repository.to_owned(), file_path.clone(), file_name.clone(), content_type, &binary_content)
            .await?;

        let file_full_path = format!("../target/repo_one/{file_path}");

        assert!(Path::new(&file_full_path).exists());

        file_store.delete_file_by_id(saved.id).await?;
        assert!(file_store.read_file_data_by_id(saved.id).await.is_err());

        assert!(!Path::new(&file_full_path).exists());

        Ok(())
    })
}

#[test]
fn should_allow_files_with_same_path_but_different_repository() -> Result<(), LsError> {
    tokio_test(async {
        let data = data(false).await;
        let file_store = &data.0.file_store_service;

        // Arrange
        let random: u32 = rand::random();
        let same_file_name = format!("folder/file_{random}");
        let operator = Operator::new(Fs::default().root("./")).unwrap().finish().into();
        let binary_content = BinaryContent::OpenDal { operator, path: SOURCE_FILE.to_owned() };

        let content_type = "application/text".to_owned();

        let repository_db_1 = "DB_ONE";
        let repository_db_2 = "DB_TWO";
        let repository_fs_1 = "FS_ONE";
        let repository_fs_2 = "FS_TWO";

        // Act
        let saved_db_1 = file_store
            .save_file(
                repository_db_1.to_owned(),
                same_file_name.clone(),
                same_file_name.clone(),
                content_type.clone(),
                &binary_content,
            )
            .await?;

        let saved_db_2 = file_store
            .save_file(
                repository_db_2.to_owned(),
                same_file_name.clone(),
                same_file_name.clone(),
                content_type.clone(),
                &binary_content,
            )
            .await?;

        let saved_fs_1 = file_store
            .save_file(
                repository_fs_1.to_owned(),
                same_file_name.clone(),
                same_file_name.clone(),
                content_type.clone(),
                &binary_content,
            )
            .await?;

        let saved_fs_2 = file_store
            .save_file(
                repository_fs_2.to_owned(),
                same_file_name.clone(),
                same_file_name.clone(),
                content_type.clone(),
                &binary_content,
            )
            .await?;

        // Assert
        assert!(file_store.read_file_data_by_id(saved_db_1.id).await.is_ok());
        assert!(file_store.read_file_data_by_id(saved_db_2.id).await.is_ok());
        assert!(file_store.read_file_data_by_id(saved_fs_1.id).await.is_ok());
        assert!(file_store.read_file_data_by_id(saved_fs_2.id).await.is_ok());

        Ok(())
    })
}

#[test]
fn should_fail_if_file_already_exists_in_db() -> Result<(), LsError> {
    tokio_test(async {
        let data = data(false).await;
        let file_store = &data.0.file_store_service;

        // Arrange
        let random: u32 = rand::random();
        let same_file_name = format!("folder/file_{random}");
        let same_file_path = format!("path_{random}");
        let same_repository_name = "DB_ONE";
        let operator = Operator::new(Fs::default().root("./")).unwrap().finish().into();
        let binary_content = BinaryContent::OpenDal { operator, path: SOURCE_FILE.to_owned() };

        let content_type = "application/text".to_owned();

        // Act & Assert
        assert!(
            file_store
                .save_file(
                    same_repository_name.to_owned(),
                    same_file_path.clone(),
                    same_file_name.clone(),
                    content_type.clone(),
                    &binary_content.clone(),
                )
                .await
                .is_ok()
        );

        // fail if file already exists
        assert!(
            file_store
                .save_file(
                    same_repository_name.to_owned(),
                    same_file_path.clone(),
                    same_file_name.clone(),
                    content_type.clone(),
                    &binary_content.clone(),
                )
                .await
                .is_err()
        );

        // success if file has different path
        assert!(
            file_store
                .save_file(
                    same_repository_name.to_owned(),
                    format!("{same_file_name}-1"),
                    same_file_name.clone(),
                    content_type.clone(),
                    &binary_content.clone(),
                )
                .await
                .is_ok()
        );

        // success if file has different repository_name
        assert!(
            file_store
                .save_file(
                    "DB_TWO".to_owned(),
                    same_file_path.clone(),
                    same_file_name.clone(),
                    content_type.clone(),
                    &binary_content.clone(),
                )
                .await
                .is_ok()
        );

        Ok(())
    })
}

#[test]
fn should_fail_if_file_already_exists_in_fs() -> Result<(), LsError> {
    tokio_test(async {
        let data = data(false).await;
        let file_store = &data.0.file_store_service;

        // Arrange
        let random: u32 = rand::random();
        let same_file_name = format!("folder/file_{random}");
        let same_file_path = format!("path_{random}");
        let same_repository_name = "FS_ONE";

        let operator = Operator::new(Fs::default().root("./")).unwrap().finish().into();
        let binary_content = BinaryContent::OpenDal { operator, path: SOURCE_FILE.to_owned() };

        let content_type = "application/text".to_owned();

        // Act & Assert
        assert!(
            file_store
                .save_file(
                    same_repository_name.to_owned(),
                    same_file_path.clone(),
                    same_file_name.clone(),
                    content_type.clone(),
                    &binary_content.clone(),
                )
                .await
                .is_ok()
        );

        // fail if file already exists
        assert!(
            file_store
                .save_file(
                    same_repository_name.to_owned(),
                    same_file_path.clone(),
                    same_file_name.clone(),
                    content_type.clone(),
                    &binary_content.clone(),
                )
                .await
                .is_err()
        );

        // success if file has different path
        assert!(
            file_store
                .save_file(
                    same_repository_name.to_owned(),
                    format!("{same_file_name}-1"),
                    same_file_name.clone(),
                    content_type.clone(),
                    &binary_content.clone(),
                )
                .await
                .is_ok()
        );

        // success if file has different repository_name
        assert!(
            file_store
                .save_file(
                    "FS_TWO".to_owned(),
                    same_file_path.clone(),
                    same_file_name.clone(),
                    content_type.clone(),
                    &binary_content.clone(),
                )
                .await
                .is_ok()
        );

        Ok(())
    })
}

#[test]
fn should_read_all_file_data_by_repository() -> Result<(), LsError> {
    tokio_test(async {
        let data = data(false).await;
        let file_store = &data.0.file_store_service;

        let random: u32 = rand::random();
        let operator = Operator::new(Fs::default().root("./")).unwrap().finish().into();
        let binary_content = BinaryContent::OpenDal { operator, path: SOURCE_FILE.to_owned() };
        let content_type = "application/text".to_owned();
        let file_name_1 = format!("file_1_{random}");
        let file_name_2 = format!("file_2_{random}");
        let file_name_3 = format!("file_3_{random}");

        let save_repository = "DB_ONE";

        file_store
            .save_file(
                save_repository.to_owned(),
                file_name_1.clone(),
                file_name_1.clone(),
                content_type.clone(),
                &binary_content,
            )
            .await?;

        let all_repo_files =
            file_store.read_all_file_data_by_repository(&save_repository, 0, 100, &OrderBy::Asc).await.unwrap();
        assert!(all_repo_files.len() >= 1);

        file_store
            .save_file(
                save_repository.to_owned(),
                file_name_2.clone(),
                file_name_2.clone(),
                content_type.clone(),
                &binary_content,
            )
            .await?;

        let all_repo_files =
            file_store.read_all_file_data_by_repository(&save_repository, 0, 2, &OrderBy::Asc).await.unwrap();
        assert_eq!(all_repo_files.len(), 2);

        file_store
            .save_file(
                save_repository.to_owned(),
                file_name_3.clone(),
                file_name_3.clone(),
                content_type,
                &binary_content,
            )
            .await?;

        let all_repo_files =
            file_store.read_all_file_data_by_repository(&save_repository, 0, 10000, &OrderBy::Asc).await.unwrap();
        assert!(all_repo_files.len() >= 3);

        assert!(all_repo_files.iter().any(|file| file.data.filename == file_name_1));
        assert!(all_repo_files.iter().any(|file| file.data.filename == file_name_2));
        assert!(all_repo_files.iter().any(|file| file.data.filename == file_name_3));

        let all_repo_files =
            file_store.read_all_file_data_by_repository(&save_repository, 1, 1, &OrderBy::Asc).await.unwrap();
        assert_eq!(1, all_repo_files.len());

        Ok(())
    })
}

#[test]
fn should_return_if_file_exists_by_repository() -> Result<(), LsError> {
    tokio_test(async {
        let data = data(false).await;
        let file_store = &data.0.file_store_service;

        // Arrange
        let random: u32 = rand::random();
        let file_name = format!("file_{random}");
        let operator = Operator::new(Fs::default().root("./")).unwrap().finish().into();
        let binary_content = BinaryContent::OpenDal { operator, path: SOURCE_FILE.to_owned() };
        let content_type = "application/text".to_owned();
        let save_repository = "FS_ONE";

        let saved = file_store
            .save_file(save_repository.to_owned(), file_name.clone(), file_name, content_type, &binary_content)
            .await?;

        // Act & Assert
        assert!(file_store.exists_by_repository(&saved.data.repository, &saved.data.file_path).await.unwrap());

        file_store.delete_file_by_id(saved.id).await?;
        assert!(!file_store.exists_by_repository(&saved.data.repository, &saved.data.file_path).await.unwrap());

        Ok(())
    })
}
