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
