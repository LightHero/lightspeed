use crate::model::auth_account::{AuthAccountData, AuthAccountModel, AuthAccountStatus};
use crate::repository::AuthAccountRepository;
use c3p0::sqlx::*;
use c3p0::*;
use lightspeed_core::error::{ErrorCodes, LsError};

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
        start_user_id: u64,
        limit: u32,
    ) -> Result<Vec<AuthAccountModel>, LsError> {
        Ok(AuthAccountModel::query_with(
            r#"
            where id >= $1 and DATA ->> 'status' = $2
            order by id asc
            limit $3
        "#,
        )
        .bind(start_user_id as i64)
        .bind(status.as_ref())
        .bind(limit as i64)
        .fetch_all(tx)
        .await?)
    }

    async fn fetch_by_id(&self, tx: &mut PgConnection, user_id: u64) -> Result<AuthAccountModel, LsError> {
        Ok(tx.fetch_one_by_id(user_id).await?)
    }

    async fn fetch_by_username(&self, tx: &mut PgConnection, username: &str) -> Result<AuthAccountModel, LsError> {
        self.fetch_by_username_optional(tx, username).await?.ok_or_else(|| LsError::BadRequest {
            message: format!("No user found with username [{username}]"),
            code: ErrorCodes::NOT_FOUND,
        })
    }

    async fn fetch_by_username_optional(
        &self,
        tx: &mut PgConnection,
        username: &str,
    ) -> Result<Option<AuthAccountModel>, LsError> {
        Ok(AuthAccountModel::query_with(
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
    ) -> Result<Option<AuthAccountModel>, LsError> {
        Ok(AuthAccountModel::query_with(
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
    ) -> Result<AuthAccountModel, LsError> {
        Ok(tx.save(model).await?)
    }

    async fn update(&self, tx: &mut PgConnection, model: AuthAccountModel) -> Result<AuthAccountModel, LsError> {
        Ok(tx.update(model).await?)
    }

    async fn delete(&self, tx: &mut PgConnection, model: AuthAccountModel) -> Result<AuthAccountModel, LsError> {
        Ok(tx.delete(model).await?)
    }

    async fn delete_by_id(&self, tx: &mut PgConnection, user_id: u64) -> Result<u64, LsError> {
        Ok(tx.delete_by_id::<AuthAccountData>(user_id).await?)
    }
}
