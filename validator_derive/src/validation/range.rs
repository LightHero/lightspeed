//! Macro support for `#[validate(range(min = ..., max = ..., exclusive_min = ...,
//! exclusive_max = ...))]`.
//!
//! Each bound is optional, but at least one must be provided. The macro emits
//! a `RangeValidator::<FieldTy>` so the bounds' types are forced to match the
//! field's declared type — any literal/expression that the compiler can't
//! coerce to that type produces a normal type-error at the macro expansion
//! site (which is the most useful error you can get without a custom trait
//! check).

use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{Expr, Type, meta::ParseNestedMeta};

#[derive(Default)]
pub struct RangeArgs {
    pub min: Option<Expr>,
    pub max: Option<Expr>,
    pub exclusive_min: Option<Expr>,
    pub exclusive_max: Option<Expr>,
}

pub fn parse_range_args(meta: &ParseNestedMeta<'_>) -> syn::Result<RangeArgs> {
    let mut args = RangeArgs::default();

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
        } else if inner.path.is_ident("exclusive_min") {
            if args.exclusive_min.is_some() {
                return Err(inner.error("duplicate `exclusive_min = ...`"));
            }
            args.exclusive_min = Some(inner.value()?.parse::<Expr>()?);
            Ok(())
        } else if inner.path.is_ident("exclusive_max") {
            if args.exclusive_max.is_some() {
                return Err(inner.error("duplicate `exclusive_max = ...`"));
            }
            args.exclusive_max = Some(inner.value()?.parse::<Expr>()?);
            Ok(())
        } else {
            Err(inner.error("unknown `range` option (expected `min`, `max`, `exclusive_min` or `exclusive_max`)"))
        }
    })?;

    if args.min.is_none() && args.max.is_none() && args.exclusive_min.is_none() && args.exclusive_max.is_none() {
        return Err(meta.error("`range` requires at least one of `min`, `max`, `exclusive_min`, `exclusive_max`"));
    }

    Ok(args)
}

/// Tokens that construct a `Box<dyn FieldValidator<...>>` for `RangeValidator<#field_ty>`.
pub fn range_validator_instance(field_ty: &Type, args: &RangeArgs) -> TokenStream2 {
    let min = expr_to_option(&args.min);
    let max = expr_to_option(&args.max);
    let exclusive_min = expr_to_option(&args.exclusive_min);
    let exclusive_max = expr_to_option(&args.exclusive_max);
    quote! {
        ::lightspeed_validator::ValidatorRef::Boxed(::std::boxed::Box::new(
            ::lightspeed_validator::range::RangeValidator::<#field_ty> {
                min: #min,
                max: #max,
                exclusive_min: #exclusive_min,
                exclusive_max: #exclusive_max,
            }
        ))
    }
}

fn expr_to_option(opt: &Option<Expr>) -> TokenStream2 {
    match opt {
        Some(e) => quote! { ::core::option::Option::Some(#e) },
        None => quote! { ::core::option::Option::None },
    }
}
