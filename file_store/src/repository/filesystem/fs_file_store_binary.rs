use crate::repository::{ FileStoreBinaryRepository};
use c3p0::pg::*;
use c3p0::*;
use lightspeed_core::error::LightSpeedError;
use crate::model::{FileStoreData, FileStoreDataCodec};
use std::path::Path;
use log::*;

#[derive(Clone)]
pub struct FsFileStoreBinaryRepository {
    base_folder: String,
}

impl FsFileStoreBinaryRepository {

    pub fn new<S: Into<String>>(base_folder: S) -> Self {
        Self {
            base_folder: base_folder.into()
        }
    }

    pub fn get_file_path(&self, file_name: &str) -> String {
        format!("{}/{}", &self.base_folder, file_name)
    }

}

#[async_trait::async_trait]
impl FileStoreBinaryRepository for FsFileStoreBinaryRepository {
    type Conn = PgConnectionAsync;

    async fn read_file<W: tokio::io::AsyncWrite + Unpin + Send>(&self, file_name: &str, output: &mut W) -> Result<u64, LightSpeedError> {

        use tokio::io::AsyncReadExt;

        let file_path = self.get_file_path(file_name);
        let mut file = tokio::fs::OpenOptions::new()
            .write(false)
            .create(false)
            .open(file_path)
            .await
            .map_err(|err| LightSpeedError::BadRequest {
                message: format!(
                    "FsFileStoreDataRepository - Cannot read file [{}]. Err: {}",
                    file_name,
                    err
                ),
            })?;

        tokio::io::copy(&mut file, output).await.map_err(|err| LightSpeedError::BadRequest {
            message: format!(
                "FsFileStoreDataRepository - Cannot read file [{}]. Err: {}",
                file_name,
                err
            ),
        })
    }

    async fn save_file(&self, source_path: &str, file_name: &str) -> Result<(), LightSpeedError> {
        let destination_file_path = self.get_file_path(file_name);
        let destination_path = Path::new(&destination_file_path);

        if destination_path.exists() {
            return Err(LightSpeedError::BadRequest {
                message: format!(
                    "FsFileStoreDataRepository - Cannot save file [{}] because it already exists.",
                    destination_file_path,
                ),
            });
        }

        match destination_path.parent() {
            Some(parent_path) => {
                tokio::fs::create_dir_all(parent_path).await.map_err(|err| LightSpeedError::BadRequest {
                    message: format!(
                        "FsFileStoreDataRepository - Create directory structure for file [{}]. Err: {}",
                        destination_file_path,
                        err
                    ),
                })?;
            }, 
            None => {
                warn!("The file does not have a parent path. Are you saving it on root? File path: [{}]", destination_file_path)
            }
        }
        
        tokio::fs::copy(source_path, destination_path).await.map_err(|err| LightSpeedError::BadRequest {
            message: format!(
                "FsFileStoreDataRepository - Cannot copy file from [{:?}] to [{}]. Err: {}",
                source_path,
                destination_file_path,
                err
            ),
        })?;
        Ok(())
    }

    async fn delete_by_filename(&self, file_name: &str) -> Result<(), LightSpeedError> {
        let to = self.get_file_path(file_name);
        if std::path::Path::new(&to).exists() {
            tokio::fs::remove_file(&to).await.map_err(|err| LightSpeedError::BadRequest {
                message: format!("FsFileStoreDataRepository - Cannot delete file [{}]. Err: {}", to, err),
            })?;
        }
        Ok(())
    }

}

#[cfg(test)]
mod test {
    use lightspeed_core::error::LightSpeedError;
    use crate::repository::filesystem::fs_file_store_binary::FsFileStoreBinaryRepository;
    use crate::repository::FileStoreBinaryRepository;

    const SOURCE_FILE: &str = "./Cargo.toml";

    #[tokio::test]
    async fn should_save_file() -> Result<(), LightSpeedError> {

        let random: u32 = rand::random();
        let file_name = format!("file_{}", random);

        let tempdir = tempfile::tempdir().unwrap();
        let temp_dir_path = tempdir.path().to_str().unwrap().to_owned();
        let file_store = FsFileStoreBinaryRepository::new(temp_dir_path.clone());

        file_store.save_file(SOURCE_FILE, &file_name).await?;

        let expected_file_path = format!("{}/{}", temp_dir_path, file_name);
        assert!(std::path::Path::new(&expected_file_path).exists());

        assert_eq!(std::fs::read_to_string(SOURCE_FILE).unwrap(), std::fs::read_to_string(&expected_file_path).unwrap());

        Ok(())
    }

    #[tokio::test]
    async fn save_file_should_fail_if_file_exists() -> Result<(), LightSpeedError> {

        let random: u32 = rand::random();
        let file_name = format!("file_{}", random);

        let tempdir = tempfile::tempdir().unwrap();
        let temp_dir_path = tempdir.path().to_str().unwrap().to_owned();
        let file_store = FsFileStoreBinaryRepository::new(temp_dir_path.clone());

        file_store.save_file(SOURCE_FILE, &file_name).await?;
        assert!(file_store.save_file(SOURCE_FILE, &file_name).await.is_err());

        Ok(())
    }

    #[tokio::test]
    async fn should_save_file_with_relative_folder() -> Result<(), LightSpeedError> {

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
    }

    #[tokio::test]
    async fn should_delete_file_with_relative_folder() -> Result<(), LightSpeedError> {

        let random: u32 = rand::random();
        let file_name = format!("/test/temp/file_{}", random);

        let tempdir = tempfile::tempdir().unwrap();
        let temp_dir_path = tempdir.path().to_str().unwrap().to_owned();
        let file_store = FsFileStoreBinaryRepository::new(temp_dir_path.clone());

        file_store.save_file(SOURCE_FILE, &file_name).await?;

        file_store.delete_by_filename(&file_name).await?;

        assert!(!std::path::Path::new(&file_name).exists());

        Ok(())
    }

}

