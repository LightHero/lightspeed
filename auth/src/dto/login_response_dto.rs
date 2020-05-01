use crate::dto::token_dto::TokenDto;
use lightspeed_core::service::auth::Auth;
use serde::{Deserialize, Serialize};
use typescript_definitions::TypeScriptify;

#[derive(Serialize, Deserialize, TypeScriptify)]
#[serde(rename_all = "camelCase")]
pub struct LoginResponseDto {
    pub auth: Auth,
    pub token: TokenDto,
}
