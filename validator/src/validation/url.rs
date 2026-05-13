use std::fmt::Display;

use crate::FieldValidator;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UrlError;

impl Display for UrlError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Url")
    }
}

/// Validates that a string-compatible value parses as an absolute URL via
/// the [`url`] crate.
pub struct UrlValidator;

impl<S: AsRef<str>, E: From<UrlError>, Ctx> FieldValidator<S, E, Ctx> for UrlValidator {
    fn validate(&self, value: &S, _context: &Ctx) -> Result<(), E> {
        if ::url::Url::parse(value.as_ref()).is_ok() { Ok(()) } else { Err(UrlError.into()) }
    }
}

#[cfg(test)]
mod test {

    use super::*;
    use crate::ValidationError;

    const OK: Result<(), ValidationError> = Ok(());

    #[test]
    fn accepts_common_absolute_urls() {
        for ok in [
            "https://example.com",
            "http://example.com:8080/path?q=1",
            "http://example.com/path#frag",
            "ftp://files.example.com/dir/",
            "mailto:user@example.com",
            "file:///tmp/foo.txt",
            "https://user:pass@example.com",
        ] {
            assert_eq!(UrlValidator.validate(&ok, &()), OK, "expected `{ok}` to be accepted");
        }
    }

    #[test]
    fn rejects_garbage_and_relative_inputs() {
        for bad in ["", "not a url", "/relative/path", "://missing-scheme", "http://", "example.com"] {
            assert_eq!(
                UrlValidator.validate(&bad, &()),
                Err(ValidationError::Url(UrlError)),
                "expected `{bad}` to be rejected",
            );
        }
    }

    #[test]
    fn validator_works_on_string_and_cow() {
        use std::borrow::Cow;
        let owned: String = "https://example.com".to_string();
        assert_eq!(UrlValidator.validate(&owned, &()), OK);
        let cow: Cow<'static, str> = Cow::Borrowed("https://example.com");
        assert_eq!(UrlValidator.validate(&cow, &()), OK);
    }

    #[test]
    fn url_error_display() {
        assert_eq!(UrlError.to_string(), "Url");
    }
}
