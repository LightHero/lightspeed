use crate::service::auth::Owned;
use crate::service::validator::ownership::WithIdAndVersion;
use serde::{Deserialize, Serialize};
use typescript_definitions::TypeScriptify;

pub mod boolean;
pub mod language;
pub mod model_dto;

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize, TypeScriptify)]
pub struct ModelWithOwner<D> {
    pub id: i64,
    pub version: i32,
    pub user_id: i64,
    pub data: D,
}

impl <D> Owned for ModelWithOwner<D> {
    fn get_owner_id(&self) -> i64 {
        self.user_id
    }
}

impl <D> WithIdAndVersion for ModelWithOwner<D> {
    fn get_id(&self) -> i64 {
        self.id
    }

    fn get_version(&self) -> i32 {
        self.version
    }
}

impl <T: Owned + WithIdAndVersion, D> From<(&T, D)> for ModelWithOwner<D> {
    fn from((model, data): (&T, D)) -> Self {
        ModelWithOwner {
            id: model.get_id(),
            version: model.get_version(),
            user_id: model.get_owner_id(),
            data,
        }
    }
}

impl <T: Owned + WithIdAndVersion> From<&T> for ModelWithOwner<()> {
    fn from(model: &T) -> Self {
        ModelWithOwner {
            id: model.get_id(),
            version: model.get_version(),
            user_id: model.get_owner_id(),
            data: (),
        }
    }
}
