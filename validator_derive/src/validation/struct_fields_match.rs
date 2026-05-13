//! Macro support for the struct-level `#[validate(fields_match(...))]` rule.
//!
//! Each helper handles one phase of the macro pipeline for this rule:
//! parsing the `(field_a, field_b[, attach_to_fields = <bool>])` arguments,
//! verifying both names refer to existing fields, emitting the per-rule unit
//! struct + `StructValidator` impl, and emitting the dispatch code that calls
//! that validator inside the generated `validate` body and routes its errors
//! either to the top-level `errors` vec or onto each named field.

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
        return Err(meta.error(format!(
            "`fields_match` requires exactly 2 field names, got {}",
            field_idents.len()
        )));
    }

    let mut iter = field_idents.into_iter();
    let a = iter.next().unwrap();
    let b = iter.next().unwrap();

    Ok(FieldsMatchArgs { a, b, attach_to_fields: attach_to_fields.unwrap_or(false) })
}

/// Ensures both field names referenced by the rule exist on the target
/// struct, producing a span-pointed error at the bad identifier otherwise.
pub fn ensure_fields_exist(fields: &FieldsNamed, args: &FieldsMatchArgs) -> syn::Result<()> {
    let exists = |needle: &Ident| {
        fields
            .named
            .iter()
            .any(|f| f.ident.as_ref().is_some_and(|i| i == needle))
    };
    if !exists(&args.a) {
        return Err(syn::Error::new(args.a.span(), format!("unknown field `{}`", args.a)));
    }
    if !exists(&args.b) {
        return Err(syn::Error::new(args.b.span(), format!("unknown field `{}`", args.b)));
    }
    Ok(())
}

/// Emits the per-rule unit struct and its `StructValidator` impl. The body
/// of the impl compares the two fields with `==` and returns
/// `ValidationError::FieldsMustMatch` on inequality.
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
    quote! {
        #[doc(hidden)]
        #[allow(non_camel_case_types)]
        struct #validator_ident;

        impl ::lightspeed_validator::StructValidator<
            #validable_name,
            ::lightspeed_validator::ValidationError,
            #ctx_ty,
        > for #validator_ident {
            fn validate(
                &self,
                value: &#validable_name,
                _context: &#ctx_ty,
            ) -> ::core::result::Result<
                (),
                ::std::vec::Vec<::lightspeed_validator::ValidationError>,
            > {
                if value.#a.get() == value.#b.get() {
                    ::core::result::Result::Ok(())
                } else {
                    ::core::result::Result::Err(::std::vec![
                        ::lightspeed_validator::ValidationError::FieldsMustMatch {
                            a: #a_str,
                            b: #b_str,
                        }
                    ])
                }
            }
        }
    }
}

/// Emits the call-site for one rule inside the generated `validate` body.
/// Routes the produced errors either to `self.top_level_errors` or onto each
/// named field's `errors` based on the rule's `attach_to_fields` flag.
pub fn generate_dispatch(
    validator_ident: &Ident,
    validable_name: &Ident,
    ctx_ty: &Type,
    ctx_expr: &TokenStream2,
    args: &FieldsMatchArgs,
) -> TokenStream2 {
    let a = &args.a;
    let b = &args.b;
    let route_errors = if args.attach_to_fields {
        quote! {
            for __e in errs {
                self.#a.push_error(::core::clone::Clone::clone(&__e));
                self.#b.push_error(__e);
            }
        }
    } else {
        quote! { self.top_level_errors.extend(errs); }
    };
    quote! {
        if let ::core::result::Result::Err(errs) =
            <#validator_ident as ::lightspeed_validator::StructValidator<
                #validable_name,
                ::lightspeed_validator::ValidationError,
                #ctx_ty,
            >>::validate(&#validator_ident, &self, #ctx_expr)
        {
            #route_errors
        }
    }
}
