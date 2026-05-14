use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct TokenDto {
    pub token: String,
    pub expiration_epoch_seconds: i64,
}
