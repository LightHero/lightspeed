use crate::model::BinaryContent;
use lightspeed_core::error::{ErrorCodes, LsError};
use log::*;
use opendal::Operator;
use std::path::Path;

#[derive(Clone)]
pub struct OpendalFileStoreBinaryRepository {
    operator: Operator,
}

impl OpendalFileStoreBinaryRepository {
    pub fn new(operator: Operator) -> Self {
        Self { operator }
    }

    pub fn get_fs_file_path<'a>(&self, file_path: &'a str) -> &'a str {
        file_path
    }

    pub async fn read_file<'a>(&'a self, file_path: &'a str) -> Result<BinaryContent<'a>, LsError> {
        let result = self.operator.read(file_path).await.map_err(|err| LsError::BadRequest {
            message: format!("FsFileStoreDataRepository - Cannot read file [{file_path}]. Err: {err:?}"),
            code: ErrorCodes::IO_ERROR,
        })?;

        let result = self.operator.reader(file_path).await.map_err(|err| LsError::BadRequest {
            message: format!("FsFileStoreDataRepository - Cannot read file [{file_path}]. Err: {err:?}"),
            code: ErrorCodes::IO_ERROR,
        })?;


        Ok(BinaryContent::OpenDal { operator: &self.operator, path: file_path })
    }

    pub async fn save_file<'a>(&self, file_path: &str, content: &'a BinaryContent<'a>) -> Result<(), LsError> {
        let destination_file_path = self.get_fs_file_path(file_path);
        let destination_path = Path::new(&destination_file_path);

        if destination_path.exists() {
            return Err(LsError::BadRequest {
                message: format!(
                    "FsFileStoreDataRepository - Cannot save file [{destination_file_path}] because it already exists.",
                ),
                code: ErrorCodes::IO_ERROR,
            });
        }

        match destination_path.parent() {
            Some(parent_path) => {
                tokio::fs::create_dir_all(parent_path).await.map_err(|err| LsError::BadRequest {
                    message: format!(
                        "FsFileStoreDataRepository - Create directory structure for file [{destination_file_path}]. Err: {err:?}"
                    ),
                    code: ErrorCodes::IO_ERROR,
                })?;
            }
            None => warn!(
                "The file does not have a parent path. Are you saving it on root? File path: [{}]",
                destination_file_path
            ),
        }

        match content {
            BinaryContent::InMemory { content } => {
                tokio::fs::write(destination_path, content.as_ref()).await.map_err(|err| LsError::BadRequest {
                    message: format!(
                        "FsFileStoreDataRepository - Cannot write data to [{destination_file_path}]. Err: {err:?}"
                    ),
                    code: ErrorCodes::IO_ERROR,
                })?;
                Ok(())
            }
            BinaryContent::FromFs { file_path } => {
                tokio::fs::copy(file_path, destination_path).await.map_err(|err| LsError::BadRequest {
                    message: format!(
                        "FsFileStoreDataRepository - Cannot copy file from [{file_path:?}] to [{destination_file_path}]. Err: {err:?}"
                    ),
                    code: ErrorCodes::IO_ERROR,
                })?;
                Ok(())
            }
            BinaryContent::OpenDal { operator, path } => {
                let TODO = 0;
                todo!()
            }
        }
    }

    pub async fn delete_by_filename(&self, file_name: &str) -> Result<u64, LsError> {
        let to = self.get_fs_file_path(file_name);
        if std::path::Path::new(&to).exists() {
            tokio::fs::remove_file(&to).await.map_err(|err| LsError::BadRequest {
                message: format!("FsFileStoreDataRepository - Cannot delete file [{to}]. Err: {err:?}"),
                code: ErrorCodes::IO_ERROR,
            })?;
            Ok(1)
        } else {
            Ok(0)
        }
    }
}

// #[cfg(test)]
// mod test {
//     use super::*;
//     use crate::utils::read_file;
//     use lightspeed_core::error::LsError;
//     use std::borrow::Cow;

//     const SOURCE_FILE: &str = "./Cargo.toml";

//     #[tokio::test]
//     async fn should_save_file_from_fs() -> Result<(), LsError> {
//         let random: u32 = rand::random();
//         let file_name = format!("file_{random}");
//         let binary_content = BinaryContent::FromFs { file_path: SOURCE_FILE.to_owned().into() };

//         let tempdir = tempfile::tempdir().unwrap();
//         let temp_dir_path = tempdir.path().to_str().unwrap().to_owned();
//         let file_store = OpendalFileStoreBinaryRepository::new(temp_dir_path.clone());

//         file_store.save_file(&file_name, &binary_content).await?;

//         let expected_file_path = format!("{temp_dir_path}/{file_name}");
//         assert!(std::path::Path::new(&expected_file_path).exists());

