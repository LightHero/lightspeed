use crate::model::auth_account::{AuthAccountData, AuthAccountModel, AuthAccountStatus};
use crate::model::token::{TokenData, TokenModel};
use c3p0::*;
use lightspeed_core::error::LsError;

pub mod pg;

#[async_trait::async_trait]
pub trait AuthRepositoryManager: Clone + Send + Sync {
    type Conn: SqlConnection;
    type C3P0: C3p0Pool<Conn = Self::Conn>;
    type AuthAccountRepo: AuthAccountRepository<Conn = Self::Conn>;
    type TokenRepo: TokenRepository<Conn = Self::Conn>;

    fn c3p0(&self) -> &Self::C3P0;
    async fn start(&self) -> Result<(), LsError>;
    fn auth_account_repo(&self) -> Self::AuthAccountRepo;
    fn token_repo(&self) -> Self::TokenRepo;
}

#[async_trait::async_trait]
pub trait AuthAccountRepository: Clone + Send + Sync {
    type Conn: SqlConnection;

    async fn fetch_all_by_status(
        &self,
        conn: &mut Self::Conn,
        status: AuthAccountStatus,
        start_user_id: i64,
        limit: u32,
    ) -> Result<Vec<AuthAccountModel>, LsError>;

    async fn fetch_by_id(&self, conn: &mut Self::Conn, user_id: i64) -> Result<AuthAccountModel, LsError>;

    async fn fetch_by_username(
        &self,
        conn: &mut Self::Conn,
        username: &str,
    ) -> Result<AuthAccountModel, LsError>;

    async fn fetch_by_username_optional(
        &self,
        conn: &mut Self::Conn,
        username: &str,
    ) -> Result<Option<AuthAccountModel>, LsError>;

    async fn fetch_by_email_optional(
        &self,
        conn: &mut Self::Conn,
        email: &str,
    ) -> Result<Option<AuthAccountModel>, LsError>;

    async fn save(
        &self,
        conn: &mut Self::Conn,
        model: NewModel<AuthAccountData>,
    ) -> Result<AuthAccountModel, LsError>;

    async fn update(
        &self,
        conn: &mut Self::Conn,
        model: Model<AuthAccountData>,
    ) -> Result<AuthAccountModel, LsError>;

    async fn delete(&self, conn: &mut Self::Conn, model: AuthAccountModel)
        -> Result<AuthAccountModel, LsError>;

    async fn delete_by_id(&self, conn: &mut Self::Conn, user_id: i64) -> Result<u64, LsError>;
}

#[async_trait::async_trait]
pub trait TokenRepository: Clone + Send + Sync {
    type Conn: SqlConnection;

    async fn fetch_by_token(&self, conn: &mut Self::Conn, token_string: &str) -> Result<TokenModel, LsError>;

    async fn fetch_by_username(
        &self,
        conn: &mut Self::Conn,
        username: &str,
    ) -> Result<Vec<TokenModel>, LsError>;

    async fn save(&self, conn: &mut Self::Conn, model: NewModel<TokenData>) -> Result<TokenModel, LsError>;

    async fn delete(&self, conn: &mut Self::Conn, model: TokenModel) -> Result<TokenModel, LsError>;
}
