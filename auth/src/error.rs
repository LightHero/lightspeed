use coreutils_jwt::JwtError;
use err_derive::Error;

#[derive(Error, Debug)]
pub enum AuthError {

    #[error(display = "JwtTokenError: [{}]", message)]
    JwtTokenError { message: String },
    #[error(display = "MissingAuthTokenError")]
    MissingAuthTokenError,
    #[error(display = "ParseAuthHeaderError: [{}]", message)]
    ParseAuthHeaderError { message: String },
}

impl From<JwtError> for AuthError {
    fn from(err: JwtError) -> Self {
        AuthError::JwtTokenError {message: format!("{}", err)}
    }
}