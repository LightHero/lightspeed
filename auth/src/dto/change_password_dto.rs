use lightspeed_core::error::{ErrorDetails, LsError};
use lightspeed_core::service::auth::Owned;
use lightspeed_core::service::validator::Validable;
use lightspeed_core::service::validator::must_match::validate_must_be_equals;
use serde::{Deserialize, Serialize};

use crate::dto::{MIN_PASSWORD_LEN, validate_min_password_len};

#[derive(Serialize, Deserialize)]
pub struct ChangePasswordDto {
    pub user_id: u64,
    pub old_password: String,
    pub new_password: String,
    pub new_password_confirm: String,
}

impl Owned for ChangePasswordDto {
    fn get_owner_id(&self) -> u64 {
        self.user_id
    }
}

impl Validable for ChangePasswordDto {
    fn validate(&self, error_details: &mut ErrorDetails) -> Result<(), LsError> {
        validate_must_be_equals(
            error_details,
            "new_password",
            &self.new_password,
            "new_password_confirm",
            &self.new_password_confirm,
        );
        validate_min_password_len(error_details, "new_password", &self.new_password, MIN_PASSWORD_LEN);
        Ok(())
    }
}
