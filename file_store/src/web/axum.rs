
use axum_ext::{
    body::StreamBody,
    http::{header, StatusCode},
    response::{Headers, IntoResponse},
    routing::get,
    Router,
};
use log::*;
use std::net::SocketAddr;
use tokio_util::io::ReaderStream;
use crate::model::BinaryContent;
use lightspeed_core::error::LightSpeedError;
use axum_ext::body::{boxed, BoxBody, Body};
use axum_ext::http::{Response, HeaderValue};
use axum_ext::http::response::Builder;

pub async fn into_response(
    content: BinaryContent<'_>,
    file_name: Option<&str>,
    set_content_disposition: bool,
) -> Result<Response<BoxBody>, LightSpeedError> {

    let (ct, body) = match content {
        BinaryContent::FromFs { file_path } => {
            debug!("Create HttpResponse from FS content");
            let ct = mime_guess::from_path(&file_path).first_or_octet_stream();
            let file = tokio::fs::File::open(&file_path).await
                .map_err(|err| LightSpeedError::BadRequest {
                    message: format!("Cannot open file {}", file_path.display()),
                    code: "",
                })?;
            // convert the `AsyncRead` into a `Stream`
            let stream = ReaderStream::new(file);
            // convert the `Stream` into an `axum::body::HttpBody`
            let body = StreamBody::new(stream);
            (ct, boxed(body))
        }
        BinaryContent::InMemory { content } => {
            debug!("Create HttpResponse from Memory content of {} bytes", content.len());
            let file_name= file_name.unwrap_or("");
            let ct = mime_guess::from_ext(&file_name).first_or_octet_stream();

            let owned_vec: Vec<u8> = content.to_owned().into();
            (ct, boxed(Body::from(owned_vec)))
        }
    };

    println!("ct is: {}", ct);

    let headers = Headers([
        (header::CONTENT_TYPE, "text/toml; charset=utf-8"),
        (
            header::CONTENT_DISPOSITION,
            "attachment; filename=\"Cargo.toml\"",
        ),
    ]);

    let mut response_builder = Builder::new();

    response_builder = response_builder.header(header::CONTENT_TYPE, "text/toml; charset=utf-8");

    // if set_content_disposition {
    //     debug!("Set content disposition");
    //     let disposition = match ct.type_() {
    //         mime::IMAGE | mime::TEXT | mime::VIDEO => http::header::DispositionType::Inline,
    //         _ => http::header::DispositionType::Attachment,
    //     };
    //     let mut parameters = vec![http::header::DispositionParam::Filename(String::from(filename.as_ref()))];
    //     if !filename.is_ascii() {
    //         parameters.push(http::header::DispositionParam::FilenameExt(http::header::ExtendedValue {
    //             charset: http::header::Charset::Ext(String::from("UTF-8")),
    //             language_tag: None,
    //             value: filename.into_owned().into_bytes(),
    //         }))
    //     }
    //     let cd = http::header::ContentDisposition { disposition, parameters };
    //
    //     res.append_header((http::header::CONTENT_DISPOSITION, cd));
    // } else {
    //     debug!("Ignore content disposition");
    // };

    response_builder.body(body)
        .map_err(|err| LightSpeedError::InternalServerError {
            message: "Cannot set body request".to_owned(),
        })

}

#[cfg(test)]
mod test {
    use super::*;
    use axum_ext::http::{self, header, HeaderMap, Request};
    use axum_ext::{Router, AddExtensionLayer};
    use axum_ext::routing::get;
    use std::sync::Arc;
    use tower::ServiceExt; // for `app.oneshot()`

    use std::path::Path;
    use axum_ext::extract::Extension;

    async fn download(Extension(data): Extension<Arc<AppData>>) -> Result<Response<BoxBody>, LightSpeedError> {
        println!("Download called");
        into_response(data.content.clone(), data.file_name.clone(), data.set_content_disposition).await
    }

    #[derive(Clone)]
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

    /*
    #[tokio::test]
    async fn should_download_bytes_with_content_disposition() {
        // Arrange
        let content = std::fs::read("./Cargo.toml").unwrap();
        let data = AppData {
            content: BinaryContent::InMemory { content: content.clone().into() },
            file_name: Some("Cargo.toml"),
            set_content_disposition: true,
        };

        let srv =
            init_service(App::new().app_data(Data::new(data.clone())).service(web::resource("/download").to(download)))
                .await;

        let request = TestRequest::get().uri("/download").to_request();

        // Act
        let resp = srv.call(request).await.unwrap();

        // Assert
        assert_eq!(resp.status(), StatusCode::OK);
        assert!(resp.headers().get(http::header::CONTENT_DISPOSITION).is_some());

        assert_eq!(
            http::header::HeaderValue::from_str("inline; filename=\"Cargo.toml\"").unwrap(),
            resp.headers().get(http::header::CONTENT_DISPOSITION).unwrap()
        );

        let body = read_body(resp).await;
        assert_eq!(body, &content);
    }

    #[tokio::test]
    async fn should_download_file_with_no_content_disposition() {
        // Arrange
        let file_path = Path::new("./Cargo.toml").to_owned();
        let content = std::fs::read(&file_path).unwrap();
        let data = AppData {
            content: BinaryContent::FromFs { file_path: file_path.clone() },
            file_name: Some("Cargo.toml"),
            set_content_disposition: false,
        };

        let srv =
            init_service(App::new().app_data(Data::new(data.clone())).service(web::resource("/download").to(download)))
                .await;

        let request = TestRequest::get().uri("/download").to_request();

        // Act
        let resp = srv.call(request).await.unwrap();

        // Assert
        assert_eq!(resp.status(), StatusCode::OK);
        assert!(resp.headers().get(http::header::CONTENT_DISPOSITION).is_none());

        let body = read_body(resp).await;
        assert_eq!(body, &content);
    }

    #[tokio::test]
    async fn should_download_file_with_content_disposition() {
        // Arrange
        let file_path = Path::new("./Cargo.toml").to_owned();
        let content = std::fs::read(&file_path).unwrap();
        let data = AppData {
            content: BinaryContent::FromFs { file_path: file_path.clone() },
            file_name: None,
            set_content_disposition: true,
        };

        let srv =
            init_service(App::new().app_data(Data::new(data.clone())).service(web::resource("/download").to(download)))
                .await;

        let request = TestRequest::get().uri("/download").to_request();

        // Act
        let resp = srv.call(request).await.unwrap();

        // Assert
        assert_eq!(resp.status(), StatusCode::OK);
        assert!(resp.headers().get(http::header::CONTENT_DISPOSITION).is_some());

        assert_eq!(
            http::header::HeaderValue::from_str("inline; filename=\"Cargo.toml\"").unwrap(),
            resp.headers().get(http::header::CONTENT_DISPOSITION).unwrap()
        );

        let body = read_body(resp).await;
        assert_eq!(body, &content);
    }

     */
}
