use c3p0::{C3p0Error, JsonCodec, Model};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::borrow::Cow;
use strum_macros::{AsRefStr, Display};
use typescript_definitions::TypeScriptify;

pub type AuthAccountModel = Model<AuthAccountData>;

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthAccountData {
    pub username: String,
    pub email: String,
    pub password: String,
    pub roles: Vec<String>,
    pub created_date_epoch_seconds: i64,
    pub status: AuthAccountStatus,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize, AsRefStr, Display, TypeScriptify)]
#[allow(non_camel_case_types)]
pub enum AuthAccountStatus {
    ACTIVE,
    PENDING_ACTIVATION,
    DISABLED,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(tag = "_json_tag")]
enum AuthAccountDataVersioning<'a> {
    V1(Cow<'a, AuthAccountData>),
}

#[derive(Clone)]
pub struct AuthAccountDataCodec {}

impl JsonCodec<AuthAccountData> for AuthAccountDataCodec {
    fn from_value(&self, value: Value) -> Result<AuthAccountData, C3p0Error> {
        let versioning = serde_json::from_value(value)?;
        let data = match versioning {
            AuthAccountDataVersioning::V1(data_v1) => data_v1.into_owned(),
        };
        Ok(data)
    }

    fn to_value(&self, data: &AuthAccountData) -> Result<Value, C3p0Error> {
        serde_json::to_value(AuthAccountDataVersioning::V1(Cow::Borrowed(data)))
            .map_err(C3p0Error::from)
    }
}
