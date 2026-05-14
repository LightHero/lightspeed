use lightspeed_core::model::language::Language;
use lightspeed_validator::Validable;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Serialize, Deserialize, Validable)]
#[validate(fields_match(password, password_confirm, attach_to_fields = true))]
pub struct CreateLoginDto {
    pub username: Option<String>,
    #[validate(email)]
    pub email: String,
    #[validate(password)]
    #[validate(length(min = 8))]
    pub password: String,
    pub password_confirm: String,
    pub language: Language,
    pub data: HashMap<String, String>,
    #[validate(isTrue)]
    pub accept_privacy_policy: bool,
}
