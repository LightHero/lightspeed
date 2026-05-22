use std::future::Future;

use crate::error::LsAccountManagementError;
use crate::model::auth_account::{AccountData, AccountStatus, AuthAccountModel};
use crate::model::token::{TokenData, TokenModel};
use c3p0::sqlx::Database;
use c3p0::*;
use lightspeed_core::error::LsError;

#[cfg(feature = "mysql")]
pub mod mysql;

#[cfg(feature = "postgres")]
pub mod postgres;

#[cfg(feature = "sqlite")]
pub mod sqlite;

pub trait AMRepositoryManager: Clone + Send + Sync {
    type DB: Database;
    type C3P0: C3p0Pool<DB = Self::DB>;
    type AccountRepo: for<'a> AccountRepository<DB = Self::DB>;
    type TokenRepo: for<'a> TokenRepository<DB = Self::DB>;

    fn c3p0(&self) -> &Self::C3P0;
    fn start(&self) -> impl Future<Output = Result<(), LsError>> + Send;
    fn account_repo(&self) -> Self::AccountRepo;
    fn token_repo(&self) -> Self::TokenRepo;
}

pub trait AccountRepository: Clone + Send + Sync {
    type DB: Database;

    fn fetch_all_by_status(
        &self,
        tx: &mut <Self::DB as Database>::Connection,
        status: AccountStatus,
        start_user_id: i64,
        limit: u32,
    ) -> impl Future<Output = Result<Vec<AuthAccountModel>, LsAccountManagementError>> + Send;

    fn fetch_by_id(
        &self,
        tx: &mut <Self::DB as Database>::Connection,
        user_id: i64,
    ) -> impl Future<Output = Result<AuthAccountModel, LsAccountManagementError>> + Send;

    fn fetch_by_username(
        &self,
        tx: &mut <Self::DB as Database>::Connection,
        username: &str,
    ) -> impl Future<Output = Result<AuthAccountModel, LsAccountManagementError>> + Send;

    fn fetch_by_username_optional(
        &self,
        tx: &mut <Self::DB as Database>::Connection,
        username: &str,
    ) -> impl Future<Output = Result<Option<AuthAccountModel>, LsAccountManagementError>> + Send;

    fn fetch_by_email_optional(
        &self,
        tx: &mut <Self::DB as Database>::Connection,
        email: &str,
    ) -> impl Future<Output = Result<Option<AuthAccountModel>, LsAccountManagementError>> + Send;

    fn save(
        &self,
        tx: &mut <Self::DB as Database>::Connection,
        model: NewRecord<AccountData>,
    ) -> impl Future<Output = Result<AuthAccountModel, LsAccountManagementError>> + Send;

    fn update(
        &self,
        tx: &mut <Self::DB as Database>::Connection,
        model: AuthAccountModel,
    ) -> impl Future<Output = Result<AuthAccountModel, LsAccountManagementError>> + Send;

    fn delete(
        &self,
        tx: &mut <Self::DB as Database>::Connection,
        model: AuthAccountModel,
    ) -> impl Future<Output = Result<AuthAccountModel, LsAccountManagementError>> + Send;

    fn delete_by_id(
        &self,
        tx: &mut <Self::DB as Database>::Connection,
        user_id: i64,
    ) -> impl Future<Output = Result<u64, LsAccountManagementError>> + Send;
}

pub trait TokenRepository: Clone + Send + Sync {
    type DB: Database;

    fn fetch_by_token(
        &self,
        tx: &mut <Self::DB as Database>::Connection,
        token_string: &str,
    ) -> impl Future<Output = Result<TokenModel, LsAccountManagementError>> + Send;

    fn fetch_by_username(
        &self,
        tx: &mut <Self::DB as Database>::Connection,
        username: &str,
    ) -> impl Future<Output = Result<Vec<TokenModel>, LsAccountManagementError>> + Send;

    fn save(
        &self,
        tx: &mut <Self::DB as Database>::Connection,
        model: NewRecord<TokenData>,
    ) -> impl Future<Output = Result<TokenModel, LsAccountManagementError>> + Send;

    fn delete(
        &self,
        tx: &mut <Self::DB as Database>::Connection,
        model: TokenModel,
    ) -> impl Future<Output = Result<TokenModel, LsAccountManagementError>> + Send;

    fn delete_expired(
        &self,
        tx: &mut <Self::DB as Database>::Connection,
        threshold_epoch_seconds: i64,
    ) -> impl Future<Output = Result<u64, LsAccountManagementError>> + Send;
}
