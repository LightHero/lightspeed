use crate::service::auth::Owned;
use crate::service::validator::ownership::WithIdAndVersion;
use c3p0::DataType;
use serde::{Deserialize, Serialize};

pub mod language;
pub mod model_dto;

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]

pub struct ModelWithOwner<Data> {
    pub id: u64,
    pub version: u32,
    pub user_id: u64,
    pub data: Data,
}

impl<Data: DataType> Owned for ModelWithOwner<Data> {
    fn get_owner_id(&self) -> u64 {
        self.user_id
    }
}

impl<Data: DataType> WithIdAndVersion for ModelWithOwner<Data> {
    fn get_id(&self) -> u64 {
        self.id
    }

    fn get_version(&self) -> u32 {
        self.version
    }
}

impl<Data: DataType, T: Owned + WithIdAndVersion> From<(&T, Data)> for ModelWithOwner<Data> {
    fn from((model, data): (&T, Data)) -> Self {
        ModelWithOwner { id: model.get_id(), version: model.get_version(), user_id: model.get_owner_id(), data }
    }
}

impl<T: Owned + WithIdAndVersion> From<&T> for ModelWithOwner<()> {
    fn from(model: &T) -> Self {
        ModelWithOwner { id: model.get_id(), version: model.get_version(), user_id: model.get_owner_id(), data: () }
    }
}
