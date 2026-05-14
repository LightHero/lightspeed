use serde::{Deserialize, Serialize};
use thiserror::Error;


#[derive(Debug, Serialize, Deserialize, Error)]
pub enum LsAccountManagerError {
    #[error("PasswordEncryptionError: {message}")]
    PasswordEncryptionError { message: String },
    #[error("TokenExpiredError")]
    TokenExpiredError,
}