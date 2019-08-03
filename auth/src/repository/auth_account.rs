use c3p0::*;
use crate::model::auth_account::AuthAccountData;

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
            repo: C3p0JsonBuilder::new("AUTH_ACCOUNT").build(),
        }
    }
}
