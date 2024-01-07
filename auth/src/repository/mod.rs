use crate::model::auth_account::{AuthAccountData, AuthAccountModel, AuthAccountStatus};
use crate::model::token::{TokenData, TokenModel};
use c3p0::*;
use lightspeed_core::error::LsError;

pub mod pg;

#[async_trait::async_trait]
pub trait AuthRepositoryManager<Id: IdType>: Clone + Send + Sync {
    type Tx: Send + Sync;
    type C3P0: C3p0Pool<Tx = Self::Tx>;
    type AuthAccountRepo: AuthAccountRepository<Id, Tx = Self::Tx>;
    type TokenRepo: TokenRepository<Id, Tx = Self::Tx>;

    fn c3p0(&self) -> &Self::C3P0;
    async fn start(&self) -> Result<(), LsError>;
    fn auth_account_repo(&self) -> Self::AuthAccountRepo;
    fn token_repo(&self) -> Self::TokenRepo;
}

#[async_trait::async_trait]
pub trait AuthAccountRepository<Id: IdType>: Clone + Send + Sync {
    type Tx: Send + Sync;

    async fn fetch_all_by_status(
        &self,
        tx: &mut Self::Tx,
        status: AuthAccountStatus,
        start_user_id: i64,
        limit: u32,
    ) -> Result<Vec<AuthAccountModel<Id>>, LsError>;

    async fn fetch_by_id(&self, tx: &mut Self::Tx, user_id: i64) -> Result<AuthAccountModel<Id>, LsError>;

    async fn fetch_by_username(&self, tx: &mut Self::Tx, username: &str) -> Result<AuthAccountModel<Id>, LsError>;

    async fn fetch_by_username_optional(
        &self,
        tx: &mut Self::Tx,
        username: &str,
    ) -> Result<Option<AuthAccountModel<Id>>, LsError>;

    async fn fetch_by_email_optional(
        &self,
        tx: &mut Self::Tx,
        email: &str,
    ) -> Result<Option<AuthAccountModel<Id>>, LsError>;

    async fn save(&self, tx: &mut Self::Tx, model: NewModel<AuthAccountData>) -> Result<AuthAccountModel<Id>, LsError>;

    async fn update(&self, tx: &mut Self::Tx, model: Model<Id, AuthAccountData>) -> Result<AuthAccountModel<Id>, LsError>;

    async fn delete(&self, tx: &mut Self::Tx, model: AuthAccountModel<Id>) -> Result<AuthAccountModel<Id>, LsError>;

    async fn delete_by_id(&self, tx: &mut Self::Tx, user_id: i64) -> Result<u64, LsError>;
}

#[async_trait::async_trait]
pub trait TokenRepository<Id: IdType>: Clone + Send + Sync {
    
    type Tx: Send + Sync;

    async fn fetch_by_token(&self, tx: &mut Self::Tx, token_string: &str) -> Result<TokenModel<Id>, LsError>;

    async fn fetch_by_username(&self, tx: &mut Self::Tx, username: &str) -> Result<Vec<TokenModel<Id>>, LsError>;

    async fn save(&self, tx: &mut Self::Tx, model: NewModel<TokenData>) -> Result<TokenModel<Id>, LsError>;

    async fn delete(&self, tx: &mut Self::Tx, model: TokenModel<Id>) -> Result<TokenModel<Id>, LsError>;
}
