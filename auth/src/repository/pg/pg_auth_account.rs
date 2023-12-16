use crate::model::auth_account::{AuthAccountData, AuthAccountDataCodec, AuthAccountModel, AuthAccountStatus};
use crate::repository::AuthAccountRepository;
use c3p0::sqlx::*;
use c3p0::*;
use lightspeed_core::error::{ErrorCodes, LsError};
use std::ops::Deref;

#[derive(Clone)]
pub struct PgAuthAccountRepository {
    repo: SqlxPgC3p0Json<AuthAccountData, AuthAccountDataCodec>,
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
    type Conn = SqlxPgConnection;

    async fn fetch_all_by_status(
        &self,
        conn: &mut Self::Conn,
        status: AuthAccountStatus,
        start_user_id: i64,
        limit: u32,
    ) -> Result<Vec<AuthAccountModel>, LsError> {
        let sql = format!(
            r#"
            {}
            where id >= $1 and DATA ->> 'status' = $2
            order by id asc
            limit $3
        "#,
            self.queries().find_base_sql_query
        );
        
        Ok(self.repo.fetch_all_with_sql(conn, ::sqlx::query(&sql).bind(start_user_id)
        .bind(status.as_ref())
        .bind(limit as i64)).await?)
    }

    async fn fetch_by_id(
        &self,
        conn: &mut Self::Conn,
        user_id: i64,
    ) -> Result<Model<AuthAccountData>, LsError> {
        Ok(self.repo.fetch_one_by_id(conn, &user_id).await?)
    }

    async fn fetch_by_username(
        &self,
        conn: &mut Self::Conn,
        username: &str,
    ) -> Result<AuthAccountModel, LsError> {
        self.fetch_by_username_optional(conn, username).await?.ok_or_else(|| LsError::BadRequest {
            message: format!("No user found with username [{username}]"),
            code: ErrorCodes::NOT_FOUND,
        })
    }

    async fn fetch_by_username_optional(
        &self,
        conn: &mut Self::Conn,
        username: &str,
    ) -> Result<Option<Model<AuthAccountData>>, LsError> {
        let sql = &format!(
            r#"
            {}
            where DATA ->> 'username' = $1
            limit 1
        "#,
            self.queries().find_base_sql_query
        );
        Ok(self.repo.fetch_one_optional_with_sql(conn, ::sqlx::query(&sql).bind(username)).await?)
    }

    async fn fetch_by_email_optional(
        &self,
        conn: &mut Self::Conn,
        email: &str,
    ) -> Result<Option<AuthAccountModel>, LsError> {
        let sql = format!(
            r#"
            {}
            where DATA ->> 'email' = $1
            limit 1
        "#,
            self.queries().find_base_sql_query
        );
        Ok(self.repo.fetch_one_optional_with_sql(conn, ::sqlx::query(&sql)
        .bind(email)).await?)
    }

    async fn save(
        &self,
        conn: &mut Self::Conn,
        model: NewModel<AuthAccountData>,
    ) -> Result<Model<AuthAccountData>, LsError> {
        Ok(self.repo.save(conn, model).await?)
    }

    async fn update(
        &self,
        conn: &mut Self::Conn,
        model: Model<AuthAccountData>,
    ) -> Result<Model<AuthAccountData>, LsError> {
        Ok(self.repo.update(conn, model).await?)
    }

    async fn delete(
        &self,
        conn: &mut Self::Conn,
        model: Model<AuthAccountData>,
    ) -> Result<Model<AuthAccountData>, LsError> {
        Ok(self.repo.delete(conn, model).await?)
    }

    async fn delete_by_id(&self, conn: &mut Self::Conn, user_id: i64) -> Result<u64, LsError> {
        Ok(self.repo.delete_by_id(conn, &user_id).await?)
    }
}

impl Deref for PgAuthAccountRepository {
    type Target = SqlxPgC3p0Json<AuthAccountData, AuthAccountDataCodec>;

    fn deref(&self) -> &Self::Target {
        &self.repo
    }
}
