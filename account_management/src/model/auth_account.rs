use c3p0::{Codec, DataType, Record};
use serde::{Deserialize, Serialize};
use strum::{AsRefStr, Display};

pub type AuthAccountModel = Record<AccountData>;

#[derive(Clone, Serialize, Deserialize)]
pub struct AccountData {
    pub username: String,
    pub email: String,
    pub password: String,
    pub roles: Vec<String>,
    pub created_date_epoch_seconds: i64,
    /// Epoch seconds at which `password` was last set
    pub password_updated_date_epoch_seconds: i64,
    pub status: AccountStatus,
}

impl DataType for AccountData {
    const TABLE_NAME: &'static str = "LS_AM_ACCOUNT";
    type CODEC = AccountDataToken;
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize, AsRefStr, Display)]
pub enum AccountStatus {
    Active,
    PendingActivation,
    Disabled,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(tag = "_codec_tag")]
pub enum AccountDataToken {
    V1(AccountData),
}

impl Codec<AccountData> for AccountDataToken {
    fn encode(data: AccountData) -> Self {
        AccountDataToken::V1(data)
    }

    fn decode(data: Self) -> AccountData {
        match data {
            AccountDataToken::V1(data) => data,
        }
    }
}
