use lightspeed_core::model::language::Language;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[cfg_attr(feature = "poem_openapi", derive(poem_openapi::Object))]
pub struct SendResetPasswordDto {
    pub email: String,
    pub language: Language,
}
