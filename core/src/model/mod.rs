use crate::service::auth::Owned;
use crate::service::validator::ownership::WithIdAndVersion;
use c3p0::{DataType, IdType, VersionType};
use serde::{Deserialize, Serialize};

pub mod language;
pub mod model_dto;

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]

pub struct ModelWithOwner<Id, Data> {
    pub id: Id,
    pub version: VersionType,
    pub user_id: Id,
    pub data: Data,
}

impl<Id: IdType, Data: DataType> Owned<Id> for ModelWithOwner<Id, Data> {
    fn get_owner_id(&self) -> &Id {
        &self.user_id
    }
}

impl<Id: IdType, Data: DataType> WithIdAndVersion<Id> for ModelWithOwner<Id, Data> {
    fn get_id(&self) -> &Id {
        &self.id
    }

    fn get_version(&self) -> VersionType {
        self.version
    }
}

impl<Id: IdType, Data: DataType, T: Owned<Id> + WithIdAndVersion<Id>> From<(&T, Data)> for ModelWithOwner<Id, Data> {
    fn from((model, data): (&T, Data)) -> Self {
        ModelWithOwner {
            id: model.get_id().clone(),
            version: model.get_version(),
            user_id: model.get_owner_id().clone(),
            data,
        }
    }
}

impl<Id: IdType, T: Owned<Id> + WithIdAndVersion<Id>> From<&T> for ModelWithOwner<Id, ()> {
    fn from(model: &T) -> Self {
        ModelWithOwner {
            id: model.get_id().clone(),
            version: model.get_version(),
            user_id: model.get_owner_id().clone(),
            data: (),
        }
    }
}
