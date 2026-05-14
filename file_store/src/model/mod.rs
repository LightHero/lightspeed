use crate::error::LsFileStoreError;
use c3p0::*;
use futures::stream::BoxStream;
use serde::{Deserialize, Serialize};
use std::{borrow::Cow, sync::Arc};
use tokio::sync::Mutex;

pub type FileStoreDataModel = Record<FileStoreDataData>;

/// `BinaryContent` is a wrapper around the source of a file's binary content.
pub enum BinaryContent<'a> {
    InMemory { content: Cow<'a, [u8]> },
    OpenDal { operator: Arc<opendal::Operator>, path: String },
    Stream { stream: Mutex<BoxStream<'static, Result<Vec<u8>, LsFileStoreError>>> },
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
