use std::fmt::Display;

use crate::FieldValidator;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreditCardError;

impl Display for CreditCardError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "CreditCard")
    }
}

/// Validates that a string-compatible value is a recognized credit card
/// number. Delegates to the [`card_validate`](https://docs.rs/card-validate)
/// crate, which runs Luhn, brand-specific length checks, and IIN range
/// matching against the major issuers.
pub struct CreditCardValidator;

impl<S: AsRef<str>, E: From<CreditCardError>, Ctx> FieldValidator<S, E, Ctx> for CreditCardValidator {
    fn validate(&self, value: &S, _context: &Ctx) -> Result<(), E> {
        let raw = value.as_ref();
        // `card_validate` doesn't normalize input, so strip the common
        // grouping characters before delegating.
        let cleaned: String = raw.chars().filter(|c| *c != ' ' && *c != '-').collect();
        if ::card_validate::Validate::from(&cleaned).is_ok() { Ok(()) } else { Err(CreditCardError.into()) }
    }
}

#[cfg(test)]
mod test {

    use super::*;
    use crate::ValidationError;

    const OK: Result<(), ValidationError> = Ok(());

    #[test]
    fn accepts_well_known_test_card_numbers() {
        for ok in [
            "4532015112830366", // Visa
            "5425233430109903", // MasterCard
            "374245455400126",  // Amex (15 digits — brand-specific length)
            "6011000990139424", // Discover
            "30569309025904",   // Diners Club (14 digits)
            "3530111333300000", // JCB
        ] {
            assert_eq!(CreditCardValidator.validate(&ok, &()), OK, "expected `{ok}` to be accepted",);
        }
    }

    #[test]
    fn accepts_numbers_with_spaces_and_dashes() {
        for ok in ["4532 0151 1283 0366", "4532-0151-1283-0366", "4532 0151-1283 0366"] {
            assert_eq!(CreditCardValidator.validate(&ok, &()), OK, "expected `{ok}` to be accepted",);
        }
    }

    #[test]
    fn rejects_luhn_invalid_and_unknown_brand() {
        let expected = Err(ValidationError::CreditCard(CreditCardError));
        for bad in [
            "",
            "1234567890123456", // Luhn-invalid
            "abcd1234efgh5678", // non-digits
            "9999999999999991", // Luhn-valid but no known brand prefix → rejected by card-validate
        ] {
            assert_eq!(CreditCardValidator.validate(&bad, &()), expected, "expected `{bad}` to be rejected",);
        }
    }

    #[test]
    fn rejects_brand_specific_wrong_length() {
        // 16 digits starting with `34` looks like Amex but Amex is 15 digits.
        // Inline-Luhn alone would accept this; card-validate rejects on brand length.
        let expected = Err(ValidationError::CreditCard(CreditCardError));
        // `3700000000000002` is a Luhn-valid 16-digit number starting with `37` (Amex prefix).
        assert_eq!(CreditCardValidator.validate(&"3700000000000002", &()), expected);
    }

    #[test]
    fn validator_works_on_string_and_cow() {
        use std::borrow::Cow;
        let owned: String = "4532015112830366".to_string();
        assert_eq!(CreditCardValidator.validate(&owned, &()), OK);
        let cow: Cow<'static, str> = Cow::Borrowed("4532015112830366");
        assert_eq!(CreditCardValidator.validate(&cow, &()), OK);
    }

    #[test]
    fn credit_card_error_display() {
        assert_eq!(CreditCardError.to_string(), "CreditCard");
    }
}
