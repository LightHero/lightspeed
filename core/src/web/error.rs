use crate::error::{ErrorDetails, LightSpeedError};
use actix_web::dev::HttpResponseBuilder;
use actix_web::{HttpResponse, ResponseError};
use serde_derive::Serialize;
use std::collections::HashMap;

#[derive(Serialize)]
pub struct WebErrorDetails<'a> {
    pub code: u16,
    pub message: &'a Option<String>,
    pub details: &'a HashMap<String, Vec<String>>,
}

impl<'a> WebErrorDetails<'a> {
    pub fn from(code: u16, error_details: &'a ErrorDetails) -> Self {
        WebErrorDetails {
            code,
            message: &error_details.message,
            details: &error_details.details,
        }
    }
}

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
            LightSpeedError::InternalServerError { .. } => {
                HttpResponse::InternalServerError().finish()
            }
            LightSpeedError::ValidationError { details } => {
                let code = actix_web::http::StatusCode::UNPROCESSABLE_ENTITY;
                HttpResponseBuilder::new(code).json(WebErrorDetails::from(code.as_u16(), details))
            }
            _ => HttpResponse::InternalServerError().finish(),
        }
    }
}
