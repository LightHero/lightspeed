use crate::dto::token_dto::TokenDto;
use lightspeed_core::service::auth::Auth;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct LoginResponseDto {
    pub auth: Auth<u64>,
    pub token: TokenDto,
}
