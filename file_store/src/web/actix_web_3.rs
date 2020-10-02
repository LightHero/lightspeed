use lightspeed_core::error::LightSpeedError;
use actix_files::NamedFile;
use actix_web::{http, HttpRequest, HttpResponse};
use log::*;
use crate::model::BinaryContent;


pub async fn into_response(content: BinaryContent, file_name: Option<&str>, set_content_disposition: bool, req: &HttpRequest) -> actix_web::Result<HttpResponse> {
    match content {
        BinaryContent::FromFs {file_path} => {
            debug!("Create HttpResponse from FS content");
            Ok(NamedFile::open(&file_path)?.disable_content_disposition().into_response(&req)?)
        },
        BinaryContent::InMemory {content} => {
            debug!("Create HttpResponse from Memory content of {} bytes", content.len());
            let path = std::path::Path::new(file_name.unwrap_or(""));
            let ct = mime_guess::from_path(&path).first_or_octet_stream();

            let filename = match path.file_name() {
                Some(name) => name.to_string_lossy(),
                None => {
                    return Err(LightSpeedError::BadRequest{
                        message: "Provided path has no filename".to_owned(),
                        code: ""
                    })?;
                }
            };

            Ok(HttpResponse::Ok()
                .set(http::header::ContentType(ct.clone()))
                .if_true(set_content_disposition, |res| {
                    debug!("Set content disposition");
                    let disposition = match ct.type_() {
                        mime::IMAGE | mime::TEXT | mime::VIDEO => http::header::DispositionType::Inline,
                        _ => http::header::DispositionType::Attachment,
                    };
                    let mut parameters =
                        vec![http::header::DispositionParam::Filename(String::from(filename.as_ref()))];
                    if !filename.is_ascii() {
                        parameters.push(http::header::DispositionParam::FilenameExt(http::header::ExtendedValue {
                            charset: http::header::Charset::Ext(String::from("UTF-8")),
                            language_tag: None,
                            value: filename.into_owned().into_bytes(),
                        }))
                    }
                    let cd = http::header::ContentDisposition {
                        disposition,
                        parameters,
                    };

                    res.header(
                        http::header::CONTENT_DISPOSITION,
                        cd,
                    );
                })
                .body(content))
        }
    }
}
