use serde::{Deserialize, Serialize};
use typescript_definitions::TypeScriptify;
use lightspeed_core::model::language::Language;

#[derive(Serialize, Deserialize, TypeScriptify)]
#[serde(rename_all = "camelCase")]
pub struct LoginDto {
    pub username: String,
    pub password: String,
}
