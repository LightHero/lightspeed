use crate::error::{LightSpeedError, WebErrorDetails, RootErrorDetails};
use crate::service::auth::{Auth, AuthContext, AuthService, RolesProvider};
use crate::service::jwt::JwtService;
use log::*;
use std::sync::Arc;
use crate::web::{JWT_TOKEN_HEADER, JWT_TOKEN_HEADER_SUFFIX_LEN, Headers};
use axum_ext::response::IntoResponse;
use axum_ext::http::{HeaderValue, Response, StatusCode};
use axum_ext::body::Body;
use axum_ext::prelude::Request;

impl IntoResponse for LightSpeedError {

    fn into_response(self) -> Response<Body> {
        match self {
            LightSpeedError::InvalidTokenError { .. }
            | LightSpeedError::ExpiredTokenError { .. }
            | LightSpeedError::GenerateTokenError { .. }
            | LightSpeedError::MissingAuthTokenError { .. }
            | LightSpeedError::ParseAuthHeaderError { .. }
            | LightSpeedError::UnauthenticatedError => {
                response_with_code(StatusCode::UNAUTHORIZED)
            },
            LightSpeedError::ForbiddenError { .. } => {
                response_with_code(StatusCode::FORBIDDEN)
            },
            LightSpeedError::ValidationError { details } => {
                response_with_error_details(StatusCode::UNPROCESSABLE_ENTITY, &details)
            }
            LightSpeedError::BadRequest { code, .. } => {
                response_with_message(StatusCode::BAD_REQUEST, &Some((code).to_string()))
            }
            LightSpeedError::C3p0Error { .. } => {
                response_with_message(StatusCode::BAD_REQUEST, &None)
            }
            LightSpeedError::RequestConflict { code, .. } |
            LightSpeedError::ServiceUnavailable { code, .. } => {
                response_with_message(StatusCode::CONFLICT, &Some((code).to_string()))
            }
            LightSpeedError::InternalServerError { .. }
            | LightSpeedError::ModuleBuilderError { .. }
            | LightSpeedError::ModuleStartError { .. }
            | LightSpeedError::ConfigurationError { .. }
            | LightSpeedError::PasswordEncryptionError { .. } => {
                response_with_code(http::StatusCode::INTERNAL_SERVER_ERROR)
            },
        }
    }
}

#[inline]
fn response_with_code(http_code: StatusCode) -> Response<Body> {
    let mut res = Response::new(Body::empty());
    *res.status_mut() = http_code;
    res
}

#[inline]
fn response_with_message(http_code: StatusCode, message: &Option<String>) -> Response<Body> {
    response(http_code, &WebErrorDetails::from_message(http_code.as_u16(), message))
}

#[inline]
fn response_with_error_details(http_code: StatusCode, details: &RootErrorDetails) -> Response<Body> {
    response(http_code, &WebErrorDetails::from_error_details(http_code.as_u16(), details))
}

#[inline]
fn response(http_code: StatusCode, details: &WebErrorDetails<'_>) -> Response<Body> {
    match serde_json::to_vec(details) {
        Ok(body) => {
            let mut res = Response::new(Body::from(body));
            *res.status_mut() = http_code;
            res
        }
        Err(err) => {
            error!("response_with_message - cannot serialize body. Err: {:?}", err);
            let mut res = Response::new(Body::empty());
            *res.status_mut() = http::StatusCode::INTERNAL_SERVER_ERROR;
            res
        }
    }
}

/*
#[cfg(test)]
mod test {

    use super::*;
    use crate::config::JwtConfig;
    use crate::service::auth::{InMemoryRolesProvider, Role};
    use crate::service::jwt::JWT;
    use actix_web_4_ext::dev::Service;
    use actix_web_4_ext::test::{init_service, TestRequest};
    use actix_web_4_ext::{http::StatusCode, web, App};
    use jsonwebtoken::Algorithm;
    use crate::web::JWT_TOKEN_HEADER_SUFFIX;

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
*/