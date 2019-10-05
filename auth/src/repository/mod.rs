use crate::model::auth_account::{AuthAccountData, AuthAccountModel};
use crate::model::token::{TokenData, TokenModel};
use c3p0::{C3p0Pool, Model, NewModel};
use lightspeed_core::error::LightSpeedError;

pub mod pg;

pub trait AuthRepositoryManager: Clone {
    type CONN;
    type C3P0: C3p0Pool<CONN = Self::CONN>;
    type AUTH_ACCOUNT_REPO: AuthAccountRepository<CONN = Self::CONN>;
    type TOKEN_REPO: TokenRepository<CONN = Self::CONN>;

    fn c3p0(&self) -> &Self::C3P0;
    fn start(&self) -> Result<(), LightSpeedError>;
    fn auth_account_repo(&self) -> Self::AUTH_ACCOUNT_REPO;
    fn token_repo(&self) -> Self::TOKEN_REPO;
}

pub trait AuthAccountRepository: Clone {
    type CONN;

    fn fetch_by_id(
        &self,
        conn: &Self::CONN,
        user_id: i64,
    ) -> Result<AuthAccountModel, LightSpeedError>;

    fn fetch_by_username(
        &self,
        conn: &Self::CONN,
        username: &str,
    ) -> Result<AuthAccountModel, LightSpeedError>;

    fn fetch_by_username_optional(
        &self,
        conn: &Self::CONN,
        username: &str,
    ) -> Result<Option<AuthAccountModel>, LightSpeedError>;

    fn fetch_by_email_optional(
        &self,
        conn: &Self::CONN,
        email: &str,
    ) -> Result<Option<AuthAccountModel>, LightSpeedError>;

    fn save(
        &self,
        conn: &Self::CONN,
        model: NewModel<AuthAccountData>,
    ) -> Result<AuthAccountModel, LightSpeedError>;

    fn update(
        &self,
        conn: &Self::CONN,
        model: Model<AuthAccountData>,
    ) -> Result<AuthAccountModel, LightSpeedError>;

    fn delete(&self, conn: &Self::CONN, model: &AuthAccountModel) -> Result<u64, LightSpeedError>;
}

pub trait TokenRepository: Clone {
    type CONN;

    fn fetch_by_token(
        &self,
        conn: &Self::CONN,
        token_string: &str,
    ) -> Result<TokenModel, LightSpeedError>;

    fn save(&self, conn: &Self::CONN, model: NewModel<TokenData>) -> Result<TokenModel, LightSpeedError>;

    fn delete(&self, conn: &Self::CONN, model: &TokenModel) -> Result<u64, LightSpeedError>;
}
