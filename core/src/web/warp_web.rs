use crate::error::{LightSpeedError, WebErrorDetails};
use crate::service::auth::{Auth, AuthContext, AuthService, RolesProvider};
use crate::service::jwt::JwtService;
use log::*;
use std::sync::Arc;
use warp_external::reply::Response;
use warp_external::{header, http, path, reply, Filter, Rejection, Reply};

pub const JWT_TOKEN_HEADER: &str = "Authorization";
pub const JWT_TOKEN_HEADER_SUFFIX: &str = "Bearer ";
pub const JWT_TOKEN_HEADER_SUFFIX_LEN: usize = JWT_TOKEN_HEADER_SUFFIX.len();

#[derive(Clone)]
pub struct WebAuthService<T: RolesProvider> {
    auth_service: Arc<AuthService<T>>,
    jwt_service: Arc<JwtService>,
}

pub struct ReqInfo {
    pub method: http::Method,
    pub path: path::FullPath,
    pub auth_header: Option<String>,
}

pub fn req_info() -> impl Filter<Extract = (ReqInfo,), Error = Rejection> + Clone {
    warp_external::method().and(path::full()).and(header::optional::<String>(JWT_TOKEN_HEADER)).map(
        |method: http::Method, path: path::FullPath, auth_header: Option<String>| ReqInfo { method, path, auth_header },
    )
}

impl<T: RolesProvider> WebAuthService<T> {
    pub fn new(auth_service: Arc<AuthService<T>>, jwt_service: Arc<JwtService>) -> Self {
        Self { auth_service, jwt_service }
    }

    pub fn token_from_auth(&self, auth: &Auth) -> Result<String, LightSpeedError> {
        Ok(self.jwt_service.generate_from_payload(auth)?.1)
    }

    pub fn auth_from_token_string(&self, auth_header: &Option<String>) -> Result<AuthContext, LightSpeedError> {
        if let Some(header) = auth_header {
            trace!("Token found in request: [{}]", header);
            if header.len() > JWT_TOKEN_HEADER_SUFFIX_LEN {
                let token = &header[JWT_TOKEN_HEADER_SUFFIX_LEN..];
                let auth = self.jwt_service.parse_payload::<Auth>(token);
                trace!("Auth built from request: [{:?}]", auth);
                Ok(self.auth_service.auth(auth?))
            } else {
                Err(LightSpeedError::ParseAuthHeaderError { message: format!("Unexpected auth header: {}", header) })
            }
        } else {
            Err(LightSpeedError::MissingAuthTokenError)
        }
    }
}

impl Reply for LightSpeedError {
    fn into_response(self) -> Response {
        match self {
            LightSpeedError::InvalidTokenError { .. }
            | LightSpeedError::ExpiredTokenError { .. }
            | LightSpeedError::GenerateTokenError { .. }
            | LightSpeedError::MissingAuthTokenError { .. }
            | LightSpeedError::ParseAuthHeaderError { .. }
            | LightSpeedError::UnauthenticatedError => http::StatusCode::UNAUTHORIZED.into_response(),
            LightSpeedError::ForbiddenError { .. } => http::StatusCode::FORBIDDEN.into_response(),
            LightSpeedError::InternalServerError { .. } => http::StatusCode::INTERNAL_SERVER_ERROR.into_response(),
            LightSpeedError::ValidationError { details } => {
                let http_code = http::StatusCode::UNPROCESSABLE_ENTITY;
                let error_details = WebErrorDetails::from_error_details(http_code.as_u16(), &details);
                reply::with_status(reply::json(&error_details), http_code).into_response()
            }
            LightSpeedError::BadRequest { code, .. } => {
                let http_code = http::StatusCode::BAD_REQUEST;
                let error_details =
                    reply::json(&WebErrorDetails::from_message(http_code.as_u16(), &Some(code.to_string())));
                reply::with_status(error_details, http_code).into_response()
            }
            LightSpeedError::RepositoryError { .. } => {
                let http_code = http::StatusCode::BAD_REQUEST;
                let error_details =
                    reply::json(&WebErrorDetails::from_message(http_code.as_u16(), &None));
                reply::with_status(error_details, http_code).into_response()
            }
            LightSpeedError::RequestConflict { code, .. } => {
                let http_code = http::StatusCode::CONFLICT;
                let error_details =
                    reply::json(&WebErrorDetails::from_message(http_code.as_u16(), &Some(code.to_string())));
                reply::with_status(error_details, http_code).into_response()
            }
            LightSpeedError::ServiceUnavailable { code, .. } => {
                let http_code = http::StatusCode::CONFLICT;
                let error_details =
                    reply::json(&WebErrorDetails::from_message(http_code.as_u16(), &Some(code.to_string())));
                reply::with_status(error_details, http_code).into_response()
            }
            LightSpeedError::ModuleBuilderError { .. }
            | LightSpeedError::ModuleStartError { .. }
            | LightSpeedError::ConfigurationError { .. }
            | LightSpeedError::PasswordEncryptionError { .. } => http::StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        }
    }
}
