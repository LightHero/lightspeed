use c3p0::Model;
use serde::{Deserialize, Serialize};

pub type AuthAccountModel = Model<AuthAccountData>;

#[derive(Clone, Serialize, Deserialize)]
pub struct AuthAccountData {
    pub username: String,
    pub email: String,
    pub password: String,
    pub roles: Vec<String>,
    pub created_date_epoch_seconds: i64,
    pub status: AuthAccountStatus,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum AuthAccountStatus {
    Active,
    PendingActivation,
    Disabled,
}
