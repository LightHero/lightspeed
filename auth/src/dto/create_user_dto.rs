use lightspeed_core::error::{ErrorDetails, LightSpeedError};
use lightspeed_core::model::language::Language;
use lightspeed_core::service::validator::boolean::validate_is_true;
use lightspeed_core::service::validator::email::validate_email;
use lightspeed_core::service::validator::must_match::validate_must_match;
use lightspeed_core::service::validator::Validable;
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Serialize, Deserialize)]
pub struct CreateLoginDto {
    pub username: String,
    pub email: String,
    pub password: String,
    pub password_confirm: String,
    pub language: Language,
    pub data: HashMap<String, String>,
    pub accept_privacy_policy: bool,
}

impl Validable for &CreateLoginDto {
    fn validate(&self, error_details: &mut ErrorDetails) -> Result<(), LightSpeedError> {
        validate_must_match(
            error_details,
            "password",
            &self.password,
            "password_confirm",
            &self.password_confirm,
        );
        validate_is_true(
            error_details,
            "accept_privacy_policy",
            self.accept_privacy_policy,
        );
        validate_email(error_details, "email", &self.email);
        Ok(())
    }
}
