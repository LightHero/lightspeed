use lightspeed_core::error::{ErrorDetails, LsError};
use lightspeed_core::service::auth::Owned;
use lightspeed_core::service::validator::must_match::validate_must_be_equals;
use lightspeed_core::service::validator::Validable;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[cfg_attr(feature = "poem_openapi", derive(poem_openapi::Object))]
pub struct ChangePasswordDto<Id: Send + Sync> {
    pub user_id: Id,
    pub old_password: String,
    pub new_password: String,
    pub new_password_confirm: String,
}

impl <Id: Send + Sync> Owned<Id> for ChangePasswordDto<Id> {
    fn get_owner_id(&self) -> &Id {
        &self.user_id
    }
}

impl <Id: Send + Sync> Validable for ChangePasswordDto<Id> {
    fn validate(&self, error_details: &mut ErrorDetails) -> Result<(), LsError> {
        validate_must_be_equals(
            error_details,
            "new_password",
            &self.new_password,
            "new_password_confirm",
            &self.new_password_confirm,
        );
        Ok(())
    }
}
