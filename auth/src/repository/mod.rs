use crate::model::auth_account::{AuthAccountData, AuthAccountModel};
use crate::model::token::{TokenData, TokenModel};
use c3p0::{C3p0Pool, Model, NewModel};
use lightspeed_core::error::LightSpeedError;

pub mod pg;

pub trait AuthRepositoryManager: Clone {
    type Conn;
    type C3P0: C3p0Pool<CONN = Self::Conn>;
    type AuthAccountRepo: AuthAccountRepository<Conn = Self::Conn>;
    type TokenRepo: TokenRepository<Conn = Self::Conn>;

    fn c3p0(&self) -> &Self::C3P0;
    fn start(&self) -> Result<(), LightSpeedError>;
    fn auth_account_repo(&self) -> Self::AuthAccountRepo;
    fn token_repo(&self) -> Self::TokenRepo;
}

pub trait AuthAccountRepository: Clone {
    type Conn;

    fn fetch_by_id(&self, conn: &mut Self::Conn, user_id: i64) -> Result<AuthAccountModel, LightSpeedError>;

    fn fetch_by_username(&self, conn: &mut Self::Conn, username: &str) -> Result<AuthAccountModel, LightSpeedError>;

    fn fetch_by_username_optional(&self, conn: &mut Self::Conn, username: &str) -> Result<Option<AuthAccountModel>, LightSpeedError>;

    fn fetch_by_email_optional(&self, conn: &mut Self::Conn, email: &str) -> Result<Option<AuthAccountModel>, LightSpeedError>;

    fn save(&self, conn: &mut Self::Conn, model: NewModel<AuthAccountData>) -> Result<AuthAccountModel, LightSpeedError>;

    fn update(&self, conn: &mut Self::Conn, model: Model<AuthAccountData>) -> Result<AuthAccountModel, LightSpeedError>;

    fn delete(&self, conn: &mut Self::Conn, model: AuthAccountModel) -> Result<AuthAccountModel, LightSpeedError>;
}

pub trait TokenRepository: Clone {
    type Conn;

    fn fetch_by_token(&self, conn: &mut Self::Conn, token_string: &str) -> Result<TokenModel, LightSpeedError>;

    fn save(&self, conn: &mut Self::Conn, model: NewModel<TokenData>) -> Result<TokenModel, LightSpeedError>;

    fn delete(&self, conn: &mut Self::Conn, model: TokenModel) -> Result<TokenModel, LightSpeedError>;
}
