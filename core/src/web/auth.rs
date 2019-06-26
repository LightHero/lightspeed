use actix_web::HttpRequest;
use log::*;
use crate::error::LightSpeedError;
use crate::service::auth::{Auth, AuthContext, AuthService, InMemoryRolesProvider};
use crate::service::jwt::JwtService;

pub const JWT_TOKEN_HEADER: &str = "Authorization";
pub const JWT_TOKEN_HEADER_SUFFIX: &str = "Bearer ";
// This must be equal to JWT_TOKEN_HEADER_SUFFIX.len()
pub const JWT_TOKEN_HEADER_SUFFIX_LEN: usize = 7;

/*
impl FromRequest for AuthContext<'static> {

    type Error = LightSpeedError;
    type Future = Result<Self, Self::Error>;
    type Config = ();

    fn from_request(req: &HttpRequest, payload: &mut Payload) -> Self::Future {
        if let Some(core) = req.get_app_data::<CoreModule>() {
            auth_from_request(req, &core.jwt, &core.auth)
        } else {
            log::debug!(
                "Failed to construct App-level Data extractor. \
                 Request path: {:?}",
                req.path()
            );
            Err(LightSpeedError::InternalServerError {
                message: "App data is not configured, to configure use App::data()",
            })
        }
    }

}
*/

#[derive(Clone)]
pub struct WebAuthService {
    auth_service: AuthService<InMemoryRolesProvider>,
    jwt_service: JwtService,
}

impl WebAuthService {
    pub fn token_string_from_request(req: &HttpRequest) -> Result<&str, LightSpeedError> {
        if let Some(header) = req.headers().get(JWT_TOKEN_HEADER) {
            return header
                .to_str()
                .map_err(|err| LightSpeedError::ParseAuthHeaderError {
                    message: format!("{}", err),
                })
                .and_then(|header| {
                    debug!("Token found in request: [{}]", header);
                    if header.len() > JWT_TOKEN_HEADER_SUFFIX_LEN {
                        Ok(&header[JWT_TOKEN_HEADER_SUFFIX_LEN..])
                    } else {
                        Err(LightSpeedError::ParseAuthHeaderError {
                            message: format!("Unexpected auth header: {}", header),
                        })
                    }
                });
        };
        Err(LightSpeedError::MissingAuthTokenError)
    }

    pub fn token_from_auth(&self, auth: &Auth) -> Result<String, LightSpeedError> {
        Ok(self.jwt_service.generate_from_payload(auth)?)
    }

    pub fn auth_from_request(&self, req: &HttpRequest) -> Result<AuthContext, LightSpeedError> {
        WebAuthService::token_string_from_request(req).and_then(|token| {
            let auth = self.jwt_service.parse_payload::<Auth>(token)?;
            Ok(self.auth_service.auth(auth))
        })
    }
}

#[cfg(test)]
mod test {

    use super::*;
    use actix_service::Service;
    use actix_web::test::{block_on, init_service, TestRequest};
    use actix_web::{http::StatusCode, web, App};
    use jsonwebtoken::Algorithm;
    use crate::config::JwtConfig;
    use crate::service::auth::Role;
    use crate::service::jwt::Token;

    #[test]
    fn access_protected_url_should_return_unauthorized_if_no_token() {
        // Arrange
        let mut srv = init_service(App::new().service(web::resource("/auth").to(username)));

        let request = TestRequest::get().uri("/auth").to_request();

        // Act
        let resp = block_on(srv.call(request)).unwrap();

        // Assert
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    #[test]
    fn access_protected_url_should_return_unauthorized_if_expired_token() {
        crate::test_root::init_context();

        // Arrange
        let token = Token {
            payload: Auth {
                username: "Amelia".to_owned(),
                id: 100,
                roles: vec![],
            },
            exp: 0,
            iat: 0,
            sub: "".to_owned(),
        };
        let token = new_service()
            .jwt_service
            .generate_from_token(&token)
            .unwrap();

        let mut srv = init_service(App::new().service(web::resource("/auth").to(username)));

        let request = TestRequest::get()
            .uri("/auth")
            .header(
                JWT_TOKEN_HEADER,
                format!("{}{}", JWT_TOKEN_HEADER_SUFFIX, token),
            )
            .to_request();

        // Act
        let resp = block_on(srv.call(request)).unwrap();

        // Assert
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    #[test]
    fn access_protected_url_should_return_ok_if_valid_token() {
        crate::test_root::init_context();

        // Arrange
        let auth = Auth {
            username: "Amelia".to_owned(),
            id: 100,
            roles: vec![],
        };
        let token = new_service().token_from_auth(&auth).unwrap();

        let mut srv = init_service(App::new().service(web::resource("/auth").to(username)));

        let request = TestRequest::get()
            .uri("/auth")
            .header(
                JWT_TOKEN_HEADER,
                format!("{}{}", JWT_TOKEN_HEADER_SUFFIX, token),
            )
            .to_request();

        // Act
        let resp = block_on(srv.call(request)).unwrap();

        // Assert
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[test]
    fn access_admin_url_should_return_forbidden_if_not_admin_role() {
        crate::test_root::init_context();

        // Arrange
        let auth = Auth {
            username: "Amelia".to_owned(),
            id: 100,
            roles: vec![],
        };
        let token = new_service().token_from_auth(&auth).unwrap();

        let mut srv = init_service(App::new().service(web::resource("/auth").to(admin)));

        let request = TestRequest::get()
            .uri("/auth")
            .header(
                JWT_TOKEN_HEADER,
                format!("{}{}", JWT_TOKEN_HEADER_SUFFIX, token),
            )
            .to_request();

        // Act
        let resp = block_on(srv.call(request)).unwrap();

        // Assert
        assert_eq!(resp.status(), StatusCode::FORBIDDEN);
    }

    fn admin(req: HttpRequest) -> actix_web::Result<String> {
        let auth_service = new_service();
        let auth_context = auth_service.auth_from_request(&req)?;
        auth_context.has_role("admin")?;
        Ok(auth_context.auth.username.clone())
    }

    fn username(req: HttpRequest) -> actix_web::Result<String> {
        let auth_service = new_service();
        let auth_context = auth_service.auth_from_request(&req)?;
        Ok(auth_context.auth.username)
    }

    fn new_service() -> WebAuthService {
        WebAuthService {
            auth_service: AuthService::new(InMemoryRolesProvider::new(vec![Role {
                name: "admin".to_owned(),
                permissions: vec![],
            }])),
            jwt_service: JwtService::new(&JwtConfig {
                secret: "secret".to_owned(),
                signature_algorithm: Algorithm::HS256,
                token_validity_minutes: 10,
            }),
        }
    }

}
