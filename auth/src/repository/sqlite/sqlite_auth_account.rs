use crate::model::auth_account::{AuthAccountData, AuthAccountDataCodec, AuthAccountModel, AuthAccountStatus};
use crate::repository::AuthAccountRepository;
use c3p0::sqlx::*;
use c3p0::*;
use lightspeed_core::error::{ErrorCodes, LsError};
use std::ops::Deref;

#[derive(Clone)]
pub struct SqliteAuthAccountRepository {
    repo: SqlxSqliteC3p0Json<u64, AuthAccountData, AuthAccountDataCodec>,
}

impl Default for SqliteAuthAccountRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl SqliteAuthAccountRepository {
    pub fn new() -> Self {
        Self { repo: SqlxSqliteC3p0JsonBuilder::new("LS_AUTH_ACCOUNT").build_with_codec(AuthAccountDataCodec {}) }
    }
}

impl AuthAccountRepository for SqliteAuthAccountRepository {
    type DB = Sqlite;

    async fn fetch_all_by_status(
        &self,
        tx: &mut SqliteConnection,
        status: AuthAccountStatus,
        start_user_id: &u64,
        limit: u32,
    ) -> Result<Vec<AuthAccountModel>, LsError> {
        let sql = format!(
            r#"
            {}
            where id >= ? and DATA ->> '$.status' = ?
            order by id asc
            limit ?
        "#,
            self.queries().find_base_sql_query
        );

        Ok(self
            .repo
            .fetch_all_with_sql(
                tx,
                self.repo.query_with_id(&sql, start_user_id).bind(status.as_ref()).bind(limit as i64),
            )
            .await?)
    }

    async fn fetch_by_id(&self, tx: &mut SqliteConnection, user_id: &u64) -> Result<AuthAccountModel, LsError> {
        Ok(self.repo.fetch_one_by_id(tx, user_id).await?)
    }

    async fn fetch_by_username(&self, tx: &mut SqliteConnection, username: &str) -> Result<AuthAccountModel, LsError> {
        self.fetch_by_username_optional(tx, username).await?.ok_or_else(|| LsError::BadRequest {
            message: format!("No user found with username [{username}]"),
            code: ErrorCodes::NOT_FOUND,
        })
    }

    async fn fetch_by_username_optional(
        &self,
        tx: &mut SqliteConnection,
        username: &str,
    ) -> Result<Option<AuthAccountModel>, LsError> {
        let sql = &format!(
            r#"
            {}
            where DATA ->> '$.username' = ?
            limit 1
        "#,
            self.queries().find_base_sql_query
        );
        Ok(self.repo.fetch_one_optional_with_sql(tx, ::sqlx::query(sql).bind(username)).await?)
    }

    async fn fetch_by_email_optional(
        &self,
        tx: &mut SqliteConnection,
        email: &str,
    ) -> Result<Option<AuthAccountModel>, LsError> {
        let sql = format!(
            r#"
            {}
            where DATA ->> '$.email' = ?
            limit 1
        "#,
            self.queries().find_base_sql_query
        );
        Ok(self.repo.fetch_one_optional_with_sql(tx, ::sqlx::query(&sql).bind(email)).await?)
    }

    async fn save(&self, tx: &mut SqliteConnection, model: NewModel<AuthAccountData>) -> Result<AuthAccountModel, LsError> {
        Ok(self.repo.save(tx, model).await?)
    }

    async fn update(&self, tx: &mut SqliteConnection, model: AuthAccountModel) -> Result<AuthAccountModel, LsError> {
        Ok(self.repo.update(tx, model).await?)
    }

    async fn delete(&self, tx: &mut SqliteConnection, model: AuthAccountModel) -> Result<AuthAccountModel, LsError> {
        Ok(self.repo.delete(tx, model).await?)
    }

    async fn delete_by_id(&self, tx: &mut SqliteConnection, user_id: &u64) -> Result<u64, LsError> {
        Ok(self.repo.delete_by_id(tx, user_id).await?)
    }
}

impl Deref for SqliteAuthAccountRepository {
    type Target = SqlxSqliteC3p0Json<u64, AuthAccountData, AuthAccountDataCodec>;

    fn deref(&self) -> &Self::Target {
        &self.repo
    }
}
