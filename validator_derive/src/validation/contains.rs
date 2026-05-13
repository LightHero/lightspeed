//! String-contains code generation for `#[validate(contains(...))]` and
//! `#[validate(not_contains(...))]`.

use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{Field, LitBool, LitStr, Type, meta::ParseNestedMeta};

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

/// Ensures a field annotated with a `contains` / `not_contains` validator is a
/// string-compatible type.
pub fn ensure_string_field(field: &Field) -> syn::Result<()> {
    if !is_string_like_type(&field.ty) {
        return Err(syn::Error::new_spanned(
            &field.ty,
            "`contains` / `not_contains` validators require a string-compatible field \
             (e.g. `String`, `&str`, `Cow<'_, str>`)",
        ));
    }
    Ok(())
}

/// Tokens that construct a `Box<dyn FieldValidator<...>>` for `MustContainValidator`.
pub fn must_contain_validator_instance(args: &ContainsArgs) -> TokenStream2 {
    let pattern = &args.pattern;
    let case_sensitive = args.case_sensitive;
    // `::new` pre-computes the lower-cased pattern once when
    // `case_sensitive == false`, so the per-call `validate` body avoids the
    // `pattern.to_lowercase()` allocation it would otherwise pay every time.
    quote! {
        ::std::boxed::Box::new(
            ::lightspeed_validator::contains::MustContainValidator::new(#pattern, #case_sensitive)
        )
    }
}

/// Tokens that construct a `Box<dyn FieldValidator<...>>` for `MustNotContainValidator`.
pub fn must_not_contain_validator_instance(args: &ContainsArgs) -> TokenStream2 {
    let pattern = &args.pattern;
    let case_sensitive = args.case_sensitive;
    quote! {
        ::std::boxed::Box::new(
            ::lightspeed_validator::contains::MustNotContainValidator::new(#pattern, #case_sensitive)
        )
    }
}
