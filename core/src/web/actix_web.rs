use crate::error::{LsError, WebErrorDetails};
use crate::web::Headers;
use ::http::HeaderValue;
use actix_web::HttpResponseBuilder;
use actix_web::{http, HttpRequest, HttpResponse, ResponseError};

impl Headers for HttpRequest {
    fn get(&self, header_name: &str) -> Option<&HeaderValue> {
        self.headers().get(header_name)
    }
}

impl ResponseError for LsError {
    fn error_response(&self) -> HttpResponse {
        match self {
            LsError::InvalidTokenError { .. }
            | LsError::ExpiredTokenError { .. }
            | LsError::GenerateTokenError { .. }
            | LsError::MissingAuthTokenError { .. }
            | LsError::ParseAuthHeaderError { .. }
            | LsError::UnauthenticatedError => HttpResponse::Unauthorized().finish(),
            LsError::ForbiddenError { .. } => HttpResponse::Forbidden().finish(),
            LsError::ValidationError { details } => {
                let http_code = http::StatusCode::UNPROCESSABLE_ENTITY;
                HttpResponseBuilder::new(http_code)
                    .json(&WebErrorDetails::from_error_details(http_code.as_u16(), details.clone()))
            }
            LsError::BadRequest { code, .. } => {
                let http_code = http::StatusCode::BAD_REQUEST;
                HttpResponseBuilder::new(http_code)
                    .json(&WebErrorDetails::from_message(http_code.as_u16(), Some((*code).into())))
            }
            LsError::C3p0Error { .. } => {
                let http_code = http::StatusCode::BAD_REQUEST;
                HttpResponseBuilder::new(http_code).json(WebErrorDetails::from_message(http_code.as_u16(), None))
            }
            LsError::RequestConflict { code, .. } | LsError::ServiceUnavailable { code, .. } => {
                let http_code = http::StatusCode::CONFLICT;
                HttpResponseBuilder::new(http_code)
                    .json(&WebErrorDetails::from_message(http_code.as_u16(), Some((*code).into())))
            }
            LsError::InternalServerError { .. }
            | LsError::ModuleBuilderError { .. }
            | LsError::ModuleStartError { .. }
            | LsError::ConfigurationError { .. }
            | LsError::PasswordEncryptionError { .. } => HttpResponse::InternalServerError().finish(),
        }
    }
}

#[cfg(test)]
mod test {

    use super::*;
    use crate::config::JwtConfig;
    use crate::error::RootErrorDetails;
    use crate::service::auth::{Auth, InMemoryRolesProvider, LsAuthService, Role};
    use crate::service::jwt::{LsJwtService, JWT};
    use crate::web::{WebAuthService, JWT_TOKEN_HEADER, JWT_TOKEN_HEADER_SUFFIX};
    use actix_web::dev::Service;
    use actix_web::test::{init_service, read_body_json, TestRequest};
    use actix_web::{
        http::{header, StatusCode},
        web, App,
    };
    use jsonwebtoken::Algorithm;
    use std::sync::Arc;

    type AuthIdType = u64;

    #[actix_web::rt::test]
    async fn access_protected_url_should_return_unauthorized_if_no_token() {
        // Arrange
        let srv = init_service(App::new().service(web::resource("/username").to(username))).await;

        let request = TestRequest::get().uri("/username").to_request();

        // Act
        let resp = srv.call(request).await.unwrap();

        // Assert
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    #[actix_web::rt::test]
    async fn access_protected_url_should_return_unauthorized_if_expired_token() {
        // Arrange
        let token = JWT {
            payload: Auth::<AuthIdType> {
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

        let srv = init_service(App::new().service(web::resource("/username").to(username))).await;

        let request = TestRequest::get()
            .uri("/username")
            .append_header((JWT_TOKEN_HEADER, format!("{}{}", JWT_TOKEN_HEADER_SUFFIX, token)))
            .to_request();

        // Act
        let resp = srv.call(request).await.unwrap();

        // Assert
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    #[actix_web::rt::test]
    async fn access_protected_url_should_return_ok_if_valid_token() {
        // Arrange
        let auth = Auth::<AuthIdType> {
            username: "Amelia".to_owned(),
            id: 100,
            session_id: "a_0".to_owned(),
            roles: vec![],
            creation_ts_seconds: 0,
            expiration_ts_seconds: i64::MAX,
        };
        let token = new_service().token_from_auth(&auth).unwrap();

        let srv = init_service(App::new().service(web::resource("/username").to(username))).await;

        let request = TestRequest::get()
            .uri("/username")
            .append_header((JWT_TOKEN_HEADER, format!("{}{}", JWT_TOKEN_HEADER_SUFFIX, token)))
            .to_request();

        // Act
        let resp = srv.call(request).await.unwrap();

        // Assert
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[actix_web::rt::test]
    async fn access_admin_url_should_return_forbidden_if_not_admin_role() {
        // Arrange
        let auth = Auth::<AuthIdType> {
            username: "Amelia".to_owned(),
            id: 100,
            session_id: "a_0".to_owned(),
            roles: vec![],
            creation_ts_seconds: 0,
            expiration_ts_seconds: i64::MAX,
        };
        let token = new_service().token_from_auth(&auth).unwrap();

        let srv = init_service(App::new().service(web::resource("/admin").to(admin))).await;

        let request = TestRequest::get()
            .uri("/admin")
            .append_header((JWT_TOKEN_HEADER, format!("{}{}", JWT_TOKEN_HEADER_SUFFIX, token)))
            .to_request();

        // Act
        let resp = srv.call(request).await.unwrap();

        // Assert
        assert_eq!(resp.status(), StatusCode::FORBIDDEN);
    }

    #[actix_web::rt::test]
    async fn should_return_json_web_error() {
        // Arrange
        let srv = init_service(App::new().service(web::resource("/web_error").to(web_error))).await;

        let request = TestRequest::get().uri("/web_error").to_request();

        // Act
        let resp = srv.call(request).await.unwrap();

        // Assert
        assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!("application/json", resp.headers().get(header::CONTENT_TYPE).unwrap().to_str().unwrap());

        let body: WebErrorDetails = read_body_json(resp).await;
        assert_eq!("error", body.message.unwrap());
    }

    async fn admin(req: HttpRequest) -> actix_web::Result<String> {
        let auth_service = new_service();
        let auth_context = auth_service.auth_from_request(&req)?;
        auth_context.has_role("admin")?;
        Ok(auth_context.auth.username.clone())
    }

    async fn username(req: HttpRequest) -> actix_web::Result<String> {
        let auth_service = new_service();
        let auth_context = auth_service.auth_from_request(&req)?;
        Ok(auth_context.auth.username)
    }

    async fn web_error() -> Result<String, LsError> {
        Err(LsError::ValidationError {
            details: RootErrorDetails { details: Default::default(), message: Some("error".to_owned()) },
        })
    }

    fn new_service() -> WebAuthService<AuthIdType, InMemoryRolesProvider> {
        WebAuthService::new(
            Arc::new(LsAuthService::new(InMemoryRolesProvider::new(
                vec![Role { name: "admin".to_owned(), permissions: vec![] }].into(),
            ))),
            Arc::new(
                LsJwtService::new(&JwtConfig {
                    secret: "secret".to_owned(),
                    signature_algorithm: Algorithm::HS256,
                    token_validity_minutes: 10,
                })
                .unwrap(),
            ),
        )
    }
}
