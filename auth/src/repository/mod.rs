use crate::model::auth_account::{AuthAccountData, AuthAccountModel};
use crate::model::token::{TokenData, TokenModel};
use c3p0::*;
use lightspeed_core::error::LightSpeedError;

pub mod pg;

#[async_trait::async_trait]
pub trait AuthRepositoryManager: Clone + Send + Sync {
    type Conn: SqlConnectionAsync;
    type C3P0: C3p0PoolAsync<Conn = Self::Conn>;
    type AuthAccountRepo: AuthAccountRepository<Conn = Self::Conn>;
    type TokenRepo: TokenRepository<Conn = Self::Conn>;

    fn c3p0(&self) -> &Self::C3P0;
    async fn start(&self) -> Result<(), LightSpeedError>;
    fn auth_account_repo(&self) -> Self::AuthAccountRepo;
    fn token_repo(&self) -> Self::TokenRepo;
}

#[async_trait::async_trait]
pub trait AuthAccountRepository: Clone + Send + Sync {
    type Conn: SqlConnectionAsync;

    async fn fetch_by_id(
        &self,
        conn: &mut Self::Conn,
        user_id: i64,
    ) -> Result<AuthAccountModel, LightSpeedError>;

    async fn fetch_by_username(
        &self,
        conn: &mut Self::Conn,
        username: &str,
    ) -> Result<AuthAccountModel, LightSpeedError>;

    async fn fetch_by_username_optional(
        &self,
        conn: &mut Self::Conn,
        username: &str,
    ) -> Result<Option<AuthAccountModel>, LightSpeedError>;

    async fn fetch_by_email_optional(
        &self,
        conn: &mut Self::Conn,
        email: &str,
    ) -> Result<Option<AuthAccountModel>, LightSpeedError>;

    async fn save(
        &self,
        conn: &mut Self::Conn,
        model: NewModel<AuthAccountData>,
    ) -> Result<AuthAccountModel, LightSpeedError>;

    async fn update(
        &self,
        conn: &mut Self::Conn,
        model: Model<AuthAccountData>,
    ) -> Result<AuthAccountModel, LightSpeedError>;

    async fn delete(
        &self,
        conn: &mut Self::Conn,
        model: AuthAccountModel,
    ) -> Result<AuthAccountModel, LightSpeedError>;
}

#[async_trait::async_trait]
pub trait TokenRepository: Clone + Send + Sync {
    type Conn: SqlConnectionAsync;

    async fn fetch_by_token(
        &self,
        conn: &mut Self::Conn,
        token_string: &str,
    ) -> Result<TokenModel, LightSpeedError>;

    async fn fetch_by_username(
        &self,
        conn: &mut Self::Conn,
        username: &str,
    ) -> Result<Vec<TokenModel>, LightSpeedError>;

    async fn save(
        &self,
        conn: &mut Self::Conn,
        model: NewModel<TokenData>,
    ) -> Result<TokenModel, LightSpeedError>;

    async fn delete(
        &self,
        conn: &mut Self::Conn,
        model: TokenModel,
    ) -> Result<TokenModel, LightSpeedError>;
}
