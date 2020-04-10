use c3p0::{C3p0Error, JsonCodec, Model};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::borrow::Cow;

pub type FileStoreModel = Model<FileStoreData>;

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileStoreData {
    pub filename: String,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(tag = "_json_tag")]
enum FileStoreDataVersioning<'a> {
    V1(Cow<'a, FileStoreData>),
}

#[derive(Clone)]
pub struct FileStoreDataCodec {}

impl JsonCodec<FileStoreData> for FileStoreDataCodec {
    fn from_value(&self, value: Value) -> Result<FileStoreData, C3p0Error> {
        let versioning = serde_json::from_value(value)?;
        let data = match versioning {
            FileStoreDataVersioning::V1(data_v1) => data_v1.into_owned(),
        };
        Ok(data)
    }

    fn to_value(&self, data: &FileStoreData) -> Result<Value, C3p0Error> {
        serde_json::to_value(FileStoreDataVersioning::V1(Cow::Borrowed(data)))
            .map_err(C3p0Error::from)
    }
}
