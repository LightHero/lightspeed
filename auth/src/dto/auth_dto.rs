use lightspeed_core::service::auth::Auth;
use serde_derive::{Deserialize, Serialize};
use typescript_definitions::TypeScriptify;

#[derive(Serialize, Deserialize, TypeScriptify)]
#[serde(rename_all = "camelCase")]
pub struct AuthDto {
    pub auth: Auth,
}
