use std::sync::Arc;

use crate::model::BinaryContent;
use futures::StreamExt;
use lightspeed_core::error::{ErrorCodes, LsError};
use opendal::Operator;

#[derive(Clone)]
pub struct OpendalFileStoreBinaryRepository {
    operator: Arc<Operator>,
}

impl OpendalFileStoreBinaryRepository {
    pub fn new(operator: Arc<Operator>) -> Self {
        Self { operator }
    }

    pub async fn read_file(&self, file_path: &str) -> Result<BinaryContent<'_>, LsError> {
        Ok(BinaryContent::OpenDal { operator: self.operator.clone(), path: file_path.to_owned() })
    }

    pub async fn exists(&self, file_path: &str) -> Result<bool, LsError> {
        self.operator.exists(file_path).await.map_err(|err| LsError::BadRequest {
            message: format!("OpendalFileStoreDataRepository - Cannot check file [{file_path}]. Err: {err:?}"),
            code: ErrorCodes::IO_ERROR,
        })
    }

    pub async fn save_file(&self, file_path: &str, content: &BinaryContent<'_>) -> Result<(), LsError> {
        match content {
            BinaryContent::InMemory { content } => {
                self.operator.write(file_path, content.to_vec()).await.map_err(|err| LsError::BadRequest {
                    message: format!(
                        "OpendalFileStoreDataRepository - Cannot write data to [{file_path}]. Err: {err:?}"
                    ),
                    code: ErrorCodes::IO_ERROR,
                })?;
                Ok(())
            }
            BinaryContent::OpenDal { operator, path } => {
                let reader = operator.reader(path).await.map_err(|err| LsError::BadRequest {
                    message: format!("OpendalFileStoreDataRepository - Cannot read file [{path}]. Err: {err:?}"),
                    code: ErrorCodes::IO_ERROR,
                })?;

                let byte_stream = reader.into_bytes_stream(..).await.map_err(|err| LsError::BadRequest {
                    message: format!(
                        "OpendalFileStoreDataRepository - Cannot create byte stream from file [{path}]. Err: {err:?}"
                    ),
                    code: ErrorCodes::IO_ERROR,
                })?;

                let byte_sink = self
                    .operator
                    .writer(file_path)
                    .await
                    .map_err(|err| LsError::BadRequest {
                        message: format!(
                            "OpendalFileStoreDataRepository - Cannot create writer to [{file_path}]. Err: {err:?}"
                        ),
                        code: ErrorCodes::IO_ERROR,
                    })?
                    .into_bytes_sink();

                byte_stream.forward(byte_sink).await.map_err(|err| LsError::BadRequest {
                    message: format!(
                        "OpendalFileStoreDataRepository - Cannot write data to [{file_path}]. Err: {err:?}"
                    ),
                    code: ErrorCodes::IO_ERROR,
                })
            }
        }
    }

    pub async fn delete_by_filename(&self, file_name: &str) -> Result<(), LsError> {
        self.operator.delete(file_name).await.map_err(|err| LsError::BadRequest {
            message: format!("OpendalFileStoreDataRepository - Cannot delete file [{file_name}]. Err: {err:?}"),
            code: ErrorCodes::IO_ERROR,
        })
    }
}

#[cfg(test)]
mod test {

    use super::*;
    use lightspeed_core::error::LsError;
    use opendal::services;
    use std::borrow::Cow;

    const SOURCE_FILE: &str = "./Cargo.toml";

    fn source_store() -> Arc<Operator> {
        new_operator("./")
    }

    fn new_operator(path: &str) -> Arc<Operator> {
        let builder = services::Fs::default().root(path);
        Operator::new(builder).unwrap().finish().into()
    }

    #[tokio::test]
    async fn should_save_file_from_fs() -> Result<(), LsError> {
        let random: u32 = rand::random();
        let file_name = format!("file_{random}");

        let source_store = source_store();
        let binary_content = BinaryContent::OpenDal { operator: source_store, path: SOURCE_FILE.to_owned() };

        let tempdir = tempfile::tempdir().unwrap();
        let temp_dir_path = tempdir.path().to_str().unwrap().to_owned();
        let file_store = OpendalFileStoreBinaryRepository::new(new_operator(&temp_dir_path));

        assert!(!file_store.exists(&file_name).await?);

        file_store.save_file(&file_name, &binary_content).await?;

        assert!(file_store.exists(&file_name).await?);

        let expected_file_path = format!("{temp_dir_path}/{file_name}");
        assert!(std::path::Path::new(&expected_file_path).exists());

        assert_eq!(
            std::fs::read_to_string(SOURCE_FILE).unwrap(),
            std::fs::read_to_string(&expected_file_path).unwrap()
        );

        Ok(())
    }

