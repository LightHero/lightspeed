use crate::model::BinaryContent;
use axum::{
    body::Body,
    http::{Response, header, response::Builder},
};
use futures::StreamExt;
use lightspeed_core::error::LsError;
use log::*;
use percent_encoding::{AsciiSet, CONTROLS, utf8_percent_encode};
use std::borrow::Cow;

/// `attr-char` per RFC 5987 §3.2.1: any byte that is NOT in this set must
/// be percent-encoded when used in `filename*=UTF-8''…`. The complement
/// of `attr-char` is "everything that isn't ALPHA / DIGIT / one of the
/// allowed punctuation characters", which we express by starting from
/// CONTROLS and adding every other byte that isn't permitted.
const RFC5987_ATTR_CHAR_DISALLOWED: &AsciiSet = &CONTROLS
    // Whitespace and quote-style characters.
    .add(b' ')
    .add(b'"')
    .add(b'\'')
    .add(b'(')
    .add(b')')
    .add(b',')
    .add(b'/')
    .add(b':')
    .add(b';')
    .add(b'<')
    .add(b'=')
    .add(b'>')
    .add(b'?')
    .add(b'@')
    .add(b'[')
    .add(b'\\')
    .add(b']')
    .add(b'{')
    .add(b'}')
    .add(b'%')
    // High-bit bytes (UTF-8 continuation/leading bytes).
    .add(0x7F);

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

            let reader = operator.reader(&path).await.map_err(|err| LsError::InternalServerError {
                message: format!("Cannot open reader for [{path}]: {err:?}"),
            })?;
            let stream = reader.into_bytes_stream(..).await.map_err(|err| LsError::InternalServerError {
                message: format!("Cannot open byte stream for [{path}]: {err:?}"),
            })?;

            (file_name, ct, Body::from_stream(stream))
        }
        BinaryContent::Stream { stream } => {
            // The caller-provided file_name is authoritative for streamed
            // content (the stream itself has no path metadata).
            let file_name = Cow::Borrowed(file_name.unwrap_or(""));
            let ct = mime_guess::from_path(std::path::Path::new(file_name.as_ref())).first_or_octet_stream();

            // Adapt our `Result<Vec<u8>, LsError>` stream into the byte
            // stream shape `Body::from_stream` expects.
            let bytes_stream =
                stream.into_inner().map(|chunk| chunk.map_err(|err| std::io::Error::other(err.to_string())));
            (file_name, ct, Body::from_stream(bytes_stream))
        }
    };

    let mut response_builder = Builder::new();

    response_builder = response_builder.header(header::CONTENT_TYPE, format!("{ct}; charset=utf-8"));

    if set_content_disposition {
        debug!("Set content disposition");

        let disposition_type = match ct.type_() {
            mime::IMAGE | mime::TEXT | mime::VIDEO => "inline",
            mime::APPLICATION => match ct.subtype() {
                mime::JAVASCRIPT | mime::JSON => "inline",
                name if name == "wasm" => "inline",
                _ => "attachment",
            },
            _ => "attachment",
        };

        let disposition = build_content_disposition(disposition_type, file_name.as_ref());
        response_builder = response_builder.header(header::CONTENT_DISPOSITION, disposition);
    } else {
        debug!("Ignore content disposition");
    };

    response_builder
        .body(body)
        .map_err(|err| LsError::InternalServerError { message: format!("Cannot set body request. Err: {err:?}") })
}

