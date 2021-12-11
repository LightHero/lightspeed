
use crate::model::BinaryContent;
use axum_ext::{
    body::{boxed, BoxBody, Body, StreamBody},
    http::{header, Response, response::Builder},
};
use lightspeed_core::error::LightSpeedError;
use log::*;
use tokio_util::io::ReaderStream;
use std::borrow::Cow;

pub async fn into_response(
    content: BinaryContent<'_>,
    file_name: Option<&str>,
    set_content_disposition: bool,
) -> Result<Response<BoxBody>, LightSpeedError> {

    let (file_name, ct, body) = match content {
        BinaryContent::FromFs { file_path } => {
            debug!("Create HttpResponse from FS content");
            let ct = mime_guess::from_path(&file_path).first_or_octet_stream();
            let file = tokio::fs::File::open(&file_path).await
                .map_err(|err| LightSpeedError::BadRequest {
                    message: format!("Cannot open file {}. Err {:?}", file_path.display(), err),
                    code: "",
                })?;
            // convert the `AsyncRead` into a `Stream`
            let stream = ReaderStream::new(file);
            // convert the `Stream` into an `axum::body::HttpBody`
            let body = StreamBody::new(stream);

            let file_name = if let Some(file_name) = file_name {
                Cow::Borrowed(file_name)
            } else {
                match file_path.file_name().to_owned() {
                    Some(name) => Cow::Owned(name.to_string_lossy().as_ref().to_owned()),
                    None => {
                        return Err(LightSpeedError::BadRequest {
                            message: "Provided path has no filename".to_owned(),
                            code: "",
                        })?;
                    }
                }
            };

            (file_name, ct, boxed(body))
        }
        BinaryContent::InMemory { content } => {
            debug!("Create HttpResponse from Memory content of {} bytes", content.len());
            let file_name = Cow::Borrowed(file_name.unwrap_or(""));
            let path = std::path::Path::new(file_name.as_ref());
            let ct = mime_guess::from_path(&path).first_or_octet_stream();
            let owned_vec: Vec<u8> = content.to_owned().into();
            (file_name, ct, boxed(Body::from(owned_vec)))
        }
    };

    let mut response_builder = Builder::new();

    response_builder = response_builder.header(header::CONTENT_TYPE, format!("{}; charset=utf-8", ct));

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
            disposition.push_str("\"");
//        }

        response_builder = response_builder.header(header::CONTENT_DISPOSITION, disposition);
    } else {
        debug!("Ignore content disposition");
    };

    response_builder.body(body)
        .map_err(|err| LightSpeedError::InternalServerError {
            message: format!("Cannot set body request. Err: {:?}", err),
        })

}

#[cfg(test)]
mod test {
    use super::*;
    use axum_ext::{Router, AddExtensionLayer};
    use axum_ext::extract::Extension;
    use axum_ext::http::{self, Request, StatusCode};
    use axum_ext::routing::get;
    use std::path::Path;
    use std::sync::Arc;
    use tower::ServiceExt; // for `app.oneshot()`

    async fn download(Extension(data): Extension<Arc<AppData>>) -> Result<Response<BoxBody>, LightSpeedError> {
        println!("Download called");
        into_response(data.content.clone(), data.file_name.clone(), data.set_content_disposition).await
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

        let app = Router::new()
            .route("/download", get(download))
            .layer(AddExtensionLayer::new(data.clone()));

        // Act
        let resp = app
            .oneshot(Request::builder().method(http::Method::GET).uri("/download").body(Body::empty()).unwrap())
            .await
            .unwrap();

        // Assert
        assert_eq!(resp.status(), StatusCode::OK);
        assert!(resp.headers().get(http::header::CONTENT_DISPOSITION).is_none());

        let body = hyper::body::to_bytes(resp.into_body()).await.unwrap();
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

        let app = Router::new()
            .route("/download", get(download))
            .layer(AddExtensionLayer::new(data.clone()));

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

        let body = hyper::body::to_bytes(resp.into_body()).await.unwrap();
        assert_eq!(body.as_ref(), &content);
    }

    #[tokio::test]
    async fn should_download_file_with_no_content_disposition() {
        // Arrange
        let file_path = Path::new("./Cargo.toml").to_owned();
        let content = std::fs::read(&file_path).unwrap();
        let data = Arc::new(AppData {
            content: BinaryContent::FromFs { file_path: file_path.clone() },
            file_name: Some("Cargo.toml"),
            set_content_disposition: false,
        });

        let app = Router::new()
            .route("/download", get(download))
            .layer(AddExtensionLayer::new(data.clone()));

        // Act
        let resp = app
            .oneshot(Request::builder().method(http::Method::GET).uri("/download").body(Body::empty()).unwrap())
            .await
            .unwrap();

        // Assert
        assert_eq!(resp.status(), StatusCode::OK);
        assert!(resp.headers().get(http::header::CONTENT_DISPOSITION).is_none());

        let body = hyper::body::to_bytes(resp.into_body()).await.unwrap();
        assert_eq!(body.as_ref(), &content);
    }

    #[tokio::test]
    async fn should_download_file_with_content_disposition() {
        // Arrange
        let file_path = Path::new("./Cargo.toml").to_owned();
        let content = std::fs::read(&file_path).unwrap();
        let data = Arc::new(AppData {
            content: BinaryContent::FromFs { file_path: file_path.clone() },
            file_name: None,
            set_content_disposition: true,
        });

        let app = Router::new()
            .route("/download", get(download))
            .layer(AddExtensionLayer::new(data.clone()));

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

        let body = hyper::body::to_bytes(resp.into_body()).await.unwrap();
        assert_eq!(body.as_ref(), &content);
    }

}
