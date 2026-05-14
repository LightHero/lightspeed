//! Field-level validators parsed from `#[validate(...)]` attributes.
//!
//! To add a new variant:
//!  1. add it to [`FieldValidator`];
//!  2. recognize its keyword in [`parse_field_validators`];
//!  3. declare its accepted field types via the per-validator `ensure_*` helper;
//!  4. emit its validator-instance tokens in [`generate_validator_instance`].

pub mod boolean;
pub mod contains;
#[cfg(feature = "credit_card")]
pub mod credit_card;
pub mod custom;
pub mod email;
pub mod ip;
pub mod length;
pub mod password;
pub mod range;
pub mod regex;
pub mod string_field;
pub mod struct_fields_match;
pub mod url;

use proc_macro2::TokenStream as TokenStream2;
use syn::{Field, Ident, Type};

use contains::ContainsArgs;
use custom::CustomArgs;
use length::LengthArgs;
use password::PasswordArgs;
use range::RangeArgs;
use regex::RegexSpec;

const VALIDATE_ATTR: &str = "validate";

/// A single validator configured on a field via `#[validate(<keyword>)]`.
pub enum FieldValidator {
    IsTrue,
    IsFalse,
    MustContain(ContainsArgs),
    MustNotContain(ContainsArgs),
    Ip,
    Ipv4,
    Ipv6,
    Url,
    Email,
    Password(PasswordArgs),
    Range(RangeArgs),
    Regex(RegexSpec),
    Length(LengthArgs),
    #[cfg(feature = "credit_card")]
    CreditCard,
    Custom(CustomArgs),
}

/// Parses every `#[validate(...)]` attribute on `field`, returning all
/// validators in source order. Also enforces that each validator is
/// compatible with the field's declared type.
pub fn parse_field_validators(field: &Field) -> syn::Result<Vec<FieldValidator>> {
    let mut out = Vec::new();
    for attr in &field.attrs {
        if !attr.path().is_ident(VALIDATE_ATTR) {
            continue;
        }
        attr.parse_nested_meta(|meta| {
            let keyword = meta.path.require_ident()?;
            let validator = match keyword.to_string().as_str() {
                "isTrue" => {
                    boolean::ensure_bool_field(field)?;
                    FieldValidator::IsTrue
                }
                "isFalse" => {
                    boolean::ensure_bool_field(field)?;
                    FieldValidator::IsFalse
                }
                "contains" => {
                    contains::ensure_string_field(field)?;
                    FieldValidator::MustContain(contains::parse_contains_args(&meta)?)
                }
                "not_contains" => {
                    contains::ensure_string_field(field)?;
                    FieldValidator::MustNotContain(contains::parse_contains_args(&meta)?)
                }
                "ip" => {
                    ip::ensure_string_field(field)?;
                    FieldValidator::Ip
                }
                "ipv4" => {
                    ip::ensure_string_field(field)?;
                    FieldValidator::Ipv4
                }
                "ipv6" => {
                    ip::ensure_string_field(field)?;
                    FieldValidator::Ipv6
                }
                "url" => {
                    url::ensure_string_field(field)?;
                    FieldValidator::Url
                }
                "email" => {
                    email::ensure_string_field(field)?;
                    FieldValidator::Email
                }
                "password" => {
                    password::ensure_string_field(field)?;
                    // Accept both bare `password` and `password(...)`.
                    let args = if meta.input.peek(syn::token::Paren) {
                        password::parse_password_args(&meta)?
                    } else {
                        password::PasswordArgs::default()
                    };
                    FieldValidator::Password(args)
                }
                "range" => FieldValidator::Range(range::parse_range_args(&meta)?),
                "regex" => {
                    regex::ensure_string_field(field)?;
                    FieldValidator::Regex(regex::parse_regex_args(&meta)?)
                }
                "length" => FieldValidator::Length(length::parse_length_args(&meta)?),
                #[cfg(feature = "credit_card")]
                "credit_card" => {
                    credit_card::ensure_string_field(field)?;
                    FieldValidator::CreditCard
                }
                // No `ensure_*_field` for `custom` — the user function's
                // signature pins the accepted field/context/error types,
                // and a mismatch surfaces as a normal type error at the
                // call site of the generated `boxed_fn(...)`.
                "custom" => FieldValidator::Custom(custom::parse_custom_args(&meta)?),
                other => {
                    return Err(syn::Error::new(keyword.span(), format!("unknown validator `{other}`")));
                }
            };
            out.push(validator);
            Ok(())
        })?;
    }
    Ok(out)
}

