//! String-contains code generation for `#[validate(contains(...))]` and
//! `#[validate(not_contains(...))]`.

use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{Field, LitBool, LitStr, meta::ParseNestedMeta};

/// Parsed `(pattern = "...", case_sensitive = ...)` arguments.
pub struct ContainsArgs {
    pub pattern: String,
    pub case_sensitive: bool,
}

pub fn parse_contains_args(meta: &ParseNestedMeta<'_>) -> syn::Result<ContainsArgs> {
    let mut pattern: Option<String> = None;
    let mut case_sensitive: Option<bool> = None;

    meta.parse_nested_meta(|inner| {
        if inner.path.is_ident("pattern") {
            if pattern.is_some() {
                return Err(inner.error("duplicate `pattern = ...`"));
            }
            let value = inner.value()?;
            let lit: LitStr = value.parse()?;
            pattern = Some(lit.value());
            Ok(())
        } else if inner.path.is_ident("case_sensitive") {
            if case_sensitive.is_some() {
                return Err(inner.error("duplicate `case_sensitive = ...`"));
            }
            let value = inner.value()?;
            let lit: LitBool = value.parse()?;
            case_sensitive = Some(lit.value);
            Ok(())
        } else {
            Err(inner.error("expected `pattern = \"...\"` or `case_sensitive = <bool>`"))
        }
    })?;

    let pattern = pattern.ok_or_else(|| meta.error("`pattern = \"...\"` is required"))?;
    Ok(ContainsArgs { pattern, case_sensitive: case_sensitive.unwrap_or(true) })
}

/// Ensures a field annotated with a `contains` / `not_contains` validator is
/// a string-compatible type. Thin wrapper around the shared
/// [`super::string_field::ensure_string_field`] helper.
pub fn ensure_string_field(field: &Field) -> syn::Result<()> {
    super::string_field::ensure_string_field(field, "contains` / `not_contains")
}

/// Tokens that construct a `Box<dyn FieldValidator<...>>` for `MustContainValidator`.
pub fn must_contain_validator_instance(args: &ContainsArgs) -> TokenStream2 {
    let pattern = &args.pattern;
    let case_sensitive = args.case_sensitive;
    // `::new` pre-computes the lower-cased pattern once when
    // `case_sensitive == false`, so the per-call `validate` body avoids the
    // `pattern.to_lowercase()` allocation it would otherwise pay every time.
    quote! {
        ::lightspeed_validator::ValidatorRef::Boxed(::std::boxed::Box::new(
            ::lightspeed_validator::contains::MustContainValidator::new(#pattern, #case_sensitive)
        ))
    }
}

/// Tokens that construct a `Box<dyn FieldValidator<...>>` for `MustNotContainValidator`.
pub fn must_not_contain_validator_instance(args: &ContainsArgs) -> TokenStream2 {
    let pattern = &args.pattern;
    let case_sensitive = args.case_sensitive;
    quote! {
        ::lightspeed_validator::ValidatorRef::Boxed(::std::boxed::Box::new(
            ::lightspeed_validator::contains::MustNotContainValidator::new(#pattern, #case_sensitive)
        ))
    }
}
