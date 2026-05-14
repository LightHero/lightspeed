//! Macro support for `#[validate(ip)]`, `#[validate(ipv4)]` and
//! `#[validate(ipv6)]`.
//!
//! All three keywords map to the same `IpValidator`, distinguished only by
//! the [`IpKind`](::lightspeed_validator::ip::IpKind) it carries.

use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::Field;

/// Ensures a field annotated with an `ip` / `ipv4` / `ipv6` validator is a
/// string-compatible type. Thin wrapper around
/// [`super::string_field::ensure_string_field`].
pub fn ensure_string_field(field: &Field) -> syn::Result<()> {
    super::string_field::ensure_string_field(field, "ip` / `ipv4` / `ipv6")
}

/// Tokens that construct a `Box<dyn FieldValidator<...>>` for `IpValidator`
/// with the configured `IpKind`.
pub fn ip_validator_instance() -> TokenStream2 {
    quote! {
        ::std::boxed::Box::new(::lightspeed_validator::ip::IpValidator {
            kind: ::lightspeed_validator::ip::IpKind::Any,
        })
    }
}

pub fn ipv4_validator_instance() -> TokenStream2 {
    quote! {
        ::std::boxed::Box::new(::lightspeed_validator::ip::IpValidator {
            kind: ::lightspeed_validator::ip::IpKind::V4,
        })
    }
}

pub fn ipv6_validator_instance() -> TokenStream2 {
    quote! {
        ::std::boxed::Box::new(::lightspeed_validator::ip::IpValidator {
            kind: ::lightspeed_validator::ip::IpKind::V6,
        })
    }
}
