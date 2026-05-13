use std::borrow::Cow;

use lightspeed_validator::Validable;
use lightspeed_validator::email::EmailError;

fn email_err<E: From<EmailError>>() -> E {
    EmailError.into()
}

#[derive(Validable)]
pub struct Account {
    #[validate(email)]
    pub address: String,
    pub untouched: String,
}

#[derive(Validable)]
pub struct CowStringFields {
    #[validate(email)]
    pub address: Cow<'static, str>,
}

#[derive(Validable)]
pub struct StaticStrFields {
    #[validate(email)]
    pub address: &'static str,
}

#[test]
fn email_accepts_common_well_formed_addresses() {
    for ok in ["user@example.com", "first.last@sub.example.co.uk", "u+tag@example.com", "x@y.z"] {
        let v = AccountValidable::new(Account { address: ok.to_string(), untouched: String::new() });
        assert!(v.validate().is_ok(), "expected `{ok}` accepted");
    }
}

#[test]
fn email_rejects_garbage() {
    let v = AccountValidable::new(Account { address: "not-an-email".to_string(), untouched: String::new() });
    let returned = match v.validate() {
        Ok(_) => panic!("expected Err"),
        Err(v) => v,
    };
    assert_eq!(returned.address.errors(), &[email_err()]);
    assert!(returned.untouched.errors().is_empty());
}

#[test]
fn email_rejects_partial_addresses() {
    for bad in ["", "user@", "@example.com", "user.example.com"] {
        let v = AccountValidable::new(Account { address: bad.to_string(), untouched: String::new() });
        let returned = match v.validate() {
            Ok(_) => panic!("expected Err for `{bad}`"),
            Err(v) => v,
        };
        assert_eq!(returned.address.errors(), &[email_err()]);
    }
}

#[test]
fn email_validator_works_on_cow_str_field() {
    let v = CowStringFieldsValidable::new(CowStringFields { address: Cow::Borrowed("user@example.com") });
    assert!(v.validate().is_ok());

    let v = CowStringFieldsValidable::new(CowStringFields { address: Cow::Owned("nope".to_string()) });
    let returned = match v.validate() {
        Ok(_) => panic!("expected Err"),
        Err(v) => v,
    };
    assert_eq!(returned.address.errors(), &[email_err()]);
}

#[test]
fn email_validator_works_on_static_str_field() {
    let v = StaticStrFieldsValidable::new(StaticStrFields { address: "user@example.com" });
    assert!(v.validate().is_ok());

    let v = StaticStrFieldsValidable::new(StaticStrFields { address: "example.com" });
    let returned = match v.validate() {
        Ok(_) => panic!("expected Err"),
        Err(v) => v,
    };
    assert_eq!(returned.address.errors(), &[email_err()]);
}

#[test]
fn macro_attaches_one_validator_per_email_attribute() {
    let v = AccountValidable::new(Account { address: "user@example.com".to_string(), untouched: String::new() });
    assert_eq!(v.address.validators().len(), 1);
    assert_eq!(v.untouched.validators().len(), 0);
}
