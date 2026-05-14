//! Macro support for `#[validate(credit_card)]`.

use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::Field;

/// Ensures a field annotated with a `credit_card` validator is a
/// string-compatible type. Thin wrapper around
/// [`super::string_field::ensure_string_field`].
pub fn ensure_string_field(field: &Field) -> syn::Result<()> {
    super::string_field::ensure_string_field(field, "credit_card")
}

/// Tokens that reference the program-wide static `CreditCardValidator` —
/// no per-validator heap allocation.
pub fn credit_card_validator_instance() -> TokenStream2 {
    quote! {
        ::lightspeed_validator::ValidatorRef::Static(
            &::lightspeed_validator::credit_card::CREDIT_CARD_VALIDATOR
        )
    }
}
