use lightspeed_core::model::language::Language;
use serde::{Deserialize, Serialize};
use typescript_definitions::TypeScriptify;

#[derive(Serialize, Deserialize, TypeScriptify)]
#[serde(rename_all = "camelCase")]
pub struct SendResetPasswordDto {
    pub email: String,
    pub language: Language,
}
