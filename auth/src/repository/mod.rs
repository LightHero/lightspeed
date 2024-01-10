use std::future::Future;

use crate::model::auth_account::{AuthAccountData, AuthAccountModel, AuthAccountStatus};
use crate::model::token::{TokenData, TokenModel};
use c3p0::*;
use lightspeed_core::error::LsError;

pub mod pg;

pub trait AuthRepositoryManager<Id: IdType>: Clone + Send + Sync {
    type Tx: Send + Sync;
    type C3P0: C3p0Pool<Tx = Self::Tx>;
    type AuthAccountRepo: AuthAccountRepository<Id, Tx = Self::Tx>;
    type TokenRepo: TokenRepository<Id, Tx = Self::Tx>;

    fn c3p0(&self) -> &Self::C3P0;
    fn start(&self) -> impl Future<Output = Result<(), LsError>> + Send;
    fn auth_account_repo(&self) -> Self::AuthAccountRepo;
    fn token_repo(&self) -> Self::TokenRepo;
}

pub trait AuthAccountRepository<Id: IdType>: Clone + Send + Sync {
    type Tx: Send + Sync;

    fn fetch_all_by_status(
        &self,
        tx: &mut Self::Tx,
        status: AuthAccountStatus,
        start_user_id: &Id,
        limit: u32,
    ) -> impl Future<Output = Result<Vec<AuthAccountModel<Id>>, LsError>> + Send;

    fn fetch_by_id(
        &self,
        tx: &mut Self::Tx,
        user_id: &Id,
    ) -> impl Future<Output = Result<AuthAccountModel<Id>, LsError>> + Send;

    fn fetch_by_username(
        &self,
        tx: &mut Self::Tx,
        username: &str,
    ) -> impl Future<Output = Result<AuthAccountModel<Id>, LsError>> + Send;

    fn fetch_by_username_optional(
        &self,
        tx: &mut Self::Tx,
        username: &str,
    ) -> impl Future<Output = Result<Option<AuthAccountModel<Id>>, LsError>> + Send;

    fn fetch_by_email_optional(
        &self,
        tx: &mut Self::Tx,
        email: &str,
    ) -> impl Future<Output = Result<Option<AuthAccountModel<Id>>, LsError>> + Send;

    fn save(
        &self,
        tx: &mut Self::Tx,
        model: NewModel<AuthAccountData>,
    ) -> impl Future<Output = Result<AuthAccountModel<Id>, LsError>> + Send;

    fn update(
        &self,
        tx: &mut Self::Tx,
        model: Model<Id, AuthAccountData>,
    ) -> impl Future<Output = Result<AuthAccountModel<Id>, LsError>> + Send;

    fn delete(
        &self,
        tx: &mut Self::Tx,
        model: AuthAccountModel<Id>,
    ) -> impl Future<Output = Result<AuthAccountModel<Id>, LsError>> + Send;

    fn delete_by_id(&self, tx: &mut Self::Tx, user_id: &Id) -> impl Future<Output = Result<u64, LsError>> + Send;
}

pub trait TokenRepository<Id: IdType>: Clone + Send + Sync {
    type Tx: Send + Sync;

    fn fetch_by_token(
        &self,
        tx: &mut Self::Tx,
        token_string: &str,
    ) -> impl Future<Output = Result<TokenModel<Id>, LsError>> + Send;

    fn fetch_by_username(
        &self,
        tx: &mut Self::Tx,
        username: &str,
    ) -> impl Future<Output = Result<Vec<TokenModel<Id>>, LsError>> + Send;

    fn save(
        &self,
        tx: &mut Self::Tx,
        model: NewModel<TokenData>,
    ) -> impl Future<Output = Result<TokenModel<Id>, LsError>> + Send;

    fn delete(
        &self,
        tx: &mut Self::Tx,
        model: TokenModel<Id>,
    ) -> impl Future<Output = Result<TokenModel<Id>, LsError>> + Send;
}
