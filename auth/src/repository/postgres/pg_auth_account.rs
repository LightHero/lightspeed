use crate::{
    model::auth_account::{AuthAccountData, AuthAccountDataCodec, AuthAccountModel, AuthAccountStatus},
    repository::AuthAccountRepository,
};
use c3p0::postgres::*;
use c3p0::*;
use lightspeed_core::error::{ErrorCodes, LsError};
use std::ops::Deref;

#[derive(Clone)]
pub struct PostgresAuthAccountRepository {
    repo: PgC3p0Json<u64, i64, AuthAccountData, AuthAccountDataCodec>,
}

impl Default for PostgresAuthAccountRepository {
    fn default() -> Self {
        PostgresAuthAccountRepository {
            repo: PgC3p0JsonBuilder::new("LS_AUTH_ACCOUNT").build_with_codec(AuthAccountDataCodec {}),
        }
    }
}

impl AuthAccountRepository for PostgresAuthAccountRepository {
    type Tx<'a> = PgTx<'a>;

    async fn fetch_all_by_status(
        &self,
        tx: &mut Self::Tx<'_>,
        status: AuthAccountStatus,
        start_user_id: &u64,
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
        Ok(self
            .repo
            .fetch_all_with_sql(tx, &sql, &[&(*start_user_id as i64), &status.as_ref(), &(limit as i64)])
            .await?)
    }

    async fn fetch_by_id(&self, tx: &mut Self::Tx<'_>, user_id: &u64) -> Result<AuthAccountModel, LsError> {
        Ok(self.repo.fetch_one_by_id(tx, user_id).await?)
    }

    async fn fetch_by_username(&self, tx: &mut Self::Tx<'_>, username: &str) -> Result<AuthAccountModel, LsError> {
        self.fetch_by_username_optional(tx, username).await?.ok_or_else(|| LsError::BadRequest {
            message: format!("No user found with username [{username}]"),
            code: ErrorCodes::NOT_FOUND,
        })
    }

    async fn fetch_by_username_optional(
        &self,
        tx: &mut Self::Tx<'_>,
        username: &str,
    ) -> Result<Option<AuthAccountModel>, LsError> {
        let sql = format!(
            r#"
            {}
            where DATA ->> 'username' = $1
            limit 1
        "#,
            self.queries().find_base_sql_query
        );
        Ok(self.repo.fetch_one_optional_with_sql(tx, &sql, &[&username]).await?)
    }

    async fn fetch_by_email_optional(
        &self,
        tx: &mut Self::Tx<'_>,
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
        Ok(self.repo.fetch_one_optional_with_sql(tx, &sql, &[&email]).await?)
    }

    async fn save(&self, tx: &mut Self::Tx<'_>, model: NewModel<AuthAccountData>) -> Result<AuthAccountModel, LsError> {
        Ok(self.repo.save(tx, model).await?)
    }

    async fn update(&self, tx: &mut Self::Tx<'_>, model: AuthAccountModel) -> Result<AuthAccountModel, LsError> {
        Ok(self.repo.update(tx, model).await?)
    }

    async fn delete(&self, tx: &mut Self::Tx<'_>, model: AuthAccountModel) -> Result<AuthAccountModel, LsError> {
        Ok(self.repo.delete(tx, model).await?)
    }

    async fn delete_by_id(&self, tx: &mut Self::Tx<'_>, user_id: &u64) -> Result<u64, LsError> {
        Ok(self.repo.delete_by_id(tx, user_id).await?)
    }
}

impl Deref for PostgresAuthAccountRepository {
    type Target = PgC3p0Json<u64, i64, AuthAccountData, AuthAccountDataCodec>;

    fn deref(&self) -> &Self::Target {
        &self.repo
    }
}
