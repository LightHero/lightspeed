use crate::{data, test};
use lightspeed_core::error::LightSpeedError;
use lightspeed_file_store::dto::FileData;

const SOURCE_FILE: &str = "./Cargo.toml";

#[test]
fn should_save_file() -> Result<(), LightSpeedError> {
    test(async {
        let data = data(false).await;
        let file_store = &data.0.file_store_service;

        let random: u32 = rand::random();
        let file_name = format!("file_{}", random);

        file_store.save_file(SOURCE_FILE, &file_name).await?;

        match file_store.read_file(&file_name).await {
            Ok(FileData::InMemory { content }) => {
                let file_content = std::str::from_utf8(&content).unwrap();
                assert_eq!(&std::fs::read_to_string(SOURCE_FILE).unwrap(), file_content);
            }
            _ => assert!(false),
        }

        Ok(())
    })
}

#[test]
fn save_file_should_fail_if_file_exists() -> Result<(), LightSpeedError> {
    test(async {
        let data = data(false).await;
        let file_store = &data.0.file_store_service;

        let random: u32 = rand::random();
        let file_name = format!("file_{}", random);

        file_store.save_file(SOURCE_FILE, &file_name).await?;
        assert!(file_store.save_file(SOURCE_FILE, &file_name).await.is_err());
        Ok(())
    })
}

#[test]
fn should_save_file_with_relative_folder() -> Result<(), LightSpeedError> {
    test(async {
        let data = data(false).await;
        let file_store = &data.0.file_store_service;

        let random: u32 = rand::random();
        let file_name = format!("/relative/folder/file_{}", random);

        file_store.save_file(SOURCE_FILE, &file_name).await?;

        match file_store.read_file(&file_name).await {
            Ok(FileData::InMemory { content }) => {
                let file_content = std::str::from_utf8(&content).unwrap();
                assert_eq!(&std::fs::read_to_string(SOURCE_FILE).unwrap(), file_content);
            }
            _ => assert!(false),
        }

        Ok(())
    })
}

#[test]
fn should_delete_file_with_relative_folder() -> Result<(), LightSpeedError> {
    test(async {
        let data = data(false).await;
        let file_store = &data.0.file_store_service;

        let random: u32 = rand::random();
        let file_name = format!("/relative/folder/file_{}", random);

        file_store.save_file(SOURCE_FILE, &file_name).await?;

        assert_eq!(1, file_store.delete_by_filename(&file_name).await?);

        assert!(file_store.read_file(&file_name).await.is_err());

        Ok(())
    })
}
