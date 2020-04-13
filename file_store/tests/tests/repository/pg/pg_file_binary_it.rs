use crate::{data, test};
use c3p0::*;
use lightspeed_core::error::LightSpeedError;
use lightspeed_core::utils::{current_epoch_seconds, new_hyphenated_uuid};
use lightspeed_file_store::repository::{FileStoreRepositoryManager, FileStoreBinaryRepository};
use lightspeed_file_store::dto::FileData;

const SOURCE_FILE: &str = "./Cargo.toml";

#[test]
fn should_save_file() -> Result<(), LightSpeedError> {
    test(async {
        let data = data(false).await;
        let file_store = &data.0.repo_manager.file_store_binary_repo();


        data.0.repo_manager.c3p0().transaction(|mut conn| async move {

            let random: u32 = rand::random();
            let file_name = format!("file_{}", random);

            file_store.save_file(&mut conn, SOURCE_FILE, &file_name).await?;

            /*
            let expected_file_path = format!("{}/{}", temp_dir_path, file_name);
            assert!(std::path::Path::new(&expected_file_path).exists());

            assert_eq!(std::fs::read_to_string(SOURCE_FILE).unwrap(), std::fs::read_to_string(&expected_file_path).unwrap());
        */
            Ok(())
        }).await
    })
}

/*
#[test]
fn save_file_should_fail_if_file_exists() -> Result<(), LightSpeedError> {
    test(async {
        let data = data(false).await;
        let file_store = &data.0.repo_manager.file_store_binary_repo();

        data.0.repo_manager.c3p0().transaction(|mut conn| async move {

        })

    let random: u32 = rand::random();
    let file_name = format!("file_{}", random);

    let tempdir = tempfile::tempdir().unwrap();
    let temp_dir_path = tempdir.path().to_str().unwrap().to_owned();
    let file_store = FsFileStoreBinaryRepository::new(temp_dir_path.clone());

    file_store.save_file(SOURCE_FILE, &file_name).await?;
    assert!(file_store.save_file(SOURCE_FILE, &file_name).await.is_err());

    Ok(())
    })
}

#[test]
fn should_save_file_with_relative_folder() -> Result<(), LightSpeedError> {
    test(async {
        let data = data(false).await;
        let file_store = &data.0.repo_manager.file_store_binary_repo();

        data.0.repo_manager.c3p0().transaction(|mut conn| async move {

        })
    let random: u32 = rand::random();
    let file_name = format!("test/temp/file_{}", random);

    let tempdir = tempfile::tempdir().unwrap();
    let temp_dir_path = tempdir.path().to_str().unwrap().to_owned();
    let file_store = FsFileStoreBinaryRepository::new(temp_dir_path.clone());

    file_store.save_file(SOURCE_FILE, &file_name).await?;

    let expected_file_path = format!("{}/{}", temp_dir_path, file_name);
    assert!(std::path::Path::new(&expected_file_path).exists());

    assert_eq!(std::fs::read_to_string(SOURCE_FILE).unwrap(), std::fs::read_to_string(&expected_file_path).unwrap());

    Ok(())
    })
}

#[test]
fn should_delete_file_with_relative_folder() -> Result<(), LightSpeedError> {
    test(async {
        let data = data(false).await;
        let file_store = &data.0.repo_manager.file_store_binary_repo();
        data.0.repo_manager.c3p0().transaction(|mut conn| async move {

        })

    let random: u32 = rand::random();
    let file_name = format!("/test/temp/file_{}", random);

    let tempdir = tempfile::tempdir().unwrap();
    let temp_dir_path = tempdir.path().to_str().unwrap().to_owned();
    let file_store = FsFileStoreBinaryRepository::new(temp_dir_path.clone());

    file_store.save_file(SOURCE_FILE, &file_name).await?;

    file_store.delete_by_filename(&file_name).await?;

    assert!(!std::path::Path::new(&file_name).exists());

    Ok(())
    })
}
*/

#[test]
fn should_read_a_saved_file() -> Result<(), LightSpeedError> {
    test(async {
        let data = data(false).await;
        let file_store = &data.0.repo_manager.file_store_binary_repo();

        data.0.repo_manager.c3p0().transaction(|mut conn| async move {

            let random: u32 = rand::random();
            let file_name = format!("file_{}", random);

            file_store.save_file(&mut conn, SOURCE_FILE, &file_name).await?;

            match file_store.read_file(&mut conn, &file_name).await {
                Ok(FileData::InMemory{ content }) => {
                    let file_content = std::str::from_utf8(&content).unwrap();
                    assert_eq!(&std::fs::read_to_string(SOURCE_FILE).unwrap(), file_content);
                },
                _ => assert!(false)
            }

            Ok(())
        }).await

    })
}
