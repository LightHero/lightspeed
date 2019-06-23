use err_derive::Error;
use actix_web::{ResponseError, HttpResponse};

#[derive(Error, Debug)]
pub enum LightSpeedError {

    // JWT
    #[error(display = "InvalidTokenError: [{}]", message)]
    InvalidTokenError { message: String },
    #[error(display = "ExpiredTokenError: [{}]", message)]
    ExpiredTokenError { message: String },
    #[error(display = "GenerateTokenError: [{}]", message)]
    GenerateTokenError { message: String },
    #[error(display = "MissingAuthTokenError")]
    MissingAuthTokenError,
    #[error(display = "ParseAuthHeaderError: [{}]", message)]
    ParseAuthHeaderError { message: String },

    // Module
    #[error(display = "ModuleBuilderError: [{}]", message)]
    ModuleBuilderError { message: String },
    #[error(display = "ModuleStartError: [{}]", message)]
    ModuleStartError { message: String },

    // Auth
    #[error(display = "UnAuthenticatedError")]
    UnAuthenticatedError,
    #[error(display = "UnAuthorizedError [{}]", message)]
    UnAuthorizedError { message: String },
}

impl ResponseError for LightSpeedError {
    fn error_response(&self) -> HttpResponse {
        match *self {
            LightSpeedError::InvalidTokenError{..} |
            LightSpeedError::ExpiredTokenError{..} |
            LightSpeedError::GenerateTokenError{..} |
            LightSpeedError::MissingAuthTokenError{..} |
            LightSpeedError::ParseAuthHeaderError{..} |
            LightSpeedError::UnAuthenticatedError |
            LightSpeedError::UnAuthorizedError{..}  => HttpResponse::Unauthorized().finish(),
            _ => HttpResponse::InternalServerError().finish(),
        }
    }
}
