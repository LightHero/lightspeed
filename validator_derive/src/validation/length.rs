//! Macro support for `#[validate(length(min = ..., max = ..., equal = ...))]`.
//!
//! All three keys are optional but at least one is required. `equal` is
//! mutually exclusive with `min`/`max` — combining them is rejected at
//! macro-expansion time. The macro doesn't constrain the field's type; the
//! runtime `HasLength` trait does, so any field type that doesn't impl
//! `HasLength` produces a normal trait-bound error at the generated
//! validator-construction site.

use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{Expr, meta::ParseNestedMeta};

#[derive(Default)]
pub struct LengthArgs {
    pub min: Option<Expr>,
    pub max: Option<Expr>,
    pub equal: Option<Expr>,
}

pub fn parse_length_args(meta: &ParseNestedMeta<'_>) -> syn::Result<LengthArgs> {
    let mut args = LengthArgs::default();

    meta.parse_nested_meta(|inner| {
        if inner.path.is_ident("min") {
            if args.min.is_some() {
                return Err(inner.error("duplicate `min = ...`"));
            }
            args.min = Some(inner.value()?.parse::<Expr>()?);
            Ok(())
        } else if inner.path.is_ident("max") {
            if args.max.is_some() {
                return Err(inner.error("duplicate `max = ...`"));
            }
            args.max = Some(inner.value()?.parse::<Expr>()?);
            Ok(())
        } else if inner.path.is_ident("equal") {
            if args.equal.is_some() {
                return Err(inner.error("duplicate `equal = ...`"));
            }
            args.equal = Some(inner.value()?.parse::<Expr>()?);
            Ok(())
        } else {
            Err(inner.error("unknown `length` option (expected `min`, `max` or `equal`)"))
        }
    })?;

    if args.min.is_none() && args.max.is_none() && args.equal.is_none() {
        return Err(meta.error("`length` requires at least one of `min`, `max`, `equal`"));
    }
    if args.equal.is_some() && (args.min.is_some() || args.max.is_some()) {
        return Err(meta.error("`equal` cannot be combined with `min` or `max`"));
    }

    Ok(args)
}

/// Tokens that construct a `Box<dyn FieldValidator<...>>` for `LengthValidator`.
pub fn length_validator_instance(args: &LengthArgs) -> TokenStream2 {
    let min = expr_to_option_usize(&args.min);
    let max = expr_to_option_usize(&args.max);
    let equal = expr_to_option_usize(&args.equal);
    quote! {
        ::lightspeed_validator::ValidatorRef::Boxed(::std::boxed::Box::new(
            ::lightspeed_validator::length::LengthValidator {
                min: #min,
                max: #max,
                equal: #equal,
            }
        ))
    }
}

/// `Option<usize>` literal expression — wraps the user's expression in
/// `Option::Some(... as usize)` so plain integer literals and `const` paths
/// both coerce to `usize` at the construction site.
fn expr_to_option_usize(opt: &Option<Expr>) -> TokenStream2 {
    match opt {
        Some(e) => quote! { ::core::option::Option::Some((#e) as ::core::primitive::usize) },
        None => quote! { ::core::option::Option::None },
    }
}
