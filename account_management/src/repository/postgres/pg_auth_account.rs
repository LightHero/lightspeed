use crate::error::LsAccountManagerError;
use crate::model::auth_account::{AuthAccountData, AuthAccountModel, AuthAccountStatus};
use crate::repository::AuthAccountRepository;
use c3p0::sqlx::*;
use c3p0::*;

#[derive(Clone)]
pub struct PgAuthAccountRepository {}

impl Default for PgAuthAccountRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl PgAuthAccountRepository {
    pub fn new() -> Self {
        Self {}
    }
}

impl AuthAccountRepository for PgAuthAccountRepository {
    type DB = Postgres;

    async fn fetch_all_by_status(
        &self,
        tx: &mut PgConnection,
        status: AuthAccountStatus,
        start_user_id: i64,
        limit: u32,
    ) -> Result<Vec<AuthAccountModel>, LsAccountManagerError> {
        Ok(AuthAccountModel::query_with_tail(
            r#"
            where id >= $1 and DATA ->> 'status' = $2
            order by id asc
            limit $3
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
        tx: &mut PgConnection,
        user_id: i64,
    ) -> Result<AuthAccountModel, LsAccountManagerError> {
        Ok(tx.fetch_one_by_id::<AuthAccountData>(user_id).await?)
    }

    async fn fetch_by_username(
        &self,
        tx: &mut PgConnection,
        username: &str,
    ) -> Result<AuthAccountModel, LsAccountManagerError> {
        self.fetch_by_username_optional(tx, username).await?.ok_or_else(|| LsAccountManagerError::BadRequest {
            message: format!("No user found with username [{username}]"),
            code: "",
        })
    }

    async fn fetch_by_username_optional(
        &self,
        tx: &mut PgConnection,
        username: &str,
    ) -> Result<Option<AuthAccountModel>, LsAccountManagerError> {
        Ok(AuthAccountModel::query_with_tail(
            r#"
            where DATA ->> 'username' = $1
            limit 1
        "#,
        )
        .bind(username)
        .fetch_optional(tx)
        .await?)
    }

    async fn fetch_by_email_optional(
        &self,
        tx: &mut PgConnection,
        email: &str,
    ) -> Result<Option<AuthAccountModel>, LsAccountManagerError> {
        Ok(AuthAccountModel::query_with_tail(
            r#"
            where DATA ->> 'email' = $1
            limit 1
        "#,
        )
        .bind(email)
        .fetch_optional(tx)
        .await?)
    }

    async fn save(
        &self,
        tx: &mut PgConnection,
        model: NewRecord<AuthAccountData>,
    ) -> Result<AuthAccountModel, LsAccountManagerError> {
        Ok(tx.save(model).await?)
    }

    async fn update(
        &self,
        tx: &mut PgConnection,
        model: AuthAccountModel,
    ) -> Result<AuthAccountModel, LsAccountManagerError> {
        Ok(tx.update(model).await?)
    }

    async fn delete(
        &self,
        tx: &mut PgConnection,
        model: AuthAccountModel,
    ) -> Result<AuthAccountModel, LsAccountManagerError> {
        Ok(tx.delete(model).await?)
    }

    async fn delete_by_id(&self, tx: &mut PgConnection, user_id: i64) -> Result<u64, LsAccountManagerError> {
        Ok(tx.delete_by_id::<AuthAccountData>(user_id).await?)
    }
}
