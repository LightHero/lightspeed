//! Macro support for `#[validate(url)]`.

use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::Field;

/// Ensures a field annotated with a `url` validator is a string-compatible
/// type. Thin wrapper around [`super::string_field::ensure_string_field`].
pub fn ensure_string_field(field: &Field) -> syn::Result<()> {
    super::string_field::ensure_string_field(field, "url")
}

/// Tokens that construct a `Box<dyn FieldValidator<...>>` for `UrlValidator`.
pub fn url_validator_instance() -> TokenStream2 {
    quote! {
        ::std::boxed::Box::new(::lightspeed_validator::url::UrlValidator)
    }
}
