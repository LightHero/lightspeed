use crate::error::{LightSpeedError, RootErrorDetails, WebErrorDetails};
use axum_ext::body::{Body, HttpBody};
use axum_ext::http::{header, HeaderValue, Response, StatusCode};
use axum_ext::response::IntoResponse;
use log::*;

impl IntoResponse for LightSpeedError {
    type Body = Body;
    type BodyError = <Self::Body as HttpBody>::Error;

    fn into_response(self) -> Response<Body> {
        match self {
            LightSpeedError::InvalidTokenError { .. }
            | LightSpeedError::ExpiredTokenError { .. }
            | LightSpeedError::GenerateTokenError { .. }
            | LightSpeedError::MissingAuthTokenError { .. }
            | LightSpeedError::ParseAuthHeaderError { .. }
            | LightSpeedError::UnauthenticatedError => response_with_code(StatusCode::UNAUTHORIZED),
            LightSpeedError::ForbiddenError { .. } => response_with_code(StatusCode::FORBIDDEN),
            LightSpeedError::ValidationError { details } => {
                response_with_error_details(StatusCode::UNPROCESSABLE_ENTITY, &details)
            }
            LightSpeedError::BadRequest { code, .. } => {
                response_with_message(StatusCode::BAD_REQUEST, &Some((code).to_string()))
            }
            LightSpeedError::C3p0Error { .. } => response_with_message(StatusCode::BAD_REQUEST, &None),
            LightSpeedError::RequestConflict { code, .. } | LightSpeedError::ServiceUnavailable { code, .. } => {
                response_with_message(StatusCode::CONFLICT, &Some((code).to_string()))
            }
            LightSpeedError::InternalServerError { .. }
            | LightSpeedError::ModuleBuilderError { .. }
            | LightSpeedError::ModuleStartError { .. }
            | LightSpeedError::ConfigurationError { .. }
            | LightSpeedError::PasswordEncryptionError { .. } => {
                response_with_code(http::StatusCode::INTERNAL_SERVER_ERROR)
            }
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
    response(http_code, &WebErrorDetails::from_message(http_code.as_u16(), message.as_ref().map(|val| val.into())))
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
            res.headers_mut().insert(header::CONTENT_TYPE, HeaderValue::from_static("application/json"));
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

#[cfg(test)]
mod test {

    use super::*;
    use crate::config::JwtConfig;
    use crate::service::auth::{Auth, AuthService, InMemoryRolesProvider, Role};
    use crate::service::jwt::{JwtService, JWT};
    use crate::web::{WebAuthService, JWT_TOKEN_HEADER, JWT_TOKEN_HEADER_SUFFIX};
    use axum_ext::http::{header, HeaderMap, Request};
    use axum_ext::Router;
    use axum_ext::routing::get;
    use jsonwebtoken::Algorithm;
    use std::sync::Arc;
    use tower::ServiceExt; // for `app.oneshot()`

    #[tokio::test]
    async fn access_protected_url_should_return_unauthorized_if_no_token() {
        // Arrange
        let app = Router::new().route("/auth", get(username));

        // Act
        let resp = app
            .oneshot(Request::builder().method(http::Method::GET).uri("/auth").body(Body::empty()).unwrap())
            .await
            .unwrap();

        // Assert
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
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

        let app = Router::new().route("/auth", get(username));

        // Act
        let resp = app
            .oneshot(
                Request::builder()
                    .method(http::Method::GET)
                    .uri("/auth")
                    .header(JWT_TOKEN_HEADER, format!("{}{}", JWT_TOKEN_HEADER_SUFFIX, token))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        // Assert
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
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

        let app = Router::new().route("/auth", get(username));

        // Act
        let resp = app
            .oneshot(
                Request::builder()
                    .method(http::Method::GET)
                    .uri("/auth")
                    .header(JWT_TOKEN_HEADER, format!("{}{}", JWT_TOKEN_HEADER_SUFFIX, token))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        // Assert
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
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

        let app = Router::new().route("/auth", get(admin));

        // Act
        let resp = app
            .oneshot(
                Request::builder()
                    .method(http::Method::GET)
                    .uri("/auth")
                    .header(JWT_TOKEN_HEADER, format!("{}{}", JWT_TOKEN_HEADER_SUFFIX, token))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        // Assert
        assert_eq!(resp.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn should_return_json_web_error() {
        // Arrange
        let app = Router::new().route("/err", get(web_error));

        // Act
        let resp = app
            .oneshot(Request::builder().method(http::Method::GET).uri("/err").body(Body::empty()).unwrap())
            .await
            .unwrap();

        // Assert
        assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);

        assert_eq!("application/json", resp.headers().get(header::CONTENT_TYPE).unwrap().to_str().unwrap());

        let body = hyper::body::to_bytes(resp.into_body()).await.unwrap();
        let body: WebErrorDetails = serde_json::from_slice(&body).unwrap();

        assert_eq!("error", body.message.unwrap());
    }

    async fn admin(req: HeaderMap) -> Result<String, LightSpeedError> {
        let auth_service = new_service();
        let auth_context = auth_service.auth_from_request(&req)?;
        auth_context.has_role("admin")?;
        Ok(auth_context.auth.username.clone())
    }

    async fn username(req: Request<Body>) -> Result<String, LightSpeedError> {
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
