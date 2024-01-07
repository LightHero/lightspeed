use c3p0::{C3p0Error, JsonCodec, Model};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::borrow::Cow;
use strum::{AsRefStr, Display};

pub type AuthAccountModel<Id> = Model<Id, AuthAccountData>;

#[derive(Clone, Serialize, Deserialize)]
pub struct AuthAccountData {
    pub username: String,
    pub email: String,
    pub password: String,
    pub roles: Vec<String>,
    pub created_date_epoch_seconds: i64,
    pub status: AuthAccountStatus,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize, AsRefStr, Display)]
#[cfg_attr(feature = "poem_openapi", derive(poem_openapi::Enum))]
pub enum AuthAccountStatus {
    Active,
    PendingActivation,
    Disabled,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(tag = "_json_tag")]
enum AuthAccountDataVersioning<'a> {
    V1(Cow<'a, AuthAccountData>),
}

#[derive(Clone)]
pub struct AuthAccountDataCodec {}

impl JsonCodec<AuthAccountData> for AuthAccountDataCodec {
    fn data_from_value(&self, value: Value) -> Result<AuthAccountData, C3p0Error> {
        let versioning = serde_json::from_value(value)?;
        let data = match versioning {
            AuthAccountDataVersioning::V1(data_v1) => data_v1.into_owned(),
        };
        Ok(data)
    }

    fn data_to_value(&self, data: &AuthAccountData) -> Result<Value, C3p0Error> {
        serde_json::to_value(AuthAccountDataVersioning::V1(Cow::Borrowed(data))).map_err(C3p0Error::from)
    }
}
