use lightspeed_core::error::{ErrorDetails, LightSpeedError};
use lightspeed_core::service::auth::Owned;
use lightspeed_core::service::validator::must_match::validate_must_match;
use lightspeed_core::service::validator::Validable;
use serde_derive::{Deserialize, Serialize};

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

impl Validable for &ChangePasswordDto {
    fn validate(&self, error_details: &mut ErrorDetails) -> Result<(), LightSpeedError> {
        validate_must_match(
            error_details,
            "new_password",
            &self.new_password,
            "new_password_confirm",
            &self.new_password_confirm,
        );
        Ok(())
    }
}
