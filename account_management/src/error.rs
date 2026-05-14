use thiserror::Error;

#[derive(Debug, Error)]
pub enum LsAccountManagerError {

    #[error("PasswordEncryptionError: {message}")]
    PasswordEncryptionError { message: String },

    #[error("BadRequest: {message} - {code}")]
    BadRequest { message: String, code: &'static str },

    #[error("TokenExpired")]
    TokenExpired,

    #[error("TokenNotValid")]
    TokenNotValid,

    #[error("UsernameAlreadyUsed")]
    UsernameAlreadyUsed,

    #[error("EmailAlreadyUsed")]
    EmailAlreadyUsed,

    #[error("C3p0Error: {source:?}")]
    C3p0Error {
        #[from]
        source: c3p0::error::C3p0Error,
    },

    #[error("SqlxError: {source:?}")]
    SqlxError {
        #[from]
        source: c3p0::sqlx::Error,
    },

    #[error("InactiveUser: {0}")]
    InactiveUser(String),

    #[error("NotDisabledUser: {0}")]
    NotDisabledUser(String),

    #[error("ExpiredPassword for user {0}")]
    ExpiredPassword(String),

    #[error("WrongCredentials")]
    WrongCredentials,

    #[error("UserNotPendingActivation")]
    UserNotPendingActivation,
}
