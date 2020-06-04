use lightspeed_core::model::language::Language;
use serde::{Deserialize, Serialize};
use typescript_definitions::TypeScriptify;

#[derive(Serialize, Deserialize, TypeScriptify)]
#[serde(rename_all = "camelCase")]
pub struct SendNewActivationTokenDto {
    pub token: String,
    pub language: Language,
}

#[derive(Serialize, Deserialize, TypeScriptify)]
#[serde(rename_all = "camelCase")]
pub struct SendNewActivationTokenByUsernameAndEmailDto {
    pub username: String,
    pub email: String,
    pub language: Language,
}
