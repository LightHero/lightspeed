use c3p0::*;
use crate::model::auth_account::{AuthAccountData, AuthAccountModel};

pub struct AuthAccountRepository {
    repo: C3p0Json<
        AuthAccountData,
        DefaultJsonCodec,
        PgJsonManager<AuthAccountData, DefaultJsonCodec>,
    >,
}

impl AuthAccountRepository {
    pub fn new() -> Self {
        AuthAccountRepository {
            repo: C3p0JsonBuilder::new("AUTH_ACCOUNT").with_data_field_name("data_json").build(),
        }
    }

    pub fn fetch_by_username(
        &self,
        conn: &PgConnection,
        token_string: &str,
    ) -> Result<Option<AuthAccountModel>, C3p0Error> {
        let sql = r#"
            select id, version, data_json from AUTH_ACCOUNT
            where DATA_JSON ->> 'username' = $1
            limit 1
        "#;
        conn.fetch_one_option(sql, &[&token_string], |row| self.repo.json().to_model(row))
    }

}
