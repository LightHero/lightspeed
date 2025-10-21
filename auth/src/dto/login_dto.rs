use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct LoginDto {
    pub username: String,
    pub password: String,
}
