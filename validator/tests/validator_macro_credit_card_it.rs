#![cfg(feature = "credit_card")]

use std::borrow::Cow;

use lightspeed_validator::credit_card::CreditCardError;
use lightspeed_validator::Validable;

fn cc_err<E: From<CreditCardError>>() -> E {
    CreditCardError.into()
}

#[derive(Validable)]
pub struct Payment {
    #[validate(credit_card)]
    pub card_number: String,
    pub untouched: String,
}

#[derive(Validable)]
pub struct CowStringFields {
    #[validate(credit_card)]
    pub card_number: Cow<'static, str>,
}

#[derive(Validable)]
pub struct StaticStrFields {
    #[validate(credit_card)]
    pub card_number: &'static str,
}

#[test]
fn credit_card_accepts_well_known_test_numbers() {
    for ok in ["4532015112830366", "5425233430109903", "374245455400126"] {
        let v = PaymentValidable::new(Payment {
            card_number: ok.to_string(),
            untouched: String::new(),
        });
        assert!(v.validate().is_ok(), "expected `{ok}` to be accepted");
    }
}

#[test]
fn credit_card_accepts_grouped_numbers_with_spaces_and_dashes() {
    let v = PaymentValidable::new(Payment {
        card_number: "4532 0151 1283 0366".to_string(),
        untouched: String::new(),
    });
    assert!(v.validate().is_ok());

    let v = PaymentValidable::new(Payment {
        card_number: "4532-0151-1283-0366".to_string(),
        untouched: String::new(),
    });
    assert!(v.validate().is_ok());
}

#[test]
fn credit_card_rejects_luhn_invalid_number() {
    let v = PaymentValidable::new(Payment {
        card_number: "1234567890123456".to_string(),
        untouched: String::new(),
    });
    let returned = match v.validate() {
        Ok(_) => panic!("expected Err"),
        Err(v) => v,
    };
    assert_eq!(returned.card_number.errors(), &[cc_err()]);
    assert!(returned.untouched.errors().is_empty());
}

#[test]
fn credit_card_rejects_non_digit_characters() {
    let v = PaymentValidable::new(Payment {
        card_number: "4532abcd1283ZZZZ".to_string(),
        untouched: String::new(),
    });
    let returned = match v.validate() {
        Ok(_) => panic!("expected Err"),
        Err(v) => v,
    };
    assert_eq!(returned.card_number.errors(), &[cc_err()]);
}

#[test]
fn credit_card_rejects_too_short_or_too_long() {
    let expected = vec![cc_err()];
    for bad in ["000000000000", "45320151128303661234"] {
        let v = PaymentValidable::new(Payment {
            card_number: bad.to_string(),
            untouched: String::new(),
        });
        let returned = match v.validate() {
            Ok(_) => panic!("expected Err for `{bad}`"),
            Err(v) => v,
        };
        assert_eq!(returned.card_number.errors(), expected.as_slice());
    }
}

#[test]
fn credit_card_validator_works_on_cow_str_field() {
    let v = CowStringFieldsValidable::new(CowStringFields {
        card_number: Cow::Borrowed("4532015112830366"),
    });
    assert!(v.validate().is_ok());

    let v = CowStringFieldsValidable::new(CowStringFields {
        card_number: Cow::Owned("nope".to_string()),
    });
    let returned = match v.validate() {
        Ok(_) => panic!("expected Err"),
        Err(v) => v,
    };
    assert_eq!(returned.card_number.errors(), &[cc_err()]);
}

#[test]
fn credit_card_validator_works_on_static_str_field() {
    let v = StaticStrFieldsValidable::new(StaticStrFields { card_number: "4532015112830366" });
    assert!(v.validate().is_ok());

    let v = StaticStrFieldsValidable::new(StaticStrFields { card_number: "0000" });
    let returned = match v.validate() {
        Ok(_) => panic!("expected Err"),
        Err(v) => v,
    };
    assert_eq!(returned.card_number.errors(), &[cc_err()]);
}

#[test]
fn macro_attaches_one_validator_per_credit_card_attribute() {
    let v = PaymentValidable::new(Payment {
        card_number: "4532015112830366".to_string(),
        untouched: String::new(),
    });
    assert_eq!(v.card_number.validators().len(), 1);
    assert_eq!(v.untouched.validators().len(), 0);
}
