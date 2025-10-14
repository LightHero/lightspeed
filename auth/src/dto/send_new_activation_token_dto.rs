use lightspeed_core::model::language::Language;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct SendNewActivationTokenDto {
    pub token: String,
    pub language: Language,
}

#[derive(Serialize, Deserialize)]
pub struct SendNewActivationTokenByUsernameAndEmailDto {
    pub username: String,
    pub email: String,
    pub language: Language,
}
