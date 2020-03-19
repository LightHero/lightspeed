use lightspeed_core::model::language::Language;
use serde_derive::{Deserialize, Serialize};
use typescript_definitions::TypeScriptify;

#[derive(Serialize, Deserialize, TypeScriptify)]
#[serde(rename_all = "camelCase")]
pub struct SendNewActivationTokenDto {
    pub token: String,
    pub language: Language,
}
