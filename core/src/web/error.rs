use actix_web::{HttpResponse};
use actix_web::dev::HttpResponseBuilder;
use serde_derive::{Deserialize, Serialize};
use crate::error::{LightSpeedError, ErrorDetails};
use std::collections::HashMap;

#[derive(Serialize, Deserialize)]
pub struct WebErrorDetails {
    pub code: u16,
    pub message: Option<String>,
    pub details: HashMap<String, Vec<String>>
}

impl WebErrorDetails {
    pub fn from(code: u16, error_details: ErrorDetails) -> Self {
        WebErrorDetails{
            code,
            message: error_details.message,
            details: error_details.details
        }
    }
}

impl From<LightSpeedError> for HttpResponse {
    fn from(err: LightSpeedError) -> Self {
        match err {
            LightSpeedError::InvalidTokenError { .. }
            | LightSpeedError::ExpiredTokenError { .. }
            | LightSpeedError::GenerateTokenError { .. }
            | LightSpeedError::MissingAuthTokenError { .. }
            | LightSpeedError::ParseAuthHeaderError { .. }
            | LightSpeedError::UnauthenticatedError => HttpResponse::Unauthorized().finish(),
            LightSpeedError::ForbiddenError { .. } => HttpResponse::Forbidden().finish(),
            LightSpeedError::InternalServerError { .. } => {
                HttpResponse::InternalServerError().finish()
            },
            LightSpeedError::ValidationError { details } => {
                let code = actix_web::http::StatusCode::UNPROCESSABLE_ENTITY;
                HttpResponseBuilder::new(code).json(WebErrorDetails::from(code.as_u16(), details))
            },
            _ => HttpResponse::InternalServerError().finish(),
        }
    }
}