impl FieldValidator {
    /// `(variant_name, error_type_path)` for this validator.
    ///
    /// - `variant_name` is the variant the corresponding `ValidationError`
    ///   wraps the narrow error in (also used as the variant name in the
    ///   macro-generated per-field error enum).
    /// - `error_type_path` is the fully-qualified path to the narrow error
    ///   type the validator emits at runtime.
    ///
    /// Multiple variants may share the same `variant_name` / error type
    /// (e.g. `Ip` / `Ipv4` / `Ipv6` all emit `IpError`); the per-field enum
    /// generator deduplicates them.
    pub fn error_info(&self) -> (&'static str, TokenStream2) {
        match self {
            FieldValidator::IsTrue => {
                ("MustBeTrue", quote::quote! { ::lightspeed_validator::boolean::MustBeTrueError })
            }
            FieldValidator::IsFalse => {
                ("MustBeFalse", quote::quote! { ::lightspeed_validator::boolean::MustBeFalseError })
            }
            FieldValidator::MustContain(_) => {
                ("MustContain", quote::quote! { ::lightspeed_validator::contains::MustContainError })
            }
            FieldValidator::MustNotContain(_) => {
                ("MustNotContain", quote::quote! { ::lightspeed_validator::contains::MustNotContainError })
            }
            FieldValidator::Ip | FieldValidator::Ipv4 | FieldValidator::Ipv6 => {
                ("Ip", quote::quote! { ::lightspeed_validator::ip::IpError })
            }
            FieldValidator::Url => ("Url", quote::quote! { ::lightspeed_validator::url::UrlError }),
            FieldValidator::Email => ("Email", quote::quote! { ::lightspeed_validator::email::EmailError }),
            FieldValidator::Password(_) => {
                ("Password", quote::quote! { ::lightspeed_validator::password::PasswordError })
            }
            FieldValidator::Range(_) => ("Range", quote::quote! { ::lightspeed_validator::range::RangeError }),
            FieldValidator::Regex(_) => ("Regex", quote::quote! { ::lightspeed_validator::regex::RegexError }),
            FieldValidator::Length(_) => ("Length", quote::quote! { ::lightspeed_validator::length::LengthError }),
            #[cfg(feature = "credit_card")]
            FieldValidator::CreditCard => {
                ("CreditCard", quote::quote! { ::lightspeed_validator::credit_card::CreditCardError })
            }
            // In `errors(tailored)` mode this surfaces as a
            // `Custom(ValidationError)` variant on the per-field enum plus
            // a `From<ValidationError>` impl — which doubles as an escape
            // hatch for user code that wants to return the wide error type
            // from a custom function and lift it into the narrow enum.
            FieldValidator::Custom(_) => ("Custom", quote::quote! { ::lightspeed_validator::ValidationError }),
        }
    }
}

/// Emits the tokens that build a `Box<dyn FieldValidator<...>>` instance for
/// `validator`, suitable for inclusion in a `vec![...]` passed to
/// `ValidableType::new`. `field_ty` is the field's declared type; most
/// validators ignore it, but generic validators like `range` need it to
/// pin their type parameter.
pub fn generate_validator_instance(validator: &FieldValidator, field_ty: &Type) -> TokenStream2 {
    match validator {
        FieldValidator::IsTrue => boolean::must_be_true_validator_instance(),
        FieldValidator::IsFalse => boolean::must_be_false_validator_instance(),
        FieldValidator::MustContain(args) => contains::must_contain_validator_instance(args),
        FieldValidator::MustNotContain(args) => contains::must_not_contain_validator_instance(args),
        FieldValidator::Ip => ip::ip_validator_instance(),
        FieldValidator::Ipv4 => ip::ipv4_validator_instance(),
        FieldValidator::Ipv6 => ip::ipv6_validator_instance(),
        FieldValidator::Url => url::url_validator_instance(),
        FieldValidator::Email => email::email_validator_instance(),
        FieldValidator::Password(args) => password::password_validator_instance(args),
        FieldValidator::Range(args) => range::range_validator_instance(field_ty, args),
        FieldValidator::Regex(spec) => regex::regex_validator_instance(spec),
        FieldValidator::Length(args) => length::length_validator_instance(args),
        #[cfg(feature = "credit_card")]
        FieldValidator::CreditCard => credit_card::credit_card_validator_instance(),
        FieldValidator::Custom(args) => custom::custom_validator_instance(args),
    }
}

/// Emits a `vec![...]` of validator instances for `validators`. An empty input
/// produces an empty vec literal. `field_ty` is forwarded to each
/// validator's instance emitter (only used by generics like `range`).
pub fn generate_validators_vec(_field_ident: &Ident, field_ty: &Type, validators: &[FieldValidator]) -> TokenStream2 {
    let items = validators.iter().map(|v| generate_validator_instance(v, field_ty));
    quote::quote! {
        ::std::vec![ #( #items ),* ]
    }
}
