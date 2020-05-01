use serde::{Deserialize, Serialize};
use typescript_definitions::TypeScriptify;

#[derive(Serialize, Deserialize, TypeScriptify)]
#[serde(rename_all = "camelCase")]
pub struct LoginDto {
    pub username: String,
    pub password: String,
}
