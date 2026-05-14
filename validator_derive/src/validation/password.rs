//! Macro support for `#[validate(password)]` and
//! `#[validate(password(upper = <bool>, lower = <bool>, number = <bool>,
//! special_char = <bool|str>, trailing_whitespaces = <bool>))]`.

use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{Field, Lit, LitBool, meta::ParseNestedMeta};

/// Parsed arguments for a single `password(...)` invocation. Defaults match
/// [`PasswordValidator::default()`](::lightspeed_validator::password::PasswordValidator):
/// every class is required, the default special-char list is used, and
/// trailing whitespace is forbidden.
pub struct PasswordArgs {
    pub upper: bool,
    pub lower: bool,
    pub number: bool,
    pub special_char: SpecialCharSpec,
    pub trailing_whitespaces: bool,
}

impl Default for PasswordArgs {
    fn default() -> Self {
        Self {
            upper: true,
            lower: true,
            number: true,
            special_char: SpecialCharSpec::Default,
            trailing_whitespaces: false,
        }
    }
}

/// What was requested for the `special_char` option.
pub enum SpecialCharSpec {
    /// `special_char = true` (or absent) — use the runtime's
    /// `DEFAULT_SPECIAL_CHARS` list.
    Default,
    /// `special_char = false` — disable the special-character check.
    Disabled,
    /// `special_char = "..."` — use the chars in the literal as the
    /// allowed set.
    CustomList(String),
}

/// Parses the inner arguments of `password(...)`. Accepts the keys `upper`,
/// `lower`, `number`, `trailing_whitespaces` (each a `bool` literal) and
/// `special_char` (a `bool` or string literal). All keys are optional.
pub fn parse_password_args(meta: &ParseNestedMeta<'_>) -> syn::Result<PasswordArgs> {
    let mut args = PasswordArgs::default();
    let mut set_keys: Vec<&'static str> = Vec::new();

    meta.parse_nested_meta(|inner| {
        if inner.path.is_ident("upper") {
            check_duplicate(&mut set_keys, "upper", &inner)?;
            args.upper = inner.value()?.parse::<LitBool>()?.value;
            Ok(())
        } else if inner.path.is_ident("lower") {
            check_duplicate(&mut set_keys, "lower", &inner)?;
            args.lower = inner.value()?.parse::<LitBool>()?.value;
            Ok(())
        } else if inner.path.is_ident("number") {
            check_duplicate(&mut set_keys, "number", &inner)?;
            args.number = inner.value()?.parse::<LitBool>()?.value;
            Ok(())
        } else if inner.path.is_ident("special_char") {
            check_duplicate(&mut set_keys, "special_char", &inner)?;
            let lit: Lit = inner.value()?.parse()?;
            args.special_char = match lit {
                Lit::Bool(b) => {
                    if b.value {
                        SpecialCharSpec::Default
                    } else {
                        SpecialCharSpec::Disabled
                    }
                }
                Lit::Str(s) => SpecialCharSpec::CustomList(s.value()),
                other => {
                    return Err(syn::Error::new_spanned(other, "`special_char` expects a `bool` or a string literal"));
                }
            };
            Ok(())
        } else if inner.path.is_ident("trailing_whitespaces") {
            check_duplicate(&mut set_keys, "trailing_whitespaces", &inner)?;
            args.trailing_whitespaces = inner.value()?.parse::<LitBool>()?.value;
            Ok(())
        } else {
            Err(inner.error(
                "unknown `password` option (expected `upper`, `lower`, `number`, \
                 `special_char` or `trailing_whitespaces`)",
            ))
        }
    })?;

    Ok(args)
}

fn check_duplicate(set_keys: &mut Vec<&'static str>, key: &'static str, meta: &ParseNestedMeta<'_>) -> syn::Result<()> {
    if set_keys.contains(&key) {
        return Err(meta.error(format!("duplicate `{key} = ...`")));
    }
    set_keys.push(key);
    Ok(())
}

/// Ensures a field annotated with a `password` validator is a
/// string-compatible type. Thin wrapper around
/// [`super::string_field::ensure_string_field`].
pub fn ensure_string_field(field: &Field) -> syn::Result<()> {
    super::string_field::ensure_string_field(field, "password")
}

/// Tokens that construct a `Box<dyn FieldValidator<...>>` for
/// `PasswordValidator` with the configured args.
pub fn password_validator_instance(args: &PasswordArgs) -> TokenStream2 {
    let upper = args.upper;
    let lower = args.lower;
    let number = args.number;
    let trailing = args.trailing_whitespaces;

    let special_chars = match &args.special_char {
        SpecialCharSpec::Default => quote! {
            ::core::option::Option::Some(
                <[char]>::to_vec(::lightspeed_validator::password::DEFAULT_SPECIAL_CHARS)
            )
        },
        SpecialCharSpec::Disabled => quote! { ::core::option::Option::None },
        SpecialCharSpec::CustomList(s) => {
            let chars = s.chars();
            quote! {
                ::core::option::Option::Some(::std::vec![ #( #chars ),* ])
            }
        }
    };

    quote! {
        ::lightspeed_validator::ValidatorRef::Boxed(::std::boxed::Box::new(
            ::lightspeed_validator::password::PasswordValidator {
                upper: #upper,
                lower: #lower,
                number: #number,
                special_chars: #special_chars,
                trailing_whitespaces: #trailing,
            }
        ))
    }
}
