use lightspeed_core::error::{ErrorDetails, LightSpeedError};
use lightspeed_core::service::validator::must_match::validate_must_be_equals;
use lightspeed_core::service::validator::Validable;
use serde::{Deserialize, Serialize};
use typescript_definitions::TypeScriptify;

#[derive(Serialize, Deserialize, TypeScriptify)]
#[serde(rename_all = "camelCase")]
pub struct ResetPasswordDto {
    pub token: String,
    pub password: String,
    pub password_confirm: String,
}

impl Validable for ResetPasswordDto {
    fn validate(&self, error_details: &mut ErrorDetails) -> Result<(), LightSpeedError> {
        validate_must_be_equals(error_details, "password", &self.password, "password_confirm", &self.password_confirm);
        Ok(())
    }
}
