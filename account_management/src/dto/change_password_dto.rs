use lightspeed_core::service::auth::Owned;
use lightspeed_validator::Validable;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Validable)]
#[validate(fields_match(new_password, new_password_confirm, attach_to_fields = true))]
pub struct ChangePasswordDto {
    pub user_id: i64,
    pub old_password: String,
    #[validate(password)]
    #[validate(length(min = 8))]
    pub new_password: String,
    pub new_password_confirm: String,
}

impl Owned for ChangePasswordDto {
    fn get_owner_id(&self) -> i64 {
        self.user_id
    }
}
