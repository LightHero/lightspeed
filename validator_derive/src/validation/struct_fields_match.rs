//! Macro support for the struct-level `#[validate(fields_match(...))]` rule.
//!
//! Each helper handles one phase of the macro pipeline for this rule:
//! parsing the `(field_a, field_b[, attach_to_fields = <bool>])` arguments,
//! verifying both names refer to existing fields, emitting the per-rule unit
//! struct + `StructValidator` impl, and emitting the dispatch code that calls
//! that validator inside the generated `validate` body and routes its errors
//! either to the top-level `errors` vec or onto each named field.
//!
//! ## Error-routing convention
//!
//! The generated unit-struct impl and the generated dispatch site share an
//! implicit contract:
//! - When `attach_to_fields = false` the impl returns a single
//!   `ValidationError::FieldsMustMatch(FieldsMustMatch { field_a, field_b })`
//!   and the dispatch extends `self.top_level_errors`.
//! - When `attach_to_fields = true` the impl returns exactly two
//!   `ValidationError::MustMatchField(MustMatchField { field: ... })` errors:
//!   the first names `field_b` (and is routed to `field_a`'s `errors`), the
//!   second names `field_a` (and is routed to `field_b`'s `errors`). Both
//!   ends are emitted by this module so they stay in lockstep.

use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{FieldsNamed, Ident, LitBool, Type, meta::ParseNestedMeta};

/// Parsed arguments of one `fields_match(...)` invocation.
pub struct FieldsMatchArgs {
    pub a: Ident,
    pub b: Ident,
    pub attach_to_fields: bool,
}

/// Parses `fields_match(field_a, field_b[, attach_to_fields = <bool>])`.
/// Positional identifiers are collected as field names; `attach_to_fields`
/// is recognized as a keyword argument. Exactly two field names are required;
/// `attach_to_fields` defaults to `false`.
pub fn parse_fields_match(meta: &ParseNestedMeta<'_>) -> syn::Result<FieldsMatchArgs> {
    let mut field_idents: Vec<Ident> = Vec::new();
    let mut attach_to_fields: Option<bool> = None;

    meta.parse_nested_meta(|inner| {
        if inner.input.peek(syn::Token![=]) {
            if inner.path.is_ident("attach_to_fields") {
                if attach_to_fields.is_some() {
                    return Err(inner.error("duplicate `attach_to_fields = ...`"));
                }
                let lit: LitBool = inner.value()?.parse()?;
                attach_to_fields = Some(lit.value);
                Ok(())
            } else {
                Err(inner.error("unknown `fields_match` option (expected `attach_to_fields = <bool>`)"))
            }
        } else {
            let ident = inner.path.require_ident()?.clone();
            field_idents.push(ident);
            Ok(())
        }
    })?;

    if field_idents.len() != 2 {
        return Err(meta.error(format!("`fields_match` requires exactly 2 field names, got {}", field_idents.len())));
    }

    let mut iter = field_idents.into_iter();
    let a = iter.next().unwrap();
    let b = iter.next().unwrap();

    Ok(FieldsMatchArgs { a, b, attach_to_fields: attach_to_fields.unwrap_or(false) })
}

/// Ensures both field names referenced by the rule exist on the target
/// struct, producing a span-pointed error at the bad identifier otherwise.
pub fn ensure_fields_exist(fields: &FieldsNamed, args: &FieldsMatchArgs) -> syn::Result<()> {
    let exists = |needle: &Ident| fields.named.iter().any(|f| f.ident.as_ref().is_some_and(|i| i == needle));
    if !exists(&args.a) {
        return Err(syn::Error::new(args.a.span(), format!("unknown field `{}`", args.a)));
    }
    if !exists(&args.b) {
        return Err(syn::Error::new(args.b.span(), format!("unknown field `{}`", args.b)));
    }
    Ok(())
}

/// Returns the narrow error type each rule emits — `FieldsMustMatch` when
/// errors go to the top-level vec, or `MustMatchField` per field when
/// `attach_to_fields = true`.
fn narrow_error_type(args: &FieldsMatchArgs) -> TokenStream2 {
    if args.attach_to_fields {
        quote! { ::lightspeed_validator::fields_match::MustMatchField }
    } else {
        quote! { ::lightspeed_validator::fields_match::FieldsMustMatch }
    }
}

/// Emits the per-rule unit struct and its `StructValidator` impl. Returns
/// errors as their narrow type so the dispatch site can `.into()` them into
/// either `ValidationError` (for top-level routing) or the targeted field's
/// per-field error enum (for `attach_to_fields = true`).
pub fn generate_validator_impl(
    validable_name: &Ident,
    ctx_ty: &Type,
    validator_ident: &Ident,
    args: &FieldsMatchArgs,
) -> TokenStream2 {
    let a = &args.a;
    let b = &args.b;
    let a_str = a.to_string();
    let b_str = b.to_string();
    let err_ty = narrow_error_type(args);

    let err_branch = if args.attach_to_fields {
        quote! {
            ::core::result::Result::Err(::std::vec![
                ::lightspeed_validator::fields_match::MustMatchField {
                    field: ::std::string::String::from(#b_str),
                },
                ::lightspeed_validator::fields_match::MustMatchField {
                    field: ::std::string::String::from(#a_str),
                },
            ])
        }
    } else {
        quote! {
            ::core::result::Result::Err(::std::vec![
                ::lightspeed_validator::fields_match::FieldsMustMatch {
                    field_a: ::std::string::String::from(#a_str),
                    field_b: ::std::string::String::from(#b_str),
                }
            ])
        }
    };

    quote! {
        #[doc(hidden)]
        #[allow(non_camel_case_types)]
        struct #validator_ident;

        impl ::lightspeed_validator::StructValidator<
            #validable_name,
            #err_ty,
            #ctx_ty,
        > for #validator_ident {
            fn validate(
                &self,
                value: &#validable_name,
                _context: &#ctx_ty,
            ) -> ::core::result::Result<
                (),
                ::std::vec::Vec<#err_ty>,
            > {
                if value.#a.get() == value.#b.get() {
                    ::core::result::Result::Ok(())
                } else {
                    #err_branch
                }
            }
        }
    }
}

/// Emits the call-site for one rule inside the generated `validate` body.
/// Each narrow error is `.into()`-converted to its routing target:
/// `ValidationError` for the top-level vec, or the targeted field's
/// per-field error enum for `attach_to_fields = true`.
pub fn generate_dispatch(
    validator_ident: &Ident,
    validable_name: &Ident,
    ctx_ty: &Type,
    ctx_expr: &TokenStream2,
    args: &FieldsMatchArgs,
) -> TokenStream2 {
    let a = &args.a;
    let b = &args.b;
    let err_ty = narrow_error_type(args);
    let route_errors = if args.attach_to_fields {
        quote! {
            let mut __it = errs.into_iter();
            if let ::core::option::Option::Some(__e) = __it.next() {
                self.#a.push_error(::core::convert::Into::into(__e));
            }
            if let ::core::option::Option::Some(__e) = __it.next() {
                self.#b.push_error(::core::convert::Into::into(__e));
            }
        }
    } else {
        quote! {
            self.top_level_errors.extend(
                errs.into_iter().map(::core::convert::Into::into)
            );
        }
    };
    quote! {
        if let ::core::result::Result::Err(errs) =
            <#validator_ident as ::lightspeed_validator::StructValidator<
                #validable_name,
                #err_ty,
                #ctx_ty,
            >>::validate(&#validator_ident, &self, #ctx_expr)
        {
            #route_errors
        }
    }
}
