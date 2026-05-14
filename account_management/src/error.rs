use lightspeed_core::error::{ErrorDetails, LsError, RootErrorDetails};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum LsAccountManagerError {
    #[error("ModuleStartError: {message}")]
    ModuleStartError { message: String },

    #[error("PasswordEncryptionError: {message}")]
    PasswordEncryptionError { message: String },

    #[error("BadRequest: {message} - {code}")]
    BadRequest { message: String, code: &'static str },

    #[error("ValidationError: {details:?}")]
    ValidationError { details: RootErrorDetails },

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

impl LsAccountManagerError {
    /// Run `f` against a fresh `ErrorDetails`; if any errors were collected,
    /// return `LsAccountManagerError::ValidationError` carrying them.
    pub fn validate<F: FnOnce(&mut ErrorDetails)>(f: F) -> Result<(), Self> {
        let mut error_details = ErrorDetails::default();
        f(&mut error_details);
        let ErrorDetails::Root(root) = error_details else { unreachable!() };
        if root.details.is_empty() { Ok(()) } else { Err(Self::ValidationError { details: root }) }
    }
}

impl From<LsAccountManagerError> for LsError {
    fn from(err: LsAccountManagerError) -> Self {
        match err {
            LsAccountManagerError::ModuleStartError { message } => LsError::ModuleStartError { message },
            LsAccountManagerError::PasswordEncryptionError { message } => {
                LsError::InternalServerError { message: format!("PasswordEncryptionError: {message}") }
            }
            LsAccountManagerError::BadRequest { message, code } => LsError::BadRequest { message, code },
            LsAccountManagerError::ValidationError { details } => LsError::ValidationError { details },
            LsAccountManagerError::C3p0Error { source } => LsError::C3p0Error { source },
            LsAccountManagerError::SqlxError { source } => LsError::SqlxError { source },
        }
    }
}
