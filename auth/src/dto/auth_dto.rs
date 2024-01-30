use lightspeed_core::{service::auth::Auth, web::types::MaybeWeb};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[cfg_attr(feature = "poem_openapi", derive(poem_openapi::Object))]
pub struct AuthDto<Id: MaybeWeb> {
    pub auth: Auth<Id>,
}
