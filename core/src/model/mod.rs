use crate::service::auth::Owned;
use crate::service::validator::ownership::WithIdAndVersion;

pub mod boolean;
pub mod language;
pub mod model_dto;

pub struct ModelWithOwner {
    pub id: i64,
    pub version: i32,
    pub user_id: i64,
}

impl Owned for ModelWithOwner {
    fn get_owner_id(&self) -> i64 {
        self.user_id
    }
}

impl WithIdAndVersion for ModelWithOwner {
    fn get_id(&self) -> i64 {
        self.id
    }

    fn get_version(&self) -> i32 {
        self.version
    }
}

impl <T: Owned + WithIdAndVersion> From<&T> for ModelWithOwner {
    fn from(model: &T) -> Self {
        ModelWithOwner {
            id: model.get_id(),
            version: model.get_version(),
            user_id: model.get_owner_id()
        }
    }
}
