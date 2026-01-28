use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct TaskDto {
    pub token: String,
    pub expiration_epoch_seconds: i64,
}
