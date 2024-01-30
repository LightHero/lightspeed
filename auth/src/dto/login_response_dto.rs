use crate::dto::token_dto::TokenDto;
use lightspeed_core::{service::auth::Auth, web::types::types::MaybeWeb};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[cfg_attr(feature = "poem_openapi", derive(poem_openapi::Object))]
pub struct LoginResponseDto<Id: MaybeWeb> {
    pub auth: Auth<Id>,
    pub token: TokenDto,
}
