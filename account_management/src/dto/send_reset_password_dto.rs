use lightspeed_core::model::language::Language;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct SendResetPasswordDto {
    pub email: String,
    pub language: Language,
}
