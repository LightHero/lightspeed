use crate::error::LsAccountManagementError;
use crate::model::auth_account::{AccountData, AccountStatus, AuthAccountModel};
use crate::repository::AccountRepository;
use c3p0::sqlx::*;
use c3p0::*;

#[derive(Clone)]
pub struct SqliteAccountRepository {}

impl Default for SqliteAccountRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl SqliteAccountRepository {
    pub fn new() -> Self {
        Self {}
    }
}

impl AccountRepository for SqliteAccountRepository {
    type DB = Sqlite;

    async fn fetch_all_by_status(
        &self,
        tx: &mut SqliteConnection,
        status: AccountStatus,
        start_user_id: i64,
        limit: u32,
    ) -> Result<Vec<AuthAccountModel>, LsAccountManagementError> {
        Ok(AuthAccountModel::query_with_tail(
            r#"
            where id >= ? and DATA ->> '$.status' = ?
            order by id asc
            limit ?
        "#,
        )
        .bind(start_user_id)
        .bind(status.as_ref())
        .bind(limit as i64)
        .fetch_all(tx)
        .await?)
    }

    async fn fetch_by_id(
        &self,
        tx: &mut SqliteConnection,
        user_id: i64,
    ) -> Result<AuthAccountModel, LsAccountManagementError> {
        Ok(tx.fetch_one_by_id::<AccountData>(user_id).await?)
    }

    async fn fetch_by_username(
        &self,
        tx: &mut SqliteConnection,
        username: &str,
    ) -> Result<AuthAccountModel, LsAccountManagementError> {
        self.fetch_by_username_optional(tx, username).await?.ok_or_else(|| LsAccountManagementError::BadRequest {
            message: format!("No user found with username [{username}]"),
            code: "",
        })
    }

    async fn fetch_by_username_optional(
        &self,
        tx: &mut SqliteConnection,
        username: &str,
    ) -> Result<Option<AuthAccountModel>, LsAccountManagementError> {
        Ok(AuthAccountModel::query_with_tail(
            r#"
            where DATA ->> '$.username' = ?
            limit 1
        "#,
        )
        .bind(username)
        .fetch_optional(tx)
        .await?)
    }

    async fn fetch_by_email_optional(
        &self,
        tx: &mut SqliteConnection,
        email: &str,
    ) -> Result<Option<AuthAccountModel>, LsAccountManagementError> {
        Ok(AuthAccountModel::query_with_tail(
            r#"
            where DATA ->> '$.email' = ?
            limit 1
        "#,
        )
        .bind(email)
        .fetch_optional(tx)
        .await?)
    }

    async fn save(
        &self,
        tx: &mut SqliteConnection,
        model: NewRecord<AccountData>,
    ) -> Result<AuthAccountModel, LsAccountManagementError> {
        Ok(tx.save(model).await?)
    }

    async fn update(
        &self,
        tx: &mut SqliteConnection,
        model: AuthAccountModel,
    ) -> Result<AuthAccountModel, LsAccountManagementError> {
        Ok(tx.update(model).await?)
    }

    async fn delete(
        &self,
        tx: &mut SqliteConnection,
        model: AuthAccountModel,
    ) -> Result<AuthAccountModel, LsAccountManagementError> {
        Ok(tx.delete(model).await?)
    }

    async fn delete_by_id(&self, tx: &mut SqliteConnection, user_id: i64) -> Result<u64, LsAccountManagementError> {
        Ok(tx.delete_by_id::<AccountData>(user_id).await?)
    }
}
