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

/// Tokens that reference the program-wide static `IpValidator` for
/// `IpKind::Any` — no per-validator heap allocation.
pub fn ip_validator_instance() -> TokenStream2 {
    quote! {
        ::lightspeed_validator::ValidatorRef::Static(
            &::lightspeed_validator::ip::IP_ANY_VALIDATOR
        )
    }
}

/// See [`ip_validator_instance`].
pub fn ipv4_validator_instance() -> TokenStream2 {
    quote! {
        ::lightspeed_validator::ValidatorRef::Static(
            &::lightspeed_validator::ip::IPV4_VALIDATOR
        )
    }
}

/// See [`ip_validator_instance`].
pub fn ipv6_validator_instance() -> TokenStream2 {
    quote! {
        ::lightspeed_validator::ValidatorRef::Static(
            &::lightspeed_validator::ip::IPV6_VALIDATOR
        )
    }
}
