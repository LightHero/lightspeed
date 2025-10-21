use crate::error::{LsError, RootErrorDetails, WebErrorDetails};
use axum::body::Body;
use axum::http::{HeaderValue, Response, StatusCode, header};
use axum::response::IntoResponse;
use log::*;

impl IntoResponse for LsError {
    fn into_response(self) -> Response<Body> {
        match self {
            LsError::InvalidTokenError { .. }
            | LsError::ExpiredTokenError { .. }
            | LsError::GenerateTokenError { .. }
            | LsError::MissingAuthTokenError
            | LsError::ParseAuthHeaderError { .. }
            | LsError::UnauthenticatedError => response_with_code(StatusCode::UNAUTHORIZED),
            LsError::ForbiddenError { .. } => response_with_code(StatusCode::FORBIDDEN),
            LsError::ValidationError { details } => {
                response_with_error_details(StatusCode::UNPROCESSABLE_ENTITY, details)
            }
            LsError::BadRequest { code, .. } => {
                response_with_message(StatusCode::BAD_REQUEST, Some((code).to_string()))
            }
            LsError::C3p0Error { .. } => response_with_message(StatusCode::BAD_REQUEST, None),
            LsError::SqlxError { .. } => response_with_message(StatusCode::BAD_REQUEST, None),
            LsError::RequestConflict { code, .. } | LsError::ServiceUnavailable { code, .. } => {
                response_with_message(StatusCode::CONFLICT, Some((code).to_string()))
            }
            LsError::InternalServerError { .. }
            | LsError::ModuleBuilderError { .. }
            | LsError::ModuleStartError { .. }
            | LsError::ConfigurationError { .. }
            | LsError::PasswordEncryptionError { .. } => response_with_code(http::StatusCode::INTERNAL_SERVER_ERROR),
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
fn response_with_message(http_code: StatusCode, message: Option<String>) -> Response<Body> {
    response(http_code, &WebErrorDetails::from_message(http_code.as_u16(), message.as_ref().map(|val| val.into())))
}

#[inline]
fn response_with_error_details(http_code: StatusCode, details: RootErrorDetails) -> Response<Body> {
    response(http_code, &WebErrorDetails::from_error_details(http_code.as_u16(), details))
}

#[inline]
fn response(http_code: StatusCode, details: &WebErrorDetails) -> Response<Body> {
    match serde_json::to_vec(details) {
        Ok(body) => {
            let mut res = Response::new(Body::from(body));
            *res.status_mut() = http_code;
            res.headers_mut().insert(header::CONTENT_TYPE, HeaderValue::from_static("application/json"));
            res
        }
        Err(err) => {
            error!("response_with_message - cannot serialize body. Err: {err:?}");
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
    use crate::service::auth::{Auth, InMemoryRolesProvider, LsAuthService, Role};
    use crate::service::jwt::{JWT, LsJwtService};
    use crate::web::{JWT_TOKEN_HEADER, JWT_TOKEN_HEADER_SUFFIX, WebAuthService};
    use axum::Router;
    use axum::http::{HeaderMap, Request, header};
    use axum::routing::get;
    use http_body_util::BodyExt;
    use jsonwebtoken::Algorithm;
    use std::sync::Arc;
    use tower::ServiceExt; // for `app.oneshot()`

    #[tokio::test]
    async fn access_protected_url_should_return_unauthorized_if_no_token() {
        // Arrange
        let app = Router::new().route("/username", get(username));

        // Act
        let resp = app
            .oneshot(Request::builder().method(http::Method::GET).uri("/username").body(Body::empty()).unwrap())
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

        let app = Router::new().route("/username", get(username));

        // Act
        let resp = app
            .oneshot(
                Request::builder()
                    .method(http::Method::GET)
                    .uri("/username")
                    .header(JWT_TOKEN_HEADER, format!("{JWT_TOKEN_HEADER_SUFFIX}{token}"))
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

        let app = Router::new().route("/username", get(username));

        // Act
        let resp = app
            .oneshot(
                Request::builder()
                    .method(http::Method::GET)
                    .uri("/username")
                    .header(JWT_TOKEN_HEADER, format!("{JWT_TOKEN_HEADER_SUFFIX}{token}"))
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

        let app = Router::new().route("/admin", get(admin));

        // Act
        let resp = app
            .oneshot(
                Request::builder()
                    .method(http::Method::GET)
                    .uri("/admin")
                    .header(JWT_TOKEN_HEADER, format!("{JWT_TOKEN_HEADER_SUFFIX}{token}"))
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
        let app = Router::new().route("/web_error", get(web_error));

        // Act
        let resp = app
            .oneshot(Request::builder().method(http::Method::GET).uri("/web_error").body(Body::empty()).unwrap())
            .await
            .unwrap();

        // Assert
        assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);

        assert_eq!("application/json", resp.headers().get(header::CONTENT_TYPE).unwrap().to_str().unwrap());

        let body = resp.into_body().collect().await.unwrap().to_bytes();
        let body: WebErrorDetails = serde_json::from_slice(&body).unwrap();

        assert_eq!("error", body.message.unwrap());
    }

    async fn admin(req: HeaderMap) -> Result<String, LsError> {
        let auth_service = new_service();
        let auth_context = auth_service.auth_from_request(&req)?;
        auth_context.has_role("admin")?;
        Ok(auth_context.auth.username.clone())
    }

    async fn username(req: Request<Body>) -> Result<String, LsError> {
        let auth_service = new_service();
        let auth_context = auth_service.auth_from_request(&req)?;
        Ok(auth_context.auth.username)
    }

    async fn web_error() -> Result<String, LsError> {
        Err(LsError::ValidationError {
            details: RootErrorDetails { details: Default::default(), message: Some("error".to_owned()) },
        })
    }

    fn new_service() -> WebAuthService {
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
