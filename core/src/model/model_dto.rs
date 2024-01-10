use c3p0::{DataType, IdType, Model, VersionType};
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct ModelDto<Id, Data> {
    pub id: Id,
    pub version: VersionType,
    pub data: Data,
}

impl<Id: IdType, Data: DataType> From<Model<Id, Data>> for ModelDto<Id, Data> {
    fn from(model: Model<Id, Data>) -> Self {
        Self { id: model.id, version: model.version, data: model.data }
    }
}
