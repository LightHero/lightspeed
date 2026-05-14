use crate::dto::validate_must_be_equals;
use lightspeed_core::error::ErrorDetails;
use lightspeed_core::service::auth::Owned;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct ChangePasswordDto {
    pub user_id: i64,
    pub old_password: String,
    pub new_password: String,
    pub new_password_confirm: String,
}

impl Owned for ChangePasswordDto {
    fn get_owner_id(&self) -> i64 {
        self.user_id
    }
}

impl ChangePasswordDto {
    pub(crate) fn validate(&self, error_details: &mut ErrorDetails) {
        validate_must_be_equals(
            error_details,
            "new_password",
            &self.new_password,
            "new_password_confirm",
            &self.new_password_confirm,
        );
    }
}
