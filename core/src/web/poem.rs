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
        match self {
            LightSpeedError::InvalidTokenError { .. }
            | LightSpeedError::ExpiredTokenError { .. }
            | LightSpeedError::GenerateTokenError { .. }
            | LightSpeedError::MissingAuthTokenError { .. }
            | LightSpeedError::ParseAuthHeaderError { .. }
            | LightSpeedError::UnauthenticatedError => response_with_code(self.status()),
            LightSpeedError::ForbiddenError { .. } => response_with_code(self.status()),
            LightSpeedError::ValidationError { details } => response_with_error_details(self.status(), details.clone()),
            LightSpeedError::BadRequest { code, .. } => response_with_message(self.status(), Some((code).to_string())),
            LightSpeedError::C3p0Error { .. } => response_with_message(self.status(), None),
            LightSpeedError::RequestConflict { code, .. } | LightSpeedError::ServiceUnavailable { code, .. } => {
                response_with_message(self.status(), Some((code).to_string()))
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
            error!("response_with_message - cannot serialize body. Err: {:?}", err);
            Response::builder().status(StatusCode::INTERNAL_SERVER_ERROR).finish()
        }
    }
}

#[cfg(feature = "poem_openapi")]
mod open_api {

    use crate::error::{LightSpeedError, WebErrorDetails};
    use poem_ext::http::StatusCode;
    use poem_openapi_ext::payload::Json;
    use poem_openapi_ext::types::{ToJSON, Type};
    use poem_openapi_ext::ApiResponse;

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

    impl From<LightSpeedError> for LightSpeedErrorResponse {
        fn from(err: LightSpeedError) -> Self {
            match err {
                LightSpeedError::InvalidTokenError { .. }
                | LightSpeedError::ExpiredTokenError { .. }
                | LightSpeedError::GenerateTokenError { .. }
                | LightSpeedError::MissingAuthTokenError { .. }
                | LightSpeedError::ParseAuthHeaderError { .. }
                | LightSpeedError::UnauthenticatedError => LightSpeedErrorResponse::Unauthorized,
                LightSpeedError::ForbiddenError { .. } => LightSpeedErrorResponse::Forbidden,
                LightSpeedError::ValidationError { details } => LightSpeedErrorResponse::UnprocessableEntity(Json(
                    WebErrorDetails::from_error_details(StatusCode::UNPROCESSABLE_ENTITY.as_u16(), details),
                )),
                LightSpeedError::BadRequest { code, .. } => LightSpeedErrorResponse::BadRequest(Json(
                    WebErrorDetails::from_message(StatusCode::BAD_REQUEST.as_u16(), Some((code).to_string())),
                )),
                LightSpeedError::C3p0Error { .. } => LightSpeedErrorResponse::BadRequest(Json(
                    WebErrorDetails::from_message(StatusCode::BAD_REQUEST.as_u16(), None),
                )),
                LightSpeedError::RequestConflict { code, .. } | LightSpeedError::ServiceUnavailable { code, .. } => {
                    LightSpeedErrorResponse::Conflict(Json(WebErrorDetails::from_message(
                        StatusCode::CONFLICT.as_u16(),
                        Some((code).to_string()),
                    )))
                }
                LightSpeedError::InternalServerError { .. }
                | LightSpeedError::ModuleBuilderError { .. }
                | LightSpeedError::ModuleStartError { .. }
                | LightSpeedError::ConfigurationError { .. }
                | LightSpeedError::PasswordEncryptionError { .. } => LightSpeedErrorResponse::InternalServerError,
            }
        }
    }

    /// A generic JSON type response with status code 200
    #[derive(ApiResponse)]
    pub enum JsonResponse<T: Type + ToJSON> {
        #[oai(status = 200)]
        Ok(Json<T>),
    }

    impl<T: Type + ToJSON> JsonResponse<T> {
        /// Returns Ok(JsonResponse::Ok(Json(json)))
        pub fn ok<E>(json: T) -> Result<Self, E> {
            Ok(JsonResponse::Ok(Json(json)))
        }
    }

    impl<T: Type + ToJSON> From<Json<T>> for JsonResponse<T> {
        fn from(json: Json<T>) -> Self {
            JsonResponse::Ok(json)
        }
    }

    impl<T: Type + ToJSON> From<T> for JsonResponse<T> {
        fn from(json: T) -> Self {
            JsonResponse::Ok(Json(json))
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

#[cfg(test)]
mod test_openapi {

    use super::*;
    use crate::config::JwtConfig;
    use crate::service::auth::{Auth, AuthService, InMemoryRolesProvider, Role};
    use crate::service::jwt::{JwtService, JWT};
    use crate::web::poem::open_api::OpenApiResponse;
    use crate::web::{WebAuthService, JWT_TOKEN_HEADER, JWT_TOKEN_HEADER_SUFFIX};
    use jsonwebtoken::Algorithm;
    use poem_ext::http::HeaderMap;
    use poem_ext::test::TestClient;
    use poem_ext::web::Json;
    use poem_ext::{handler, Request, Route};
    use poem_openapi_ext::payload::PlainText;
    use poem_openapi_ext::registry::{MetaResponses, Registry};
    use poem_openapi_ext::*;
    use std::sync::Arc;

    #[tokio::test]
    async fn should_use_lightspeederror_with_openapi() {
        // Arrange
        let api_service = OpenApiService::new(Api, "Hello World", "1.0").server("http://localhost:3000/api");
        let ui = api_service.swagger_ui();

        let app = Route::new().nest("/api", api_service).nest("/", ui);
        let cli = TestClient::new(app);

        // Act
        let resp = cli.get("/api").send().await.into_inner();

        // Assert
        assert_eq!(resp.status(), StatusCode::OK);
        let body = resp.into_body().into_string().await.unwrap();
        println!("{:?}", body)
    }

    struct Api;

    #[OpenApi]
    impl Api {
        #[oai(path = "/ok", method = "get")]
        async fn web_ok(&self) -> OpenApiResponse<PlainText<String>> {
            Ok("".to_owned()).into()
        }

        #[oai(path = "/admin", method = "get")]
        async fn admin(&self, req: &HeaderMap) -> poem_ext::Result<PlainText<String>> {
            let auth_service = new_service();
            let auth_context = auth_service.auth_from_request(req)?;
            auth_context.has_role("admin")?;
            Ok(PlainText(auth_context.auth.username.clone()))
        }

        #[oai(path = "/username", method = "get")]
        async fn username(&self, req: &Request) -> poem_ext::Result<PlainText<String>> {
            let auth_service = new_service();
            let auth_context = auth_service.auth_from_request(req)?;
            Ok(PlainText(auth_context.auth.username))
        }

        #[oai(path = "/error", method = "get")]
        async fn web_error(&self) -> poem_ext::Result<PlainText<String>> {
            Err(LightSpeedError::ValidationError {
                details: RootErrorDetails { details: Default::default(), message: Some("error".to_owned()) },
            })?
        }
    }

    #[derive(ApiResponse)]
    enum GetResponse {
        #[oai(status = 200)]
        Todo(PlainText<String>),

        #[oai(status = 404)]
        NotFound(PlainText<String>),
    }

    // impl ApiResponse for Result<(), LightSpeedError> {
    //     fn meta() -> MetaResponses {
    //         todo!()
    //     }
    //
    //     fn register(registry: &mut Registry) {
    //         todo!()
    //     }
    // }

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