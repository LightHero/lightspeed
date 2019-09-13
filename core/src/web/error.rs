use crate::error::{ErrorDetail, ErrorDetails, LightSpeedError};
use actix_web::dev::HttpResponseBuilder;
use actix_web::{HttpResponse, ResponseError};
use serde_derive::Serialize;
use std::cell::RefCell;
use std::collections::HashMap;
use std::ops::Deref;

#[derive(Serialize)]
pub struct WebErrorDetails<'a> {
    pub code: u16,
    pub message: &'a Option<String>,
    pub details: &'a RefCell<HashMap<String, Vec<ErrorDetail>>>,
}

impl<'a> WebErrorDetails<'a> {
    pub fn from(code: u16, error_details: &'a ErrorDetails) -> Self {
        match error_details {
            ErrorDetails::Root { message, details } => WebErrorDetails {
                code,
                message,
                details: details.deref(),
            },
            ErrorDetails::Child { details, .. } => WebErrorDetails {
                code,
                message: &None,
                details: details.deref(),
            },
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
            LightSpeedError::BadRequest { .. } => HttpResponse::BadRequest().finish(),
            _ => HttpResponse::InternalServerError().finish(),
        }
    }
}