//         assert_eq!(
//             std::fs::read_to_string(SOURCE_FILE).unwrap(),
//             std::fs::read_to_string(&expected_file_path).unwrap()
//         );

//         Ok(())
//     }

//     #[tokio::test]
//     async fn should_save_file_from_memory() -> Result<(), LsError> {
//         let random: u32 = rand::random();
//         let file_name = format!("file_{random}");

//         let binary_content = BinaryContent::InMemory { content: Cow::Owned("Hello world!".to_owned().into_bytes()) };

//         let tempdir = tempfile::tempdir().unwrap();
//         let temp_dir_path = tempdir.path().to_str().unwrap().to_owned();
//         let file_store = OpendalFileStoreBinaryRepository::new(temp_dir_path.clone());

//         file_store.save_file(&file_name, &binary_content).await?;

//         let expected_file_path = format!("{temp_dir_path}/{file_name}");
//         assert!(std::path::Path::new(&expected_file_path).exists());

//         assert_eq!("Hello world!", std::fs::read_to_string(&expected_file_path).unwrap());

//         Ok(())
//     }

//     #[tokio::test]
//     async fn save_file_should_fail_if_file_exists() -> Result<(), LsError> {
//         let random: u32 = rand::random();
//         let file_name = format!("file_{random}");
//         let binary_content = BinaryContent::FromFs { file_path: SOURCE_FILE.to_owned().into() };

//         let tempdir = tempfile::tempdir().unwrap();
//         let temp_dir_path = tempdir.path().to_str().unwrap().to_owned();
//         let file_store = OpendalFileStoreBinaryRepository::new(temp_dir_path.clone());

//         file_store.save_file(&file_name, &binary_content).await?;
//         assert!(file_store.save_file(&file_name, &binary_content).await.is_err());

//         Ok(())
//     }

//     #[tokio::test]
//     async fn should_save_file_with_relative_folder() -> Result<(), LsError> {
//         let random: u32 = rand::random();
//         let file_name = format!("test/temp/file_{random}");
//         let binary_content = BinaryContent::FromFs { file_path: SOURCE_FILE.to_owned().into() };

//         let tempdir = tempfile::tempdir().unwrap();
//         let temp_dir_path = tempdir.path().to_str().unwrap().to_owned();
//         let file_store = OpendalFileStoreBinaryRepository::new(temp_dir_path.clone());

//         file_store.save_file(&file_name, &binary_content).await?;

//         let expected_file_path = format!("{temp_dir_path}/{file_name}");
//         assert!(std::path::Path::new(&expected_file_path).exists());

//         assert_eq!(
//             std::fs::read_to_string(SOURCE_FILE).unwrap(),
//             std::fs::read_to_string(&expected_file_path).unwrap()
//         );

//         Ok(())
//     }

//     #[tokio::test]
//     async fn should_delete_file_with_relative_folder() -> Result<(), LsError> {
//         let random: u32 = rand::random();
//         let file_name = format!("/test/temp/file_{random}");
//         let binary_content = BinaryContent::FromFs { file_path: SOURCE_FILE.to_owned().into() };

//         let tempdir = tempfile::tempdir().unwrap();
//         let temp_dir_path = tempdir.path().to_str().unwrap().to_owned();
//         let file_store = OpendalFileStoreBinaryRepository::new(temp_dir_path.clone());

//         file_store.save_file(&file_name, &binary_content).await?;

//         file_store.delete_by_filename(&file_name).await?;

//         assert!(!std::path::Path::new(&file_name).exists());

//         Ok(())
//     }

//     #[tokio::test]
//     async fn should_read_a_saved_file() -> Result<(), LsError> {
//         let random: u32 = rand::random();
//         let file_name = format!("file_{random}");
//         let binary_content = BinaryContent::FromFs { file_path: SOURCE_FILE.to_owned().into() };

//         let tempdir = tempfile::tempdir().unwrap();
//         let temp_dir_path = tempdir.path().to_str().unwrap().to_owned();
//         let file_store = OpendalFileStoreBinaryRepository::new(temp_dir_path.clone());

//         file_store.save_file(&file_name, &binary_content).await?;

//         match file_store.read_file(&file_name).await {
//             Ok(BinaryContent::FromFs { file_path }) => {
//                 let mut buffer: Vec<u8> = vec![];
//                 read_file(&file_path, &mut buffer).await?;
//                 let file_content = std::str::from_utf8(&buffer).unwrap();
//                 assert_eq!(&std::fs::read_to_string(SOURCE_FILE).unwrap(), file_content);
//             }
//             _ => panic!(),
//         }

//         Ok(())
//     }
// }
