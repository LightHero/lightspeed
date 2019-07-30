use c3p0::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct AuthAccountData {
    pub username: String,
    pub email: String,
    pub password: String,
    pub roles: Vec<String>,
    pub created_date_epoch_seconds: i64,
    pub status: AuthAccountStatus,
}

#[derive(Clone, Serialize, Deserialize)]
pub enum AuthAccountStatus {
    ACTIVE,
    PENDING_ACTIVATION,
    DISABLED,
}

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
