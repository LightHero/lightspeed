use crate::model::BinaryContent;
use actix_files::NamedFile;
use actix_web::{http, HttpRequest, HttpResponse};
use lightspeed_core::error::LightSpeedError;
use log::*;

pub async fn into_response(
    content: BinaryContent<'_>,
    file_name: Option<&str>,
    set_content_disposition: bool,
    req: &HttpRequest,
) -> actix_web::Result<HttpResponse> {
    match content {
        BinaryContent::FromFs { file_path } => {
            debug!("Create HttpResponse from FS content");
            let mut named_file = NamedFile::open(&file_path)?;

            if !set_content_disposition {
                debug!("Ignore content disposition");
                named_file = named_file.disable_content_disposition();
            }

            Ok(named_file.into_response(&req))
        }
        BinaryContent::InMemory { content } => {
            debug!("Create HttpResponse from Memory content of {} bytes", content.len());
            let path = std::path::Path::new(file_name.unwrap_or(""));
            let ct = mime_guess::from_path(&path).first_or_octet_stream();

            let filename = match path.file_name() {
                Some(name) => name.to_string_lossy(),
                None => {
                    return Err(LightSpeedError::BadRequest {
                        message: "Provided path has no filename".to_owned(),
                        code: "",
                    })?;
                }
            };

            let mut res = HttpResponse::Ok();

            if set_content_disposition {
                debug!("Set content disposition");
                let disposition = match ct.type_() {
                    mime::IMAGE | mime::TEXT | mime::VIDEO => http::header::DispositionType::Inline,
                    _ => http::header::DispositionType::Attachment,
                };
                let mut parameters = vec![http::header::DispositionParam::Filename(String::from(filename.as_ref()))];
                if !filename.is_ascii() {
                    parameters.push(http::header::DispositionParam::FilenameExt(http::header::ExtendedValue {
                        charset: http::header::Charset::Ext(String::from("UTF-8")),
                        language_tag: None,
                        value: filename.into_owned().into_bytes(),
                    }))
                }
                let cd = http::header::ContentDisposition { disposition, parameters };

                res.append_header((http::header::CONTENT_DISPOSITION, cd));
            } else {
                debug!("Ignore content disposition");
            };

            res.content_type(http::header::ContentType(ct));

            Ok(res.body(content.into_owned()))
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use actix_web::dev::Service;
    use actix_web::test::{init_service, read_body, TestRequest};
    use actix_web::web::Data;
    use actix_web::{http::StatusCode, web, App};
    use std::path::Path;

    async fn download(req: HttpRequest, data: Data<AppData>) -> actix_web::Result<HttpResponse> {
        println!("Download called");
        into_response(data.content.clone(), data.file_name.clone(), data.set_content_disposition, &req).await
    }

    #[derive(Clone)]
    pub struct AppData {
        content: BinaryContent<'static>,
        file_name: Option<&'static str>,
        set_content_disposition: bool,
    }

    #[actix_web::rt::test]
    async fn should_download_bytes_with_no_content_disposition() {
        // Arrange
        let content = std::fs::read("./Cargo.toml").unwrap();
        let data = AppData {
            content: BinaryContent::InMemory { content: content.clone().into() },
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

    #[actix_web::rt::test]
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

    #[actix_web::rt::test]
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

    #[actix_web::rt::test]
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
}
