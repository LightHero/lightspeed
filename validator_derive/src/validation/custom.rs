//! Macro support for `#[validate(custom(function = "<path>"))]`.
//!
//! Accepts either `function = "path::to::fn"` (string literal — matching the
//! `validator` crate's convention) or `function = path::to::fn` (bare path
//! expression). The referenced item must be a free function — or any path
//! the compiler can coerce to a function pointer — with the signature
//! `fn(&FieldTy, &CtxTy) -> Result<(), FieldErrTy>`, where `FieldErrTy` is
//! the field's error type as selected by the struct-level `errors(...)`
//! strategy (defaults to `lightspeed_validator::ValidationError`).
//!
//! Field-type checking is delegated to the user's function signature: the
//! macro emits `lightspeed_validator::custom::boxed_fn(<path>)`, and any
//! signature mismatch surfaces as a normal type error at the call site.

use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{LitStr, Path, meta::ParseNestedMeta};

pub struct CustomArgs {
    pub function: Path,
}

/// Parses the contents of `custom(...)`. Expects exactly one
/// `function = <path>` entry; rejects unknown keys, duplicates, and the
/// empty form.
pub fn parse_custom_args(meta: &ParseNestedMeta<'_>) -> syn::Result<CustomArgs> {
    let mut function: Option<Path> = None;

    meta.parse_nested_meta(|inner| {
        if inner.path.is_ident("function") {
            if function.is_some() {
                return Err(inner.error("duplicate `function = ...`"));
            }
            let value = inner.value()?;
            // `function = "path::to::fn"` is the documented form; the bare-path
            // form is supported for symmetry with `regex(path = <expr>)` and
            // other Rust-native attributes.
            if value.peek(LitStr) {
                let lit: LitStr = value.parse()?;
                function = Some(lit.parse::<Path>()?);
            } else {
                function = Some(value.parse::<Path>()?);
            }
            Ok(())
        } else {
            Err(inner.error("unknown `custom` option (expected `function = \"<path>\"`)"))
        }
    })?;

    function
        .map(|function| CustomArgs { function })
        .ok_or_else(|| meta.error("`custom` requires `function = \"<path>\"`"))
}

/// Tokens that wrap `args.function` as a `ValidatorRef` via the runtime
/// [`lightspeed_validator::custom::boxed_fn`] helper. The helper takes a
/// `fn` pointer, so the function-item-to-fn-pointer coercion happens in
/// argument position — no brittle `as fn(...)` cast with `_` placeholders
/// in the generated code.
pub fn custom_validator_instance(args: &CustomArgs) -> TokenStream2 {
    let function = &args.function;
    quote! {
        ::lightspeed_validator::custom::boxed_fn(#function)
    }
}
