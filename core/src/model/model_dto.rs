use c3p0_common::json::model::{IdType, VersionType};
use c3p0_common::Model;
use serde_derive::{Deserialize, Serialize};
use typescript_definitions::TypeScriptify;

#[derive(Clone, Serialize, Deserialize, TypeScriptify)]
pub struct ModelDto<DATA>
where
    DATA: Clone + serde::ser::Serialize,
{
    pub id: IdType,
    pub version: VersionType,
    #[serde(bound(deserialize = "DATA: serde::Deserialize<'de>"))]
    pub data: DATA,
}

impl<DATA> From<Model<DATA>> for ModelDto<DATA>
where
    DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned,
{
    fn from(model: Model<DATA>) -> Self {
        Self {
            id: model.id,
            version: model.version,
            data: model.data,
        }
    }
}
