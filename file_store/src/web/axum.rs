use crate::model::BinaryContent;
use axum::{
    body::Body,
    http::{Response, header, response::Builder},
};
use lightspeed_core::error::LsError;
use log::*;
use std::borrow::Cow;

pub async fn into_response(
    content: BinaryContent<'_>,
    file_name: Option<&str>,
    set_content_disposition: bool,
) -> Result<Response<Body>, LsError> {
    let (file_name, ct, body) = match content {
        BinaryContent::InMemory { content } => {
            debug!("Create HttpResponse from Memory content of {} bytes", content.len());
            let file_name = Cow::Borrowed(file_name.unwrap_or(""));
            let path = std::path::Path::new(file_name.as_ref());
            let ct = mime_guess::from_path(path).first_or_octet_stream();
            let owned_vec: Vec<u8> = content.into();
            (file_name, ct, Body::from(owned_vec))
        }
        BinaryContent::OpenDal { operator, path } => {
            let file_path = std::path::Path::new(&path);
            let ct = mime_guess::from_path(&path).first_or_octet_stream();

            let file_name = if let Some(file_name) = file_name {
                Cow::Borrowed(file_name)
            } else {
                match file_path.file_name().to_owned() {
                    Some(name) => Cow::Owned(name.to_string_lossy().as_ref().to_owned()),
                    None => {
                        return Err(LsError::BadRequest {
                            message: "Provided path has no filename".to_owned(),
                            code: "",
                        })?;
                    }
                }
            };

            let reader = operator.reader(&path).await.unwrap();
            let stream = reader.into_bytes_stream(..).await.unwrap();

            (file_name, ct, Body::from_stream(stream))
        }
    };

    let mut response_builder = Builder::new();

    response_builder = response_builder.header(header::CONTENT_TYPE, format!("{ct}; charset=utf-8"));

    if set_content_disposition {
        debug!("Set content disposition");

        let mut disposition = String::new();

        let disposition_type = match ct.type_() {
            mime::IMAGE | mime::TEXT | mime::VIDEO => "inline; ",
            mime::APPLICATION => match ct.subtype() {
                mime::JAVASCRIPT | mime::JSON => "inline; ",
                name if name == "wasm" => "inline; ",
                _ => "attachment; ",
            },
            _ => "attachment; ",
        };

        disposition.push_str(disposition_type);

        //        if !file_name.is_ascii() {
        //        } else {
        disposition.push_str("filename=\"");
        disposition.push_str(file_name.as_ref());
        disposition.push('\"');
        //        }

        response_builder = response_builder.header(header::CONTENT_DISPOSITION, disposition);
    } else {
        debug!("Ignore content disposition");
    };

    response_builder
        .body(body)
        .map_err(|err| LsError::InternalServerError { message: format!("Cannot set body request. Err: {err:?}") })
}

#[cfg(test)]
mod test {
    use super::*;
    use axum::http::{self, Request, StatusCode};
    use axum::routing::get;
    use axum::{Router, extract::Extension};
    use http_body_util::BodyExt;
    use opendal::{Operator, services};
    use std::sync::Arc;
    use tower::ServiceExt; // for `app.oneshot()`

    async fn download(Extension(data): Extension<Arc<AppData>>) -> Result<Response<Body>, LsError> {
        println!("Download called");
        into_response(data.content.clone(), data.file_name, data.set_content_disposition).await
    }

    pub struct AppData {
        content: BinaryContent<'static>,
        file_name: Option<&'static str>,
        set_content_disposition: bool,
    }

