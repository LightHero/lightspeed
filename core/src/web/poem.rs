use crate::error::{LsError, RootErrorDetails, WebErrorDetails};
use crate::web::Headers;
use log::*;
use poem::{Request, Response, error::ResponseError, http::StatusCode};
use std::error::Error as StdError;

impl Headers for Request {
    fn get_as_str(&self, header_name: &str) -> Option<Result<&str, LsError>> {
        self.headers().get_as_str(header_name)
    }
}

impl ResponseError for LsError {
    fn status(&self) -> StatusCode {
        match self {
            LsError::InvalidTokenError { .. }
            | LsError::ExpiredTokenError { .. }
            | LsError::GenerateTokenError { .. }
            | LsError::MissingAuthTokenError
            | LsError::ParseAuthHeaderError { .. }
            | LsError::UnauthenticatedError => StatusCode::UNAUTHORIZED,
            LsError::ForbiddenError { .. } => StatusCode::FORBIDDEN,
            LsError::ValidationError { .. } => StatusCode::UNPROCESSABLE_ENTITY,
            LsError::BadRequest { .. } => StatusCode::BAD_REQUEST,
            LsError::C3p0Error { .. } => StatusCode::BAD_REQUEST,
            LsError::RequestConflict { .. } | LsError::ServiceUnavailable { .. } => StatusCode::CONFLICT,
            LsError::InternalServerError { .. }
            | LsError::ModuleBuilderError { .. }
            | LsError::ModuleStartError { .. }
            | LsError::ConfigurationError { .. }
            | LsError::PasswordEncryptionError { .. } => http::StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn as_response(&self) -> Response
    where
        Self: StdError + Send + Sync + 'static,
    {
        error!("Converting error into poem response. Err: {self:?}");
        match self {
            LsError::InvalidTokenError { .. }
            | LsError::ExpiredTokenError { .. }
            | LsError::GenerateTokenError { .. }
            | LsError::MissingAuthTokenError
            | LsError::ParseAuthHeaderError { .. }
            | LsError::UnauthenticatedError => response_with_code(self.status()),
            LsError::ForbiddenError { .. } => response_with_code(self.status()),
            LsError::ValidationError { details } => response_with_error_details(self.status(), details.clone()),
            LsError::BadRequest { code, .. } => response_with_message(self.status(), Some((code).to_string())),
            LsError::C3p0Error { .. } => response_with_message(self.status(), None),
            LsError::RequestConflict { code, .. } | LsError::ServiceUnavailable { code, .. } => {
                response_with_message(self.status(), Some((code).to_string()))
            }
            LsError::InternalServerError { .. }
            | LsError::ModuleBuilderError { .. }
            | LsError::ModuleStartError { .. }
            | LsError::ConfigurationError { .. }
            | LsError::PasswordEncryptionError { .. } => response_with_code(self.status()),
        }
    }
}

#[inline]
fn response_with_code(http_code: StatusCode) -> Response {
    Response::builder().status(http_code).finish()
}

#[inline]
fn response_with_message(http_code: StatusCode, message: Option<String>) -> Response {
    response(http_code, &WebErrorDetails::from_message(http_code.as_u16(), message))
}

#[inline]
fn response_with_error_details(http_code: StatusCode, details: RootErrorDetails) -> Response {
    response(http_code, &WebErrorDetails::from_error_details(http_code.as_u16(), details))
}

#[inline]
fn response(http_code: StatusCode, details: &WebErrorDetails) -> Response {
    match serde_json::to_vec(details) {
        Ok(body) => Response::builder()
            .status(http_code)
            .header(http::header::CONTENT_TYPE, http::HeaderValue::from_static("application/json"))
            .body(body),
        Err(err) => {
            error!("response_with_message - cannot serialize body. Err: {err:?}");
            Response::builder().status(StatusCode::INTERNAL_SERVER_ERROR).finish()
        }
    }
}

#[cfg(feature = "poem_openapi")]
pub mod openapi {
    use crate::error::{LsError, WebErrorDetails};
    use log::error;
    use poem::http::StatusCode;
    use poem_openapi::ApiResponse;
    use poem_openapi::payload::Json;

    #[derive(ApiResponse, Debug)]
    pub enum LightSpeedErrorResponse {
        #[oai(status = 400)]
        BadRequest(Json<WebErrorDetails>),
        #[oai(status = 401)]
        Unauthorized,
        #[oai(status = 403)]
        Forbidden,
        #[oai(status = 409)]
        Conflict(Json<WebErrorDetails>),
        #[oai(status = 422)]
        UnprocessableEntity(Json<WebErrorDetails>),
        #[oai(status = 500)]
        InternalServerError,
    }

    impl From<LsError> for LightSpeedErrorResponse {
        fn from(err: LsError) -> Self {
            error!("Converting error into poem response. Err: {err:?}");
            match err {
                LsError::InvalidTokenError { .. }
                | LsError::ExpiredTokenError { .. }
                | LsError::GenerateTokenError { .. }
                | LsError::MissingAuthTokenError
                | LsError::ParseAuthHeaderError { .. }
                | LsError::UnauthenticatedError => LightSpeedErrorResponse::Unauthorized,
                LsError::ForbiddenError { .. } => LightSpeedErrorResponse::Forbidden,
                LsError::ValidationError { details } => LightSpeedErrorResponse::UnprocessableEntity(Json(
                    WebErrorDetails::from_error_details(StatusCode::UNPROCESSABLE_ENTITY.as_u16(), details),
                )),
                LsError::BadRequest { code, .. } => LightSpeedErrorResponse::BadRequest(Json(
                    WebErrorDetails::from_message(StatusCode::BAD_REQUEST.as_u16(), Some((code).to_string())),
                )),
                LsError::C3p0Error { .. } => LightSpeedErrorResponse::BadRequest(Json(WebErrorDetails::from_message(
                    StatusCode::BAD_REQUEST.as_u16(),
                    None,
                ))),
                LsError::RequestConflict { code, .. } | LsError::ServiceUnavailable { code, .. } => {
                    LightSpeedErrorResponse::Conflict(Json(WebErrorDetails::from_message(
                        StatusCode::CONFLICT.as_u16(),
                        Some((code).to_string()),
                    )))
                }
                LsError::InternalServerError { .. }
                | LsError::ModuleBuilderError { .. }
                | LsError::ModuleStartError { .. }
                | LsError::ConfigurationError { .. }
                | LsError::PasswordEncryptionError { .. } => LightSpeedErrorResponse::InternalServerError,
            }
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
    use jsonwebtoken::Algorithm;
    use poem::http::HeaderMap;
    use poem::test::TestClient;
    use poem::{Request, Route, handler};
    use std::sync::Arc;

    type AuthIdType = u64;

    #[tokio::test]
    async fn access_protected_url_should_return_unauthorized_if_no_token() {
        // Arrange

        let app = Route::new().at("/username", username);
        let cli = TestClient::new(app);

        // Act
        let resp = cli.get("/username").send().await.0;

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

        let app = Route::new().at("/username", username);
        let cli = TestClient::new(app);

        // Act
        let resp =
            cli.get("/username").header(JWT_TOKEN_HEADER, format!("{JWT_TOKEN_HEADER_SUFFIX}{token}")).send().await;

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

        let app = Route::new().at("/username", username);
        let cli = TestClient::new(app);

        // Act
        let resp =
            cli.get("/username").header(JWT_TOKEN_HEADER, format!("{JWT_TOKEN_HEADER_SUFFIX}{token}")).send().await;

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

        let app = Route::new().at("/admin", admin);
        let cli = TestClient::new(app);

        // Act
        let resp = cli.get("/admin").header(JWT_TOKEN_HEADER, format!("{JWT_TOKEN_HEADER_SUFFIX}{token}")).send().await;

        // Assert
        resp.assert_status(StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn should_return_json_web_error() {
        // Arrange
        let app = Route::new().at("/web_error", web_error);
        let cli = TestClient::new(app);

        // Act
        let resp = cli.get("/web_error").send().await;

        // Assert
        resp.assert_status(StatusCode::UNPROCESSABLE_ENTITY);
        resp.assert_header(http::header::CONTENT_TYPE, "application/json");

        let body: WebErrorDetails = resp.0.into_body().into_json().await.unwrap();
        assert_eq!("error", body.message.unwrap());
    }

    #[handler]
    async fn admin(req: &HeaderMap) -> Result<String, LsError> {
        let auth_service = new_service();
        let auth_context = auth_service.auth_from_request(req)?;
        auth_context.has_role("admin")?;
        Ok(auth_context.auth.username.clone())
    }

    #[handler]
    async fn username(req: &Request) -> Result<String, LsError> {
        let auth_service = new_service();
        let auth_context = auth_service.auth_from_request(req)?;
        Ok(auth_context.auth.username)
    }

    #[handler]
    async fn web_error() -> Result<String, LsError> {
        Err(LsError::ValidationError {
            details: RootErrorDetails { details: Default::default(), message: Some("error".to_owned()) },
        })
    }

    fn new_service() -> WebAuthService<AuthIdType> {
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

    #[cfg(feature = "poem_openapi")]
    #[cfg(test)]
    mod test_openapi {
        use super::*;
        use crate::web::poem::openapi::LightSpeedErrorResponse;
        use poem::Route;
        use poem::test::TestClient;
        use poem_openapi::payload::{Json, PlainText};
        use poem_openapi::*;
        use serde::Serialize;

        #[tokio::test]
        async fn should_use_lightspeederror_with_openapi_ok_text() {
            // Arrange
            let api_service = OpenApiService::new(Api, "Hello World", "1.0").server("http://localhost:3000/api");
            let ui = api_service.swagger_ui();

            let app = Route::new().nest("/api", api_service).nest("/", ui);
            let cli = TestClient::new(app);

            // Act
            let resp = cli.get("/api/ok_text").send().await.0;

            // Assert
            assert_eq!(resp.status(), StatusCode::OK);
            let body = resp.into_body().into_string().await.unwrap();
            println!("{body:?}")
        }

        #[tokio::test]
        async fn should_use_lightspeederror_with_openapi_ok_json() {
            // Arrange
            let api_service = OpenApiService::new(Api, "Hello World", "1.0").server("http://localhost:3000/api");
            let ui = api_service.swagger_ui();

            let app = Route::new().nest("/api", api_service).nest("/", ui);
            let cli = TestClient::new(app);

            // Act
            let resp = cli.get("/api/ok_json").send().await;

            // Assert
            resp.assert_status_is_ok();
            resp.assert_json(MyJson { data: "ok json".to_owned() }).await;
        }

        #[tokio::test]
        async fn access_protected_url_should_return_unauthorized_if_no_token() {
            // Arrange
            let api_service = OpenApiService::new(Api, "Hello World", "1.0").server("http://localhost:3000/api");
            let ui = api_service.swagger_ui();

            let app = Route::new().nest("/api", api_service).nest("/", ui);
            let cli = TestClient::new(app);

            // Act
            let resp = cli.get("/api/username").send().await.0;

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

            let api_service = OpenApiService::new(Api, "Hello World", "1.0").server("http://localhost:3000/api");
            let ui = api_service.swagger_ui();

            let app = Route::new().nest("/api", api_service).nest("/", ui);
            let cli = TestClient::new(app);

            // Act
            let resp = cli
                .get("/api/username")
                .header(JWT_TOKEN_HEADER, format!("{JWT_TOKEN_HEADER_SUFFIX}{token}"))
                .send()
                .await;

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

            let api_service = OpenApiService::new(Api, "Hello World", "1.0").server("http://localhost:3000/api");
            let ui = api_service.swagger_ui();

            let app = Route::new().nest("/api", api_service).nest("/", ui);
            let cli = TestClient::new(app);

            // Act
            let resp = cli
                .get("/api/username")
                .header(JWT_TOKEN_HEADER, format!("{JWT_TOKEN_HEADER_SUFFIX}{token}"))
                .send()
                .await;

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

            let api_service = OpenApiService::new(Api, "Hello World", "1.0").server("http://localhost:3000/api");
            let ui = api_service.swagger_ui();

            let app = Route::new().nest("/api", api_service).nest("/", ui);
            let cli = TestClient::new(app);

            // Act
            let resp = cli
                .get("/api/admin")
                .header(JWT_TOKEN_HEADER, format!("{JWT_TOKEN_HEADER_SUFFIX}{token}"))
                .send()
                .await;

            // Assert
            resp.assert_status(StatusCode::FORBIDDEN);
        }

        #[tokio::test]
        async fn should_return_json_web_error() {
            // Arrange
            let api_service = OpenApiService::new(Api, "Hello World", "1.0").server("http://localhost:3000/api");
            let ui = api_service.swagger_ui();

            let app = Route::new().nest("/api", api_service).nest("/", ui);
            let cli = TestClient::new(app);

            // Act
            let resp = cli.get("/api/web_error").send().await;

            // Assert
            resp.assert_status(StatusCode::UNPROCESSABLE_ENTITY);
            resp.assert_header(http::header::CONTENT_TYPE, "application/json; charset=utf-8");

            let body: WebErrorDetails = resp.0.into_body().into_json().await.unwrap();
            assert_eq!("error", body.message.unwrap());
        }

        #[derive(Serialize, Object)]
        pub struct MyJson {
            data: String,
        }

        struct Api;

        #[OpenApi]
        impl Api {
            #[oai(path = "/ok_text", method = "get")]
            async fn ok_text(&self) -> Result<PlainText<String>, LightSpeedErrorResponse> {
                Ok(PlainText("ok text".to_owned()))
            }

            #[oai(path = "/ok_json", method = "get")]
            async fn ok_json(&self) -> Result<Json<MyJson>, LightSpeedErrorResponse> {
                Ok(Json(MyJson { data: "ok json".to_owned() }))
            }

            #[oai(path = "/admin", method = "get")]
            async fn admin(&self, req: &Request) -> Result<PlainText<String>, LightSpeedErrorResponse> {
                let auth_service = new_service();
                let auth_context = auth_service.auth_from_request(req)?;
                auth_context.has_role("admin")?;
                Ok(PlainText(auth_context.auth.username.clone()))
            }

            #[oai(path = "/username", method = "get")]
            async fn username(&self, req: &Request) -> Result<PlainText<String>, LightSpeedErrorResponse> {
                let auth_service = new_service();
                let auth_context = auth_service.auth_from_request(req)?;
                Ok(PlainText(auth_context.auth.username))
            }

            #[oai(path = "/web_error", method = "get")]
            async fn web_error(&self) -> Result<PlainText<String>, LightSpeedErrorResponse> {
                Err(LsError::ValidationError {
                    details: RootErrorDetails { details: Default::default(), message: Some("error".to_owned()) },
                })?
            }
        }
    }
}
