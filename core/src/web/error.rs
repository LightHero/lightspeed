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
            | LightSpeedError::UnauthenticatedError => {
                warn!("User is unauthorized: {}", self);
                HttpResponse::Unauthorized().finish()
            }
            LightSpeedError::ForbiddenError { .. } => {
                warn!("Access is forbidden: {}", self);
                HttpResponse::Forbidden().finish()
            }
            LightSpeedError::InternalServerError { message } => {
                error!("Internal server error: {}", message);
                HttpResponse::InternalServerError().finish()
            }
            LightSpeedError::ValidationError { details } => {
                warn!("ValidationError: {}", self);
                let http_code = http::StatusCode::UNPROCESSABLE_ENTITY;
                HttpResponseBuilder::new(http_code).json(WebErrorDetails::from_error_details(
                    http_code.as_u16(),
                    details,
                ))
            }
            LightSpeedError::BadRequest { .. } => {
                warn!("BadRequest: {}", self);
                HttpResponse::BadRequest().finish()
            }
            LightSpeedError::RequestConflict { code, .. } => {
                error!("RequestConflict: {}", self);
                let http_code = http::StatusCode::CONFLICT;
                HttpResponseBuilder::new(http_code).json(WebErrorDetails::from_message(
                    http_code.as_u16(),
                    &Some((*code).to_string()),
                ))
            }
            LightSpeedError::ServiceUnavailable { code, .. } => {
                error!("ServiceUnavailable: {}", self);
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
                error!("InternalServerError: {}", self);
                HttpResponse::InternalServerError().finish()
            }
        }
    }
}
