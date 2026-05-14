use thiserror::Error;

#[derive(Error, Debug)]
pub enum LsError {
    #[error("InvalidTokenError: {message}")]
    InvalidTokenError { message: String },
    #[error("ExpiredTokenError: {message}")]
    ExpiredTokenError { message: String },
    #[error("GenerateTokenError: {message}")]
    GenerateTokenError { message: String },
    #[error("MissingAuthTokenError")]
    MissingAuthTokenError,
    #[error("ParseAuthHeaderError: {message}")]
    ParseAuthHeaderError { message: String },

    // Module
    #[error("ModuleBuilderError: {message}")]
    ModuleBuilderError { message: String },
    #[error("ModuleStartError: {message}")]
    ModuleStartError { message: String },
    #[error("ConfigurationError: {message}")]
    ConfigurationError { message: String },

    // Auth
    #[error("UnauthenticatedError")]
    UnauthenticatedError,
    #[error("ForbiddenError: {message}")]
    ForbiddenError { message: String },

    #[error("InternalServerError: {message}")]
    InternalServerError { message: String },

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

    #[error("BadRequest: {message} - {code}")]
    BadRequest { message: String, code: &'static str },

    #[error("RequestConflict: {message} - {code}")]
    RequestConflict { message: String, code: &'static str },

    #[error("ServiceUnavailable: {message} - {code}")]
    ServiceUnavailable { message: String, code: &'static str },
}

impl From<serde_json::Error> for LsError {
    fn from(err: serde_json::Error) -> Self {
        LsError::BadRequest { message: format!("{err:?}"), code: "" }
    }
}
