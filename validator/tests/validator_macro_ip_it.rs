use std::borrow::Cow;

use lightspeed_validator::ip::{IpError, IpKind};
use lightspeed_validator::Validable;

#[derive(Validable)]
pub struct AnyIpHost {
    #[validate(ip)]
    pub address: String,
    pub untouched: String,
}

#[derive(Validable)]
pub struct V4Only {
    #[validate(ipv4)]
    pub address: String,
}

#[derive(Validable)]
pub struct V6Only {
    #[validate(ipv6)]
    pub address: String,
}

#[derive(Validable)]
pub struct CowStringFields {
    #[validate(ip)]
    pub address: Cow<'static, str>,
}

#[derive(Validable)]
pub struct StaticStrFields {
    #[validate(ipv4)]
    pub address: &'static str,
}

fn ip_err<E: From<IpError>>(kind: IpKind) -> E {
    IpError { kind }.into()
}

#[test]
fn ip_accepts_v4_and_v6() {
    let v = AnyIpHostValidable::new(AnyIpHost {
        address: "1.1.1.1".to_string(),
        untouched: String::new(),
    });
    assert!(v.validate().is_ok());

    let v = AnyIpHostValidable::new(AnyIpHost {
        address: "fe80::1".to_string(),
        untouched: String::new(),
    });
    assert!(v.validate().is_ok());
}

#[test]
fn ip_rejects_garbage() {
    let v = AnyIpHostValidable::new(AnyIpHost {
        address: "not-an-ip".to_string(),
        untouched: String::new(),
    });
    let returned = match v.validate() {
        Ok(_) => panic!("expected Err"),
        Err(v) => v,
    };
    assert_eq!(returned.address.errors(), &[ip_err(IpKind::Any)]);
    assert!(returned.untouched.errors().is_empty());
}

#[test]
fn ipv4_accepts_v4_only() {
    let v = V4OnlyValidable::new(V4Only { address: "192.168.0.1".to_string() });
    assert!(v.validate().is_ok());

    let v = V4OnlyValidable::new(V4Only { address: "fe80::1".to_string() });
    let returned = match v.validate() {
        Ok(_) => panic!("expected Err"),
        Err(v) => v,
    };
    assert_eq!(returned.address.errors(), &[ip_err(IpKind::V4)]);
}

#[test]
fn ipv6_accepts_v6_only() {
    let v = V6OnlyValidable::new(V6Only { address: "::1".to_string() });
    assert!(v.validate().is_ok());

    let v = V6OnlyValidable::new(V6Only { address: "127.0.0.1".to_string() });
    let returned = match v.validate() {
        Ok(_) => panic!("expected Err"),
        Err(v) => v,
    };
    assert_eq!(returned.address.errors(), &[ip_err(IpKind::V6)]);
}

#[test]
fn ip_validator_works_on_cow_str_field() {
    let v = CowStringFieldsValidable::new(CowStringFields {
        address: Cow::Borrowed("10.0.0.1"),
    });
    assert!(v.validate().is_ok());

    let v = CowStringFieldsValidable::new(CowStringFields {
        address: Cow::Owned("nope".to_string()),
    });
    let returned = match v.validate() {
        Ok(_) => panic!("expected Err"),
        Err(v) => v,
    };
    assert_eq!(returned.address.errors(), &[ip_err(IpKind::Any)]);
}

#[test]
fn ipv4_validator_works_on_static_str_field() {
    let v = StaticStrFieldsValidable::new(StaticStrFields { address: "1.2.3.4" });
    assert!(v.validate().is_ok());

    let v = StaticStrFieldsValidable::new(StaticStrFields { address: "::1" });
    let returned = match v.validate() {
        Ok(_) => panic!("expected Err"),
        Err(v) => v,
    };
    assert_eq!(returned.address.errors(), &[ip_err(IpKind::V4)]);
}

#[test]
fn macro_attaches_one_validator_per_ip_attribute() {
    let v = AnyIpHostValidable::new(AnyIpHost {
        address: "1.1.1.1".to_string(),
        untouched: String::new(),
    });
    assert_eq!(v.address.validators().len(), 1);
    assert_eq!(v.untouched.validators().len(), 0);
}
