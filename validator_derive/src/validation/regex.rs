//! Macro support for `#[validate(regex(path = <expr>))]` and
//! `#[validate(regex(pattern = "..."))]`.
//!
//! - `path` — the expression should evaluate to `&'static ::regex::Regex`.
//!   The user is responsible for backing it with a `OnceLock<Regex>`,
//!   `LazyLock<Regex>`, or similar.
//! - `pattern` — a string literal. The macro emits an inline
//!   `static OnceLock<Regex>` keyed at the call site, initialized on first
//!   `validate` via `get_or_init`.
//!
//! Exactly one of the two must be supplied.

use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{Expr, Field, LitStr, meta::ParseNestedMeta};

pub enum RegexSpec {
    /// User-provided expression that already evaluates to `&'static Regex`.
    Path(Expr),
    /// Regex source text — the macro will generate an inline `OnceLock`.
    Pattern(String),
}

pub fn parse_regex_args(meta: &ParseNestedMeta<'_>) -> syn::Result<RegexSpec> {
    let mut spec: Option<RegexSpec> = None;

    meta.parse_nested_meta(|inner| {
        if inner.path.is_ident("path") {
            if spec.is_some() {
                return Err(inner.error("`path` and `pattern` are mutually exclusive"));
            }
            spec = Some(RegexSpec::Path(inner.value()?.parse::<Expr>()?));
            Ok(())
        } else if inner.path.is_ident("pattern") {
            if spec.is_some() {
                return Err(inner.error("`path` and `pattern` are mutually exclusive"));
            }
            let lit: LitStr = inner.value()?.parse()?;
            spec = Some(RegexSpec::Pattern(lit.value()));
            Ok(())
        } else {
            Err(inner.error("expected `path = <expr>` or `pattern = \"...\"`"))
        }
    })?;

    spec.ok_or_else(|| meta.error("`regex` requires either `path = <expr>` or `pattern = \"...\"`"))
}

/// Ensures a field annotated with a `regex` validator is a string-compatible
/// type. Thin wrapper around [`super::string_field::ensure_string_field`].
pub fn ensure_string_field(field: &Field) -> syn::Result<()> {
    super::string_field::ensure_string_field(field, "regex")
}

/// Tokens that construct a `Box<dyn FieldValidator<...>>` for `RegexValidator`.
/// For the `Pattern` variant the expansion lifts the regex into a per-call-site
/// `static OnceLock<Regex>`, so each `validate` call after the first reuses
/// the compiled regex.
pub fn regex_validator_instance(spec: &RegexSpec) -> TokenStream2 {
    let regex_expr = match spec {
        RegexSpec::Path(p) => quote! { #p },
        RegexSpec::Pattern(s) => quote! {
            {
                static __REGEX: ::std::sync::OnceLock<::regex::Regex> =
                    ::std::sync::OnceLock::new();
                __REGEX.get_or_init(|| {
                    ::regex::Regex::new(#s)
                        .expect(::core::concat!("invalid regex pattern: ", #s))
                })
            }
        },
    };
    quote! {
        ::std::boxed::Box::new(::lightspeed_validator::regex::RegexValidator {
            regex: #regex_expr,
        })
    }
}
