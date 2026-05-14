use crate::error::LsError;
use axum::body::Body;
use axum::http::{Response, StatusCode};
use axum::response::IntoResponse;

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

            LsError::BadRequest { .. } => response_with_code(StatusCode::BAD_REQUEST),
            LsError::C3p0Error { .. } => response_with_code(StatusCode::BAD_REQUEST),
            LsError::SqlxError { .. } => response_with_code(StatusCode::BAD_REQUEST),
            LsError::ModuleStartError { .. } | LsError::ConfigurationError { .. } => {
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

#[cfg(test)]
mod test {

    use super::*;
    use crate::config::JwtConfig;
    use crate::service::auth::{Auth, InMemoryRolesProvider, LsAuthService, Role};
    use crate::service::jwt::{JWT, LsJwtService};
    use crate::web::{JWT_TOKEN_HEADER, JWT_TOKEN_HEADER_SUFFIX, WebAuthService};
    use axum::Router;
    use axum::http::{HeaderMap, Request};
    use axum::routing::get;
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

    fn new_service() -> WebAuthService {
        WebAuthService::new(
            Arc::new(LsAuthService::new(InMemoryRolesProvider::new(
                vec![Role { name: "admin".to_owned(), permissions: vec![] }].into(),
            ))),
            Arc::new(
                LsJwtService::new(&JwtConfig {
                    secret: "secret".into(),
                    signature_algorithm: Algorithm::HS256,
                    token_validity_minutes: 10,
                })
                .unwrap(),
            ),
        )
    }
}