/// Builds an RFC 6266 / RFC 5987-compliant `Content-Disposition` header value.
///
/// Two safety properties matter here:
///
/// 1. **No header injection.** A user-controlled filename containing `\r`,
///    `\n`, `"`, `\` or other shenanigans must NOT be able to terminate
///    the header value early or splice in additional headers. We strip
///    every byte that isn't safe printable ASCII for the `filename=`
///    fallback — control characters become `_`, embedded quotes/backslashes
///    become `_`, and the result is then surrounded by quotes.
///
/// 2. **Lossless non-ASCII.** RFC 5987 `filename*=UTF-8''<percent-encoded>`
///    carries the original (UTF-8) name losslessly for every modern
///    browser. We emit both parameters so legacy `filename=` clients get
///    a safe fallback, and `filename*=` clients get the real name.
fn build_content_disposition(disposition_type: &str, file_name: &str) -> String {
    let ascii_fallback: String = file_name
        .chars()
        .map(|c| match c {
            // Control chars (incl. CR, LF, NUL), backslash, double-quote
            // and DEL — all unsafe in a quoted-string header value.
            '\\' | '"' => '_',
            c if (c as u32) < 0x20 || c == '\x7f' => '_',
            // Everything outside ASCII gets replaced in the fallback;
            // the real value lives in `filename*=UTF-8''…` below.
            c if !c.is_ascii() => '_',
            c => c,
        })
        .collect();

    let encoded = utf8_percent_encode(file_name, RFC5987_ATTR_CHAR_DISALLOWED);
    format!("{disposition_type}; filename=\"{ascii_fallback}\"; filename*=UTF-8''{encoded}")
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
        // Build a fresh `BinaryContent` per request from the underlying
        // source. `BinaryContent` is intentionally not `Clone` (its `Stream`
        // variant carries a one-shot stream), so the shared `Arc<AppData>`
        // owns the source data and each request constructs its own content.
        let content = match &data.source {
            ContentSource::InMemory(bytes) => BinaryContent::InMemory { content: Cow::Borrowed(bytes) },
            ContentSource::OpenDal { operator, path } => {
                BinaryContent::OpenDal { operator: operator.clone(), path: path.clone() }
            }
        };
        into_response(content, data.file_name, data.set_content_disposition).await
    }

    pub enum ContentSource {
        InMemory(Vec<u8>),
        OpenDal { operator: Arc<opendal::Operator>, path: String },
    }

    pub struct AppData {
        source: ContentSource,
        file_name: Option<&'static str>,
        set_content_disposition: bool,
    }

    #[tokio::test]
    async fn should_download_bytes_with_no_content_disposition() {
        // Arrange
        let content = std::fs::read("./Cargo.toml").unwrap();
        let data = Arc::new(AppData {
            source: ContentSource::InMemory(content.clone()),
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
            source: ContentSource::InMemory(content.clone()),
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
            http::header::HeaderValue::from_str("inline; filename=\"Cargo.toml\"; filename*=UTF-8''Cargo.toml")
                .unwrap(),
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
            source: ContentSource::OpenDal { operator, path: file_path.to_owned() },
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
            source: ContentSource::OpenDal { operator, path: file_path.to_owned() },
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
            http::header::HeaderValue::from_str("inline; filename=\"Cargo.toml\"; filename*=UTF-8''Cargo.toml")
                .unwrap(),
            resp.headers().get(http::header::CONTENT_DISPOSITION).unwrap()
        );

        let body = resp.into_body().collect().await.unwrap().to_bytes();
        assert_eq!(body.as_ref(), &content);
    }

    /// CR/LF/quotes/backslashes in an attacker-controlled filename must be
    /// neutralised so they cannot break out of the `filename="..."` quoted
    /// value or splice an extra header. The actual UTF-8 sequence still
    /// round-trips losslessly via `filename*=UTF-8''…`.
    #[test]
    fn build_content_disposition_should_neutralise_header_injection() {
        let evil = "evil\r\nSet-Cookie: x=y\r\n\"\\.txt";
        let header = super::build_content_disposition("attachment", evil);

        // No CR/LF anywhere in the produced header — `HeaderValue::from_str`
        // would also reject those, so confirming this means we're never even
        // *attempting* to inject.
        assert!(!header.contains('\r'), "header contains CR: {header}");
        assert!(!header.contains('\n'), "header contains LF: {header}");

        // The ASCII fallback escapes every dangerous char to `_`: the input
        // ends with `\r \n " \\ . t x t` → four underscores then `.txt`.
        assert!(header.contains("filename=\"evil__Set-Cookie: x=y____.txt\""), "fallback wrong: {header}");

        // The exact original value is preserved by filename* via percent-encoding.
        assert!(
            header.contains("filename*=UTF-8''evil%0D%0ASet-Cookie%3A%20x%3Dy%0D%0A%22%5C.txt"),
            "encoded wrong: {header}"
        );

        // And the result is a valid HTTP header value (no control bytes).
        http::header::HeaderValue::from_str(&header).expect("valid header value");
    }

    /// Non-ASCII filenames are losslessly carried by RFC 5987 `filename*=`,
    /// while `filename=` falls back to underscores so legacy clients still
    /// see *something* but no garbled bytes.
    #[test]
    fn build_content_disposition_should_handle_non_ascii_filenames() {
        let header = super::build_content_disposition("attachment", "résumé.pdf");

        // ASCII fallback: non-ASCII chars become `_`, ASCII passes through.
        assert!(header.contains("filename=\"r_sum_.pdf\""), "fallback: {header}");
        // RFC 5987: UTF-8 bytes percent-encoded.
        // 'é' = U+00E9 = UTF-8 0xC3 0xA9
        assert!(header.contains("filename*=UTF-8''r%C3%A9sum%C3%A9.pdf"), "encoded: {header}");

        http::header::HeaderValue::from_str(&header).expect("valid header value");
    }

    /// Smoke test for the `unwrap` removal: a non-existent path through the
    /// OpenDal variant must NOT panic. (For the FS backend, `reader()` and
    /// `into_bytes_stream()` succeed lazily — the actual I/O error surfaces
    /// during stream consumption — so this test only fences in the panic
    /// behaviour. Backends that error eagerly are caught by the new
    /// `?`-with-`InternalServerError` mapping in the OpenDal arm.)
    #[tokio::test]
    async fn into_response_should_not_panic_on_opendal_open() {
        let operator: Arc<opendal::Operator> =
            Operator::new(services::Fs::default().root("./")).unwrap().finish().into();
        let content = BinaryContent::OpenDal { operator, path: "this/file/definitely/does/not/exist.bin".to_owned() };

        // Either Ok (lazy backend) or Err — both are acceptable; what we
        // explicitly forbid is a panic crashing the worker thread.
        let _ = into_response(content, Some("missing.bin"), false).await;
    }
}
