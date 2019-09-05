use crate::model::auth_account::{AuthAccountData, AuthAccountModel};
use crate::repository::AuthAccountRepository;
use c3p0::pg::*;
use c3p0::*;
use lightspeed_core::error::LightSpeedError;
use std::ops::Deref;

#[derive(Clone)]
pub struct PgAuthAccountRepository {
    repo: C3p0JsonPg<AuthAccountData, DefaultJsonCodec>,
}

impl Default for PgAuthAccountRepository {
    fn default() -> Self {
        PgAuthAccountRepository {
            repo: C3p0JsonBuilder::new("AUTH_ACCOUNT")
                .with_data_field_name("data_json")
                .build(),
        }
    }
}

impl AuthAccountRepository for PgAuthAccountRepository {
    type CONN = PgConnection;

    fn fetch_by_id(
        &self,
        conn: &Self::CONN,
        user_id: i64,
    ) -> Result<Model<AuthAccountData>, LightSpeedError> {
        self.repo
            .fetch_one_by_id(conn, &user_id)?
            .ok_or_else(|| LightSpeedError::BadRequest {
                message: format!("No user found with id [{}]", user_id),
            })
    }

    fn fetch_by_username(
        &self,
        conn: &PgConnection,
        username: &str,
    ) -> Result<AuthAccountModel, LightSpeedError> {
        self.fetch_by_username_optional(conn, username)?
            .ok_or_else(|| LightSpeedError::BadRequest {
                message: format!("No user found with username [{}]", username),
            })
    }

    fn fetch_by_username_optional(
        &self,
        conn: &Self::CONN,
        username: &str,
    ) -> Result<Option<Model<AuthAccountData>>, C3p0Error> {
        let sql = r#"
            select id, version, data_json from AUTH_ACCOUNT
            where DATA_JSON ->> 'username' = $1
            limit 1
        "#;
        self.repo.fetch_one_with_sql(conn, sql, &[&username])
    }

    fn fetch_by_email_optional(
        &self,
        conn: &PgConnection,
        email: &str,
    ) -> Result<Option<AuthAccountModel>, C3p0Error> {
        let sql = r#"
            select id, version, data_json from AUTH_ACCOUNT
            where DATA_JSON ->> 'email' = $1
            limit 1
        "#;
        self.repo.fetch_one_with_sql(conn, sql, &[&email])
    }

    fn save(
        &self,
        conn: &Self::CONN,
        model: NewModel<AuthAccountData>,
    ) -> Result<Model<AuthAccountData>, C3p0Error> {
        self.repo.save(conn, model)
    }

    fn update(
        &self,
        conn: &Self::CONN,
        model: Model<AuthAccountData>,
    ) -> Result<Model<AuthAccountData>, C3p0Error> {
        self.repo.update(conn, model)
    }

    fn delete(&self, conn: &Self::CONN, model: &Model<AuthAccountData>) -> Result<u64, C3p0Error> {
        self.repo.delete(conn, model)
    }
}

impl Deref for PgAuthAccountRepository {
    type Target = C3p0JsonPg<AuthAccountData, DefaultJsonCodec>;

    fn deref(&self) -> &Self::Target {
        &self.repo
    }
}
