use crate::error::LsError;
use crate::service::auth::{Auth, AuthContext, LsAuthService, RolesProvider};
use crate::service::jwt::LsJwtService;
use http::{HeaderMap, HeaderValue, Request};
use log::*;
use std::sync::Arc;

#[cfg(feature = "actix_web")]
pub mod actix_web;
#[cfg(feature = "axum")]
pub mod axum;
#[cfg(feature = "poem")]
pub mod poem;

pub const JWT_TOKEN_HEADER: &str = "Authorization";
pub const JWT_TOKEN_HEADER_SUFFIX: &str = "Bearer ";
pub const JWT_TOKEN_HEADER_SUFFIX_LEN: usize = JWT_TOKEN_HEADER_SUFFIX.len();

pub trait Headers {
    fn get(&self, header_name: &str) -> Option<&HeaderValue>;
}

impl Headers for HeaderMap {
    fn get(&self, header_name: &str) -> Option<&HeaderValue> {
        self.get(header_name)
    }
}

impl<T> Headers for Request<T> {
    fn get(&self, header_name: &str) -> Option<&HeaderValue> {
        self.headers().get(header_name)
    }
}

#[derive(Clone)]
pub struct WebAuthService<T: RolesProvider> {
    auth_service: Arc<LsAuthService<T>>,
    jwt_service: Arc<LsJwtService>,
}

impl<T: RolesProvider> WebAuthService<T> {
    pub fn new(auth_service: Arc<LsAuthService<T>>, jwt_service: Arc<LsJwtService>) -> Self {
        Self { auth_service, jwt_service }
    }

    pub fn token_string_from_request<'a, H: Headers>(&self, req: &'a H) -> Result<&'a str, LsError> {
        if let Some(header) = req.get(JWT_TOKEN_HEADER) {
            return header
                .to_str()
                .map_err(|err| LsError::ParseAuthHeaderError { message: format!("{:?}", err) })
                .and_then(|header| {
                    trace!("Token found in request: [{}]", header);
                    if header.len() > JWT_TOKEN_HEADER_SUFFIX_LEN {
                        Ok(&header[JWT_TOKEN_HEADER_SUFFIX_LEN..])
                    } else {
                        Err(LsError::ParseAuthHeaderError {
                            message: format!("Unexpected auth header: {}", header),
                        })
                    }
                });
        };
        Err(LsError::MissingAuthTokenError)
    }

    pub fn token_from_auth(&self, auth: &Auth) -> Result<String, LsError> {
        Ok(self.jwt_service.generate_from_payload(auth)?.1)
    }

    pub fn auth_from_request<H: Headers>(&self, req: &H) -> Result<AuthContext, LsError> {
        self.token_string_from_request(req).and_then(|token| self.auth_from_token_string(token))
    }

    pub fn auth_from_token_string(&self, token: &str) -> Result<AuthContext, LsError> {
        let auth = self.jwt_service.parse_payload::<Auth>(token);
        trace!("Auth built from request: [{:?}]", auth);
        Ok(self.auth_service.auth(auth?))
    }
}
