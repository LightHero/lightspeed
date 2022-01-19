use crate::error::{LightSpeedError, WebErrorDetails};
use crate::web::Headers;
use actix_web_4_ext::HttpResponseBuilder;
use actix_web_4_ext::{http, HttpRequest, HttpResponse, ResponseError};
use ::http::HeaderValue;

impl Headers for HttpRequest {
    fn get(&self, header_name: &str) -> Option<&HeaderValue> {
        self.headers().get(header_name)
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
            LightSpeedError::ValidationError { details } => {
                let http_code = http::StatusCode::UNPROCESSABLE_ENTITY;
                HttpResponseBuilder::new(http_code)
                    .json(&WebErrorDetails::from_error_details(http_code.as_u16(), details))
            }
            LightSpeedError::BadRequest { code, .. } => {
                let http_code = http::StatusCode::BAD_REQUEST;
                HttpResponseBuilder::new(http_code)
                    .json(&WebErrorDetails::from_message(http_code.as_u16(), Some((*code).into())))
            }
            LightSpeedError::C3p0Error { .. } => {
                let http_code = http::StatusCode::BAD_REQUEST;
                HttpResponseBuilder::new(http_code).json(WebErrorDetails::from_message(http_code.as_u16(), None))
            }
            LightSpeedError::RequestConflict { code, .. } | LightSpeedError::ServiceUnavailable { code, .. } => {
                let http_code = http::StatusCode::CONFLICT;
                HttpResponseBuilder::new(http_code)
                    .json(&WebErrorDetails::from_message(http_code.as_u16(), Some((*code).into())))
            }
            LightSpeedError::InternalServerError { .. }
            | LightSpeedError::ModuleBuilderError { .. }
            | LightSpeedError::ModuleStartError { .. }
            | LightSpeedError::ConfigurationError { .. }
            | LightSpeedError::PasswordEncryptionError { .. } => HttpResponse::InternalServerError().finish(),
        }
    }
}

#[cfg(test)]
mod test {

    use super::*;
    use crate::config::JwtConfig;
    use crate::error::RootErrorDetails;
    use crate::service::auth::{Auth, AuthService, InMemoryRolesProvider, Role};
    use crate::service::jwt::{JwtService, JWT};
    use crate::web::{WebAuthService, JWT_TOKEN_HEADER, JWT_TOKEN_HEADER_SUFFIX};
    use actix_web_4_ext::dev::Service;
    use actix_web_4_ext::test::{init_service, read_body_json, TestRequest};
    use actix_web_4_ext::{
        http::{header, StatusCode},
        web, App,
    };
    use jsonwebtoken::Algorithm;
    use std::sync::Arc;

    #[actix_web_4_ext::rt::test]
    async fn access_protected_url_should_return_unauthorized_if_no_token() {
        // Arrange
        let srv = init_service(App::new().service(web::resource("/auth").to(username))).await;

        let request = TestRequest::get().uri("/auth").to_request();

        // Act
        let resp = srv.call(request).await.unwrap();

        // Assert
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    #[actix_web_4_ext::rt::test]
    async fn access_protected_url_should_return_unauthorized_if_expired_token() {
        // Arrange
        let token = JWT {
            payload: Auth {
                username: "Amelia".to_owned(),
                id: 100,
                session_id: "a_0".to_owned(),
                roles: vec![],
                creation_ts_seconds: 0,
                expiration_ts_seconds: i64::MAX,
            },
            exp: 0,
            iat: 0,
            sub: "".to_owned(),
        };
        let token = new_service().jwt_service.generate_from_token(&token).unwrap();

        let srv = init_service(App::new().service(web::resource("/auth").to(username))).await;

        let request = TestRequest::get()
            .uri("/auth")
            .append_header((JWT_TOKEN_HEADER, format!("{}{}", JWT_TOKEN_HEADER_SUFFIX, token)))
            .to_request();

        // Act
        let resp = srv.call(request).await.unwrap();

        // Assert
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    #[actix_web_4_ext::rt::test]
    async fn access_protected_url_should_return_ok_if_valid_token() {
        // Arrange
        let auth = Auth {
            username: "Amelia".to_owned(),
            id: 100,
            session_id: "a_0".to_owned(),
            roles: vec![],
            creation_ts_seconds: 0,
            expiration_ts_seconds: i64::MAX,
        };
        let token = new_service().token_from_auth(&auth).unwrap();

        let srv = init_service(App::new().service(web::resource("/auth").to(username))).await;

        let request = TestRequest::get()
            .uri("/auth")
            .append_header((JWT_TOKEN_HEADER, format!("{}{}", JWT_TOKEN_HEADER_SUFFIX, token)))
            .to_request();

        // Act
        let resp = srv.call(request).await.unwrap();

        // Assert
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[actix_web_4_ext::rt::test]
    async fn access_admin_url_should_return_forbidden_if_not_admin_role() {
        // Arrange
        let auth = Auth {
            username: "Amelia".to_owned(),
            id: 100,
            session_id: "a_0".to_owned(),
            roles: vec![],
            creation_ts_seconds: 0,
            expiration_ts_seconds: i64::MAX,
        };
        let token = new_service().token_from_auth(&auth).unwrap();

        let srv = init_service(App::new().service(web::resource("/auth").to(admin))).await;

        let request = TestRequest::get()
            .uri("/auth")
            .append_header((JWT_TOKEN_HEADER, format!("{}{}", JWT_TOKEN_HEADER_SUFFIX, token)))
            .to_request();

        // Act
        let resp = srv.call(request).await.unwrap();

        // Assert
        assert_eq!(resp.status(), StatusCode::FORBIDDEN);
    }

    #[actix_web_4_ext::rt::test]
    async fn should_return_json_web_error() {
        // Arrange
        let srv = init_service(App::new().service(web::resource("/err").to(web_error))).await;

        let request = TestRequest::get().uri("/err").to_request();

        // Act
        let resp = srv.call(request).await.unwrap();

        // Assert
        assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!("application/json", resp.headers().get(header::CONTENT_TYPE).unwrap().to_str().unwrap());

        let body: WebErrorDetails = read_body_json(resp).await;
        assert_eq!("error", body.message.unwrap());
    }

    async fn admin(req: HttpRequest) -> actix_web_4_ext::Result<String> {
        let auth_service = new_service();
        let auth_context = auth_service.auth_from_request(&req)?;
        auth_context.has_role("admin")?;
        Ok(auth_context.auth.username.clone())
    }

    async fn username(req: HttpRequest) -> actix_web_4_ext::Result<String> {
        let auth_service = new_service();
        let auth_context = auth_service.auth_from_request(&req)?;
        Ok(auth_context.auth.username)
    }

    async fn web_error() -> Result<String, LightSpeedError> {
        Err(LightSpeedError::ValidationError {
            details: RootErrorDetails { details: Default::default(), message: Some("error".to_owned()) },
        })
    }

    fn new_service() -> WebAuthService<InMemoryRolesProvider> {
        WebAuthService {
            auth_service: Arc::new(AuthService::new(InMemoryRolesProvider::new(
                vec![Role { name: "admin".to_owned(), permissions: vec![] }].into(),
            ))),
            jwt_service: Arc::new(
                JwtService::new(&JwtConfig {
                    secret: "secret".to_owned(),
                    signature_algorithm: Algorithm::HS256,
                    token_validity_minutes: 10,
                })
                .unwrap(),
            ),
        }
    }
}
