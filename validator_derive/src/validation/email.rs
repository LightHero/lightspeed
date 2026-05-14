//! Macro support for `#[validate(email)]`.

use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::Field;

/// Ensures a field annotated with an `email` validator is a string-compatible
/// type. Thin wrapper around [`super::string_field::ensure_string_field`].
pub fn ensure_string_field(field: &Field) -> syn::Result<()> {
    super::string_field::ensure_string_field(field, "email")
}

/// Tokens that construct a `Box<dyn FieldValidator<...>>` for `EmailValidator`.
pub fn email_validator_instance() -> TokenStream2 {
    quote! {
        ::std::boxed::Box::new(::lightspeed_validator::email::EmailValidator)
    }
}
