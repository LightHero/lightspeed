use c3p0_common::Model;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct ModelDto<DATA>
where
    DATA: Clone + serde::ser::Serialize + Send,
{
    pub id: u64,
    pub version: u32,
    #[serde(bound(deserialize = "DATA: serde::Deserialize<'de>"))]
    pub data: DATA,
}

impl<DATA> From<Model<u64, DATA>> for ModelDto<DATA>
where
    DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned + Send,
{
    fn from(model: Model<u64, DATA>) -> Self {
        Self { id: model.id, version: model.version, data: model.data }
    }
}
