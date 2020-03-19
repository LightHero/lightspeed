use crate::error::{LightSpeedError, WebErrorDetails};
use actix_web_external::dev::HttpResponseBuilder;
use actix_web_external::error::BlockingError;
use actix_web_external::web::Json;
use actix_web_external::{http, HttpResponse, ResponseError};
use log::*;

pub async fn web_block<I, F>(f: F) -> Result<I, LightSpeedError>
where
    F: FnOnce() -> Result<I, LightSpeedError> + Send + 'static,
    I: Send + 'static,
{
    actix_web_external::web::block(f).await.map_err(|err| match err {
        BlockingError::Error(e) => e,
        _ => LightSpeedError::InternalServerError { message: format!("{}", err) },
    })
}

pub async fn web_block_json<I, F>(f: F) -> Result<Json<I>, LightSpeedError>
where
    F: FnOnce() -> Result<I, LightSpeedError> + Send + 'static,
    I: Send + 'static,
{
    web_block(f).await.map(Json)
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
            LightSpeedError::InternalServerError { message } => {
                error!("Internal server error: {}", message);
                HttpResponse::InternalServerError().finish()
            }
            LightSpeedError::ValidationError { details } => {
                let http_code = http::StatusCode::UNPROCESSABLE_ENTITY;
                HttpResponseBuilder::new(http_code).json(WebErrorDetails::from_error_details(http_code.as_u16(), details))
            }
            LightSpeedError::BadRequest { .. } => HttpResponse::BadRequest().finish(),
            LightSpeedError::RequestConflict { code, .. } => {
                let http_code = http::StatusCode::CONFLICT;
                HttpResponseBuilder::new(http_code).json(WebErrorDetails::from_message(http_code.as_u16(), &Some((*code).to_string())))
            }
            LightSpeedError::ServiceUnavailable { code, .. } => {
                let http_code = http::StatusCode::CONFLICT;
                HttpResponseBuilder::new(http_code).json(WebErrorDetails::from_message(http_code.as_u16(), &Some((*code).to_string())))
            }
            LightSpeedError::ModuleBuilderError { .. }
            | LightSpeedError::ModuleStartError { .. }
            | LightSpeedError::ConfigurationError { .. }
            | LightSpeedError::PasswordEncryptionError { .. }
            | LightSpeedError::RepositoryError { .. } => HttpResponse::InternalServerError().finish(),
        }
    }
}
