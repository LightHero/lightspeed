use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[cfg_attr(feature = "poem_openapi_", derive(poem_openapi::Object))]
pub struct TokenDto {
    pub token: String,
    pub expiration_epoch_seconds: i64,
}
