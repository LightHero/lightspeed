use lightspeed_core::service::auth::Auth;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[cfg_attr(feature = "poem_openapi", derive(poem_openapi::Object))]
pub struct AuthDto {
    pub auth: Auth<u64>,
}
