use crate::error::{LightSpeedError, WebErrorDetails};
use actix_web_external::dev::HttpResponseBuilder;
use actix_web_external::{http, HttpResponse, ResponseError};
use log::*;

impl ResponseError for LightSpeedError {
    fn error_response(&self) -> HttpResponse {
        match self {
            LightSpeedError::InvalidTokenError { .. }
            | LightSpeedError::ExpiredTokenError { .. }
            | LightSpeedError::GenerateTokenError { .. }
            | LightSpeedError::MissingAuthTokenError { .. }
            | LightSpeedError::ParseAuthHeaderError { .. }
            | LightSpeedError::UnauthenticatedError => HttpResponse::Unauthorized().finish(),
            LightSpeedError::ForbiddenError { .. } => HttpResponse::Forbidden().finish(),
            LightSpeedError::InternalServerError { message } => {
                error!("Internal server error: {}", message);
                HttpResponse::InternalServerError().finish()
            }
            LightSpeedError::ValidationError { details } => {
                let http_code = http::StatusCode::UNPROCESSABLE_ENTITY;
                HttpResponseBuilder::new(http_code).json(WebErrorDetails::from_error_details(
                    http_code.as_u16(),
                    details,
                ))
            }
            LightSpeedError::BadRequest { .. } => HttpResponse::BadRequest().finish(),
            LightSpeedError::RequestConflict { code, .. } => {
                let http_code = http::StatusCode::CONFLICT;
                HttpResponseBuilder::new(http_code).json(WebErrorDetails::from_message(
                    http_code.as_u16(),
                    &Some((*code).to_string()),
                ))
            }
            LightSpeedError::ServiceUnavailable { code, .. } => {
                let http_code = http::StatusCode::CONFLICT;
                HttpResponseBuilder::new(http_code).json(WebErrorDetails::from_message(
                    http_code.as_u16(),
                    &Some((*code).to_string()),
                ))
            }
            LightSpeedError::ModuleBuilderError { .. }
            | LightSpeedError::ModuleStartError { .. }
            | LightSpeedError::ConfigurationError { .. }
            | LightSpeedError::PasswordEncryptionError { .. }
            | LightSpeedError::RepositoryError { .. } => {
                HttpResponse::InternalServerError().finish()
            }
        }
    }
}
