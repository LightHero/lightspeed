use actix_web::{HttpResponse, ResponseError};
use c3p0_common::error::C3p0Error;
use err_derive::Error;

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
    #[error(display = "ConfigurationError: [{}]", message)]
    ConfigurationError { message: String },

    // Auth
    #[error(display = "UnauthenticatedError")]
    UnauthenticatedError,
    #[error(display = "ForbiddenError [{}]", message)]
    ForbiddenError { message: String },
    #[error(display = "PasswordEncryptionError [{}]", message)]
    PasswordEncryptionError { message: String },

    #[error(display = "InternalServerError [{}]", message)]
    InternalServerError { message: String },

    #[error(display = "RepositoryError [{}]", message)]
    RepositoryError { message: String },
}

impl ResponseError for LightSpeedError {
    fn error_response(&self) -> HttpResponse {
        match *self {
            LightSpeedError::InvalidTokenError { .. }
            | LightSpeedError::ExpiredTokenError { .. }
            | LightSpeedError::GenerateTokenError { .. }
            | LightSpeedError::MissingAuthTokenError { .. }
            | LightSpeedError::ParseAuthHeaderError { .. }
            | LightSpeedError::UnauthenticatedError => HttpResponse::Unauthorized().finish(),
            LightSpeedError::ForbiddenError { .. } => HttpResponse::Forbidden().finish(),
            LightSpeedError::InternalServerError { .. } => {
                HttpResponse::InternalServerError().finish()
            }
            _ => HttpResponse::InternalServerError().finish(),
        }
    }
}

impl From<C3p0Error> for LightSpeedError {
    fn from(err: C3p0Error) -> Self {
        LightSpeedError::RepositoryError {
            message: format!("{}", err),
        }
    }
}
