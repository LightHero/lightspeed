use crate::{data, test};
use c3p0::*;
use lightspeed_core::error::LsError;
use lightspeed_core::utils::new_hyphenated_uuid;
use lightspeed_file_store::model::{BinaryContent, Repository, RepositoryFile, SaveRepository};
use lightspeed_file_store::repository::db::{DBFileStoreBinaryRepository, DBFileStoreRepositoryManager};
use std::path::{Path, PathBuf};

const SOURCE_FILE: &str = "./Cargo.toml";

#[test]
fn should_save_file_to_db() -> Result<(), LsError> {
    test(async {
        let data = data(false).await;
        let file_store = &data.0.file_store_service;

        let random: u32 = rand::random();
        let file_name = format!("file_{random}");
        let binary_content = BinaryContent::FromFs { file_path: SOURCE_FILE.to_owned().into() };
        let content_type = "application/text".to_owned();
        let save_repository = SaveRepository::DB { subfolder: None, repository_name: "MY_REPO".to_owned() };

        let saved = file_store.save_file(file_name.clone(), content_type, &binary_content, save_repository).await?;

        let loaded = file_store.read_file_data_by_id(saved.id).await?;
        assert_eq!(loaded.data, saved.data);
        match &loaded.data.repository {
            RepositoryFile::DB { repository_name, file_path } => {
                assert_eq!("MY_REPO", repository_name);
                assert_eq!(&file_name, file_path);
            }
            _ => panic!(),
        }

        match file_store.read_file_content(&loaded.data.repository).await {
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
    test(async {
        let data = data(false).await;
        let file_store = &data.0.file_store_service;

        let random: u32 = rand::random();
        let file_name = format!("file_{random}");
        let binary_content = BinaryContent::FromFs { file_path: SOURCE_FILE.to_owned().into() };
        let content_type = "application/text".to_owned();
        let save_repository = SaveRepository::FS { subfolder: None, repository_name: "REPO_ONE".to_owned() };

        let saved = file_store.save_file(file_name.clone(), content_type, &binary_content, save_repository).await?;

        let loaded = file_store.read_file_data_by_id(saved.id).await?;
        assert_eq!(loaded.data, saved.data);
        match &loaded.data.repository {
            RepositoryFile::FS { repository_name, file_path } => {
                assert_eq!("REPO_ONE", repository_name);
                assert_eq!(&file_name, file_path);
            }
            _ => panic!(),
        }

        println!("Data: [{:#?}]", loaded.data);

        match &file_store.read_file_content(&loaded.data.repository).await {
            Ok(BinaryContent::FromFs { file_path }) => {
                assert_eq!(&PathBuf::from(format!("../target/repo_one/{file_name}")), file_path);
                assert_eq!(
                    &std::fs::read_to_string(SOURCE_FILE).unwrap(),
                    &std::fs::read_to_string(file_path).unwrap()
                );
            }
            _ => panic!(),
        }
        Ok(())
    })
}

#[test]
fn should_save_file_to_db_with_specific_repo() -> Result<(), LsError> {
    test(async {
        let data = data(false).await;
        let file_store = &data.0.file_store_service;

        let random: u32 = rand::random();
        let binary_content = BinaryContent::FromFs { file_path: SOURCE_FILE.to_owned().into() };
        let content_type = "application/text".to_owned();
        let file_name_1 = format!("file_2_{random}");
        let file_name_2 = format!("file_1_{random}");

        let save_repository_1 = SaveRepository::DB { subfolder: None, repository_name: "REPO_ONE".to_owned() };

        let save_repository_2 = SaveRepository::DB { subfolder: None, repository_name: "REPO_TWO".to_owned() };

        let saved_1 =
            file_store.save_file(file_name_1.clone(), content_type.clone(), &binary_content, save_repository_1).await?;
        let saved_2 =
            file_store.save_file(file_name_2.clone(), content_type, &binary_content, save_repository_2).await?;

        match file_store.read_file_content(&saved_1.data.repository).await {
            Ok(BinaryContent::InMemory { content }) => {
                let file_content = std::str::from_utf8(&content).unwrap();
                assert_eq!(&std::fs::read_to_string(SOURCE_FILE).unwrap(), file_content);
            }
            _ => panic!(),
        }

        match file_store.read_file_content(&saved_2.data.repository).await {
            Ok(BinaryContent::InMemory { content }) => {
                let file_content = std::str::from_utf8(&content).unwrap();
                assert_eq!(&std::fs::read_to_string(SOURCE_FILE).unwrap(), file_content);
            }
            _ => panic!(),
        }

        println!("{:#?}", serde_json::to_string(&saved_1).unwrap());

        let loaded_data_by_repository_1 =
            file_store.read_file_data_by_repository(&saved_1.data.repository).await.unwrap();
        assert_eq!(saved_1, loaded_data_by_repository_1);

        let loaded_data_by_repository_2 =
            file_store.read_file_data_by_repository(&saved_2.data.repository).await.unwrap();
        assert_eq!(saved_2, loaded_data_by_repository_2);

        Ok(())
    })
}

#[test]
fn should_save_file_to_fs_with_specific_repo() -> Result<(), LsError> {
    test(async {
        let data = data(false).await;
        let file_store = &data.0.file_store_service;

        let random: u32 = rand::random();
        let binary_content = BinaryContent::FromFs { file_path: SOURCE_FILE.to_owned().into() };
        let content_type = "application/text".to_owned();
        let file_name_1 = format!("file_2_{random}");
        let file_name_2 = format!("file_1_{random}");
        let save_repository_1 = SaveRepository::FS { subfolder: None, repository_name: "REPO_ONE".to_owned() };

        let save_repository_2 = SaveRepository::FS { subfolder: None, repository_name: "REPO_TWO".to_owned() };

        let save_1 =
            file_store.save_file(file_name_1.clone(), content_type.clone(), &binary_content, save_repository_1).await?;
        let save_2 =
            file_store.save_file(file_name_2.clone(), content_type, &binary_content, save_repository_2).await?;

        match file_store.read_file_content(&save_1.data.repository).await {
            Ok(BinaryContent::FromFs { file_path }) => {
                assert_eq!(PathBuf::from(format!("../target/repo_one/{file_name_1}")), file_path);
            }
            _ => panic!(),
        }

        match file_store.read_file_content(&save_2.data.repository).await {
            Ok(BinaryContent::FromFs { file_path }) => {
                assert_eq!(PathBuf::from(format!("../target/repo_two/{file_name_2}")), file_path);
            }
            _ => panic!(),
        }

        let loaded_data_by_repository_1 =
            file_store.read_file_data_by_repository(&save_1.data.repository).await.unwrap();
        assert_eq!(save_1, loaded_data_by_repository_1);

        let loaded_data_by_repository_2 =
            file_store.read_file_data_by_repository(&save_2.data.repository).await.unwrap();
        assert_eq!(save_2, loaded_data_by_repository_2);

        Ok(())
    })
}

#[test]
fn save_should_fails_if_fs_repo_does_not_exist() -> Result<(), LsError> {
    test(async {
        let data = data(false).await;
        let file_store = &data.0.file_store_service;

        let random: u32 = rand::random();
        let binary_content = BinaryContent::FromFs { file_path: SOURCE_FILE.to_owned().into() };
        let content_type = "application/text".to_owned();
        let file_name_1 = format!("file_2_{random}");
        let save_repository_1 = SaveRepository::FS { subfolder: None, repository_name: "REPO_NOT_EXISTING".to_owned() };

        assert!(file_store
            .save_file(file_name_1.clone(), content_type.clone(), &binary_content, save_repository_1)
            .await
            .is_err());

        Ok(())
    })
}

#[test]
fn should_save_file_to_db_with_relative_folder() -> Result<(), LsError> {
    test(async {
        let data = data(false).await;
        let file_store = &data.0.file_store_service;

        let random: u32 = rand::random();
        let file_name = format!("relative/folder/file_{random}");
        let binary_content = BinaryContent::FromFs { file_path: SOURCE_FILE.to_owned().into() };
        let content_type = "application/text".to_owned();
        let save_repository = SaveRepository::DB { subfolder: None, repository_name: "REPO_ONE".to_owned() };

        let saved = file_store.save_file(file_name.clone(), content_type, &binary_content, save_repository).await?;

        let loaded = file_store.read_file_data_by_id(saved.id).await?;
        assert_eq!(loaded.data, saved.data);
        match &loaded.data.repository {
            RepositoryFile::DB { repository_name, file_path } => {
                assert_eq!("REPO_ONE", repository_name);
                assert_eq!(&file_name, file_path);
            }
            _ => panic!(),
        }

        match file_store.read_file_content(&saved.data.repository).await {
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
    test(async {
        let data = data(false).await;
        let file_store = &data.0.file_store_service;

        let random: u32 = rand::random();
        let file_name = format!("relative/folder/file_{random}");
        let binary_content = BinaryContent::FromFs { file_path: SOURCE_FILE.to_owned().into() };
        let content_type = "application/text".to_owned();
        let save_repository = SaveRepository::FS { subfolder: None, repository_name: "REPO_ONE".to_owned() };

        let saved = file_store.save_file(file_name.clone(), content_type, &binary_content, save_repository).await?;

        let loaded = file_store.read_file_data_by_id(saved.id).await?;
        assert_eq!(loaded.data, saved.data);
        match &loaded.data.repository {
            RepositoryFile::FS { repository_name, file_path } => {
                assert_eq!("REPO_ONE", repository_name);
                assert_eq!(&file_name, file_path);
            }
            _ => panic!(),
        }

        match file_store.read_file_content(&saved.data.repository).await {
            Ok(BinaryContent::FromFs { file_path }) => {
                assert_eq!(PathBuf::from(format!("../target/repo_one/{file_name}")), file_path);
                assert_eq!(
                    &std::fs::read_to_string(SOURCE_FILE).unwrap(),
                    &std::fs::read_to_string(file_path).unwrap()
                );
            }
            _ => panic!(),
        }

        Ok(())
    })
}

#[test]
fn should_save_file_to_db_with_relative_folder_in_repository() -> Result<(), LsError> {
    test(async {
        let data = data(false).await;
        let file_store = &data.0.file_store_service;

        let random: u32 = rand::random();
        let file_name = format!("folder/file_{random}");
        let binary_content = BinaryContent::FromFs { file_path: SOURCE_FILE.to_owned().into() };
        let content_type = "application/text".to_owned();
        let save_repository =
            SaveRepository::DB { subfolder: Some("relative".to_owned()), repository_name: "REPO_ONE".to_owned() };

        let saved = file_store.save_file(file_name.clone(), content_type, &binary_content, save_repository).await?;

        let loaded = file_store.read_file_data_by_id(saved.id).await?;
        assert_eq!(loaded.data, saved.data);
        match &loaded.data.repository {
            RepositoryFile::DB { repository_name, file_path } => {
                assert_eq!("REPO_ONE", repository_name);
                assert_eq!(&format!("relative/{file_name}"), file_path);
            }
            _ => panic!(),
        }

        match file_store.read_file_content(&saved.data.repository).await {
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
    test(async {
        let data = data(false).await;
        let file_store = &data.0.file_store_service;

        let random: u32 = rand::random();
        let file_name = format!("folder/file_{random}");
        let binary_content = BinaryContent::FromFs { file_path: SOURCE_FILE.to_owned().into() };
        let content_type = "application/text".to_owned();
        let save_repository =
            SaveRepository::FS { subfolder: Some("relative".to_owned()), repository_name: "REPO_ONE".to_owned() };

        let saved = file_store.save_file(file_name.clone(), content_type, &binary_content, save_repository).await?;

        let loaded = file_store.read_file_data_by_id(saved.id).await?;
        assert_eq!(loaded.data, saved.data);
        match &loaded.data.repository {
            RepositoryFile::FS { repository_name, file_path } => {
                assert_eq!("REPO_ONE", repository_name);
                assert_eq!(&format!("relative/{file_name}"), file_path);
            }
            _ => panic!(),
        }

        match file_store.read_file_content(&saved.data.repository).await {
            Ok(BinaryContent::FromFs { file_path }) => {
                assert_eq!(PathBuf::from(format!("../target/repo_one/relative/{file_name}")), file_path);
                assert_eq!(
                    &std::fs::read_to_string(SOURCE_FILE).unwrap(),
                    &std::fs::read_to_string(file_path).unwrap()
                );
            }
            _ => panic!(),
        }

        Ok(())
    })
}

#[test]
fn should_delete_file_from_db() -> Result<(), LsError> {
    test(async {
        let data = data(false).await;
        let file_store = &data.0.file_store_service;

        let db_file_binary_repo = &data.0.repo_manager.file_store_binary_repo();

        let random: u32 = rand::random();
        let file_name = format!("file_{random}");
        let binary_content = BinaryContent::FromFs { file_path: SOURCE_FILE.to_owned().into() };
        let content_type = "application/text".to_owned();
        let save_repository =
            SaveRepository::DB { subfolder: Some("relative".to_owned()), repository_name: "REPO_ONE".to_owned() };

        let saved = file_store.save_file(file_name, content_type, &binary_content, save_repository).await?;

        let (repository_name, file_path) = match &saved.data.repository {
            RepositoryFile::DB { repository_name, file_path } => (repository_name.as_str(), file_path.as_str()),
            _ => {
                panic!();
            }
        };

        data.0
            .repo_manager
            .c3p0()
            .transaction::<_, LsError, _, _>(|conn| async {
                assert!(db_file_binary_repo.read_file(conn, repository_name, file_path).await.is_ok());
                Ok(())
            })
            .await
            .unwrap();

        assert_eq!(1, file_store.delete_file_by_id(saved.id).await?);
        assert!(file_store.read_file_data_by_id(saved.id).await.is_err());

        data.0
            .repo_manager
            .c3p0()
            .transaction::<_, LsError, _, _>(|conn| async {
                assert!(db_file_binary_repo.read_file(conn, repository_name, file_path).await.is_err());
                Ok(())
            })
            .await
            .unwrap();

        Ok(())
    })
}

#[test]
fn should_delete_file_from_fs() -> Result<(), LsError> {
    test(async {
        let data = data(false).await;
        let file_store = &data.0.file_store_service;

        let random: u32 = rand::random();
        let file_name = format!("relative/folder/file_{random}");
        let binary_content = BinaryContent::FromFs { file_path: SOURCE_FILE.to_owned().into() };
        let content_type = "application/text".to_owned();
        let save_repository = SaveRepository::FS { subfolder: None, repository_name: "REPO_ONE".to_owned() };

        let saved = file_store.save_file(file_name.clone(), content_type, &binary_content, save_repository).await?;

        let file_path = match &saved.data.repository {
            RepositoryFile::FS { file_path, .. } => format!("../target/repo_one/{file_path}"),
            _ => {
                panic!();
            }
        };

        println!("file_path: {file_path}");

        assert!(Path::new(&file_path).exists());

        assert_eq!(1, file_store.delete_file_by_id(saved.id).await?);
        assert!(file_store.read_file_data_by_id(saved.id).await.is_err());

        assert!(!Path::new(&file_path).exists());

        Ok(())
    })
}

#[test]
fn should_allow_same_files_with_same_repository_name_and_path_but_different_repository_type() -> Result<(), LsError> {
    test(async {
        let data = data(false).await;
        let file_store = &data.0.file_store_service;

        // Arrange
        let random: u32 = rand::random();
        let same_file_name = format!("folder/file_{random}");
        let same_file_path = format!("path_{random}");
        let same_repository_name = "REPO_ONE";

        let binary_content = BinaryContent::FromFs { file_path: SOURCE_FILE.to_owned().into() };

        let content_type = "application/text".to_owned();

        let save_repository_db = SaveRepository::DB {
            subfolder: Some(same_file_path.clone()),
            repository_name: same_repository_name.to_owned(),
        };

        let save_repository_fs = SaveRepository::FS {
            subfolder: Some(same_file_path.clone()),
            repository_name: same_repository_name.to_owned(),
        };

        // Act
        let saved_db = file_store
            .save_file(same_file_name.clone(), content_type.clone(), &binary_content, save_repository_db.clone())
            .await?;

        let saved_fs = file_store
            .save_file(same_file_name.clone(), content_type.clone(), &binary_content, save_repository_fs.clone())
            .await?;

        // Assert
        let loaded_db = file_store.read_file_data_by_id(saved_db.id).await?;
        assert_eq!(loaded_db, saved_db);
        match &loaded_db.data.repository {
            RepositoryFile::DB { repository_name, file_path } => {
                assert_eq!(same_repository_name, repository_name);
                assert_eq!(&format!("{same_file_path}/{same_file_name}"), file_path);
            }
            _ => panic!(),
        }

        let loaded_fs = file_store.read_file_data_by_id(saved_fs.id).await?;
        assert_eq!(loaded_fs, saved_fs);
        match &loaded_fs.data.repository {
            RepositoryFile::FS { repository_name, file_path } => {
                assert_eq!(same_repository_name, repository_name);
                assert_eq!(&format!("{same_file_path}/{same_file_name}"), file_path);
            }
            _ => panic!(),
        }

        Ok(())
    })
}

#[test]
fn should_fail_if_file_already_exists_in_db() -> Result<(), LsError> {
    test(async {
        let data = data(false).await;
        let file_store = &data.0.file_store_service;

        // Arrange
        let random: u32 = rand::random();
        let same_file_name = format!("folder/file_{random}");
        let same_file_path = format!("path_{random}");
        let same_repository_name = "REPO_ONE";

        let binary_content = BinaryContent::FromFs { file_path: SOURCE_FILE.to_owned().into() };

        let content_type = "application/text".to_owned();

        let save_repository_db = SaveRepository::DB {
            subfolder: Some(same_file_path.clone()),
            repository_name: same_repository_name.to_owned(),
        };

        // Act & Assert
        assert!(file_store
            .save_file(same_file_name.clone(), content_type.clone(), &binary_content, save_repository_db.clone(),)
            .await
            .is_ok());

        // fail if file already exists
        assert!(file_store
            .save_file(same_file_name.clone(), content_type.clone(), &binary_content, save_repository_db.clone(),)
            .await
            .is_err());

        // success if file has different name
        assert!(
            file_store
                .save_file(
                    format!("{same_file_name}-1"),
                    content_type.clone(),
                    &binary_content,
                    save_repository_db.clone(),
                )
                .await
                .is_ok()
        );

        // success if file has different repository_name
        assert!(file_store
            .save_file(
                same_file_name.clone(),
                content_type.clone(),
                &binary_content,
                SaveRepository::DB { subfolder: Some(same_file_path.clone()), repository_name: "REPO_TWO".to_owned() },
            )
            .await
            .is_ok());

        // success if file has different file_path
        assert!(file_store
            .save_file(
                same_file_name.clone(),
                content_type.clone(),
                &binary_content,
                SaveRepository::DB {
                    subfolder: Some(format!("{same_file_path}-1")),
                    repository_name: same_repository_name.to_owned(),
                },
            )
            .await
            .is_ok());

        Ok(())
    })
}

#[test]
fn should_fail_if_file_already_exists_in_fs() -> Result<(), LsError> {
    test(async {
        let data = data(false).await;
        let file_store = &data.0.file_store_service;

        // Arrange
        let random: u32 = rand::random();
        let same_file_name = format!("folder/file_{random}");
        let same_file_path = format!("path_{random}");
        let same_repository_name = "REPO_ONE";

        let binary_content = BinaryContent::FromFs { file_path: SOURCE_FILE.to_owned().into() };

        let content_type = "application/text".to_owned();

        let save_repository_db = SaveRepository::FS {
            subfolder: Some(same_file_path.clone()),
            repository_name: same_repository_name.to_owned(),
        };

        // Act & Assert
        assert!(file_store
            .save_file(same_file_name.clone(), content_type.clone(), &binary_content, save_repository_db.clone(),)
            .await
            .is_ok());

        // fail if file already exists
        assert!(file_store
            .save_file(same_file_name.clone(), content_type.clone(), &binary_content, save_repository_db.clone(),)
            .await
            .is_err());

        // success if file has different name
        assert!(
            file_store
                .save_file(
                    format!("{same_file_name}-1"),
                    content_type.clone(),
                    &binary_content,
                    save_repository_db.clone(),
                )
                .await
                .is_ok()
        );

        // success if file has different repository_name
        assert!(file_store
            .save_file(
                same_file_name.clone(),
                content_type.clone(),
                &binary_content,
                SaveRepository::FS { subfolder: Some(same_file_path.clone()), repository_name: "REPO_TWO".to_owned() },
            )
            .await
            .is_ok());

        // success if file has different file_path
        assert!(file_store
            .save_file(
                same_file_name.clone(),
                content_type.clone(),
                &binary_content,
                SaveRepository::FS {
                    subfolder: Some(format!("{same_file_path}-1")),
                    repository_name: same_repository_name.to_owned(),
                },
            )
            .await
            .is_ok());

        Ok(())
    })
}

#[test]
fn should_read_all_file_data_by_repository() -> Result<(), LsError> {
    test(async {
        let data = data(false).await;
        let file_store = &data.0.file_store_service;

        let random: u32 = rand::random();
        let binary_content = BinaryContent::FromFs { file_path: SOURCE_FILE.to_owned().into() };
        let content_type = "application/text".to_owned();
        let file_name_1 = format!("file_1_{random}");
        let file_name_2 = format!("file_2_{random}");
        let file_name_3 = format!("file_3_{random}");
        let repository_name = new_hyphenated_uuid();

        let save_repository = SaveRepository::DB { subfolder: None, repository_name: repository_name.clone() };

        let repository = Repository::DB { repository_name: repository_name.clone() };

        let all_repo_files =
            file_store.read_all_file_data_by_repository(&repository, 0, 100, &OrderBy::Asc).await.unwrap();
        assert_eq!(0, all_repo_files.len());

        file_store
            .save_file(file_name_1.clone(), content_type.clone(), &binary_content, save_repository.clone())
            .await?;

        let all_repo_files =
            file_store.read_all_file_data_by_repository(&repository, 0, 100, &OrderBy::Asc).await.unwrap();
        assert_eq!(1, all_repo_files.len());

        file_store
            .save_file(file_name_2.clone(), content_type.clone(), &binary_content, save_repository.clone())
            .await?;

        let all_repo_files =
            file_store.read_all_file_data_by_repository(&repository, 0, 100, &OrderBy::Asc).await.unwrap();
        assert_eq!(2, all_repo_files.len());

        file_store.save_file(file_name_3.clone(), content_type, &binary_content, save_repository).await?;

        let all_repo_files =
            file_store.read_all_file_data_by_repository(&repository, 0, 100, &OrderBy::Asc).await.unwrap();
        assert_eq!(3, all_repo_files.len());

        assert!(all_repo_files.iter().any(|file| file.data.filename == file_name_1));
        assert!(all_repo_files.iter().any(|file| file.data.filename == file_name_2));
        assert!(all_repo_files.iter().any(|file| file.data.filename == file_name_3));

        let all_repo_files =
            file_store.read_all_file_data_by_repository(&repository, 1, 1, &OrderBy::Asc).await.unwrap();
        assert_eq!(1, all_repo_files.len());

        assert!(all_repo_files.iter().any(|file| file.data.filename == file_name_2));

        Ok(())
    })
}

#[test]
fn should_return_if_file_exists_by_repository() -> Result<(), LsError> {
    test(async {
        let data = data(false).await;
        let file_store = &data.0.file_store_service;

        // Arrange
        let random: u32 = rand::random();
        let file_name = format!("file_{random}");
        let binary_content = BinaryContent::FromFs { file_path: SOURCE_FILE.to_owned().into() };
        let content_type = "application/text".to_owned();
        let save_repository =
            SaveRepository::DB { subfolder: Some("relative".to_owned()), repository_name: "REPO_ONE".to_owned() };

        let saved = file_store.save_file(file_name, content_type, &binary_content, save_repository).await?;

        // Act & Assert
        assert!(file_store.exists_by_repository(&saved.data.repository).await.unwrap());

        assert_eq!(1, file_store.delete_file_by_id(saved.id).await?);
        assert!(!file_store.exists_by_repository(&saved.data.repository).await.unwrap());

        Ok(())
    })
}
