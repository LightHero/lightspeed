use c3p0::*;
use lightspeed_core::error::{ErrorCodes, LsError};
use serde::{Deserialize, Serialize};
use std::{borrow::Cow, sync::Arc};

pub type FileStoreDataModel = Record<FileStoreDataData>;

#[derive(Clone)]
pub enum BinaryContent<'a> {
    InMemory { content: Cow<'a, [u8]> },
    OpenDal { operator: Arc<opendal::Operator>, path: String },
}

impl<'a> BinaryContent<'a> {
    pub async fn read(&self) -> Result<Cow<'a, [u8]>, LsError> {
        match self {
            BinaryContent::InMemory { content } => Ok(content.clone()),
            BinaryContent::OpenDal { operator, path } => Ok(operator
                .read(path)
                .await
                .map_err(|err| LsError::BadRequest {
                    message: format!("Failed to read file from store: {err}"),
                    code: ErrorCodes::IO_ERROR,
                })?
                .to_vec()
                .into()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FileStoreDataData {
    pub filename: String,
    pub file_path: String,
    pub repository: String,
    pub content_type: String,
    pub created_date_epoch_seconds: i64,
}

impl DataType for FileStoreDataData {
    const TABLE_NAME: &'static str = "LS_FILE_STORE_DATA";
    type CODEC = FileStoreDataVersioning;
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(tag = "_codec_tag")]
pub enum FileStoreDataVersioning {
    V1(FileStoreDataData),
}

impl Codec<FileStoreDataData> for FileStoreDataVersioning {
    fn encode(data: FileStoreDataData) -> Self {
        FileStoreDataVersioning::V1(data)
    }

    fn decode(data: Self) -> FileStoreDataData {
        match data {
            FileStoreDataVersioning::V1(data) => data,
        }
    }
}
