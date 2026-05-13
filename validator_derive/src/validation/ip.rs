//! Macro support for `#[validate(ip)]`, `#[validate(ipv4)]` and
//! `#[validate(ipv6)]`.
//!
//! All three keywords map to the same `IpValidator`, distinguished only by
//! the [`IpKind`](::lightspeed_validator::ip::IpKind) it carries.

use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{Field, Type};

/// Returns true when `ty` looks like a string-compatible type (`String`,
/// `&str`, `Cow<_, str>`, `Box<str>`, `Rc<str>`, `Arc<str>`, etc.).
fn is_string_like_type(ty: &Type) -> bool {
    match ty {
        Type::Path(p) => {
            if p.qself.is_some() {
                return false;
            }
            let Some(last) = p.path.segments.last() else { return false };
            matches!(last.ident.to_string().as_str(), "String" | "Cow" | "Box" | "Rc" | "Arc" | "str")
        }
        Type::Reference(r) => is_string_like_type(&r.elem),
        _ => false,
    }
}

/// Ensures a field annotated with an `ip` / `ipv4` / `ipv6` validator is a
/// string-compatible type.
pub fn ensure_string_field(field: &Field) -> syn::Result<()> {
    if !is_string_like_type(&field.ty) {
        return Err(syn::Error::new_spanned(
            &field.ty,
            "`ip` / `ipv4` / `ipv6` validators require a string-compatible field \
             (e.g. `String`, `&str`, `Cow<'_, str>`)",
        ));
    }
    Ok(())
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
