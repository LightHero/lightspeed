use crate::model::auth_account::{AuthAccountData, AuthAccountModel};
use crate::repository::AuthAccountRepository;
use c3p0::*;
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

    fn fetch_by_username(
        &self,
        conn: &PgConnection,
        username: &str,
    ) -> Result<Option<AuthAccountModel>, C3p0Error> {
        let sql = r#"
            select id, version, data_json from AUTH_ACCOUNT
            where DATA_JSON ->> 'username' = $1
            limit 1
        "#;
        self.repo.fetch_one_by_sql(conn, sql, &[&username])
    }

    fn fetch_by_email(
        &self,
        conn: &PgConnection,
        email: &str,
    ) -> Result<Option<AuthAccountModel>, C3p0Error> {
        let sql = r#"
            select id, version, data_json from AUTH_ACCOUNT
            where DATA_JSON ->> 'email' = $1
            limit 1
        "#;
        self.repo.fetch_one_by_sql(conn, sql, &[&email])
    }

    fn save(
        &self,
        conn: &Self::CONN,
        model: NewModel<AuthAccountData>,
    ) -> Result<Model<AuthAccountData>, C3p0Error> {
        self.repo.save(conn, model)
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
