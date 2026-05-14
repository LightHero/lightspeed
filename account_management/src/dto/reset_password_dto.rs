use lightspeed_validator::Validable;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Validable)]
#[validate(fields_match(password, password_confirm, attach_to_fields = true))]
pub struct ResetPasswordDto {
    pub token: String,
    #[validate(password)]
    #[validate(length(min = 8))]
    pub password: String,
    pub password_confirm: String,
}
