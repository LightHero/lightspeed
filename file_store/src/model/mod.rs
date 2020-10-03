use c3p0::{C3p0Error, JsonCodec, Model};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::borrow::Cow;
use strum_macros::{AsRefStr, Display};

pub type FileStoreDataModel = Model<FileStoreDataData>;

pub enum BinaryContent {
    FromFs { file_path: String },
    InMemory { content: Vec<u8> },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileStoreDataData {
    pub filename: String,
    pub repository: Repository,
    pub content_type: String,
    pub created_date_epoch_seconds: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, AsRefStr, Display)]
#[serde(tag = "_json_tag")]
pub enum Repository {
    DB {
        file_path: String,
        repository_name: String,
    },
    FS {
        file_path: String,
        repository_name: String,
    },
}

#[derive(Debug, Clone)]
pub enum SaveRepository {
    DB {
        file_path: Option<String>,
        repository_name: String,
    },
    FS {
        file_path: Option<String>,
        repository_name: String,
    },
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(tag = "_json_tag")]
enum FileStoreDataVersioning<'a> {
    V1(Cow<'a, FileStoreDataData>),
}

#[derive(Clone)]
pub struct FileStoreDataDataCodec {}

impl JsonCodec<FileStoreDataData> for FileStoreDataDataCodec {
    fn from_value(&self, value: Value) -> Result<FileStoreDataData, C3p0Error> {
        let versioning = serde_json::from_value(value)?;
        let data = match versioning {
            FileStoreDataVersioning::V1(data_v1) => data_v1.into_owned(),
        };
        Ok(data)
    }

    fn to_value(&self, data: &FileStoreDataData) -> Result<Value, C3p0Error> {
        serde_json::to_value(FileStoreDataVersioning::V1(Cow::Borrowed(data)))
            .map_err(C3p0Error::from)
    }
}
