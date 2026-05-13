use std::borrow::Cow;

use lightspeed_validator::Validable;
use lightspeed_validator::url::UrlError;

#[derive(Validable)]
pub struct Website {
    #[validate(url)]
    pub homepage: String,
    pub untouched: String,
}

#[derive(Validable)]
pub struct CowStringFields {
    #[validate(url)]
    pub homepage: Cow<'static, str>,
}

#[derive(Validable)]
pub struct StaticStrFields {
    #[validate(url)]
    pub homepage: &'static str,
}

fn url_err<E: From<UrlError>>() -> E {
    UrlError.into()
}

#[test]
fn url_accepts_common_absolute_urls() {
    for ok in ["https://example.com", "http://example.com:8080/path?q=1", "mailto:user@example.com"] {
        let v = WebsiteValidable::new(Website { homepage: ok.to_string(), untouched: String::new() });
        assert!(v.validate().is_ok(), "expected `{ok}` to be accepted");
    }
}

#[test]
fn url_rejects_garbage() {
    let v = WebsiteValidable::new(Website { homepage: "not a url".to_string(), untouched: String::new() });
    let returned = match v.validate() {
        Ok(_) => panic!("expected Err"),
        Err(v) => v,
    };
    assert_eq!(returned.homepage.errors(), &[url_err()]);
    assert!(returned.untouched.errors().is_empty());
}

#[test]
fn url_rejects_relative_path() {
    let v = WebsiteValidable::new(Website { homepage: "/relative/path".to_string(), untouched: String::new() });
    let returned = match v.validate() {
        Ok(_) => panic!("expected Err"),
        Err(v) => v,
    };
    assert_eq!(returned.homepage.errors(), &[url_err()]);
}

#[test]
fn url_validator_works_on_cow_str_field() {
    let v = CowStringFieldsValidable::new(CowStringFields { homepage: Cow::Borrowed("https://example.com") });
    assert!(v.validate().is_ok());

    let v = CowStringFieldsValidable::new(CowStringFields { homepage: Cow::Owned("nope".to_string()) });
    let returned = match v.validate() {
        Ok(_) => panic!("expected Err"),
        Err(v) => v,
    };
    assert_eq!(returned.homepage.errors(), &[url_err()]);
}

#[test]
fn url_validator_works_on_static_str_field() {
    let v = StaticStrFieldsValidable::new(StaticStrFields { homepage: "https://example.com" });
    assert!(v.validate().is_ok());

    let v = StaticStrFieldsValidable::new(StaticStrFields { homepage: "example.com" });
    let returned = match v.validate() {
        Ok(_) => panic!("expected Err"),
        Err(v) => v,
    };
    assert_eq!(returned.homepage.errors(), &[url_err()]);
}

#[test]
fn macro_attaches_one_validator_per_url_attribute() {
    let v = WebsiteValidable::new(Website { homepage: "https://example.com".to_string(), untouched: String::new() });
    assert_eq!(v.homepage.validators().len(), 1);
    assert_eq!(v.untouched.validators().len(), 0);
}
