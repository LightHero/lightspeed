use c3p0::{DataType, IdType, Model, VersionType};
use serde::{Deserialize, Serialize};

use crate::web::types::types::MaybeWeb;

#[derive(Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "poem_openapi", derive(poem_openapi::Object))]
pub struct ModelDto<Id: MaybeWeb, Data: MaybeWeb> {
    pub id: Id,
    pub version: VersionType,
    pub data: Data,
}

impl<Id: IdType + MaybeWeb, Data: DataType + MaybeWeb> From<Model<Id, Data>> for ModelDto<Id, Data> {
    fn from(model: Model<Id, Data>) -> Self {
        Self { id: model.id, version: model.version, data: model.data }
    }
}