    #[tokio::test]
    async fn should_save_file_from_memory() -> Result<(), LsError> {
        let random: u32 = rand::random();
        let file_name = format!("file_{random}");

        let binary_content = BinaryContent::InMemory { content: Cow::Owned("Hello world!".to_owned().into_bytes()) };

        let tempdir = tempfile::tempdir().unwrap();
        let temp_dir_path = tempdir.path().to_str().unwrap().to_owned();
        let file_store = OpendalFileStoreBinaryRepository::new(new_operator(&temp_dir_path));

        assert!(!file_store.exists(&file_name).await?);

        file_store.save_file(&file_name, &binary_content).await?;

        assert!(file_store.exists(&file_name).await?);

        let expected_file_path = format!("{temp_dir_path}/{file_name}");
        assert!(std::path::Path::new(&expected_file_path).exists());

        assert_eq!("Hello world!", std::fs::read_to_string(&expected_file_path).unwrap());

        Ok(())
    }

    // #[tokio::test]
    // async fn save_file_should_fail_if_file_exists() -> Result<(), LsError> {
    //     let random: u32 = rand::random();
    //     let file_name = format!("file_{random}");
    //     let source_store = source_store();
    //     let binary_content = BinaryContent::OpenDal { operator: &source_store, path: SOURCE_FILE };

    //     let tempdir = tempfile::tempdir().unwrap();
    //     let temp_dir_path = tempdir.path().to_str().unwrap().to_owned();
    //     let file_store = OpendalFileStoreBinaryRepository::new(new_operator(&temp_dir_path));

    //     file_store.save_file(&file_name, &binary_content).await?;
    //     assert!(file_store.save_file(&file_name, &binary_content).await.is_err());

    //     Ok(())
    // }

    #[tokio::test]
    async fn should_save_file_with_relative_folder() -> Result<(), LsError> {
        let random: u32 = rand::random();
        let file_name = format!("test/temp/file_{random}");
        let source_store = source_store();
        let binary_content = BinaryContent::OpenDal { operator: source_store, path: SOURCE_FILE.to_owned() };

        let tempdir = tempfile::tempdir().unwrap();
        let temp_dir_path = tempdir.path().to_str().unwrap().to_owned();
        let file_store = OpendalFileStoreBinaryRepository::new(new_operator(&temp_dir_path));

        file_store.save_file(&file_name, &binary_content).await?;

        let expected_file_path = format!("{temp_dir_path}/{file_name}");
        assert!(std::path::Path::new(&expected_file_path).exists());

        assert_eq!(
            std::fs::read_to_string(SOURCE_FILE).unwrap(),
            std::fs::read_to_string(&expected_file_path).unwrap()
        );

        Ok(())
    }

    #[tokio::test]
    async fn should_delete_file_with_relative_folder() -> Result<(), LsError> {
        let random: u32 = rand::random();
        let file_name = format!("/test/temp/file_{random}");
        let source_store = source_store();
        let binary_content = BinaryContent::OpenDal { operator: source_store, path: SOURCE_FILE.to_owned() };

        let tempdir = tempfile::tempdir().unwrap();
        let temp_dir_path = tempdir.path().to_str().unwrap().to_owned();
        let file_store = OpendalFileStoreBinaryRepository::new(new_operator(&temp_dir_path));

        file_store.save_file(&file_name, &binary_content).await?;

        file_store.delete_by_filename(&file_name).await?;

        assert!(!std::path::Path::new(&file_name).exists());

        Ok(())
    }

    #[tokio::test]
    async fn should_read_a_saved_file() -> Result<(), LsError> {
        let random: u32 = rand::random();
        let file_name = format!("file_{random}");
        let source_store = source_store();
        let binary_content = BinaryContent::OpenDal { operator: source_store, path: SOURCE_FILE.to_owned() };

        let tempdir = tempfile::tempdir().unwrap();
        let temp_dir_path = tempdir.path().to_str().unwrap().to_owned();
        let file_store = OpendalFileStoreBinaryRepository::new(new_operator(&temp_dir_path));

        file_store.save_file(&file_name, &binary_content).await?;

        match file_store.read_file(&file_name).await {
            Ok(BinaryContent::OpenDal { operator, path }) => {
                let buffer = operator.read(&path).await.unwrap().to_vec();
                let file_content = std::str::from_utf8(&buffer).unwrap();
                assert_eq!(&std::fs::read_to_string(SOURCE_FILE).unwrap(), file_content);
            }
            _ => panic!(),
        }

        Ok(())
    }
}
