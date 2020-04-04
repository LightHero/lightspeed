use crate::model::auth_account::{AuthAccountData, AuthAccountDataCodec, AuthAccountModel};
use crate::repository::AuthAccountRepository;
use c3p0::pg_async::*;
use c3p0::*;
use lightspeed_core::error::LightSpeedError;
use std::ops::Deref;

#[derive(Clone)]
pub struct PgAuthAccountRepository {
    repo: PgC3p0JsonAsync<AuthAccountData, AuthAccountDataCodec>,
}

impl Default for PgAuthAccountRepository {
    fn default() -> Self {
        PgAuthAccountRepository {
            repo: C3p0JsonBuilder::new("LS_AUTH_ACCOUNT").build_with_codec(AuthAccountDataCodec {}),
        }
    }
}

#[async_trait::async_trait]
impl AuthAccountRepository for PgAuthAccountRepository {
    type Conn = PgConnectionAsync;

    async fn fetch_by_id(
        &self,
        conn: &mut Self::Conn,
        user_id: i64,
    ) -> Result<Model<AuthAccountData>, LightSpeedError> {
        Ok(self.repo.fetch_one_by_id(conn, &user_id).await?)
    }

    async fn fetch_by_username(
        &self,
        conn: &mut PgConnectionAsync,
        username: &str,
    ) -> Result<AuthAccountModel, LightSpeedError> {
        self.fetch_by_username_optional(conn, username)
            .await?
            .ok_or_else(|| LightSpeedError::BadRequest {
                message: format!("No user found with username [{}]", username),
            })
    }

    async fn fetch_by_username_optional(
        &self,
        conn: &mut Self::Conn,
        username: &str,
    ) -> Result<Option<Model<AuthAccountData>>, LightSpeedError> {
        let sql = r#"
            select id, version, data from LS_AUTH_ACCOUNT
            where DATA ->> 'username' = $1
            limit 1
        "#;
        Ok(self
            .repo
            .fetch_one_optional_with_sql(conn, sql, &[&username])
            .await?)
    }

    async fn fetch_by_email_optional(
        &self,
        conn: &mut PgConnectionAsync,
        email: &str,
    ) -> Result<Option<AuthAccountModel>, LightSpeedError> {
        let sql = r#"
            select id, version, data from LS_AUTH_ACCOUNT
            where DATA ->> 'email' = $1
            limit 1
        "#;
        Ok(self
            .repo
            .fetch_one_optional_with_sql(conn, sql, &[&email])
            .await?)
    }

    async fn save(
        &self,
        conn: &mut Self::Conn,
        model: NewModel<AuthAccountData>,
    ) -> Result<Model<AuthAccountData>, LightSpeedError> {
        Ok(self.repo.save(conn, model).await?)
    }

    async fn update(
        &self,
        conn: &mut Self::Conn,
        model: Model<AuthAccountData>,
    ) -> Result<Model<AuthAccountData>, LightSpeedError> {
        Ok(self.repo.update(conn, model).await?)
    }

    async fn delete(
        &self,
        conn: &mut Self::Conn,
        model: Model<AuthAccountData>,
    ) -> Result<Model<AuthAccountData>, LightSpeedError> {
        Ok(self.repo.delete(conn, model).await?)
    }
}

impl Deref for PgAuthAccountRepository {
    type Target = PgC3p0JsonAsync<AuthAccountData, AuthAccountDataCodec>;

    fn deref(&self) -> &Self::Target {
        &self.repo
    }
}
