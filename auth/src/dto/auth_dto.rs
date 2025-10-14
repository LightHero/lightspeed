use lightspeed_core::service::auth::Auth;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct AuthDto {
    pub auth: Auth,
}
