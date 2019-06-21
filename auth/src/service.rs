use actix_web::HttpRequest;
use coreutils_auth::{AuthService, InMemoryRolesProvider, AuthContext};
use coreutils_auth::model::Auth;
use coreutils_jwt::JwtService;
use crate::error::AuthError;

pub const JWT_TOKEN_HEADER: &str = "Authorization";
pub const JWT_TOKEN_HEADER_SUFFIX: &str = "Bearer ";
// This must be equal to JWT_TOKEN_HEADER_SUFFIX.len()
pub const JWT_TOKEN_HEADER_SUFFIX_LEN: usize = 7;


pub struct JwtAuthService {
    auth_service: AuthService<InMemoryRolesProvider>,
    jwt_service: JwtService
}

impl JwtAuthService {

    pub fn token_string_from_request(req: &HttpRequest) -> Result<&str, AuthError> {
        if let Some(header) = req.headers().get(JWT_TOKEN_HEADER) {
            return header.to_str()
                .map_err(|err| AuthError::ParseAuthHeaderError {message: format!("{}", err)})
                .and_then(|header|
                    if header.len() > JWT_TOKEN_HEADER_SUFFIX_LEN {
                        Ok(&header[JWT_TOKEN_HEADER_SUFFIX_LEN..])
                    } else {
                        Err(AuthError::ParseAuthHeaderError {message: format!("Unexpected auth header: {}", header)})
                    }
                );
        };
        Err(AuthError::MissingAuthTokenError)
    }

    pub fn token_from_auth(&self, auth: &Auth) -> Result<String, AuthError> {
        Ok(self.jwt_service.generate_from_payload(auth)?)
    }

    pub fn auth_from_request(&self, req: &HttpRequest) -> Result<AuthContext, AuthError> {
        JwtAuthService::token_string_from_request(req)
            .and_then(|token| {
                let auth = self.jwt_service.parse_payload::<Auth>(token)?;
                Ok(self.auth_service.auth(auth))
            })
    }

}