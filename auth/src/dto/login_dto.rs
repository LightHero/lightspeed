use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[cfg_attr(feature = "poem_openapi_", derive(poem_openapi::Object))]
pub struct LoginDto {
    pub username: String,
    pub password: String,
}
