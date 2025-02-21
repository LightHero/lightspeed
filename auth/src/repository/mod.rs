use std::future::Future;

use crate::model::auth_account::{AuthAccountData, AuthAccountModel, AuthAccountStatus};
use crate::model::token::{TokenData, TokenModel};
use c3p0::*;
use lightspeed_core::error::LsError;

#[cfg(feature = "mysql")]
pub mod mysql;

#[cfg(feature = "postgres")]
pub mod pg;

#[cfg(feature = "sqlite")]
pub mod sqlite;

pub trait AuthRepositoryManager: Clone + Send + Sync {
    type Tx<'a>: Send + Sync;
    type C3P0: for<'a> C3p0Pool<Tx<'a> = Self::Tx<'a>>;
    type AuthAccountRepo: for<'a> AuthAccountRepository<Tx<'a> = Self::Tx<'a>>;
    type TokenRepo: for<'a> TokenRepository<Tx<'a> = Self::Tx<'a>>;

    fn c3p0(&self) -> &Self::C3P0;
    fn start(&self) -> impl Future<Output = Result<(), LsError>> + Send;
    fn auth_account_repo(&self) -> Self::AuthAccountRepo;
    fn token_repo(&self) -> Self::TokenRepo;
}

pub trait AuthAccountRepository: Clone + Send + Sync {
    type Tx<'a>: Send + Sync;

    fn fetch_all_by_status(
        &self,
        tx: &mut Self::Tx<'_>,
        status: AuthAccountStatus,
        start_user_id: &u64,
        limit: u32,
    ) -> impl Future<Output = Result<Vec<AuthAccountModel>, LsError>> + Send;

    fn fetch_by_id(
        &self,
        tx: &mut Self::Tx<'_>,
        user_id: &u64,
    ) -> impl Future<Output = Result<AuthAccountModel, LsError>> + Send;

    fn fetch_by_username(
        &self,
        tx: &mut Self::Tx<'_>,
        username: &str,
    ) -> impl Future<Output = Result<AuthAccountModel, LsError>> + Send;

    fn fetch_by_username_optional(
        &self,
        tx: &mut Self::Tx<'_>,
        username: &str,
    ) -> impl Future<Output = Result<Option<AuthAccountModel>, LsError>> + Send;

    fn fetch_by_email_optional(
        &self,
        tx: &mut Self::Tx<'_>,
        email: &str,
    ) -> impl Future<Output = Result<Option<AuthAccountModel>, LsError>> + Send;

    fn save(
        &self,
        tx: &mut Self::Tx<'_>,
        model: NewModel<AuthAccountData>,
    ) -> impl Future<Output = Result<AuthAccountModel, LsError>> + Send;

    fn update(
        &self,
        tx: &mut Self::Tx<'_>,
        model: AuthAccountModel,
    ) -> impl Future<Output = Result<AuthAccountModel, LsError>> + Send;

    fn delete(
        &self,
        tx: &mut Self::Tx<'_>,
        model: AuthAccountModel,
    ) -> impl Future<Output = Result<AuthAccountModel, LsError>> + Send;

    fn delete_by_id(&self, tx: &mut Self::Tx<'_>, user_id: &u64) -> impl Future<Output = Result<u64, LsError>> + Send;
}

pub trait TokenRepository: Clone + Send + Sync {
    type Tx<'a>: Send + Sync;

    fn fetch_by_token(
        &self,
        tx: &mut Self::Tx<'_>,
        token_string: &str,
    ) -> impl Future<Output = Result<TokenModel, LsError>> + Send;

    fn fetch_by_username(
        &self,
        tx: &mut Self::Tx<'_>,
        username: &str,
    ) -> impl Future<Output = Result<Vec<TokenModel>, LsError>> + Send;

    fn save(
        &self,
        tx: &mut Self::Tx<'_>,
        model: NewModel<TokenData>,
    ) -> impl Future<Output = Result<TokenModel, LsError>> + Send;

    fn delete(
        &self,
        tx: &mut Self::Tx<'_>,
        model: TokenModel,
    ) -> impl Future<Output = Result<TokenModel, LsError>> + Send;
}