    #[tokio::test]
    async fn should_download_bytes_with_no_content_disposition() {
        // Arrange
        let content = std::fs::read("./Cargo.toml").unwrap();
        let data = Arc::new(AppData {
            content: BinaryContent::InMemory { content: content.clone().into() },
            file_name: Some("Cargo.toml"),
            set_content_disposition: false,
        });

        let app = Router::new().route("/download", get(download)).layer(Extension(data.clone()));

        // Act
        let resp = app
            .oneshot(Request::builder().method(http::Method::GET).uri("/download").body(Body::empty()).unwrap())
            .await
            .unwrap();

        // Assert
        assert_eq!(resp.status(), StatusCode::OK);
        assert!(resp.headers().get(http::header::CONTENT_DISPOSITION).is_none());

        let body = resp.into_body().collect().await.unwrap().to_bytes();
        assert_eq!(body.as_ref(), &content);
    }

    #[tokio::test]
    async fn should_download_bytes_with_content_disposition() {
        // Arrange
        let content = std::fs::read("./Cargo.toml").unwrap();
        let data = Arc::new(AppData {
            content: BinaryContent::InMemory { content: content.clone().into() },
            file_name: Some("Cargo.toml"),
            set_content_disposition: true,
        });

        let app = Router::new().route("/download", get(download)).layer(Extension(data.clone()));

        // Act
        let resp = app
            .oneshot(Request::builder().method(http::Method::GET).uri("/download").body(Body::empty()).unwrap())
            .await
            .unwrap();

        // Assert
        assert_eq!(resp.status(), StatusCode::OK);
        assert!(resp.headers().get(http::header::CONTENT_DISPOSITION).is_some());

        assert_eq!(
            http::header::HeaderValue::from_str("inline; filename=\"Cargo.toml\"").unwrap(),
            resp.headers().get(http::header::CONTENT_DISPOSITION).unwrap()
        );

        let body = resp.into_body().collect().await.unwrap().to_bytes();
        assert_eq!(body.as_ref(), &content);
    }

    #[tokio::test]
    async fn should_download_file_with_no_content_disposition() {
        // Arrange
        let file_path = "./Cargo.toml";
        let content = std::fs::read(file_path).unwrap();

        let operator = Operator::new(services::Fs::default().root("./")).unwrap().finish().into();

        let data = Arc::new(AppData {
            content: BinaryContent::OpenDal { operator, path: file_path.to_owned() },
            file_name: Some("Cargo.toml"),
            set_content_disposition: false,
        });

        let app = Router::new().route("/download", get(download)).layer(Extension(data.clone()));

        // Act
        let resp = app
            .oneshot(Request::builder().method(http::Method::GET).uri("/download").body(Body::empty()).unwrap())
            .await
            .unwrap();

        // Assert
        assert_eq!(resp.status(), StatusCode::OK);
        assert!(resp.headers().get(http::header::CONTENT_DISPOSITION).is_none());

        let body = resp.into_body().collect().await.unwrap().to_bytes();
        assert_eq!(body.as_ref(), &content);
    }

    #[tokio::test]
    async fn should_download_file_with_content_disposition() {
        // Arrange
        let file_path = "./Cargo.toml";
        let content = std::fs::read(file_path).unwrap();

        let operator = Operator::new(services::Fs::default().root("./")).unwrap().finish().into();

        let data = Arc::new(AppData {
            content: BinaryContent::OpenDal { operator, path: file_path.to_owned() },
            file_name: None,
            set_content_disposition: true,
        });

        let app = Router::new().route("/download", get(download)).layer(Extension(data.clone()));

        // Act
        let resp = app
            .oneshot(Request::builder().method(http::Method::GET).uri("/download").body(Body::empty()).unwrap())
            .await
            .unwrap();

        // Assert
        assert_eq!(resp.status(), StatusCode::OK);
        assert!(resp.headers().get(http::header::CONTENT_DISPOSITION).is_some());

        assert_eq!(
            http::header::HeaderValue::from_str("inline; filename=\"Cargo.toml\"").unwrap(),
            resp.headers().get(http::header::CONTENT_DISPOSITION).unwrap()
        );

        let body = resp.into_body().collect().await.unwrap().to_bytes();
        assert_eq!(body.as_ref(), &content);
    }
}
