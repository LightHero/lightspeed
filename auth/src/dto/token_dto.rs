use serde_derive::{Deserialize, Serialize};
use typescript_definitions::TypeScriptify;

#[derive(Serialize, Deserialize, TypeScriptify)]
#[serde(rename_all = "camelCase")]
pub struct TokenDto {
    pub token: String,
    pub expiration_epoch_seconds: i64,
}
