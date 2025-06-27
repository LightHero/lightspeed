use c3p0::{C3p0Error, JsonCodec, Model};
use lightspeed_core::error::{ErrorCodes, LsError};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{borrow::Cow, sync::Arc};

pub type FileStoreDataModel = Model<u64, FileStoreDataData>;

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

#[derive(Clone, Serialize, Deserialize)]
#[serde(tag = "_json_tag")]
enum FileStoreDataVersioning<'a> {
    V1(Cow<'a, FileStoreDataData>),
}

#[derive(Clone)]
pub struct FileStoreDataDataCodec {}

impl JsonCodec<FileStoreDataData> for FileStoreDataDataCodec {
    fn data_from_value(&self, value: Value) -> Result<FileStoreDataData, C3p0Error> {
        let versioning = serde_json::from_value(value)?;
        let data = match versioning {
            FileStoreDataVersioning::V1(data_v1) => data_v1.into_owned(),
        };
        Ok(data)
    }

    fn data_to_value(&self, data: &FileStoreDataData) -> Result<Value, C3p0Error> {
        serde_json::to_value(FileStoreDataVersioning::V1(Cow::Borrowed(data))).map_err(C3p0Error::from)
    }
}
