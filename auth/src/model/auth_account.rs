use c3p0::{Codec, DataType, Record};
use serde::{Deserialize, Serialize};
use strum::{AsRefStr, Display};

pub type AuthAccountModel = Record<AuthAccountData>;

#[derive(Clone, Serialize, Deserialize)]
pub struct AuthAccountData {
    pub username: String,
    pub email: String,
    pub password: String,
    pub roles: Vec<String>,
    pub created_date_epoch_seconds: i64,
    pub status: AuthAccountStatus,
}

impl DataType for AuthAccountData {
    const TABLE_NAME: &'static str = "LS_AUTH_ACCOUNT";
    type CODEC = AuthAccountDataToken;
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize, AsRefStr, Display)]
pub enum AuthAccountStatus {
    Active,
    PendingActivation,
    Disabled,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(tag = "_json_tag")]
pub enum AuthAccountDataToken {
    V1(AuthAccountData),
}

impl Codec<AuthAccountData> for AuthAccountDataToken {
    fn encode(data: AuthAccountData) -> Self {
        AuthAccountDataToken::V1(data)
    }

    fn decode(data: Self) -> AuthAccountData {
        match data {
            AuthAccountDataToken::V1(data) => data,
        }
    }
}
