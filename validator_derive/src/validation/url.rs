//! Macro support for `#[validate(url)]`.

use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::Field;

/// Ensures a field annotated with a `url` validator is a string-compatible
/// type. Thin wrapper around [`super::string_field::ensure_string_field`].
pub fn ensure_string_field(field: &Field) -> syn::Result<()> {
    super::string_field::ensure_string_field(field, "url")
}

/// Tokens that reference the program-wide static `UrlValidator` — no
/// per-validator heap allocation.
pub fn url_validator_instance() -> TokenStream2 {
    quote! {
        ::lightspeed_validator::ValidatorRef::Static(
            &::lightspeed_validator::url::URL_VALIDATOR
        )
    }
}
