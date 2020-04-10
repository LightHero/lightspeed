use crate::repository::{ FileStoreBinaryRepository};
use c3p0::pg::*;
use c3p0::*;
use lightspeed_core::error::LightSpeedError;
use crate::model::{FileStoreData, FileStoreDataCodec};
use std::path::Path;
use log::*;

#[derive(Clone)]
pub struct FsFileStoreDataRepository {
    base_folder: String,
}

impl FsFileStoreDataRepository {

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
impl FileStoreBinaryRepository for FsFileStoreDataRepository {
    type Conn = PgConnectionAsync;

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

