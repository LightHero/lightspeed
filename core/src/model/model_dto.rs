use c3p0::{DataType, Record};
use serde::{Deserialize, Serialize};

use crate::web::types::MaybeWeb;

#[derive(Clone, Serialize, Deserialize)]
pub struct RecordDto<Data: MaybeWeb> {
    pub id: u64,
    pub version: i32,
    pub data: Data,
}

impl<Data: DataType + MaybeWeb> From<Record<Data>> for RecordDto<Data> {
    fn from(model: Record<Data>) -> Self {
        Self { id: model.id, version: model.version, data: model.data }
    }
}
