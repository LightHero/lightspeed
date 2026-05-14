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
}
