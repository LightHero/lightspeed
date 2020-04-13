use crate::dto::FileData;
use actix_files::NamedFile;
use actix_web_external::dev::HttpResponseBuilder;
use actix_web_external::{http, HttpResponse, ResponseError, Responder,HttpRequest};
use lightspeed_core::error::LightSpeedError;
use futures::TryFutureExt;
use futures::future::{ready, Ready};
use log::*;

impl Responder for FileData {
    type Error = actix_web_external::Error;
    type Future = Ready<Result<HttpResponse, Self::Error>>;

    fn respond_to(self, req: &HttpRequest) -> Self::Future {
        match self {
            FileData::FromFs {file_path} => {
                match NamedFile::open(&file_path) {
                    Ok(named_file) => named_file.respond_to(req),
                    Err(err) => {
                        error!("Cannot open NamedFile. file_path: [{}]. Err: {}", file_path, err);
                        ready(Ok(HttpResponse::build(http::StatusCode::NOT_FOUND ).finish()))
                    }
                }
            },
            FileData::InMemory {content} => {
                let mut resp = HttpResponse::build(http::StatusCode::OK );
                ready(Ok(resp.body(content)))
            }
        }
    }
}
