use crate::error::{LightSpeedError, RootErrorDetails, WebErrorDetails};
use crate::web::Headers;
use http::HeaderValue;
use log::*;
use poem_ext::{error::ResponseError, http::StatusCode, Request, Response};
use std::error::Error as StdError;

impl Headers for Request {
    fn get(&self, header_name: &str) -> Option<&HeaderValue> {
        self.headers().get(header_name)
    }
}

impl ResponseError for LightSpeedError {
    fn status(&self) -> StatusCode {
        match self {
            LightSpeedError::InvalidTokenError { .. }
            | LightSpeedError::ExpiredTokenError { .. }
            | LightSpeedError::GenerateTokenError { .. }
            | LightSpeedError::MissingAuthTokenError { .. }
            | LightSpeedError::ParseAuthHeaderError { .. }
            | LightSpeedError::UnauthenticatedError => StatusCode::UNAUTHORIZED,
            LightSpeedError::ForbiddenError { .. } => StatusCode::FORBIDDEN,
            LightSpeedError::ValidationError { .. } => StatusCode::UNPROCESSABLE_ENTITY,
            LightSpeedError::BadRequest { .. } => StatusCode::BAD_REQUEST,
            LightSpeedError::C3p0Error { .. } => StatusCode::BAD_REQUEST,
            LightSpeedError::RequestConflict { .. } | LightSpeedError::ServiceUnavailable { .. } => {
                StatusCode::CONFLICT
            }
            LightSpeedError::InternalServerError { .. }
            | LightSpeedError::ModuleBuilderError { .. }
            | LightSpeedError::ModuleStartError { .. }
            | LightSpeedError::ConfigurationError { .. }
            | LightSpeedError::PasswordEncryptionError { .. } => http::StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn as_response(&self) -> Response
    where
        Self: StdError + Send + Sync + 'static,
    {
        //     Response::builder()
        //         .status(self.status())
        //         .body(self.to_string())
        // }

        match self {
            LightSpeedError::InvalidTokenError { .. }
            | LightSpeedError::ExpiredTokenError { .. }
            | LightSpeedError::GenerateTokenError { .. }
            | LightSpeedError::MissingAuthTokenError { .. }
            | LightSpeedError::ParseAuthHeaderError { .. }
            | LightSpeedError::UnauthenticatedError => response_with_code(self.status()),
            LightSpeedError::ForbiddenError { .. } => response_with_code(self.status()),
            LightSpeedError::ValidationError { details } => response_with_error_details(self.status(), &details),
            LightSpeedError::BadRequest { code, .. } => response_with_message(self.status(), &Some((code).to_string())),
            LightSpeedError::C3p0Error { .. } => response_with_message(self.status(), &None),
            LightSpeedError::RequestConflict { code, .. } | LightSpeedError::ServiceUnavailable { code, .. } => {
                response_with_message(self.status(), &Some((code).to_string()))
            }
            LightSpeedError::InternalServerError { .. }
            | LightSpeedError::ModuleBuilderError { .. }
            | LightSpeedError::ModuleStartError { .. }
            | LightSpeedError::ConfigurationError { .. }
            | LightSpeedError::PasswordEncryptionError { .. } => response_with_code(self.status()),
        }
    }
}

#[inline]
fn response_with_code(http_code: StatusCode) -> Response {
    Response::builder().status(http_code).finish()
}

#[inline]
fn response_with_message(http_code: StatusCode, message: &Option<String>) -> Response {
    response(http_code, &WebErrorDetails::from_message(http_code.as_u16(), message.as_ref().map(|val| val.into())))
}

#[inline]
fn response_with_error_details(http_code: StatusCode, details: &RootErrorDetails) -> Response {
    response(http_code, &WebErrorDetails::from_error_details(http_code.as_u16(), details))
}

#[inline]
fn response(http_code: StatusCode, details: &WebErrorDetails<'_>) -> Response {
    match serde_json::to_vec(details) {
        Ok(body) => Response::builder()
            .status(http_code)
            .header(http::header::CONTENT_TYPE, http::HeaderValue::from_static("application/json"))
            .body(body),
        Err(err) => {
            error!("response_with_message - cannot serialize body. Err: {:?}", err);
            Response::builder().status(StatusCode::INTERNAL_SERVER_ERROR).finish()
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
    use jsonwebtoken::Algorithm;
    use poem_ext::http::HeaderMap;
    use poem_ext::test::TestClient;
    use poem_ext::{handler, Request, Route};
    use std::sync::Arc;

    #[tokio::test]
    async fn access_protected_url_should_return_unauthorized_if_no_token() {
        // Arrange

        let app = Route::new().at("/auth", username);
        let cli = TestClient::new(app);

        // Act
        let resp = cli.get("/auth").send().await.into_inner();

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

        let app = Route::new().at("/auth", username);
        let cli = TestClient::new(app);

        // Act
        let resp =
            cli.get("/auth").header(JWT_TOKEN_HEADER, format!("{}{}", JWT_TOKEN_HEADER_SUFFIX, token)).send().await;

        // Assert
        resp.assert_status(StatusCode::UNAUTHORIZED);
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

        let app = Route::new().at("/auth", username);
        let cli = TestClient::new(app);

        // Act
        let resp =
            cli.get("/auth").header(JWT_TOKEN_HEADER, format!("{}{}", JWT_TOKEN_HEADER_SUFFIX, token)).send().await;

        // Assert
        resp.assert_status_is_ok();
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

        let app = Route::new().at("/auth", admin);
        let cli = TestClient::new(app);

        // Act
        let resp =
            cli.get("/auth").header(JWT_TOKEN_HEADER, format!("{}{}", JWT_TOKEN_HEADER_SUFFIX, token)).send().await;

        // Assert
        resp.assert_status(StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn should_return_json_web_error() {
        // Arrange
        let app = Route::new().at("/err", web_error);
        let cli = TestClient::new(app);

        // Act
        let resp = cli.get("/err").send().await;

        // Assert
        resp.assert_status(StatusCode::UNPROCESSABLE_ENTITY);
        resp.assert_header(http::header::CONTENT_TYPE, "application/json");

        let body: WebErrorDetails = resp.into_body().into_json().await.unwrap();
        assert_eq!("error", body.message.unwrap());
    }

    #[handler]
    async fn admin(req: &HeaderMap) -> Result<String, LightSpeedError> {
        let auth_service = new_service();
        let auth_context = auth_service.auth_from_request(req)?;
        auth_context.has_role("admin")?;
        Ok(auth_context.auth.username.clone())
    }

    #[handler]
    async fn username(req: &Request) -> Result<String, LightSpeedError> {
        let auth_service = new_service();
        let auth_context = auth_service.auth_from_request(req)?;
        Ok(auth_context.auth.username)
    }

    #[handler]
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
