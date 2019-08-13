use std::collections::HashMap;
use lightspeed_core::service::validator::Validable;
use lightspeed_core::error::{LightSpeedError, ErrorDetails};
use lightspeed_core::service::validator::must_match::validate_must_match;
use lightspeed_core::service::validator::boolean::validate_is_true;
use lightspeed_core::model::language::Language;
use serde_derive::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct CreateLoginDto{
    pub email: String,
    pub password: String,
    pub password_confirm: String,
    pub language: Language,
    pub data: HashMap<String, String>,
    pub accept_privacy_policy: bool
}

impl Validable for &CreateLoginDto {
    fn validate(&self, error_details: &mut ErrorDetails) -> Result<(), LightSpeedError> {
        validate_must_match(error_details, "password", &self.password, "password_confirm", &self.password_confirm);
        validate_is_true(error_details, "accept_privacy_policy", self.accept_privacy_policy);
        Ok(())
    }
}
