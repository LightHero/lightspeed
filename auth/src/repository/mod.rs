use crate::model::auth_account::{AuthAccountData, AuthAccountModel, AuthAccountStatus};
use crate::model::token::{TokenData, TokenModel};
use c3p0::*;
use lightspeed_core::error::LsError;

pub mod pg;

#[async_trait::async_trait]
pub trait AuthRepositoryManager: Clone + Send + Sync {
    type Tx: SqlTx;
    type C3P0: C3p0Pool<Tx = Self::Tx>;
    type AuthAccountRepo: AuthAccountRepository<Tx = Self::Tx>;
    type TokenRepo: TokenRepository<Tx = Self::Tx>;

    fn c3p0(&self) -> &Self::C3P0;
    async fn start(&self) -> Result<(), LsError>;
    fn auth_account_repo(&self) -> Self::AuthAccountRepo;
    fn token_repo(&self) -> Self::TokenRepo;
}

#[async_trait::async_trait]
pub trait AuthAccountRepository: Clone + Send + Sync {
    type Tx: SqlTx;

    async fn fetch_all_by_status(
        &self,
        tx: &mut Self::Tx,
        status: AuthAccountStatus,
        start_user_id: i64,
        limit: u32,
    ) -> Result<Vec<AuthAccountModel>, LsError>;

    async fn fetch_by_id(&self, tx: &mut Self::Tx, user_id: i64) -> Result<AuthAccountModel, LsError>;

    async fn fetch_by_username(&self, tx: &mut Self::Tx, username: &str) -> Result<AuthAccountModel, LsError>;

    async fn fetch_by_username_optional(
        &self,
        tx: &mut Self::Tx,
        username: &str,
    ) -> Result<Option<AuthAccountModel>, LsError>;

    async fn fetch_by_email_optional(
        &self,
        tx: &mut Self::Tx,
        email: &str,
    ) -> Result<Option<AuthAccountModel>, LsError>;

    async fn save(&self, tx: &mut Self::Tx, model: NewModel<AuthAccountData>) -> Result<AuthAccountModel, LsError>;

    async fn update(&self, tx: &mut Self::Tx, model: Model<AuthAccountData>) -> Result<AuthAccountModel, LsError>;

    async fn delete(&self, tx: &mut Self::Tx, model: AuthAccountModel) -> Result<AuthAccountModel, LsError>;

    async fn delete_by_id(&self, tx: &mut Self::Tx, user_id: i64) -> Result<u64, LsError>;
}

#[async_trait::async_trait]
pub trait TokenRepository: Clone + Send + Sync {
    type Tx: SqlTx;

    async fn fetch_by_token(&self, tx: &mut Self::Tx, token_string: &str) -> Result<TokenModel, LsError>;

    async fn fetch_by_username(&self, tx: &mut Self::Tx, username: &str) -> Result<Vec<TokenModel>, LsError>;

    async fn save(&self, tx: &mut Self::Tx, model: NewModel<TokenData>) -> Result<TokenModel, LsError>;

    async fn delete(&self, tx: &mut Self::Tx, model: TokenModel) -> Result<TokenModel, LsError>;
}
