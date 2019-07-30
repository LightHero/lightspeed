use crate::repository::token::{TokenModel, TokenRepository};
use crate::PoolManager;
use c3p0::*;
use ls_core::error::LightSpeedError;

#[derive(Clone)]
pub struct TokenService {
    c3p0: C3p0Pool<PoolManager>,
    token_repo: TokenRepository,
}

impl TokenService {
    pub fn new(c3p0: C3p0Pool<PoolManager>, token_repo: TokenRepository) -> Self {
        TokenService { c3p0, token_repo }
    }

    pub fn delete(&self, token_model: TokenModel) -> Result<u64, LightSpeedError> {
        let result = self
            .c3p0
            .transaction(|conn| Ok(self.token_repo.delete(conn, &token_model)?))?;
        Ok(result)
    }
}
